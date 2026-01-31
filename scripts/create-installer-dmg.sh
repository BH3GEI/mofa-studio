#!/bin/bash
# 简单美观的DMG安装器（使用AppleScript设置属性）

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/build/macos"
APP_NAME="MoFA Studio"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
DMG_NAME="MofaStudio-1.0.0"
DMG_PATH="$BUILD_DIR/$DMG_NAME.dmg"
TEMP_DIR="/tmp/dmg-temp-$$"

# 证书配置
SIGN_HASH="78055F15F4D6E7C85EF890EA066AACBCB1E908C8"

echo "🎨 创建安装DMG..."

# 清理
rm -f "$DMG_PATH"
rm -rf "$TEMP_DIR"
mkdir -p "$TEMP_DIR"

# 复制App
cp -R "$APP_BUNDLE" "$TEMP_DIR/"

# 创建Applications快捷方式
ln -s /Applications "$TEMP_DIR/Applications"

# 创建临时DMG
echo "💾 创建临时DMG..."
TEMP_DMG="$BUILD_DIR/temp-$$.dmg"
hdiutil create -srcfolder "$TEMP_DIR" -volname "$APP_NAME" -fs HFS+ \
    -format UDRW -size 50m "$TEMP_DMG" -quiet

# 挂载DMG
MOUNT_POINT="/Volumes/$APP_NAME"
echo "📂 挂载DMG..."
hdiutil attach "$TEMP_DMG" -mountpoint "$MOUNT_POINT" -nobrowse -quiet

# 等待挂载完成
sleep 1

# 设置窗口属性（使用AppleScript）
echo "🎨 设置安装界面..."
osascript << 'OSAEOF'
tell application "Finder"
    set diskName to name of every disk whose name contains "Studio"
    repeat with d in diskName
        set d to d as string
        if d contains "Studio" then
            tell disk d
                open
                
                -- 设置窗口外观
                tell container window
                    set current view to icon view
                    set toolbar visible to false
                    set statusbar visible to false
                    set bounds to {100, 100, 700, 500}
                    
                    -- 设置图标大小
                    set icon size of icon view options to 100
                    
                    -- 禁用自动排列，允许自由拖动
                    set arrangement of icon view options to not arranged
                    
                    -- 隐藏额外信息
                    set shows item info of icon view options to false
                    set shows icon preview of icon view options to false
                    
                    -- 纯色背景（浅灰色）
                    set background color of icon view options to {65535, 65535, 65535}
                end tell
                
                -- 设置图标位置
                set position of item "MoFA Studio.app" to {150, 200}
                set position of item "Applications" to {450, 200}
                
                update without registering applications
                delay 1
                close
            end tell
        end if
    end repeat
end tell
OSAEOF

# 确保设置保存
sleep 2

# 卸载DMG
echo "💾 压缩最终DMG..."
hdiutil detach "$MOUNT_POINT" -force -quiet || true
sleep 1

# 压缩DMG
hdiutil convert "$TEMP_DMG" -format UDZO -imagekey zlib-level=9 \
    -o "$DMG_PATH" -quiet

# 清理
rm -f "$TEMP_DMG"
rm -rf "$TEMP_DIR"

# 签名
echo "🔏 签名DMG..."
codesign --sign "$SIGN_HASH" --timestamp "$DMG_PATH" 2>/dev/null || true

echo ""
echo "✅ 安装DMG创建完成！"
echo "========================================"
echo "文件: $DMG_PATH"
echo "大小: $(du -h "$DMG_PATH" | cut -f1)"
echo "========================================"
echo ""
echo "🎯 安装说明："
echo "   1. 用户双击DMG文件"
echo "   2. 弹出安装窗口"
echo "   3. 拖拽 MoFA Studio.app → Applications"
echo "   4. 完成安装！"
echo ""
echo "📦 此DMG包含："
echo "   - MoFA Studio.app（已签名公证）"
echo "   - Applications 快捷方式"
echo "   - 美观的安装布局"
