#!/bin/bash
set -e

# MoFA Studio macOS Packaging Script
# 用法: ./scripts/package-macos.sh [--sign] [--notarize]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/build/macos"
APP_NAME="MoFA Studio"
BUNDLE_ID="org.mofa.studio"
VERSION="1.0.0"
PYTHON_VERSION="${PYTHON_VERSION:-3.11.8}"
PYTHON_EMBED_VERSION="${PYTHON_EMBED_VERSION:-3.11}"
DEFAULT_PYTHON_FRAMEWORK="/Library/Frameworks/Python.framework"
if [ ! -d "$DEFAULT_PYTHON_FRAMEWORK" ]; then
    if [ -d "/opt/homebrew/Frameworks/Python.framework" ]; then
        DEFAULT_PYTHON_FRAMEWORK="/opt/homebrew/Frameworks/Python.framework"
    elif [ -d "/usr/local/Frameworks/Python.framework" ]; then
        DEFAULT_PYTHON_FRAMEWORK="/usr/local/Frameworks/Python.framework"
    fi
fi
PYTHON_FRAMEWORK_SRC="${PYTHON_FRAMEWORK_SRC:-$DEFAULT_PYTHON_FRAMEWORK}"
PYTHON_PKG_URL_DEFAULT="https://www.python.org/ftp/python/${PYTHON_VERSION}/python-${PYTHON_VERSION}-macos11.pkg"
PYTHON_PKG_URL="${PYTHON_PKG_URL:-$PYTHON_PKG_URL_DEFAULT}"

# 签名身份
SIGN_IDENTITY="Developer ID Application: Yao Li (SX7GH8L8YB)"

# 公证凭据 (从环境变量或 Keychain 获取)
APPLE_ID="${APPLE_ID:-li@mofa.ai}"
TEAM_ID="SX7GH8L8YB"
# APP_PASSWORD 应从环境变量或 Keychain 获取

# 解析参数
DO_SIGN=false
DO_NOTARIZE=false
for arg in "$@"; do
    case $arg in
        --sign)
            DO_SIGN=true
            ;;
        --notarize)
            DO_NOTARIZE=true
            DO_SIGN=true  # 公证必须先签名
            ;;
    esac
done

echo "========================================"
echo "  MoFA Studio macOS Packager"
echo "========================================"
echo "Project:   $PROJECT_ROOT"
echo "Output:    $BUILD_DIR"
echo "Sign:      $DO_SIGN"
echo "Notarize:  $DO_NOTARIZE"
echo "Python:    $PYTHON_VERSION"
echo ""

# 清理旧构建
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"
TMPDIR="$BUILD_DIR/tmp"
mkdir -p "$TMPDIR"
export TMPDIR

# Step 1: 编译 Release
echo "[1/7] Building release binary..."
cd "$PROJECT_ROOT"
cargo build --release -p mofa-studio-shell

# Step 2: 创建 .icns 图标
echo "[2/7] Creating app icon..."
ICONSET_DIR="$BUILD_DIR/MofaStudio.iconset"
mkdir -p "$ICONSET_DIR"

LOGO_PATH="$PROJECT_ROOT/mofa-studio-shell/resources/mofa-logo.png"

# 生成各尺寸图标
sips -z 16 16     "$LOGO_PATH" --out "$ICONSET_DIR/icon_16x16.png" >/dev/null
sips -z 32 32     "$LOGO_PATH" --out "$ICONSET_DIR/icon_16x16@2x.png" >/dev/null
sips -z 32 32     "$LOGO_PATH" --out "$ICONSET_DIR/icon_32x32.png" >/dev/null
sips -z 64 64     "$LOGO_PATH" --out "$ICONSET_DIR/icon_32x32@2x.png" >/dev/null
sips -z 128 128   "$LOGO_PATH" --out "$ICONSET_DIR/icon_128x128.png" >/dev/null
sips -z 256 256   "$LOGO_PATH" --out "$ICONSET_DIR/icon_128x128@2x.png" >/dev/null
sips -z 256 256   "$LOGO_PATH" --out "$ICONSET_DIR/icon_256x256.png" >/dev/null
sips -z 512 512   "$LOGO_PATH" --out "$ICONSET_DIR/icon_256x256@2x.png" >/dev/null
sips -z 512 512   "$LOGO_PATH" --out "$ICONSET_DIR/icon_512x512.png" >/dev/null
sips -z 1024 1024 "$LOGO_PATH" --out "$ICONSET_DIR/icon_512x512@2x.png" >/dev/null

if ! iconutil -c icns "$ICONSET_DIR" -o "$BUILD_DIR/MofaStudio.icns"; then
    echo "   iconutil failed, generating icns via python..."
    if command -v python3 &> /dev/null; then
        if ! python3 - << PY
import struct
from pathlib import Path

iconset = Path("$ICONSET_DIR")
out_path = Path("$BUILD_DIR/MofaStudio.icns")

mapping = {
    "icp4": "icon_16x16.png",
    "icp5": "icon_32x32.png",
    "icp6": "icon_32x32@2x.png",
    "ic07": "icon_128x128.png",
    "ic08": "icon_256x256.png",
    "ic09": "icon_512x512.png",
    "ic10": "icon_512x512@2x.png",
}

