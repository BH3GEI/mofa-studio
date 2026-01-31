#!/bin/bash
set -e
set -o pipefail

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
PYTHON_FRAMEWORK_SRC="${PYTHON_FRAMEWORK_SRC:-}"
PYTHON_FRAMEWORK_SRC_VERSION=""
ALLOW_NON_311="${ALLOW_NON_311:-0}"

pick_python_framework() {
    local candidate
    local ver
    local selected=""
    local selected_ver=""
    local best=""
    local best_ver=""
    local brew_prefix=""
    if command -v brew >/dev/null 2>&1; then
        brew_prefix="$(brew --prefix python@3.11 2>/dev/null || true)"
    fi

    local candidates=(
        "/Library/Frameworks/Python.framework"
        "${brew_prefix:+$brew_prefix/Frameworks/Python.framework}"
        "/opt/homebrew/opt/python@3.11/Frameworks/Python.framework"
        "/usr/local/opt/python@3.11/Frameworks/Python.framework"
        "/opt/homebrew/Frameworks/Python.framework"
        "/usr/local/Frameworks/Python.framework"
    )

    for candidate in "${candidates[@]}"; do
        local cand_bin=""
        if [ -x "$candidate/Versions/Current/bin/python3" ]; then
            cand_bin="$candidate/Versions/Current/bin/python3"
        else
            for b in "$candidate/Versions"/*/bin/python3*; do
                if [ -x "$b" ]; then
                    cand_bin="$b"
                    break
                fi
            done
        fi
        if [ -n "$cand_bin" ]; then
            ver="$("$cand_bin" -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')"
            if [ "$ver" = "3.11" ]; then
                selected="$candidate"
                selected_ver="$ver"
                break
            elif [ -z "$best" ]; then
                best="$candidate"
                best_ver="$ver"
            fi
        fi
    done

    if [ -n "$selected" ]; then
        PYTHON_FRAMEWORK_SRC="$selected"
        PYTHON_FRAMEWORK_SRC_VERSION="$selected_ver"
    elif [ -n "$best" ]; then
        PYTHON_FRAMEWORK_SRC="$best"
        PYTHON_FRAMEWORK_SRC_VERSION="$best_ver"
    fi
}

if [ -z "$PYTHON_FRAMEWORK_SRC" ]; then
    pick_python_framework
fi

if [ -n "$PYTHON_FRAMEWORK_SRC" ] && [ "$ALLOW_NON_311" != "1" ]; then
    if [ "$PYTHON_FRAMEWORK_SRC_VERSION" != "3.11" ]; then
        echo "❌ Python 3.11 framework required. Found: ${PYTHON_FRAMEWORK_SRC_VERSION:-unknown}"
        echo "   Install python@3.11 or set PYTHON_FRAMEWORK_SRC to a 3.11 framework."
        exit 1
    fi
fi
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
if [ -n "$PYTHON_FRAMEWORK_SRC" ]; then
    echo "Python.framework: $PYTHON_FRAMEWORK_SRC (${PYTHON_FRAMEWORK_SRC_VERSION:-unknown})"
else
    echo "Python.framework: (not found)"
fi
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

# Step 5: 打包Python运行时（简化版）
echo "[5/7] Bundling Python runtime..."
echo "   Python framework source: $PYTHON_FRAMEWORK_SRC"
PYTHON_BUNDLE="$APP_BUNDLE/Contents/Resources/python"
PYTHON_FRAMEWORK_DST="$PYTHON_BUNDLE/Python.framework"
mkdir -p "$PYTHON_BUNDLE"

# 检查本地Python framework
if [ -d "$PYTHON_FRAMEWORK_SRC" ]; then
    echo "   ✅ Found Python.framework: $PYTHON_FRAMEWORK_SRC"
    echo "   Copying Python framework..."
    
    # 复制Python framework（包含基本解释器，排除大型site-packages）
    rsync -aL "$PYTHON_FRAMEWORK_SRC/" "$PYTHON_FRAMEWORK_DST/" \
        --exclude='lib/python*/site-packages/*' \
        --exclude='lib/python*/test/*' \
        --exclude='share/*' \
        --exclude='include/*' 2>&1 | head -5 || {
        echo "   ⚠️ rsync failed, trying cp -R..."
        cp -R "$PYTHON_FRAMEWORK_SRC" "$PYTHON_FRAMEWORK_DST"
    }
    
    # 显示复制后的大小
    echo "   Python framework size: $(du -sh "$PYTHON_FRAMEWORK_DST" | cut -f1)"

    PYTHON_BIN="$PYTHON_FRAMEWORK_DST/Versions/Current/bin/python3"
    if [ ! -x "$PYTHON_BIN" ]; then
        for candidate in "$PYTHON_FRAMEWORK_DST/Versions"/*/bin/python3*; do
            if [ -x "$candidate" ]; then
                PYTHON_BIN="$candidate"
                break
            fi
        done
    fi
    if [ ! -x "$PYTHON_BIN" ]; then
        echo "   ❌ Python binary not found in embedded framework"
        exit 1
    fi
    PYTHON_EMBED_VERSION="$("$PYTHON_BIN" -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')"
    if [ -z "$PYTHON_EMBED_VERSION" ]; then
        echo "   ❌ Failed to detect embedded Python version"
        exit 1
    fi
    if [ ! -e "$PYTHON_FRAMEWORK_DST/Versions/Current" ]; then
        ln -s "$PYTHON_EMBED_VERSION" "$PYTHON_FRAMEWORK_DST/Versions/Current"
    fi
    PYTHON_HOME="$PYTHON_FRAMEWORK_DST/Versions/Current"
    PYTHON_CURRENT_BIN="$PYTHON_HOME/bin"
    if [ ! -x "$PYTHON_CURRENT_BIN/python3" ]; then
        if [ -x "$PYTHON_CURRENT_BIN/python$PYTHON_EMBED_VERSION" ]; then
            ln -sf "python$PYTHON_EMBED_VERSION" "$PYTHON_CURRENT_BIN/python3"
        else
            for cand in "$PYTHON_CURRENT_BIN"/python3.*; do
                if [ -x "$cand" ]; then
                    ln -sf "$(basename "$cand")" "$PYTHON_CURRENT_BIN/python3"
                    break
                fi
            done
        fi
    fi
    PYTHON_BIN="$PYTHON_CURRENT_BIN/python3"

    # 创建Python启动脚本（设置PYTHONPATH）
    echo "   Creating Python launcher..."
    mkdir -p "$PYTHON_BUNDLE/bin"
    cat > "$PYTHON_BUNDLE/bin/python3" << 'PYEOF'
#!/bin/bash
# Python launcher with correct PYTHONPATH
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PY_FRAMEWORK="$(dirname "$SCRIPT_DIR")/Python.framework"
PY_VERSION="__PY_VERSION__"
PY_BIN="$PY_FRAMEWORK/Versions/Current/bin/python3"
SITE_PACKAGES="$PY_FRAMEWORK/Versions/Current/lib/python$PY_VERSION/site-packages"
export PYTHONHOME="$PY_FRAMEWORK/Versions/Current"
export PYTHONPATH="$SITE_PACKAGES:${PYTHONPATH:-}"
exec "$PY_BIN" "$@"
PYEOF
    sed -i '' "s/__PY_VERSION__/$PYTHON_EMBED_VERSION/g" "$PYTHON_BUNDLE/bin/python3"
    chmod +x "$PYTHON_BUNDLE/bin/python3"
    
    # 创建python命令的软链接
    ln -sf python3 "$PYTHON_BUNDLE/bin/python" 2>/dev/null || true

    # 准备 embedded Python 的 site-packages 与 pip
    SITE_PACKAGES="$PYTHON_HOME/lib/python$PYTHON_EMBED_VERSION/site-packages"
    mkdir -p "$SITE_PACKAGES"
    EXTERNALLY_MANAGED="$PYTHON_HOME/lib/python$PYTHON_EMBED_VERSION/EXTERNALLY-MANAGED"
    if [ -f "$EXTERNALLY_MANAGED" ]; then
        echo "   Removing EXTERNALLY-MANAGED marker for embedded Python..."
        rm -f "$EXTERNALLY_MANAGED"
    fi
    echo "   Bootstrapping pip in embedded Python..."
    if ! PYTHONHOME="$PYTHON_HOME" PYTHONPATH="$SITE_PACKAGES" \
        PIP_BREAK_SYSTEM_PACKAGES=1 \
        "$PYTHON_BIN" -m ensurepip --upgrade; then
        echo "   ❌ ensurepip failed for embedded Python"
        exit 1
    fi
    if ! PYTHONHOME="$PYTHON_HOME" PYTHONPATH="$SITE_PACKAGES" \
        PIP_BREAK_SYSTEM_PACKAGES=1 \
        "$PYTHON_BIN" -m pip --version >/dev/null 2>&1; then
        echo "   ❌ pip not available in embedded Python"
        exit 1
    fi
    
else
    echo "   ⚠️ Python.framework not found at $PYTHON_FRAMEWORK_SRC"
    echo "   Install with: brew install python@3.11"
    echo "   Continuing without embedded Python..."
fi

# 复制Python应用到Resources
echo "   Copying Python apps..."
APPS_DIR="$APP_BUNDLE/Contents/Resources/apps"
mkdir -p "$APPS_DIR"
APPS_COPIED=0

# Transcriber (faster-whisper)
if [ -d "$PROJECT_ROOT/apps/mofa-transcriber/python" ]; then
    echo "     - mofa-transcriber"
    cp -r "$PROJECT_ROOT/apps/mofa-transcriber/python" "$APPS_DIR/mofa-transcriber"
    APPS_COPIED=$((APPS_COPIED + 1))
    
    # Install dependencies
    if [ -f "$APPS_DIR/mofa-transcriber/requirements.txt" ]; then
        echo "       Installing transcriber dependencies..."
        if [ -x "$PYTHON_BIN" ]; then
            PYTHONHOME="$PYTHON_HOME" PYTHONPATH="$SITE_PACKAGES" PIP_BREAK_SYSTEM_PACKAGES=1 \
            "$PYTHON_BIN" -m pip install -r "$APPS_DIR/mofa-transcriber/requirements.txt" \
                --target "$SITE_PACKAGES" --quiet 2>&1 | head -3 || echo "       ⚠️ Some dependencies may have failed"
        fi
    fi
fi

# Personal News
if [ -d "$PROJECT_ROOT/apps/mofa-personal-news/python" ]; then
    echo "     - mofa-personal-news"
    cp -r "$PROJECT_ROOT/apps/mofa-personal-news/python" "$APPS_DIR/mofa-personal-news"
    APPS_COPIED=$((APPS_COPIED + 1))
fi

# Podcast Factory
if [ -d "$PROJECT_ROOT/apps/mofa-podcast-factory/python" ]; then
    echo "     - mofa-podcast-factory"
    cp -r "$PROJECT_ROOT/apps/mofa-podcast-factory/python" "$APPS_DIR/mofa-podcast-factory"
    APPS_COPIED=$((APPS_COPIED + 1))
fi

# MoFA.fm Web
if [ -d "$PROJECT_ROOT/apps/mofa-fm-web/python" ]; then
    echo "     - mofa-fm-web"
    cp -r "$PROJECT_ROOT/apps/mofa-fm-web/python" "$APPS_DIR/mofa-fm-web"
    APPS_COPIED=$((APPS_COPIED + 1))
fi

# Hello World
if [ -d "$PROJECT_ROOT/apps/mofa-hello-world/python" ]; then
    echo "     - mofa-hello-world"
    cp -r "$PROJECT_ROOT/apps/mofa-hello-world/python" "$APPS_DIR/mofa-hello-world"
    APPS_COPIED=$((APPS_COPIED + 1))
fi

echo "   ✅ Copied $APPS_COPIED Python apps"

# 安装Python依赖
echo "   Installing Python dependencies..."
PYTHON_BIN="${PYTHON_BIN:-}"
SITE_PACKAGES="${SITE_PACKAGES:-}"
PYTHON_HOME="${PYTHON_HOME:-}"

if [ -x "$PYTHON_BIN" ]; then
    # 基础依赖（所有应用共用）
    echo "       Installing base dependencies (requests, fastapi, uvicorn)..."
    PYTHONHOME="$PYTHON_HOME" PYTHONPATH="$SITE_PACKAGES" PIP_BREAK_SYSTEM_PACKAGES=1 \
    "$PYTHON_BIN" -m pip install requests fastapi uvicorn pydub \
        --target "$SITE_PACKAGES" --quiet 2>&1 | tail -3 || true
    
    # 逐个安装应用特定依赖
    for app_dir in "$APPS_DIR"/*/; do
        app_name=$(basename "$app_dir")
        
        # 如果有requirements.txt，优先使用
        if [ -f "$app_dir/requirements.txt" ]; then
            echo "       Installing $app_name from requirements.txt..."
            PYTHONHOME="$PYTHON_HOME" PYTHONPATH="$SITE_PACKAGES" PIP_BREAK_SYSTEM_PACKAGES=1 \
            "$PYTHON_BIN" -m pip install -r "$app_dir/requirements.txt" \
                --target "$SITE_PACKAGES" --quiet 2>&1 | tail -3 || true
        else
            # 根据应用名称安装特定依赖
            case "$app_name" in
                "mofa-transcriber")
                    echo "       Installing $app_name (faster-whisper, openai)..."
                    PYTHONHOME="$PYTHON_HOME" PYTHONPATH="$SITE_PACKAGES" PIP_BREAK_SYSTEM_PACKAGES=1 \
                    "$PYTHON_BIN" -m pip install faster-whisper openai \
                        --target "$SITE_PACKAGES" --quiet 2>&1 | tail -3 || true
                    ;;
                "mofa-personal-news")
                    echo "       Installing $app_name (requests, feedparser)..."
                    PYTHONHOME="$PYTHON_HOME" PYTHONPATH="$SITE_PACKAGES" PIP_BREAK_SYSTEM_PACKAGES=1 \
                    "$PYTHON_BIN" -m pip install requests feedparser \
                        --target "$SITE_PACKAGES" --quiet 2>&1 | tail -3 || true
                    ;;
                "mofa-hello-world")
                    echo "       $app_name uses stdlib only"
                    ;;
                *)
                    echo "       Installing $app_name (standard packages)..."
                    PYTHONHOME="$PYTHON_HOME" PYTHONPATH="$SITE_PACKAGES" PIP_BREAK_SYSTEM_PACKAGES=1 \
                    "$PYTHON_BIN" -m pip install requests \
                        --target "$SITE_PACKAGES" --quiet 2>&1 | tail -3 || true
                    ;;
            esac
        fi
    done
    
    # 显示site-packages大小
    echo "   Python packages size: $(du -sh "$SITE_PACKAGES" | cut -f1)"
    echo "   ✅ Dependencies installed"

    echo "   Verifying embedded Python packages..."
    PYTHONHOME="$PYTHON_HOME" PYTHONPATH="$SITE_PACKAGES" \
    "$PYTHON_BIN" - << 'PY'
import importlib.util
missing = [m for m in ("openai", "faster_whisper") if importlib.util.find_spec(m) is None]
if missing:
    raise SystemExit("Missing packages in embedded Python: " + ", ".join(missing))
print("Embedded Python packages OK")
PY
else
    echo "   ⚠️ Python binary not found: $PYTHON_BIN"
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
