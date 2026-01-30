#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/build/macos-full"
APP_NAME="MoFA Studio"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
ENTITLEMENTS="$BUILD_DIR/entitlements.plist"
APP_RES="$APP_BUNDLE/Contents/Resources"

SIGN_IDENTITY="${SIGN_IDENTITY:-}"
NOTARY_PROFILE="${NOTARY_PROFILE:-mofa-notary}"
DMG_PATH="${DMG_PATH:-}"

pick_identity() {
  local line hash name
  local hashes=()
  local names=()
  while IFS= read -r line; do
    case "$line" in
      *"Developer ID Application:"*)
        hash="$(echo "$line" | sed -n 's/^[[:space:]]*[0-9][0-9]*).* \\([A-F0-9]\\{40\\}\\) .*/\\1/p')"
        name="$(echo "$line" | sed -n 's/.*\"\\(Developer ID Application:[^\"]*\\)\".*/\\1/p')"
        if [ -n "$hash" ] && [ -n "$name" ]; then
          hashes+=("$hash")
          names+=("$name")
        fi
        ;;
    esac
  done < <(security find-identity -p codesigning -v 2>/dev/null || true)

  if [ "${#hashes[@]}" -eq 0 ]; then
    echo "No Developer ID Application identities found."
    return 1
  fi

  if [ "${#hashes[@]}" -eq 1 ]; then
    SIGN_IDENTITY="${hashes[0]}"
    return 0
  fi

  echo "Multiple Developer ID Application identities found:"
  local i=1
  while [ $i -le "${#hashes[@]}" ]; do
    echo "  [$i] ${names[$((i-1))]} (${hashes[$((i-1))]})"
    i=$((i+1))
  done
  echo -n "Pick one by number (or paste hash/name): "
  read -r choice < /dev/tty || true
  if [ -z "${choice:-}" ]; then
    return 1
  fi
  if echo "$choice" | grep -Eq '^[0-9]+$'; then
    local idx=$((choice-1))
    if [ "$idx" -ge 0 ] && [ "$idx" -lt "${#hashes[@]}" ]; then
      SIGN_IDENTITY="${hashes[$idx]}"
      return 0
    fi
    return 1
  fi
  SIGN_IDENTITY="$choice"
  return 0
}

if [ -z "$SIGN_IDENTITY" ]; then
  if ! pick_identity; then
    echo "Missing SIGN_IDENTITY and unable to auto-select."
    exit 1
  fi
fi

if [ ! -d "$APP_BUNDLE" ]; then
  echo "App bundle not found: $APP_BUNDLE"
  exit 1
fi

if [ ! -f "$ENTITLEMENTS" ]; then
  cat > "$ENTITLEMENTS" << 'EOF'
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
fi

if [ "${PRUNE_REPO_TARGETS:-0}" = "1" ]; then
  if [ -d "$APP_RES/repo" ]; then
    echo "Pruning target dirs under repo..."
    find "$APP_RES/repo" -type d -name target -prune -exec rm -rf {} + 2>/dev/null || true
  fi
fi

VERSION="0.0.0"
VERSION_FILE="$APP_BUNDLE/Contents/Resources/version.txt"
if [ -f "$VERSION_FILE" ]; then
  VERSION="$(cat "$VERSION_FILE" 2>/dev/null || echo "0.0.0")"
fi

if [ -z "$DMG_PATH" ]; then
  DMG_PATH="$BUILD_DIR/MoFA-Studio-$VERSION.dmg"
fi

sign_one() {
  codesign --force --options runtime --timestamp \
    -s "$SIGN_IDENTITY" "$1"
}

echo "Signing nested Mach-O binaries..."
while IFS= read -r -d '' f; do
  if file "$f" | rg -q "Mach-O"; then
    sign_one "$f"
  fi
done < <(find "$APP_BUNDLE" -type f -print0)

echo "Signing embedded app bundles..."
while IFS= read -r -d '' app; do
  if [ "$app" != "$APP_BUNDLE" ]; then
    codesign --force --options runtime --timestamp -s "$SIGN_IDENTITY" "$app"
  fi
done < <(find "$APP_BUNDLE" -type d -name "*.app" -print0)

echo "Signing main app with entitlements: $SIGN_IDENTITY"
codesign --deep --force --options runtime --timestamp \
  --entitlements "$ENTITLEMENTS" \
  -s "$SIGN_IDENTITY" "$APP_BUNDLE"

echo "Verifying signature..."
codesign --verify --deep --strict --verbose=2 "$APP_BUNDLE"

echo "Creating DMG: $DMG_PATH"
hdiutil create -volname "$APP_NAME" \
  -srcfolder "$APP_BUNDLE" \
  -ov -format UDZO "$DMG_PATH"

echo "Submitting for notarization..."
xcrun notarytool submit "$DMG_PATH" \
  --keychain-profile "$NOTARY_PROFILE" --wait

echo "Stapling..."
xcrun stapler staple "$DMG_PATH"

echo "Done."
