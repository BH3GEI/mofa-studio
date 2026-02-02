#!/bin/bash
# 完整的签名和公证脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/build/macos"
APP_NAME="MoFA Studio"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
DMG_PATH="$BUILD_DIR/MofaStudio-1.0.0.dmg"

# 签名身份（使用证书哈希避免歧义）
SIGN_HASH="78055F15F4D6E7C85EF890EA066AACBCB1E908C8"
TEAM_ID="SX7GH8L8YB"
APPLE_ID="li@mofa.ai"

# 检查证书
if ! security find-identity -p codesigning -v | grep -q "$SIGN_HASH"; then
    echo "❌ 错误：未找到证书哈希 $SIGN_HASH"
    echo "可用证书："
    security find-identity -p codesigning -v
    exit 1
fi

echo "✅ 找到签名证书"

# 创建 entitlements 文件
ENTITLEMENTS="$BUILD_DIR/entitlements.plist"
cat > "$ENTITLEMENTS" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.device.audio-input</key>
    <true/>
    <key>com.apple.security.device.microphone</key>
    <true/>
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>
</dict>
</plist>
EOF

echo "🔏 步骤 1: 清理旧的签名和隔离属性..."
# 移除隔离属性
xattr -r -d com.apple.quarantine "$APP_BUNDLE" 2>/dev/null || true
xattr -r -d com.apple.provenance "$APP_BUNDLE" 2>/dev/null || true

echo "🔏 步骤 2: 签名应用..."
# 使用 --force 覆盖之前的 adhoc 签名
codesign --deep --force --options runtime \
    --entitlements "$ENTITLEMENTS" \
    --sign "$SIGN_HASH" \
    --timestamp \
    "$APP_BUNDLE"

echo "✅ 签名完成"

# 验证签名
echo "🔍 验证签名..."
if codesign --verify --deep --strict --verbose=2 "$APP_BUNDLE" 2>&1 | grep -q "valid"; then
    echo "✅ 签名验证通过"
else
    echo "⚠️ 签名验证可能有问题，继续创建 DMG..."
fi

# 显示签名信息
echo "📋 签名信息："
codesign -dv --verbose=4 "$APP_BUNDLE" 2>&1 | grep -E "(Authority|Signature|TeamIdentifier)"

echo ""
echo "📦 步骤 3: 创建 DMG..."

# 删除旧 DMG
rm -f "$DMG_PATH"

# 创建 DMG
if command -v create-dmg &> /dev/null; then
    create-dmg \
        --volname "$APP_NAME" \
        --window-size 600 400 \
        --icon-size 100 \
        --icon "$APP_NAME.app" 150 190 \
        --app-drop-link 450 190 \
        --hide-extension "$APP_NAME.app" \
        "$DMG_PATH" \
        "$APP_BUNDLE"
else
    echo "使用 hdiutil 创建 DMG..."
    hdiutil create -volname "$APP_NAME" \
        -srcfolder "$APP_BUNDLE" \
        -ov -format UDZO \
        "$DMG_PATH"
fi

echo "✅ DMG 创建完成: $DMG_PATH"

# 签名 DMG
echo "🔏 步骤 4: 签名 DMG..."
codesign --sign "$SIGN_HASH" --timestamp "$DMG_PATH"

echo "✅ DMG 签名完成"

echo ""
echo "========================================"
echo "  签名和打包完成！"
echo "========================================"
echo "DMG: $DMG_PATH"
echo ""
echo "下一步（可选）- 公证:"
echo "  xcrun notarytool submit \"$DMG_PATH\" --apple-id \"$APPLE_ID\" --team-id \"$TEAM_ID\" --wait"
echo ""

# 询问是否公证
read -p "是否立即进行公证？(需要 Apple ID 密码) [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🔐 开始公证..."
    
    # 检查 notarytool 凭证
    if ! xcrun notarytool list-credentials 2>/dev/null | grep -q "$TEAM_ID"; then
        echo "⚠️ 需要配置公证凭证"
        echo "运行: xcrun notarytool store-credentials mofa-notary --apple-id \"$APPLE_ID\" --team-id \"$TEAM_ID\""
        read -p "按回车键配置凭证..."
        xcrun notarytool store-credentials mofa-notary --apple-id "$APPLE_ID" --team-id "$TEAM_ID"
    fi
    
    # 提交公证
    echo "提交公证请求..."
    xcrun notarytool submit "$DMG_PATH" --keychain-profile mofa-notary --wait
    
    # 验证公证
    echo "验证公证状态..."
    xcrun stapler staple "$DMG_PATH" 2>/dev/null || echo "⚠️ 无法 stapler，请手动检查"
    
    echo "✅ 公证流程完成"
fi

echo ""
echo "🎉 全部完成！"
echo "最终文件: $DMG_PATH"
ls -lh "$DMG_PATH"
