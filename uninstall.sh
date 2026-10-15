#!/usr/bin/env bash
# Antigravity Manager Tools — Linux Uninstaller
#
# Usage:
#   ./uninstall.sh                  Remove the app (keeps user data)
#   ./uninstall.sh --purge          Remove the app AND user data
#   ./uninstall.sh --dry-run        Print what would be done, do nothing
#   ./uninstall.sh --help           Show this help
#
# Flags:
#   --purge       Also remove user data dirs (~/.config, ~/.local/share,
#                 ~/.cache entries for antigravity-manager-tools)
#   --dry-run     Print commands without executing (implies DRY_RUN=1)
#   --help, -h    Show help and exit

set -euo pipefail
set -E

# Visual Formatting & Colors (mirrors install.sh)
INDENT="    "
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

FULL_NAME="Antigravity Manager Tools"

# Package names: real one first, legacy fallbacks after
PKG_NAMES=(
    "antigravity-manager-tools"
    "antigravity-tools"
    "anti-gravity-tools"
    "com.lbjlaq.antigravity-tools"
)

DESKTOP_FILES=(
    "agm-alim.desktop"
    "anti-gravity-tools-by-alim.desktop"
    "antigravity-tools.desktop"
    "com.lbjlaq.antigravity-tools.desktop"
    "com.alimtvnetwork.antigravity-tools.desktop"
)

BIN_LINKS=(
    "${HOME}/.local/bin/agm"
    "${HOME}/.local/bin/agm-alim"
)

PURGE_DIRS=(
    "${HOME}/.config/antigravity-manager-tools"
    "${HOME}/.local/share/antigravity-manager-tools"
    "${HOME}/.cache/antigravity-manager-tools"
)

PURGE=0

# Helper functions with left indentation
info()    { echo -e "${INDENT}${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${INDENT}${GREEN}[OK]${NC} $1"; }
warn()    { echo -e "${INDENT}${YELLOW}[WARN]${NC} $1"; }
error()   {
    echo -e "${INDENT}${RED}[ERROR]${NC} $1" >&2
    exit 1
}
step()    { echo -e "\n${INDENT}${CYAN}==>${NC} ${BOLD}$1${NC}"; }

run() {
    if [[ "${DRY_RUN:-0}" == "1" ]]; then
        echo -e "${INDENT}${YELLOW}[DRY-RUN]${NC} $*"
    else
        "$@"
    fi
}

show_help() {
    cat <<EOF
${FULL_NAME} — Linux Uninstaller

Usage:
  ./uninstall.sh [flags]

Flags:
  --purge       Also remove user data directories:
                  ~/.config/antigravity-manager-tools
                  ~/.local/share/antigravity-manager-tools
                  ~/.cache/antigravity-manager-tools
  --dry-run     Print what would be done without doing it
  --help, -h    Show this help

Without --purge, user data is kept and only the application package,
.desktop launchers, and CLI symlinks are removed.
EOF
    exit 0
}

detect_pkg_manager() {
    if command -v apt-get &>/dev/null; then
        PKG_MANAGER="apt"
    elif command -v dnf &>/dev/null; then
        PKG_MANAGER="dnf"
    elif command -v yum &>/dev/null; then
        PKG_MANAGER="yum"
    else
        error "No supported package manager found (apt/dnf/yum). Nothing to remove via package manager."
    fi
    info "Package manager: ${PKG_MANAGER}"
}

sudo_cmd() {
    if command -v sudo &>/dev/null && [[ ${EUID:-$(id -u)} -ne 0 ]]; then
        echo "sudo"
    else
        echo ""
    fi
}

