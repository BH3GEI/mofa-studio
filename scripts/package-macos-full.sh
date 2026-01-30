#!/usr/bin/env bash
set -euo pipefail

# MoFA Studio macOS Full Bundle Packager
# Strategy: ship the full repo inside the app bundle, extract to a writable
# app support directory on first run, and run the bundled binaries/tools.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/build/macos-full"
APP_NAME="MoFA Studio"
BUNDLE_ID="org.mofa.studio"

PYTHON_VERSION="${PYTHON_VERSION:-3.11.8}"
PYTHON_EMBED_VERSION="${PYTHON_EMBED_VERSION:-}"
PYTHON_PKG_URL_DEFAULT="https://www.python.org/ftp/python/${PYTHON_VERSION}/python-${PYTHON_VERSION}-macos11.pkg"
PYTHON_PKG_URL="${PYTHON_PKG_URL:-$PYTHON_PKG_URL_DEFAULT}"

TARGET_DORA_CLI_VERSION="${MOFA_DORA_CLI_VERSION:-0.4.1}"
TARGET_DORA_RS_VERSION="${MOFA_DORA_RS_VERSION:-0.4.1}"

# Signing / notarization (optional)
SIGN_IDENTITY="${SIGN_IDENTITY:-Developer ID Application: Your Name (TEAMID)}"
APPLE_ID="${APPLE_ID:-}"
TEAM_ID="${TEAM_ID:-}"

DO_SIGN=false
DO_NOTARIZE=false
for arg in "$@"; do
  case "$arg" in
    --sign) DO_SIGN=true ;;
    --notarize) DO_NOTARIZE=true; DO_SIGN=true ;;
  esac
done

version_from_cargo() {
  local shell_toml="$PROJECT_ROOT/mofa-studio-shell/Cargo.toml"
  local root_toml="$PROJECT_ROOT/Cargo.toml"
  local ver=""
  if [ -f "$shell_toml" ]; then
    ver="$(rg -n '^version\s*=' "$shell_toml" 2>/dev/null | head -n 1 | awk -F'"' '{print $2}' || true)"
  fi
  if [ -z "$ver" ] && [ -f "$root_toml" ]; then
    ver="$(rg -n '^version\s*=' "$root_toml" 2>/dev/null | head -n 1 | awk -F'"' '{print $2}' || true)"
  fi
  if [ -z "$ver" ]; then
    ver="0.0.0"
  fi
  echo "$ver"
}

VERSION="$(version_from_cargo)"

echo "========================================"
echo "  MoFA Studio Full Bundle Packager"
echo "========================================"
echo "Project:   $PROJECT_ROOT"
echo "Output:    $BUILD_DIR"
echo "Version:   $VERSION"
echo "Sign:      $DO_SIGN"
echo "Notarize:  $DO_NOTARIZE"
echo "Python:    $PYTHON_VERSION"
echo ""

if [ -d "$BUILD_DIR" ]; then
  rm -rf "$BUILD_DIR"/* "$BUILD_DIR"/.[!.]* "$BUILD_DIR"/..?* 2>/dev/null || true
fi
mkdir -p "$BUILD_DIR"
TMPDIR="$BUILD_DIR/tmp"
mkdir -p "$TMPDIR"
export TMPDIR

APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
APP_CONTENTS="$APP_BUNDLE/Contents"
APP_RES="$APP_CONTENTS/Resources"
APP_MACOS="$APP_CONTENTS/MacOS"

mkdir -p "$APP_RES" "$APP_MACOS"

echo "[1/9] Building Rust binaries..."
cd "$PROJECT_ROOT"
cargo build --release --bin mofa-studio
for manifest in \
  "node-hub/dora-conference-bridge/Cargo.toml" \
  "node-hub/dora-conference-controller/Cargo.toml" \
  "node-hub/dora-maas-client/Cargo.toml"
do
  if [ -f "$manifest" ]; then
    cargo build --release --manifest-path "$manifest"
  fi
done

echo "[2/9] Building dora-cli..."
TOOLS_DIR="$BUILD_DIR/tools"
mkdir -p "$TOOLS_DIR"
cargo install --locked dora-cli --version "$TARGET_DORA_CLI_VERSION" --root "$TOOLS_DIR"

echo "[3/9] Creating app icon..."
ICONSET_DIR="$BUILD_DIR/MofaStudio.iconset"
mkdir -p "$ICONSET_DIR"
LOGO_PATH="$PROJECT_ROOT/mofa-studio-shell/resources/mofa-logo.png"
if [ -f "$LOGO_PATH" ]; then
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
  iconutil -c icns "$ICONSET_DIR" -o "$BUILD_DIR/MofaStudio.icns" || true
fi
if [ ! -f "$BUILD_DIR/MofaStudio.icns" ]; then
  cp "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericApplicationIcon.icns" \
    "$BUILD_DIR/MofaStudio.icns"
fi

echo "[4/9] Writing Info.plist..."
cat > "$APP_CONTENTS/Info.plist" << EOF
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
  <string>MoFAStudio</string>
  <key>CFBundleIconFile</key>
  <string>MofaStudio.icns</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSMicrophoneUsageDescription</key>
  <string>MoFA Studio needs microphone access for real-time voice chat.</string>
</dict>
</plist>
EOF

mkdir -p "$APP_RES"
cp "$BUILD_DIR/MofaStudio.icns" "$APP_RES/"

echo "[5/9] Embedding full repo..."
REPO_DST="$APP_RES/repo"
mkdir -p "$REPO_DST"
rsync -a \
  --exclude ".git" \
  --exclude ".nix-mofa" \
  --exclude ".venv-mofa" \
  --exclude "build" \
  --exclude "target/debug" \
  --exclude "target/tmp" \
  --exclude "apps/**/dataflow/out" \
  "$PROJECT_ROOT/" "$REPO_DST/"
