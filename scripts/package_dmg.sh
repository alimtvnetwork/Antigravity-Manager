#!/bin/bash
set -euo pipefail
set -E

# Error stack trace trap for maximum transparency and failure diagnostics
report_error_stack() {
    local exit_code="$?"
    local line_no="${1:-$LINENO}"
    local cmd="${2:-${BASH_COMMAND:-unknown}}"
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
trap 'report_error_stack "$LINENO" "$BASH_COMMAND"' ERR

# Resolve script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

# Parse CLI arguments
ARCH_ARG=""
TARGET_ARG=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --arch)
            ARCH_ARG="$2"
            shift 2
            ;;
        --arch=*)
            ARCH_ARG="${1#*=}"
            shift
            ;;
        --target)
            TARGET_ARG="$2"
            shift 2
            ;;
        --target=*)
            TARGET_ARG="${1#*=}"
            shift
            ;;
        *)
            shift
            ;;
    esac
done

# Architecture resolution
if [ -z "$ARCH_ARG" ]; then
    if [[ "$TARGET_ARG" == *"aarch64"* ]]; then
        ARCH_ARG="aarch64"
    elif [[ "$TARGET_ARG" == *"x86_64"* ]]; then
        ARCH_ARG="x64"
    elif [[ "$TARGET_ARG" == *"universal"* ]]; then
        ARCH_ARG="universal"
    else
        NATIVE_ARCH="$(uname -m 2>/dev/null || echo "unknown")"
        case "$NATIVE_ARCH" in
            arm64|aarch64) ARCH_ARG="aarch64" ;;
            x86_64|amd64) ARCH_ARG="x64" ;;
            *) ARCH_ARG="universal" ;;
        esac
    fi
fi

# Configuration
APP_NAME="Antigravity Manager Tools"
VERSION=$(grep '"version":' package.json | head -n 1 | awk -F: '{ print $2 }' | sed 's/[", ]//g')
DIST_DIR="dist_dmg_${ARCH_ARG}"

# Staging cleanup trap
trap 'rm -rf "$DIST_DIR"' EXIT

echo "📦 Starting DMG package build (with quarantine fix scripts & installer)..."
echo "Version     : $VERSION"
echo "Architecture: $ARCH_ARG"

# 1. Dynamically discover .app bundle across target directories
RUST_TARGET=""
if [[ "$ARCH_ARG" == "aarch64" ]]; then
    RUST_TARGET="aarch64-apple-darwin"
elif [[ "$ARCH_ARG" == "x64" || "$ARCH_ARG" == "x86_64" ]]; then
    RUST_TARGET="x86_64-apple-darwin"
elif [[ "$ARCH_ARG" == "universal" ]]; then
    RUST_TARGET="universal-apple-darwin"
fi

SRC_APP_PATH=""
SEARCH_PATHS=(
    "src-tauri/target/${TARGET_ARG}/release/bundle/macos/${APP_NAME}.app"
    "src-tauri/target/${RUST_TARGET}/release/bundle/macos/${APP_NAME}.app"
    "src-tauri/target/release/bundle/macos/${APP_NAME}.app"
    "src-tauri/target/${ARCH_ARG}-apple-darwin/release/bundle/macos/${APP_NAME}.app"
)
for sp in "${SEARCH_PATHS[@]}"; do
    if [ -n "$sp" ] && [ -d "$sp" ]; then
        SRC_APP_PATH="$sp"
        break
    fi
done

if [ -z "$SRC_APP_PATH" ] || [ ! -d "$SRC_APP_PATH" ]; then
    DISCOVERED_APP=$(find src-tauri/target -name "${APP_NAME}.app" -type d 2>/dev/null | head -n 1)
    if [ -z "$DISCOVERED_APP" ]; then
        DISCOVERED_APP=$(find src-tauri/target -name "*.app" -type d 2>/dev/null | head -n 1)
    fi
    if [ -n "$DISCOVERED_APP" ] && [ -d "$DISCOVERED_APP" ]; then
        SRC_APP_PATH="$DISCOVERED_APP"
        APP_NAME="$(basename "$SRC_APP_PATH" .app)"
        echo "ℹ️  Discovered application bundle: $SRC_APP_PATH ($APP_NAME)"
    else
        echo "❌ Error: Built application not found in src-tauri/target"
        echo "Please run first: npm run tauri build"
        exit 1
    fi
fi

echo "Source app bundle: $SRC_APP_PATH"

# 2. Prepare temporary distribution directory
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# 3. Copy application bundle
echo "Copying source app ($APP_NAME) into staging..."
cp -R "$SRC_APP_PATH" "$DIST_DIR/"

TARGET_BUNDLE="$DIST_DIR/$(basename "$SRC_APP_PATH")"