remove_packages() {
    step "Removing packages..."
    local sudo
    sudo="$(sudo_cmd)"
    local removed=0

    for pkg in "${PKG_NAMES[@]}"; do
        local installed=0
        case "$PKG_MANAGER" in
            apt)
                dpkg -s "$pkg" &>/dev/null && installed=1 || true
                ;;
            dnf|yum)
                rpm -q "$pkg" &>/dev/null && installed=1 || true
                ;;
        esac
        if [[ "$installed" -eq 1 ]]; then
            info "Removing package: ${pkg}"
            case "$PKG_MANAGER" in
                apt)
                    run $sudo apt-get remove -y "$pkg" 2>/dev/null \
                        || run $sudo dpkg -r "$pkg" 2>/dev/null \
                        || warn "Could not remove ${pkg}"
                    ;;
                dnf|yum)
                    run $sudo "$PKG_MANAGER" remove -y "$pkg" 2>/dev/null \
                        || warn "Could not remove ${pkg}"
                    ;;
            esac
            removed=1
        fi
    done

    if [[ "$removed" -eq 0 ]]; then
        info "No installed package found (already removed or never installed via package manager)."
    fi
}

remove_desktop_entries() {
    step "Removing .desktop launchers..."
    local apps_dir="${HOME}/.local/share/applications"
    local removed=0
    for f in "${DESKTOP_FILES[@]}"; do
        if [[ -f "${apps_dir}/${f}" ]]; then
            info "Removing ${apps_dir}/${f}"
            run rm -f "${apps_dir}/${f}"
            removed=1
        fi
    done
    if [[ "$removed" -eq 0 ]]; then
        info "No .desktop entries found."
    fi
    # Refresh desktop database if available (best effort)
    if command -v update-desktop-database &>/dev/null; then
        run update-desktop-database "$apps_dir" 2>/dev/null || true
    fi
}

remove_bin_links() {
    step "Removing CLI symlinks..."
    local removed=0
    for link in "${BIN_LINKS[@]}"; do
        if [[ -L "$link" ]]; then
            local target
            target="$(readlink "$link" 2>/dev/null || true)"
            # Only remove if the symlink points at our app
            if [[ "${target,,}" == *"antigravity"* || "${target,,}" == *"agm-alim"* || "${target,,}" == *"/agm"* ]]; then
                info "Removing symlink ${link} -> ${target}"
                run rm -f "$link"
                removed=1
            else
                warn "Skipping ${link}: points at '${target}' (not our app)"
            fi
        elif [[ -e "$link" ]]; then
            warn "Skipping ${link}: not a symlink, leaving untouched"
        fi
    done
    if [[ "$removed" -eq 0 ]]; then
        info "No CLI symlinks found."
    fi
}

purge_user_data() {
    step "Purging user data..."
    for d in "${PURGE_DIRS[@]}"; do
        if [[ -d "$d" ]]; then
            info "Removing ${d}"
            run rm -rf "$d"
        else
            info "Not found (skipped): ${d}"
        fi
    done
    success "User data purged."
}

main() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --help|-h)   show_help ;;
            --purge)     PURGE=1; shift ;;
            --dry-run)   DRY_RUN=1; shift ;;
            *)
                error "Unknown flag: $1 (see --help)"
                ;;
        esac
    done

    echo ""
    echo -e "${INDENT}${BLUE}========================================${NC}"
    echo -e "${INDENT}${BLUE}    ${FULL_NAME} Uninstaller${NC}"
    echo -e "${INDENT}${BLUE}========================================${NC}"

    if [[ "${DRY_RUN:-0}" == "1" ]]; then
        warn "DRY-RUN mode: nothing will be removed."
    fi

    detect_pkg_manager
    remove_packages
    remove_desktop_entries
    remove_bin_links

    if [[ "$PURGE" -eq 1 ]]; then
        purge_user_data
    else
        echo ""
        info "User data kept at:"
        for d in "${PURGE_DIRS[@]}"; do
            echo -e "${INDENT}  - ${d}"
        done
        info "Re-run with --purge to remove user data as well."
    fi

    echo ""
    success "Uninstall complete."
}

main "$@"