chunks = []
for code, filename in mapping.items():
    data = (iconset / filename).read_bytes()
    length = 8 + len(data)
    chunks.append(code.encode("ascii") + struct.pack(">I", length) + data)

total_len = 8 + sum(len(c) for c in chunks)
with out_path.open("wb") as f:
    f.write(b"icns" + struct.pack(">I", total_len))
    for chunk in chunks:
        f.write(chunk)
PY
        then
            echo "   Python icns generation failed, using system generic icon."
            cp "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericApplicationIcon.icns" "$BUILD_DIR/MofaStudio.icns"
        fi
    else
        echo "   python3 not found, using system generic icon."
        cp "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericApplicationIcon.icns" "$BUILD_DIR/MofaStudio.icns"
    fi
fi
rm -rf "$ICONSET_DIR"

# Step 3: 创建 App Bundle 结构
echo "[3/7] Creating app bundle..."
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Step 4: 创建 Info.plist
echo "[4/7] Creating Info.plist..."
cat > "$APP_BUNDLE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleExecutable</key>
    <string>mofa-studio</string>
    <key>CFBundleIconFile</key>
    <string>MofaStudio</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>NSMicrophoneUsageDescription</key>
    <string>MoFA Studio needs microphone access for audio features.</string>
</dict>
</plist>
EOF

# 复制二进制和资源
cp "$PROJECT_ROOT/target/release/mofa-studio" "$APP_BUNDLE/Contents/MacOS/"
cp "$BUILD_DIR/MofaStudio.icns" "$APP_BUNDLE/Contents/Resources/"

# 复制资源文件 (字体、图标等)
cp -r "$PROJECT_ROOT/mofa-studio-shell/resources" "$APP_BUNDLE/Contents/Resources/"

# NOTE: Python apps removed for pure version (models downloaded on demand)
echo "   Skipping Python apps (pure version)"

# Step 5: Skip Python runtime and models (pure version)
echo "[5/7] Skipping Python runtime and models (pure version - models downloaded on demand)"

# Step 5: 代码签名 (可选)
if $DO_SIGN; then
    echo "[5/6] Code signing..."
    codesign --deep --force --options runtime \
        --sign "$SIGN_IDENTITY" \
        --entitlements /dev/stdin \
        "$APP_BUNDLE" << ENTITLEMENTS
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
</dict>
</plist>
ENTITLEMENTS
    echo "   Signed successfully!"
else
    echo "[5/6] Skipping code signing (use --sign to enable)"
fi

# Step 6: 创建 DMG
echo "[6/6] Creating DMG..."
DMG_PATH="$BUILD_DIR/MofaStudio-$VERSION.dmg"

# 检查是否有 create-dmg
if command -v create-dmg &> /dev/null; then
    create-dmg \
        --volname "$APP_NAME" \
        --volicon "$BUILD_DIR/MofaStudio.icns" \
        --window-size 600 400 \
        --icon-size 100 \
        --icon "$APP_NAME.app" 150 190 \
        --app-drop-link 450 190 \
        --hide-extension "$APP_NAME.app" \
        "$DMG_PATH" \
        "$APP_BUNDLE"
else
    echo "   (create-dmg not found, using hdiutil)"
    hdiutil create -volname "$APP_NAME" \
        -srcfolder "$APP_BUNDLE" \
        -ov -format UDZO \
        "$DMG_PATH"
fi

echo ""
echo "========================================"
echo "  Build Complete!"
echo "========================================"
echo "App Bundle: $APP_BUNDLE"
echo "DMG:        $DMG_PATH"
echo ""

# Step 8: 公证 (可选)
if $DO_NOTARIZE; then
    echo "[8/8] Notarizing..."

    if [ -z "$APP_PASSWORD" ]; then
        echo "Error: APP_PASSWORD environment variable not set"
        echo "Set it with: export APP_PASSWORD='your-app-specific-password'"
        echo ""
        echo "Or run notarization manually:"
        echo "  xcrun notarytool submit \"$DMG_PATH\" --apple-id \"$APPLE_ID\" --password YOUR_PASSWORD --team-id $TEAM_ID --wait"
        exit 1
    fi

    xcrun notarytool submit "$DMG_PATH" \
        --apple-id "$APPLE_ID" \
        --password "$APP_PASSWORD" \
        --team-id "$TEAM_ID" \
        --wait

    echo "   Stapling notarization ticket..."
    xcrun stapler staple "$DMG_PATH"

    echo "   Notarization complete!"
fi

echo ""
echo "To install create-dmg for prettier DMGs:"
echo "  brew install create-dmg"
echo ""
if ! $DO_SIGN; then
    echo "Note: App is unsigned. To sign, run:"
    echo "  ./scripts/package-macos.sh --sign"
fi
if $DO_SIGN && ! $DO_NOTARIZE; then
    echo "Note: App is signed but not notarized. To notarize, run:"
    echo "  APP_PASSWORD='your-app-password' ./scripts/package-macos.sh --notarize"
fi