echo "$VERSION" > "$APP_RES/version.txt"

echo "[6/9] Embedding Python..."
PYTHON_BUNDLE="$APP_RES/python"
PYTHON_FRAMEWORK_DST="$PYTHON_BUNDLE/Python.framework"
PYTHON_SITE_PACKAGES="$PYTHON_BUNDLE/site-packages"
mkdir -p "$PYTHON_BUNDLE" "$PYTHON_SITE_PACKAGES"

DEFAULT_PYTHON_FRAMEWORK="/Library/Frameworks/Python.framework"
PYTHON_FRAMEWORK_SRC="${PYTHON_FRAMEWORK_SRC:-$DEFAULT_PYTHON_FRAMEWORK}"

if [ -d "$PYTHON_FRAMEWORK_SRC" ]; then
  mkdir -p "$PYTHON_FRAMEWORK_DST"
  rsync -aL "$PYTHON_FRAMEWORK_SRC/" "$PYTHON_FRAMEWORK_DST/"
else
  PYTHON_PKG="$BUILD_DIR/python-${PYTHON_VERSION}.pkg"
  PYTHON_EXPAND="$BUILD_DIR/python-pkg"
  PYTHON_PAYLOAD_DIR="$BUILD_DIR/python-payload"
  if [ ! -f "$PYTHON_PKG" ]; then
    echo "Downloading Python pkg: $PYTHON_PKG_URL"
    curl -L "$PYTHON_PKG_URL" -o "$PYTHON_PKG"
  fi
  rm -rf "$PYTHON_EXPAND" "$PYTHON_PAYLOAD_DIR"
  pkgutil --expand "$PYTHON_PKG" "$PYTHON_EXPAND"
  PYTHON_PAYLOAD=$(find "$PYTHON_EXPAND" -name Payload -path "*Python_Framework.pkg*" | head -n 1)
  mkdir -p "$PYTHON_PAYLOAD_DIR"
  tar -xf "$PYTHON_PAYLOAD" -C "$PYTHON_PAYLOAD_DIR" || true
  mkdir -p "$PYTHON_FRAMEWORK_DST"
  if [ -d "$PYTHON_PAYLOAD_DIR/Library/Frameworks/Python.framework" ]; then
    rsync -a "$PYTHON_PAYLOAD_DIR/Library/Frameworks/Python.framework/" "$PYTHON_FRAMEWORK_DST/"
  else
    rsync -a "$PYTHON_PAYLOAD_DIR/" "$PYTHON_FRAMEWORK_DST/"
  fi
fi

if [ -z "$PYTHON_EMBED_VERSION" ] && [ -d "$PYTHON_FRAMEWORK_DST/Versions" ]; then
  PYTHON_EMBED_VERSION="$(ls -1 "$PYTHON_FRAMEWORK_DST/Versions" | grep -v '^Current$' | sort -V | tail -n 1)"
