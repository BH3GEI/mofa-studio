#!/bin/bash
# 一键创建完整安装包：签名 + 公证 + 安装DMG

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/build/macos"
APP_NAME="MoFA Studio"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
DMG_NAME="MofaStudio-1.0.0"
DMG_PATH="$BUILD_DIR/$DMG_NAME.dmg"

SIGN_HASH="78055F15F4D6E7C85EF890EA066AACBCB1E908C8"
TEAM_ID="SX7GH8L8YB"

echo "========================================"
echo "  创建完整安装包"
echo "========================================"
echo ""

# 检查App是否存在
if [ ! -d "$APP_BUNDLE" ]; then
    echo "❌ 错误：未找到 $APP_BUNDLE"
    echo "请先运行: ./scripts/package-macos.sh"
    exit 1
fi

# 步骤1: 签名App
echo "🔏 步骤1: 签名应用..."
codesign --deep --force --options runtime \
    --sign "$SIGN_HASH" \
    --timestamp \
    "$APP_BUNDLE"

echo "✅ 应用已签名"

# 步骤2: 创建安装DMG（带Applications快捷方式）
echo ""
echo "📦 步骤2: 创建安装DMG..."

# 清理旧DMG
rm -f "$DMG_PATH"

# 创建临时目录
TEMP_DIR="/tmp/mofa-installer-$$"
rm -rf "$TEMP_DIR"
mkdir -p "$TEMP_DIR"

# 复制App
cp -R "$APP_BUNDLE" "$TEMP_DIR/"

# 创建Applications快捷方式
ln -s /Applications "$TEMP_DIR/Applications"

# 创建DMG
hdiutil create -srcfolder "$TEMP_DIR" \
    -volname "$APP_NAME Installer" \
    -fs HFS+ \
    -format UDZO \
    -imagekey zlib-level=9 \
    -o "$DMG_PATH"

# 清理临时目录
rm -rf "$TEMP_DIR"

echo "✅ DMG创建完成"

# 步骤3: 签名DMG
echo ""
echo "🔏 步骤3: 签名DMG..."
codesign --sign "$SIGN_HASH" --timestamp "$DMG_PATH"

echo "✅ DMG已签名"

# 步骤4: 公证
echo ""
echo "📋 步骤4: 公证..."
echo "提交到Apple公证服务..."

# 使用保存的凭证
RESULT=$(xcrun notarytool submit "$DMG_PATH" \
    --keychain-profile "mofa-notary" \
    --wait 2>&1)

# 检查公证状态
if echo "$RESULT" | grep -q "Accepted"; then
    echo "✅ 公证通过！"
    
    # 获取提交ID
    SUBMISSION_ID=$(echo "$RESULT" | grep "id:" | head -1 | awk '{print $2}')
    echo "提交ID: $SUBMISSION_ID"
else
    echo "⚠️ 公证状态异常，请检查："
    echo "$RESULT"
    exit 1
fi

echo ""
echo "========================================"
echo "  ✅ 安装包创建完成！"
echo "========================================"
echo ""
echo "📦 文件信息："
echo "   位置: $DMG_PATH"
echo "   大小: $(du -h "$DMG_PATH" | cut -f1)"
echo ""
echo "🔏 签名信息："
codesign -dv --verbose=4 "$APP_BUNDLE" 2>&1 | grep -E "Authority|TeamIdentifier" | head -3
echo ""
echo "✅ 公证状态：已通过"
echo ""
echo "💡 用户安装步骤："
echo "   1. 双击DMG文件"
echo "   2. 将MoFA Studio拖拽到Applications"
echo "   3. 从Applications启动应用"
echo ""
echo "🎉 无需右键'打开'，双击即用！"

# 打开Finder显示文件
read -p "是否在Finder中显示？ [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    open -R "$DMG_PATH"
fi