# Strip quarantine, provenance, and apply ad-hoc code signature to bundle in staging directory
echo "Stripping quarantine and applying ad-hoc signature to staging bundle..."
xattr -cr "$TARGET_BUNDLE" 2>/dev/null || true
xattr -rd com.apple.quarantine "$TARGET_BUNDLE" 2>/dev/null || true
xattr -rd com.apple.provenance "$TARGET_BUNDLE" 2>/dev/null || true
find "$TARGET_BUNDLE" -exec xattr -c {} + 2>/dev/null || true
macos_major=0
if command -v sw_vers &>/dev/null; then
    macos_major=$(sw_vers -productVersion | cut -d. -f1)
fi
if [[ "$macos_major" -lt 15 ]] && command -v codesign &>/dev/null; then
    codesign --force --deep --sign - "$TARGET_BUNDLE" 2>/dev/null || true
fi

# 4. Copy and configure Install_Or_Repair.command, Fix_Damaged.command, and fix_app.sh
echo "Including Install_Or_Repair.command, Fix_Damaged.command, and fix_app.sh in DMG root..."
if [ -f "scripts/Install_Or_Repair.command" ]; then
    cp "scripts/Install_Or_Repair.command" "$DIST_DIR/Install_Or_Repair.command"
    chmod +x "$DIST_DIR/Install_Or_Repair.command"
    xattr -cr "$DIST_DIR/Install_Or_Repair.command" 2>/dev/null || true
    xattr -d com.apple.quarantine "$DIST_DIR/Install_Or_Repair.command" 2>/dev/null || true

    # Provide 1-Click alias so it sorts first in Finder window
    cp "scripts/Install_Or_Repair.command" "$DIST_DIR/1-Click_Install_Or_Repair.command"
    chmod +x "$DIST_DIR/1-Click_Install_Or_Repair.command"
    xattr -cr "$DIST_DIR/1-Click_Install_Or_Repair.command" 2>/dev/null || true
    xattr -d com.apple.quarantine "$DIST_DIR/1-Click_Install_Or_Repair.command" 2>/dev/null || true
fi

if [ -f "scripts/Fix_Damaged.command" ]; then
    cp "scripts/Fix_Damaged.command" "$DIST_DIR/Fix_Damaged.command"
    chmod +x "$DIST_DIR/Fix_Damaged.command"
    xattr -cr "$DIST_DIR/Fix_Damaged.command" 2>/dev/null || true
    xattr -d com.apple.quarantine "$DIST_DIR/Fix_Damaged.command" 2>/dev/null || true

    # Provide 2-Click alias for repairing trash lockout
    cp "scripts/Fix_Damaged.command" "$DIST_DIR/2-Click_Fix_Trash_Error.command"
    chmod +x "$DIST_DIR/2-Click_Fix_Trash_Error.command"
    xattr -cr "$DIST_DIR/2-Click_Fix_Trash_Error.command" 2>/dev/null || true
    xattr -d com.apple.quarantine "$DIST_DIR/2-Click_Fix_Trash_Error.command" 2>/dev/null || true
fi

if [ -f "scripts/fix_app.sh" ]; then
    cp "scripts/fix_app.sh" "$DIST_DIR/"
    chmod +x "$DIST_DIR/fix_app.sh"
    xattr -cr "$DIST_DIR/fix_app.sh" 2>/dev/null || true
    xattr -d com.apple.quarantine "$DIST_DIR/fix_app.sh" 2>/dev/null || true
fi