fi
if [ -z "$PYTHON_EMBED_VERSION" ]; then
  PYTHON_EMBED_VERSION="$(echo "$PYTHON_VERSION" | awk -F. '{print $1 "." $2}')"
fi

mkdir -p "$PYTHON_BUNDLE/bin"
cat > "$PYTHON_BUNDLE/bin/python3" << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
export PYTHONHOME="$HERE/Python.framework/Versions/__PY_EMBED_VERSION__"
export PYTHONPATH="$HERE/site-packages"
export DYLD_LIBRARY_PATH="$PYTHONHOME:$PYTHONHOME/lib:${DYLD_LIBRARY_PATH:-}"
exec "$PYTHONHOME/bin/python3" "$@"
EOF
sed -i '' "s/__PY_EMBED_VERSION__/${PYTHON_EMBED_VERSION}/g" "$PYTHON_BUNDLE/bin/python3"
chmod +x "$PYTHON_BUNDLE/bin/python3"

PYTHON_BIN="$PYTHON_BUNDLE/bin/python3"
PIP_ENV="PYTHONPATH=$PYTHON_SITE_PACKAGES"

echo "Patching embedded Python loader paths..."
PY_VER_DIR="$PYTHON_FRAMEWORK_DST/Versions/$PYTHON_EMBED_VERSION"
PY_LIB_ORIG="/Library/Frameworks/Python.framework/Versions/$PYTHON_EMBED_VERSION/Python"
if [ -f "$PY_VER_DIR/Python" ] && command -v install_name_tool >/dev/null; then
  install_name_tool -id "@rpath/Python" "$PY_VER_DIR/Python" || true
  for exe in "$PY_VER_DIR/bin/python3" "$PY_VER_DIR/bin/python3."* "$PY_VER_DIR/bin/python3-intel64" "$PY_VER_DIR/bin/python3."*-intel64; do
    [ -f "$exe" ] || continue
    if file "$exe" | grep -q "Mach-O"; then
      install_name_tool -change "$PY_LIB_ORIG" "@loader_path/../Python" "$exe" || true
    fi
  done
  if [ -f "$PY_VER_DIR/Resources/Python.app/Contents/MacOS/Python" ]; then
    if file "$PY_VER_DIR/Resources/Python.app/Contents/MacOS/Python" | grep -q "Mach-O"; then
      install_name_tool -change "$PY_LIB_ORIG" "@loader_path/../../../../Python" \
        "$PY_VER_DIR/Resources/Python.app/Contents/MacOS/Python" || true
    fi
  fi
fi
chmod -R u+w "$PYTHON_FRAMEWORK_DST" 2>/dev/null || true
/usr/bin/xattr -dr com.apple.quarantine "$PYTHON_FRAMEWORK_DST" 2>/dev/null || true
if command -v codesign >/dev/null; then
  if [ -f "$PY_VER_DIR/Python" ]; then
    codesign --force --sign - "$PY_VER_DIR/Python" || true
  fi
  for exe in "$PY_VER_DIR/bin/python3" "$PY_VER_DIR/bin/python3."* "$PY_VER_DIR/bin/python3-intel64" "$PY_VER_DIR/bin/python3."*-intel64; do
    [ -f "$exe" ] || continue
    if file "$exe" | grep -q "Mach-O"; then
      codesign --force --sign - "$exe" || true
    fi
  done
  if [ -f "$PY_VER_DIR/Resources/Python.app/Contents/MacOS/Python" ]; then
    if file "$PY_VER_DIR/Resources/Python.app/Contents/MacOS/Python" | grep -q "Mach-O"; then
      codesign --force --sign - "$PY_VER_DIR/Resources/Python.app/Contents/MacOS/Python" || true
    fi
  fi
fi

echo "[7/9] Installing Python dependencies..."
PIP_BREAK_SYSTEM_PACKAGES=1 "$PYTHON_BIN" -m ensurepip --upgrade || true
PIP_BREAK_SYSTEM_PACKAGES=1 "$PYTHON_BIN" -m pip install --upgrade pip wheel setuptools
PIP_BREAK_SYSTEM_PACKAGES=1 "$PYTHON_BIN" -m pip install --no-cache-dir --target "$PYTHON_SITE_PACKAGES" \
  "dora-rs==$TARGET_DORA_RS_VERSION" \
  "$PROJECT_ROOT/libs/dora-common" \
  "$PROJECT_ROOT/node-hub/dora-text-segmenter" \
  "$PROJECT_ROOT/node-hub/dora-primespeech" \
  "$PROJECT_ROOT/node-hub/dora-asr"

