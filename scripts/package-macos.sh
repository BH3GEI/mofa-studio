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

# 复制 Python WebView 应用
echo "   Copying Python apps..."
APPS_DIR="$APP_BUNDLE/Contents/Resources/apps"
mkdir -p "$APPS_DIR"

# Hello World (python/ -> mofa-hello-world/)
if [ -d "$PROJECT_ROOT/apps/mofa-hello-world/python" ]; then
    cp -r "$PROJECT_ROOT/apps/mofa-hello-world/python" "$APPS_DIR/mofa-hello-world"
fi

# Note Taker (python/ -> mofa-note-taker/)
if [ -d "$PROJECT_ROOT/apps/mofa-note-taker/python" ]; then
    cp -r "$PROJECT_ROOT/apps/mofa-note-taker/python" "$APPS_DIR/mofa-note-taker"
fi

# Personal News (python/ -> mofa-personal-news/, keeps web/ subfolder)
if [ -d "$PROJECT_ROOT/apps/mofa-personal-news/python" ]; then
    cp -r "$PROJECT_ROOT/apps/mofa-personal-news/python" "$APPS_DIR/mofa-personal-news"
fi

# Transcriber (python/ -> mofa-transcriber/, keeps web/ subfolder)
if [ -d "$PROJECT_ROOT/apps/mofa-transcriber/python" ]; then
    cp -r "$PROJECT_ROOT/apps/mofa-transcriber/python" "$APPS_DIR/mofa-transcriber"
fi

# Podcast Factory (python/ -> mofa-podcast-factory/, keeps web/ subfolder)
if [ -d "$PROJECT_ROOT/apps/mofa-podcast-factory/python" ]; then
    cp -r "$PROJECT_ROOT/apps/mofa-podcast-factory/python" "$APPS_DIR/mofa-podcast-factory"
fi

# MoFA.fm Web (python/ -> mofa-fm-web/, keeps web/ subfolder)
if [ -d "$PROJECT_ROOT/apps/mofa-fm-web/python" ]; then
    cp -r "$PROJECT_ROOT/apps/mofa-fm-web/python" "$APPS_DIR/mofa-fm-web"
fi

# Step 5: 打包 Python 运行时与 Whisper tiny
echo "[5/7] Bundling Python runtime and Whisper tiny..."
PYTHON_BUNDLE="$APP_BUNDLE/Contents/Resources/python"
PYTHON_FRAMEWORK_DST="$PYTHON_BUNDLE/Python.framework"
PYTHON_SITE_PACKAGES="$PYTHON_BUNDLE/site-packages"
MODEL_DIR="$APP_BUNDLE/Contents/Resources/models/whisper"
CACHE_DIR="$PROJECT_ROOT/build/cache"
PYTHON_PKG="$CACHE_DIR/python-${PYTHON_VERSION}.pkg"
PYTHON_EXPAND="$CACHE_DIR/python-${PYTHON_VERSION}-expand"

mkdir -p "$PYTHON_BUNDLE"
mkdir -p "$CACHE_DIR"

if [ -d "$PYTHON_FRAMEWORK_SRC" ]; then
    echo "   Using local Python.framework: $PYTHON_FRAMEWORK_SRC"
    rm -rf "$PYTHON_FRAMEWORK_DST"
    rsync -aL "$PYTHON_FRAMEWORK_SRC/" "$PYTHON_FRAMEWORK_DST/"
else
    echo "   Python.framework not found at $PYTHON_FRAMEWORK_SRC"
    echo "   Downloading Python from: $PYTHON_PKG_URL"
    if [ ! -f "$PYTHON_PKG" ]; then
        curl -L "$PYTHON_PKG_URL" -o "$PYTHON_PKG"
    fi
    rm -rf "$PYTHON_EXPAND"
    pkgutil --expand "$PYTHON_PKG" "$PYTHON_EXPAND"
    PYTHON_PAYLOAD=$(find "$PYTHON_EXPAND" -name Payload -path "*Python_Framework.pkg*" | head -n 1)
    if [ -z "$PYTHON_PAYLOAD" ]; then
        echo "Error: Python framework payload not found in package."
        exit 1
    fi
    PYTHON_PAYLOAD_DIR="$PYTHON_EXPAND/payload"
    rm -rf "$PYTHON_PAYLOAD_DIR"
    mkdir -p "$PYTHON_PAYLOAD_DIR"
    bsdtar -xf "$PYTHON_PAYLOAD" -C "$PYTHON_PAYLOAD_DIR"
    if [ ! -d "$PYTHON_PAYLOAD_DIR/Library/Frameworks/Python.framework" ]; then
        echo "Error: Python.framework not found in extracted payload."
        exit 1
    fi
    rm -rf "$PYTHON_FRAMEWORK_DST"
    rsync -aL "$PYTHON_PAYLOAD_DIR/Library/Frameworks/Python.framework/" "$PYTHON_FRAMEWORK_DST/"
fi

mkdir -p "$PYTHON_BUNDLE/bin"
cat > "$PYTHON_BUNDLE/bin/python3" << 'EOF'
#!/bin/bash
set -e
PY_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PY_FRAMEWORK="$PY_ROOT/Python.framework/Versions/__PY_EMBED_VERSION__"
if [ ! -x "$PY_FRAMEWORK/bin/python3" ]; then
  PY_FRAMEWORK="$PY_ROOT/Python.framework/Versions/Current"
fi
if [ ! -x "$PY_FRAMEWORK/bin/python3" ]; then
  for v in "$PY_ROOT/Python.framework/Versions/"*; do
    if [ -x "$v/bin/python3" ]; then
      PY_FRAMEWORK="$v"
      break
    fi
  done