# 5. Optionally build native macOS PKG package with root postinstall script and include inside DMG
PRIMARY_PKG_NAME="Antigravity.Manager.Tools_${VERSION}_${ARCH_ARG}.pkg"
if command -v pkgbuild &>/dev/null; then
    echo "Creating native macOS PKG installer ($PRIMARY_PKG_NAME)..."
    rm -f "$PRIMARY_PKG_NAME"
    PKG_ARGS=(--component "$SRC_APP_PATH" --install-location /Applications)
    if [ -d "scripts/pkg-scripts" ]; then
        chmod +x scripts/pkg-scripts/* 2>/dev/null || true
        PKG_ARGS+=(--scripts "scripts/pkg-scripts")
    fi
    if pkgbuild "${PKG_ARGS[@]}" "$PRIMARY_PKG_NAME" 2>/dev/null; then
        echo "✅ Created PKG installer: $PRIMARY_PKG_NAME"
        cp "$PRIMARY_PKG_NAME" "$DIST_DIR/Install_Antigravity_Manager.pkg" 2>/dev/null || true
        xattr -cr "$DIST_DIR/Install_Antigravity_Manager.pkg" 2>/dev/null || true
        xattr -d com.apple.quarantine "$DIST_DIR/Install_Antigravity_Manager.pkg" 2>/dev/null || true
    fi
fi

# Include quick installation and troubleshooting guide
cat << 'EOF' > "$DIST_DIR/Install_Guide.txt"
======================================================================
  Antigravity Manager Tools - macOS 安装与故障排除指南
  Antigravity Manager Tools - macOS Installation & Troubleshooting
======================================================================

【方案一：官方 PKG 安装器 (推荐，彻底免除隔离报错) / Option 1: macOS PKG Installer】
  双击本目录下的 "Install_Antigravity_Manager.pkg"。
  通过 macOS 原生系统安装器一键安装，自动配置看门人权限、清理废纸篓锁定并刷新系统注册。
  Double-click "Install_Antigravity_Manager.pkg" to install via Apple's
  standard installer wizard with automated permissions & trash self-healing.

----------------------------------------------------------------------

【方案二：一键脚本安装与全自动修复 / Option 2: 1-Click Script Installer】
  双击本目录下的 "1-Click_Install_Or_Repair.command"。
  脚本将自动解除废纸篓锁定、净化隔离属性、安装到 /Applications 并启动。
  Double-click "1-Click_Install_Or_Repair.command" to install and launch.

----------------------------------------------------------------------

【方案三：手动拖拽安装 / Option 3: Manual Drag-and-Drop Installation】
  1. 拖拽 "Antigravity Manager Tools.app" 图标到 "Applications" 文件夹。
     Drag "Antigravity Manager Tools.app" into the "Applications" folder.

  2. 若启动时提示 "已损坏" 或 "因为已将它丢弃到废纸篓"：
     If prompted "App is damaged" or "... because it is in the Trash":
     双击本目录下的 "2-Click_Fix_Trash_Error.command" 即可一秒解除锁定。
     Double-click "2-Click_Fix_Trash_Error.command" in this folder.

----------------------------------------------------------------------

【方案四：终端一键安装命令 / Option 4: Terminal Command】
  curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
======================================================================
EOF

# 6. Create /Applications symlink for drag-and-drop install
ln -s /Applications "$DIST_DIR/Applications"

# 7. Build DMG with hdiutil
PRIMARY_DMG_NAME="Antigravity.Manager.Tools_${VERSION}_${ARCH_ARG}.dmg"
echo "Creating DMG image ($PRIMARY_DMG_NAME)..."
rm -f "$PRIMARY_DMG_NAME"
hdiutil create -volname "${APP_NAME}" -srcfolder "$DIST_DIR" -ov -format UDZO "$PRIMARY_DMG_NAME"

# 8. Strip quarantine from DMG itself
echo "Stripping quarantine attributes from output DMG..."
xattr -d com.apple.quarantine "$PRIMARY_DMG_NAME" 2>/dev/null || true
xattr -cr "$PRIMARY_DMG_NAME" 2>/dev/null || true

# 9. Place DMG and PKG in release bundle directory if present
BUNDLE_DMG_DIRS=(
    "src-tauri/target/${TARGET_ARG}/release/bundle/dmg"
    "src-tauri/target/${RUST_TARGET}/release/bundle/dmg"
    "src-tauri/target/release/bundle/dmg"
    "src-tauri/target/${ARCH_ARG}-apple-darwin/release/bundle/dmg"
)
for bdd in "${BUNDLE_DMG_DIRS[@]}"; do
    if [ -n "$bdd" ]; then
        mkdir -p "$bdd"
        echo "Updating release bundle directory: $bdd/$PRIMARY_DMG_NAME"
        cp "$PRIMARY_DMG_NAME" "$bdd/$PRIMARY_DMG_NAME"
        xattr -cr "$bdd/$PRIMARY_DMG_NAME" 2>/dev/null || true
        if [ -f "$PRIMARY_PKG_NAME" ]; then
            cp "$PRIMARY_PKG_NAME" "$bdd/$PRIMARY_PKG_NAME" 2>/dev/null || true
        fi
        # Overwrite any unpatched Tauri DMG in the bundle folder
        for existing_dmg in "$bdd"/*.dmg; do
            if [ -f "$existing_dmg" ] && [ "$existing_dmg" != "$bdd/$PRIMARY_DMG_NAME" ] && [ "$existing_dmg" != "$bdd/$MANUAL_DMG_NAME" ]; then
                echo "Replacing unpatched Tauri DMG ($existing_dmg) with patched DMG..."
                cp "$PRIMARY_DMG_NAME" "$existing_dmg"
                xattr -cr "$existing_dmg" 2>/dev/null || true
            fi
        done
    fi
done

# Also generate friendly ManualFix named DMG for direct setup downloads
MANUAL_DMG_NAME="Antigravity_Manager_Tools_${VERSION}_ManualFix.dmg"
cp "$PRIMARY_DMG_NAME" "$MANUAL_DMG_NAME"
xattr -cr "$MANUAL_DMG_NAME" 2>/dev/null || true

echo "✅ DMG & PKG Packaging complete!"
echo "Primary DMG: $PWD/$PRIMARY_DMG_NAME"
echo "Manual Fix : $PWD/$MANUAL_DMG_NAME"
[ -f "$PRIMARY_PKG_NAME" ] && echo "Primary PKG: $PWD/$PRIMARY_PKG_NAME"