# Ensure package data for MoYoYo TTS is bundled (pip may skip non-code assets)
MOYOYO_SRC="$PROJECT_ROOT/node-hub/dora-primespeech/dora_primespeech/moyoyo_tts"
MOYOYO_DST="$PYTHON_SITE_PACKAGES/dora_primespeech/moyoyo_tts"
if [ -d "$MOYOYO_SRC" ]; then
  mkdir -p "$MOYOYO_DST"
  rsync -a "$MOYOYO_SRC/" "$MOYOYO_DST/"
fi

echo "[8/9] Creating tool wrappers..."
APP_BIN="$APP_RES/bin"
mkdir -p "$APP_BIN"
cp "$TOOLS_DIR/bin/dora" "$APP_BIN/dora"
chmod +x "$APP_BIN/dora"

cat > "$APP_BIN/dora-asr" << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
export PYTHONHOME="$HERE/python/Python.framework/Versions/__PY_EMBED_VERSION__"
export PYTHONPATH="$HERE/python/site-packages"
export DYLD_LIBRARY_PATH="$PYTHONHOME:$PYTHONHOME/lib:${DYLD_LIBRARY_PATH:-}"
if [ -x /usr/bin/arch ] && /usr/bin/arch -arm64 /usr/bin/true >/dev/null 2>&1; then
  exec /usr/bin/arch -arm64 "$HERE/python/bin/python3" -m dora_asr.main "$@"
fi
exec "$HERE/python/bin/python3" -m dora_asr.main "$@"
EOF
sed -i '' "s/__PY_EMBED_VERSION__/${PYTHON_EMBED_VERSION}/g" "$APP_BIN/dora-asr"
chmod +x "$APP_BIN/dora-asr"

cat > "$APP_BIN/dora-text-segmenter" << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
export PYTHONHOME="$HERE/python/Python.framework/Versions/__PY_EMBED_VERSION__"
export PYTHONPATH="$HERE/python/site-packages"
export DYLD_LIBRARY_PATH="$PYTHONHOME:$PYTHONHOME/lib:${DYLD_LIBRARY_PATH:-}"
if [ -x /usr/bin/arch ] && /usr/bin/arch -arm64 /usr/bin/true >/dev/null 2>&1; then
  exec /usr/bin/arch -arm64 "$HERE/python/bin/python3" -m dora_text_segmenter.main "$@"
fi
exec "$HERE/python/bin/python3" -m dora_text_segmenter.main "$@"
EOF
sed -i '' "s/__PY_EMBED_VERSION__/${PYTHON_EMBED_VERSION}/g" "$APP_BIN/dora-text-segmenter"
chmod +x "$APP_BIN/dora-text-segmenter"

cat > "$APP_BIN/dora-primespeech" << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
export PYTHONHOME="$HERE/python/Python.framework/Versions/__PY_EMBED_VERSION__"
export PYTHONPATH="$HERE/python/site-packages"
export DYLD_LIBRARY_PATH="$PYTHONHOME:$PYTHONHOME/lib:${DYLD_LIBRARY_PATH:-}"
if [ -x /usr/bin/arch ] && /usr/bin/arch -arm64 /usr/bin/true >/dev/null 2>&1; then
  exec /usr/bin/arch -arm64 "$HERE/python/bin/python3" -m dora_primespeech.main "$@"
fi
exec "$HERE/python/bin/python3" -m dora_primespeech.main "$@"
EOF
sed -i '' "s/__PY_EMBED_VERSION__/${PYTHON_EMBED_VERSION}/g" "$APP_BIN/dora-primespeech"
chmod +x "$APP_BIN/dora-primespeech"

cp "$PROJECT_ROOT/target/release/mofa-studio" "$APP_BIN/mofa-studio"
chmod +x "$APP_BIN/mofa-studio"

MODEL_SRC="${MODEL_SRC:-$HOME/.dora/models}"
if [ -d "$MODEL_SRC" ]; then
  echo "Copying models from: $MODEL_SRC"
  mkdir -p "$APP_RES/models"
  rsync -a "$MODEL_SRC/" "$APP_RES/models/"
fi