fi
export PYTHONHOME="$PY_FRAMEWORK"
PY_SITE="$PY_ROOT/site-packages"
if [ -d "$PY_SITE" ]; then
  if [ -n "$PYTHONPATH" ]; then
    export PYTHONPATH="$PY_SITE:$PYTHONPATH"
  else
    export PYTHONPATH="$PY_SITE"
  fi
fi
if [ -z "$WHISPER_MODEL_DIR" ]; then
  CANDIDATE="$PY_ROOT/../models/whisper"
  if [ -d "$CANDIDATE" ]; then
    export WHISPER_MODEL_DIR="$CANDIDATE"
  fi
fi
exec "$PY_FRAMEWORK/bin/python3" "$@"
EOF
sed -i '' "s/__PY_EMBED_VERSION__/${PYTHON_EMBED_VERSION}/g" "$PYTHON_BUNDLE/bin/python3"
chmod +x "$PYTHON_BUNDLE/bin/python3"

PYTHON_BIN="$PYTHON_BUNDLE/bin/python3"
if [ ! -x "$PYTHON_BIN" ]; then
    echo "Error: Embedded python binary not found at $PYTHON_BIN"
    exit 1
fi

echo "   Installing Python packages..."
mkdir -p "$PYTHON_SITE_PACKAGES"
PIP_CACHE_DIR="$CACHE_DIR/pip"
mkdir -p "$PIP_CACHE_DIR"
VENDOR_SITE_PACKAGES_DEFAULT="$HOME/.mofa/venv_cache/base_venv/lib/python${PYTHON_EMBED_VERSION}/site-packages"
VENDOR_SITE_PACKAGES="${VENDOR_SITE_PACKAGES:-$VENDOR_SITE_PACKAGES_DEFAULT}"
EXTRA_SITE_PACKAGES_DEFAULT="/opt/homebrew/lib/python${PYTHON_EMBED_VERSION}/site-packages"
EXTRA_SITE_PACKAGES="${EXTRA_SITE_PACKAGES:-$EXTRA_SITE_PACKAGES_DEFAULT}"

if [ "${MOFA_OFFLINE:-0}" = "1" ]; then
    PIP_STATUS=1
else
    set +e
    PIP_DISABLE_PIP_VERSION_CHECK=1 PIP_BREAK_SYSTEM_PACKAGES=1 PIP_CACHE_DIR="$PIP_CACHE_DIR" \
        "$PYTHON_BIN" -m pip install \
        --target "$PYTHON_SITE_PACKAGES" \
        -r "$PROJECT_ROOT/apps/mofa-transcriber/python/requirements.txt" \
        requests
    PIP_STATUS=$?
    set -e
fi

if [ $PIP_STATUS -ne 0 ]; then
    echo "   pip failed, falling back to vendored site-packages..."
    if [ -d "$VENDOR_SITE_PACKAGES" ]; then
        rsync -a "$VENDOR_SITE_PACKAGES/" "$PYTHON_SITE_PACKAGES/"
    else
        echo "Error: vendored site-packages not found at $VENDOR_SITE_PACKAGES"
        exit 1
    fi

    if [ -d "$EXTRA_SITE_PACKAGES" ]; then
        shopt -s nullglob
        for pkg in \
            faster_whisper faster_whisper-*.dist-info \
            ctranslate2 ctranslate2-*.dist-info; do
            if [ -e "$EXTRA_SITE_PACKAGES/$pkg" ] && [ ! -e "$PYTHON_SITE_PACKAGES/$pkg" ]; then
                rsync -aL "$EXTRA_SITE_PACKAGES/$pkg" "$PYTHON_SITE_PACKAGES/"
            fi
        done
        shopt -u nullglob
    fi
fi

mkdir -p "$MODEL_DIR"
if [ -d "$MODEL_DIR/faster-whisper-tiny" ]; then
    echo "   Whisper tiny already exists, skipping download."
else
    HF_TINY_CACHE_DEFAULT="$HOME/.cache/huggingface/hub/models--Systran--faster-whisper-tiny/snapshots"
    HF_TINY_CACHE="${HF_TINY_CACHE:-$HF_TINY_CACHE_DEFAULT}"
    SNAPSHOT_DIR=""
    if [ -d "$HF_TINY_CACHE" ]; then
        SNAPSHOT_DIR="$(ls -1 "$HF_TINY_CACHE" | head -n 1)"
    fi
    if [ -n "$SNAPSHOT_DIR" ] && [ -d "$HF_TINY_CACHE/$SNAPSHOT_DIR" ]; then
        echo "   Copying Whisper tiny from HuggingFace cache..."
        rsync -aL "$HF_TINY_CACHE/$SNAPSHOT_DIR/" "$MODEL_DIR/faster-whisper-tiny/"
    else
        echo "   Downloading Whisper tiny model..."
        WHISPER_MODEL_DIR="$MODEL_DIR" PIP_DISABLE_PIP_VERSION_CHECK=1 PYTHONPATH="$PYTHON_SITE_PACKAGES" "$PYTHON_BIN" - << 'PY'
import os
from faster_whisper import WhisperModel

model_dir = os.environ.get("WHISPER_MODEL_DIR")
WhisperModel("tiny", device="cpu", compute_type="int8", download_root=model_dir)
print(f"Whisper tiny downloaded to {model_dir}")
PY
    fi
fi

# Step 6: 代码签名 (可选)
if $DO_SIGN; then
    echo "[6/7] Code signing..."
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
    echo "[6/7] Skipping code signing (use --sign to enable)"
fi

# Step 7: 创建 DMG
echo "[7/7] Creating DMG..."
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
