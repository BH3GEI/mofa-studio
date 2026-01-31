#!/bin/bash
# 简化的DMG安装器 - 使用icon视图和预设布局

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/build/macos"
APP_NAME="MoFA Studio"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
DMG_NAME="MofaStudio-1.0.0"
DMG_PATH="$BUILD_DIR/$DMG_NAME.dmg"
TEMP_DIR="/tmp/dmg-temp-$$"

SIGN_HASH="78055F15F4D6E7C85EF890EA066AACBCB1E908C8"

echo "🎨 创建安装DMG..."

# 清理旧文件
rm -f "$DMG_PATH"
rm -rf "$TEMP_DIR"
mkdir -p "$TEMP_DIR"

# 复制App和创建Applications链接
cp -R "$APP_BUNDLE" "$TEMP_DIR/"
ln -s /Applications "$TEMP_DIR/Applications"

# 直接创建压缩DMG
echo "💾 创建DMG..."
hdiutil create -srcfolder "$TEMP_DIR" -volname "$APP_NAME Installer" \
    -fs HFS+ -format UDZO -imagekey zlib-level=9 \
    -o "$DMG_PATH" -quiet

# 签名
echo "🔏 签名DMG..."
codesign --sign "$SIGN_HASH" --timestamp "$DMG_PATH" 2>/dev/null || echo "⚠️ 签名失败"

# 清理
rm -rf "$TEMP_DIR"

echo ""
echo "✅ DMG创建完成！"
echo "文件: $DMG_PATH"
echo "大小: $(du -h "$DMG_PATH" | cut -f1)"
echo ""
echo "📦 包含内容:"
echo "   - MoFA Studio.app"
echo "   - Applications (快捷方式)"
echo ""
echo "💡 提示："
echo "   用户打开DMG后，只需将App拖拽到Applications文件夹即可"

# 可选：打开预览
read -p "是否现在打开预览？ [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    open "$DMG_PATH"
fi