echo "[9/9] Creating launcher..."
cat > "$APP_MACOS/MoFAStudio" << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
APP_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_RES="$APP_ROOT/Resources"
APP_BIN="$APP_RES/bin"
APP_VERSION_FILE="$APP_RES/version.txt"
export PATH="/usr/bin:/bin:/usr/sbin:/sbin:$PATH"

APP_SUPPORT="$HOME/Library/Application Support/MoFA Studio"
RUNTIME_REPO="$APP_SUPPORT/repo"
RUNTIME_VERSION_FILE="$RUNTIME_REPO/.mofa-version"
LOG_DIR="$APP_SUPPORT/logs"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/launch.log"
exec >>"$LOG_FILE" 2>&1
echo "---- $(date) ----"

if [ ! -d "$RUNTIME_REPO" ]; then
  mkdir -p "$APP_SUPPORT"
  rsync -a "$APP_RES/repo/" "$RUNTIME_REPO/"
  cp "$APP_VERSION_FILE" "$RUNTIME_VERSION_FILE"
else
  if [ -f "$APP_VERSION_FILE" ] && [ -f "$RUNTIME_VERSION_FILE" ]; then
    if ! cmp -s "$APP_VERSION_FILE" "$RUNTIME_VERSION_FILE"; then
      rsync -a --delete --exclude "apps/**/dataflow/out" "$APP_RES/repo/" "$RUNTIME_REPO/"
      cp "$APP_VERSION_FILE" "$RUNTIME_VERSION_FILE"
    fi
  fi
fi

if [ -d "$APP_RES/models" ]; then
  if [ ! -d "$HOME/.dora/models" ]; then
    mkdir -p "$HOME/.dora/models"
    rsync -a "$APP_RES/models/" "$HOME/.dora/models/"
  fi
fi

export MOFA_STUDIO_DIR="$RUNTIME_REPO"
export MOFA_PACKAGED=1
export MOFA_FORCE_DORA_RESTART=1
export MOFA_AUTO_START=1
export MOFA_DORA_BIN="$APP_BIN/dora"
export PYTHONHOME="$APP_RES/python/Python.framework/Versions/__PY_EMBED_VERSION__"
export PYTHONPATH="$APP_RES/python/site-packages"
export DYLD_LIBRARY_PATH="$PYTHONHOME:$PYTHONHOME/lib:${DYLD_LIBRARY_PATH:-}"
export PATH="$APP_BIN:$APP_RES/python/bin:$PYTHONHOME/bin:$PATH"

if [ -x /usr/bin/arch ] && /usr/bin/arch -arm64 /usr/bin/true >/dev/null 2>&1; then
  exec /usr/bin/arch -arm64 "$APP_BIN/mofa-studio"
fi
exec "$APP_BIN/mofa-studio"
EOF
sed -i '' "s/__PY_EMBED_VERSION__/${PYTHON_EMBED_VERSION}/g" "$APP_MACOS/MoFAStudio"
chmod +x "$APP_MACOS/MoFAStudio"

if $DO_SIGN; then
  echo "Signing app..."
  cat > "$BUILD_DIR/entitlements.plist" << 'EOF'
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>com.apple.security.device.audio-input</key>
  <true/>
  <key>com.apple.security.cs.allow-jit</key>
  <true/>
  <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
  <true/>
</dict>
</plist>
EOF
  codesign --deep --force --options runtime \
    --entitlements "$BUILD_DIR/entitlements.plist" \
    -s "$SIGN_IDENTITY" "$APP_BUNDLE"
fi

DMG_PATH="$BUILD_DIR/MoFA-Studio-$VERSION.dmg"
echo "Creating DMG: $DMG_PATH"
hdiutil create -volname "$APP_NAME" -srcfolder "$APP_BUNDLE" -ov -format UDZO "$DMG_PATH"

if $DO_NOTARIZE; then
  if [ -z "$APPLE_ID" ] || [ -z "${APP_PASSWORD:-}" ] || [ -z "$TEAM_ID" ]; then
    echo "Missing APPLE_ID / APP_PASSWORD / TEAM_ID for notarization."
    exit 1
  fi
  xcrun notarytool submit "$DMG_PATH" \
    --apple-id "$APPLE_ID" \
    --password "$APP_PASSWORD" \
    --team-id "$TEAM_ID" --wait
  xcrun stapler staple "$DMG_PATH"
fi

echo ""
echo "App Bundle: $APP_BUNDLE"
echo "DMG:        $DMG_PATH"
echo "Done."
