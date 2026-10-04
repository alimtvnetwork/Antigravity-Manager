#!/bin/bash
set -euo pipefail

# Error stack trace trap
report_error_stack() {
    local exit_code="$?"
    local line_no="${1:-$LINENO}"
    local cmd="${BASH_COMMAND:-unknown}"
    if [ "$exit_code" -ne 0 ]; then
        echo "❌ [ERROR] Command '$cmd' failed at line $line_no with exit code $exit_code" >&2
        if [ ${#FUNCNAME[@]} -gt 1 ]; then
            echo "   [STACK TRACE]" >&2
            for ((i = 1; i < ${#FUNCNAME[@]}; i++)); do
                local fn="${FUNCNAME[$i]}"
                local src="${BASH_SOURCE[$i]:-package_dmg.sh}"
                local ln="${BASH_LINENO[$((i - 1))]}"
                echo "     -> at ${fn}() in ${src}:${ln}" >&2
            done
        fi
    fi
}
trap 'report_error_stack "$LINENO"' ERR

# Resolve script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

# Configuration
APP_NAME="Antigravity Manager Tools"
VERSION=$(grep '"version":' package.json | head -n 1 | awk -F: '{ print $2 }' | sed 's/[", ]//g')
DMG_NAME="Antigravity_Manager_Tools_${VERSION}_ManualFix.dmg"
SRC_APP_PATH="src-tauri/target/release/bundle/macos/${APP_NAME}.app"
DIST_DIR="dist_dmg"

# Staging cleanup trap
trap 'rm -rf "$DIST_DIR"' EXIT

echo "📦 Starting DMG package build (with quarantine fix script)..."
echo "Version: $VERSION"

# 1. Check if build artifact exists or dynamically discover .app
if [ ! -d "$SRC_APP_PATH" ]; then
    DISCOVERED_APP=$(find src-tauri/target/release/bundle -maxdepth 3 -name "*.app" -type d 2>/dev/null | head -n 1)
    if [ -n "$DISCOVERED_APP" ] && [ -d "$DISCOVERED_APP" ]; then
        SRC_APP_PATH="$DISCOVERED_APP"
        APP_NAME="$(basename "$SRC_APP_PATH" .app)"
        echo "ℹ️  Discovered application bundle: $SRC_APP_PATH ($APP_NAME)"
    else
        echo "❌ Error: Built application not found at $SRC_APP_PATH"
        echo "Please run first: npm run tauri build"
        exit 1
    fi
fi

# 2. Prepare temporary distribution directory
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# 3. Copy application bundle
echo "Checking and copying source app ($APP_NAME)..."
cp -R "$SRC_APP_PATH" "$DIST_DIR/"

TARGET_BUNDLE="$DIST_DIR/$(basename "$SRC_APP_PATH")"

# Strip quarantine and apply ad-hoc code signature to bundle in staging directory
echo "Stripping quarantine and applying ad-hoc signature to bundle..."
xattr -cr "$TARGET_BUNDLE" 2>/dev/null || true
xattr -d com.apple.quarantine "$TARGET_BUNDLE" 2>/dev/null || true
find "$TARGET_BUNDLE" -exec xattr -d com.apple.quarantine {} + 2>/dev/null || true
if command -v codesign &>/dev/null; then
    codesign --force --deep --sign - "$TARGET_BUNDLE" 2>/dev/null || true
fi

# 4. Copy and configure Fix_Damaged.command
echo "Including Fix_Damaged.command in DMG..."
chmod +x "scripts/Fix_Damaged.command"
cp "scripts/Fix_Damaged.command" "$DIST_DIR/"
chmod +x "$DIST_DIR/Fix_Damaged.command"
xattr -cr "$DIST_DIR/Fix_Damaged.command" 2>/dev/null || true
xattr -d com.apple.quarantine "$DIST_DIR/Fix_Damaged.command" 2>/dev/null || true

# 5. Create /Applications symlink for drag-and-drop install
ln -s /Applications "$DIST_DIR/Applications"

# 6. Build DMG with hdiutil
echo "Creating DMG image ($DMG_NAME)..."
rm -f "$DMG_NAME"
hdiutil create -volname "${APP_NAME}" -srcfolder "$DIST_DIR" -ov -format UDZO "$DMG_NAME"

# 7. Strip quarantine from DMG itself
echo "Stripping quarantine attributes from output DMG..."
xattr -d com.apple.quarantine "$DMG_NAME" 2>/dev/null || true
xattr -cr "$DMG_NAME" 2>/dev/null || true

echo "✅ DMG Packaging complete!"
echo "Artifact location: $PWD/$DMG_NAME"

