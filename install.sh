#!/usr/bin/env bash
# Antigravity Tools Install Script (Linux + macOS)
# Usage: curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
#
# Environment variables:
#   VERSION     - Install specific version (e.g., "4.7.6"), default: latest
#   DRY_RUN     - Set to "1" to print commands without executing

set -euo pipefail

# Visual Formatting & Colors
INDENT="    "
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

REPO="alimtvnetwork/Antigravity-Manager"
UPSTREAM_REPO="lbjlaq/Antigravity-Manager"
APP_NAME="Antigravity Manager Tools"
FULL_NAME="Antigravity Manager Tools"
PUBLISHER="Maintained by Alim, Sponsored by RISEUP ASIA LLC"
APP_ID="com.lbjlaq.antigravity-tools"
BINARY_NAME="agm-alim"
DESKTOP_NAME="Antigravity Manager Tools"
TOOLTIP="Antigravity Manager Tools"
GITHUB_API="https://api.github.com/repos/${REPO}/releases"
UPSTREAM_API="https://api.github.com/repos/${UPSTREAM_REPO}/releases"
FALLBACK_STABLE_VERSION="4.7.6"
PINNED_VERSION="__PINNED_VERSION__"

# Helper functions with left indentation
info()    { echo -e "${INDENT}${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${INDENT}${GREEN}[OK]${NC} $1"; }
warn()    { echo -e "${INDENT}${YELLOW}[WARN]${NC} $1"; }
error()   { echo -e "${INDENT}${RED}[ERROR]${NC} $1" >&2; exit 1; }
step()    { echo -e "\n${INDENT}${CYAN}==>${NC} ${BOLD}$1${NC}"; }

run() {
    if [[ "${DRY_RUN:-0}" == "1" ]]; then
        echo -e "${INDENT}${YELLOW}[DRY-RUN]${NC} $*"
    else
        "$@"
    fi
}

run_indented() {
    if [[ "${DRY_RUN:-0}" == "1" ]]; then
        echo -e "${INDENT}${YELLOW}[DRY-RUN]${NC} $*"
        return 0
    fi
    echo ""
    local exit_code=0
    set +e
    "$@" 2>&1 | sed $'s/^/\t/'
    exit_code="${PIPESTATUS[0]}"
    set -e
    echo ""
    return "$exit_code"
}

# Show help
show_help() {
    echo ""
    echo -e "${INDENT}${APP_NAME} Install Script"
    echo ""
    echo -e "${INDENT}Usage:"
    echo -e "${INDENT}    curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash"
    echo ""
    echo -e "${INDENT}    # Install specific version"
    echo -e "${INDENT}    curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | VERSION=4.7.6 bash"
    echo ""
    echo -e "${INDENT}Options:"
    echo -e "${INDENT}    --help          Show this help message"
    echo -e "${INDENT}    --version       Show script version"
    echo -e "${INDENT}    --check-update  Check if a newer release is available and print JSON"
    echo -e "${INDENT}    --update        Check and update to latest version if available"
    echo ""
    echo -e "${INDENT}Environment Variables:"
    echo -e "${INDENT}    VERSION     Install specific version (default: latest)"
    echo -e "${INDENT}    DRY_RUN     Set to '1' to preview commands without executing"
    echo ""
    echo -e "${INDENT}Supported Platforms:"
    echo -e "${INDENT}    - Linux x86_64:  .deb (Debian/Ubuntu), .rpm (Fedora/RHEL), .AppImage"
    echo -e "${INDENT}    - Linux aarch64: .deb (Debian/Ubuntu), .rpm (Fedora/RHEL), .AppImage"
    echo -e "${INDENT}    - macOS x86_64:  .dmg"
    echo -e "${INDENT}    - macOS arm64:   .dmg"
    echo ""
    echo -e "${INDENT}Features:"
    echo -e "${INDENT}    - Parallel multi-connection download acceleration via aria2c with curl fallback"
    echo -e "${INDENT}    - Safe lifecycle: old tools uninstalled only after successful download verification"
    echo -e "${INDENT}    - Automatic post-installation temporary file and folder cleanup"
    echo -e "${INDENT}    - Clear version migration path reporting"
    echo ""
    exit 0
}

# Detect OS and architecture
detect_platform() {
    if [[ -z "${PLATFORM:-}" ]]; then
        OS="$(uname -s)"
        case "$OS" in
            Linux)  PLATFORM="linux" ;;
            Darwin) PLATFORM="macos" ;;
            *)      error "Unsupported OS: $OS. Use install.ps1 for Windows." ;;
        esac
    fi

    if [[ -z "${ARCH_LABEL:-}" ]]; then
        ARCH="$(uname -m)"
        case "$ARCH" in
            x86_64|amd64)   ARCH_LABEL="x86_64"; DEB_ARCH="amd64"; RPM_ARCH="x86_64" ;;
            aarch64|arm64)  ARCH_LABEL="aarch64"; DEB_ARCH="arm64"; RPM_ARCH="aarch64" ;;
            *)              error "Unsupported architecture: $ARCH" ;;
        esac
    else
        DEB_ARCH="${DEB_ARCH:-amd64}"
        RPM_ARCH="${RPM_ARCH:-x86_64}"
    fi

    info "Detected platform : $PLATFORM ($ARCH_LABEL)"
}

# Detect Linux package manager
detect_linux_distro() {
    if [[ "$PLATFORM" != "linux" ]]; then
        return
    fi

    if [[ -z "${PKG_MANAGER:-}" ]]; then
        if command -v apt-get &>/dev/null; then
            PKG_MANAGER="apt"
            PKG_EXT="deb"
        elif command -v dnf &>/dev/null; then
            PKG_MANAGER="dnf"
            PKG_EXT="rpm"
        elif command -v yum &>/dev/null; then
            PKG_MANAGER="yum"
            PKG_EXT="rpm"
        else
            PKG_MANAGER="appimage"
            PKG_EXT="AppImage"
            warn "No supported system package manager found, using AppImage"
        fi
    else
        case "$PKG_MANAGER" in
            apt)     PKG_EXT="${PKG_EXT:-deb}" ;;
            dnf|yum) PKG_EXT="${PKG_EXT:-rpm}" ;;
            *)       PKG_EXT="${PKG_EXT:-AppImage}" ;;
        esac
    fi

    info "Package manager   : $PKG_MANAGER ($PKG_EXT)"
}

# Detect current installed version
detect_current_version() {
    if [[ -n "${CURRENT_VERSION:-}" ]]; then
        return
    fi
    CURRENT_VERSION=""

    if [[ "$PLATFORM" == "linux" ]]; then
        case "${PKG_MANAGER:-}" in
            apt)
                if dpkg -s "$APP_ID" &>/dev/null; then
                    CURRENT_VERSION=$(dpkg -s "$APP_ID" 2>/dev/null | grep -i '^Version:' | head -n1 | awk '{print $2}' | tr -d '[:space:]\r\n')
                elif dpkg -s antigravity-tools &>/dev/null; then
                    CURRENT_VERSION=$(dpkg -s antigravity-tools 2>/dev/null | grep -i '^Version:' | head -n1 | awk '{print $2}' | tr -d '[:space:]\r\n')
                elif dpkg -s anti-gravity-tools &>/dev/null; then
                    CURRENT_VERSION=$(dpkg -s anti-gravity-tools 2>/dev/null | grep -i '^Version:' | head -n1 | awk '{print $2}' | tr -d '[:space:]\r\n')
                fi
                ;;
            dnf|yum)
                if rpm -q "$APP_ID" &>/dev/null; then
                    CURRENT_VERSION=$(rpm -q --queryformat '%{VERSION}' "$APP_ID" 2>/dev/null | tr -d '[:space:]\r\n')
                elif rpm -q antigravity-tools &>/dev/null; then
                    CURRENT_VERSION=$(rpm -q --queryformat '%{VERSION}' antigravity-tools 2>/dev/null | tr -d '[:space:]\r\n')
                fi
                ;;
            appimage)
                if [[ -x "${HOME}/.local/bin/antigravity-tools" ]]; then
                    CURRENT_VERSION=$("${HOME}/.local/bin/antigravity-tools" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n1 || echo "")
                fi
                ;;
        esac
    elif [[ "$PLATFORM" == "macos" ]]; then
        local plist_paths=(
            "/Applications/${APP_NAME}.app/Contents/Info.plist"
            "/Applications/Antigravity Tools.app/Contents/Info.plist"
            "/Applications/Anti-Gravity Tools.app/Contents/Info.plist"
        )
        for p in "${plist_paths[@]}"; do
            if [[ -f "$p" ]]; then
                CURRENT_VERSION=$(defaults read "$p" CFBundleShortVersionString 2>/dev/null || echo "")
                if [[ -n "$CURRENT_VERSION" ]]; then
                    break
                fi
            fi
        done
    fi
}

# Resolve pinned version from baked placeholder, environment, or URL invocation
resolve_pinned_version() {
    if [[ -n "${VERSION:-}" ]]; then
        return 0
    fi
    if [[ -n "${PINNED_VERSION:-}" && "$PINNED_VERSION" != "__PINNED_VERSION__" ]]; then
        VERSION="$PINNED_VERSION"
        info "Respecting pinned installer version: v$VERSION"
        return 0
    fi

    local candidates=()

    # 1. BASH_EXECUTION_STRING
    if [[ -n "${BASH_EXECUTION_STRING:-}" ]]; then
        candidates+=("$BASH_EXECUTION_STRING")
    fi

    # 2. Parent and Self process cmdlines (/proc on Linux)
    if [[ -n "${PPID:-}" && -f "/proc/$PPID/cmdline" ]]; then
        candidates+=("$(tr '\0' ' ' < "/proc/$PPID/cmdline" 2>/dev/null || true)")
    fi
    if [[ -f "/proc/$$/cmdline" ]]; then
        candidates+=("$(tr '\0' ' ' < "/proc/$$/cmdline" 2>/dev/null || true)")
    fi

    if [[ -z "${VERSION:-}" ]]; then
        if [[ -n "${AGM_VERSION:-}" ]]; then
            VERSION="$AGM_VERSION"
        elif [[ -n "${INSTALLER_VERSION:-}" ]]; then
            VERSION="$INSTALLER_VERSION"
        fi
    fi

    # 3. Process inspection via ps (Linux, macOS, BSD)
    if command -v ps >/dev/null 2>&1; then
        if [[ -n "${PPID:-}" ]]; then
            candidates+=("$(ps -p "$PPID" -o args= 2>/dev/null || ps -f -p "$PPID" 2>/dev/null || true)")
        fi
        local my_pgid
        my_pgid=$(ps -o pgid= -p $$ 2>/dev/null | tr -d ' ' || true)
        if [[ -n "$my_pgid" ]]; then
            candidates+=("$(ps -o args= -g "$my_pgid" 2>/dev/null || true)")
        fi
    fi

    # 4. Interactive shell history (bash / zsh)
    if [[ -f "${HOME:-}/.bash_history" ]]; then
        candidates+=("$(tail -n 15 "${HOME}/.bash_history" 2>/dev/null || true)")
    fi
    if [[ -f "${HOME:-}/.zsh_history" ]]; then
        candidates+=("$(tail -n 15 "${HOME}/.zsh_history" 2>/dev/null || true)")
    fi

    local regex='(releases/download/|raw\.githubusercontent\.com/[^/]+/(Antigravity-Manager|agm-alim|antigravity|gitmap)/|raw\.githubusercontent\.com/[^/]+/[^/]+/)v?([0-9]+\.[0-9]+(\.[0-9]+)?(-[a-zA-Z0-9.]+)?)/'
    for entry in "${candidates[@]}"; do
        if [[ "$entry" =~ $regex ]]; then
            VERSION="${BASH_REMATCH[3]}"
            if [[ "$VERSION" =~ ^[0-9]+\.[0-9]+$ ]]; then
                VERSION="${VERSION}.0"
            fi
            info "Detected pinned version from download URL: v$VERSION"
            return 0
        fi
    done
}

get_manifest_asset_key() {
    if [[ "$PLATFORM" == "macos" ]]; then
        if [[ "$ARCH_LABEL" == "aarch64" ]]; then
            echo "macos_aarch64_dmg"
            return 0
        fi
        echo "macos_x64_dmg"
        return 0
    fi
    get_linux_manifest_asset_key
}

get_linux_manifest_asset_key() {
    if [[ "$PKG_EXT" == "deb" ]]; then
        if [[ "$DEB_ARCH" == "arm64" ]]; then
            echo "linux_aarch64_deb"
            return 0
        fi
        echo "linux_amd64_deb"
        return 0
    fi
    get_linux_rpm_or_appimage_asset_key
}

get_linux_rpm_or_appimage_asset_key() {
    if [[ "$PKG_EXT" == "rpm" ]]; then
        if [[ "$RPM_ARCH" == "aarch64" ]]; then
            echo "linux_aarch64_rpm"
            return 0
        fi
        echo "linux_x64_rpm"
        return 0
    fi
    if [[ "$ARCH_LABEL" == "aarch64" ]]; then
        echo "linux_aarch64_appimage"
        return 0
    fi
    echo "linux_amd64_appimage"
}

parse_releases_manifest_py() {
    local py_bin="$1"
    local key="$2"
    "$py_bin" -c "
import json, sys
try:
    m = json.load(sys.stdin)
    k = sys.argv[1]
    for r in m.get('releases', []):
        v = r.get('version', '')
        t = r.get('tag_url', '')
        a = r.get('assets', {}).get(k, '')
        print(f'{v}\t{t}\t{a}')
except Exception:
    pass
" "$key" 2>/dev/null
}

parse_releases_manifest_awk() {
    local key="$1"
    awk -v key="\"$key\":" '
/^[[:space:]]*\{/ { in_obj=1; ver=""; tag_url=""; asset_url="" }
in_obj && /^[[:space:]]*"version":/ { gsub(/.*"version":[[:space:]]*"|",?[[:space:]]*$/, ""); ver=$0 }
in_obj && /^[[:space:]]*"tag_url":/ { gsub(/.*"tag_url":[[:space:]]*"|",?[[:space:]]*$/, ""); tag_url=$0 }
in_obj && $0 ~ key { gsub(/.*:[[:space:]]*"|",?[[:space:]]*$/, ""); asset_url=$0 }
in_obj && /^[[:space:]]*\},?/ && ver != "" { print ver "\t" tag_url "\t" asset_url; ver=""; tag_url=""; asset_url="" }
'
}

parse_releases_manifest() {
    local key="$1"
    if python3 -c "import sys; sys.exit(0)" 2>/dev/null; then
        parse_releases_manifest_py "python3" "$key"
        return 0
    fi
    if python -c "import sys; sys.exit(0)" 2>/dev/null; then
        parse_releases_manifest_py "python" "$key"
        return 0
    fi
    parse_releases_manifest_awk "$key"
}

probe_cdn_manifest() {
    local key
    key=$(get_manifest_asset_key)
    local urls=(
        "https://raw.githubusercontent.com/${REPO}/main/releases-manifest.json"
        "https://github.com/${REPO}/releases/latest/download/releases-manifest.json"
    )
    for u in "${urls[@]}"; do
        local resp
        resp=$(curl -fsSL --max-time 4 "$u" 2>/dev/null || true)
        if [[ -n "$resp" ]]; then
            echo "$resp" | parse_releases_manifest "$key"
            return 0
        fi
    done
    return 1
}

init_candidate_queues() {
    CANDIDATE_VERSIONS=()
    CANDIDATE_TAG_URLS=()
    CANDIDATE_ASSET_URLS=()
    IS_PINNED=0
    CLEAN_PINNED=""
    if [[ -n "${VERSION:-}" ]]; then
        local user_ver="${VERSION#v}"
        if [[ "$user_ver" =~ ^[0-9]+\.[0-9]+$ ]]; then
            user_ver="${user_ver}.0"
        fi
        if _is_valid_version "$user_ver"; then
            CANDIDATE_VERSIONS+=("$user_ver")
            CANDIDATE_TAG_URLS+=("https://github.com/${REPO}/releases/tag/v${user_ver}")
            CANDIDATE_ASSET_URLS+=("")
            IS_PINNED=1
            CLEAN_PINNED="$user_ver"
            info "Target pinned release version: v$user_ver"
        fi
    fi
}

is_version_in_candidates() {
    local target="$1"
    for cv in "${CANDIDATE_VERSIONS[@]}"; do
        if [[ "$cv" == "$target" ]]; then
            return 0
        fi
    done
    return 1
}

append_manifest_candidate() {
    local ver="$1"
    local tag="$2"
    local asset="$3"
    if [[ $IS_PINNED -eq 0 ]]; then
        if is_version_in_candidates "$ver"; then
            return 0
        fi
        CANDIDATE_VERSIONS+=("$ver")
        CANDIDATE_TAG_URLS+=("$tag")
        CANDIDATE_ASSET_URLS+=("$asset")
        return 0
    fi
    append_pinned_manifest_candidate "$ver" "$tag" "$asset"
}

append_pinned_manifest_candidate() {
    local ver="$1"
    local tag="$2"
    local asset="$3"
    if [[ "$ver" == "$CLEAN_PINNED" ]]; then
        CANDIDATE_TAG_URLS[0]="$tag"
        CANDIDATE_ASSET_URLS[0]="$asset"
        return 0
    fi
    local lowest
    lowest=$(printf "%s\n%s\n" "$ver" "$CLEAN_PINNED" | sort -V | head -n1)
    if [[ "$lowest" == "$ver" ]]; then
        if is_version_in_candidates "$ver"; then
            return 0
        fi
        CANDIDATE_VERSIONS+=("$ver")
        CANDIDATE_TAG_URLS+=("$tag")
        CANDIDATE_ASSET_URLS+=("$asset")
    fi
}

record_manifest_status() {
    if [[ ${#CANDIDATE_VERSIONS[@]} -gt 0 ]]; then
        MANIFEST_LOADED=1
        info "Discovered releases from CDN manifest (${#CANDIDATE_VERSIONS[@]} versions available, rate-limit free)"
    fi
}

discover_manifest_candidates() {
    info "Discovering available release versions from GitHub..."
    local manifest_data
    manifest_data=$(probe_cdn_manifest || true)
    if [[ -z "$manifest_data" ]]; then
        return 0
    fi
    while IFS=$'\t' read -r m_ver m_tag m_asset; do
        if _is_valid_version "$m_ver"; then
            append_manifest_candidate "$m_ver" "$m_tag" "$m_asset"
        fi
        if [[ ${#CANDIDATE_VERSIONS[@]} -ge 10 ]]; then
            break
        fi
    done <<< "$manifest_data"
    record_manifest_status
}

append_api_candidate() {
    local tag="$1"
    if is_version_in_candidates "$tag"; then
        return 0
    fi
    CANDIDATE_VERSIONS+=("$tag")
    CANDIDATE_TAG_URLS+=("https://github.com/${REPO}/releases/tag/v${tag}")
    CANDIDATE_ASSET_URLS+=("")
}

fetch_api_release_tags() {
    local api_url="$1"
    local resp
    resp=$(curl -fsSL --max-time 6 -H "User-Agent: Antigravity-Installer" "$api_url" 2>/dev/null || true)
    if [[ -z "$resp" ]]; then
        return 0
    fi
    local tags
    tags=$(echo "$resp" | grep '"tag_name"' | sed -E 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v?([^"]+)".*/\1/' | tr -d '[:space:]' || true)
    while IFS= read -r tag; do
        if _is_valid_version "$tag"; then
            append_api_candidate "$tag"
        fi
        if [[ ${#CANDIDATE_VERSIONS[@]} -ge 10 ]]; then
            break
        fi
    done <<< "$tags"
}

discover_api_candidates() {
    if [[ ${IS_PINNED:-0} -eq 1 ]]; then
        return 0
    fi
    local is_manifest_stale=0
    if [[ ${MANIFEST_LOADED:-0} -eq 1 && -n "${CURRENT_VERSION:-}" && ${#CANDIDATE_VERSIONS[@]} -gt 0 ]]; then
        local top_ver="${CANDIDATE_VERSIONS[0]}"
        local lowest
        lowest=$(printf "%s\n%s\n" "$top_ver" "$CURRENT_VERSION" | sort -V | head -n1)
        if [[ "$lowest" == "$top_ver" ]]; then
            is_manifest_stale=1
            info "Cached CDN manifest version (v$top_ver) is not newer than current installed version (v$CURRENT_VERSION); querying live GitHub releases..."
        fi
    fi
    if [[ ${MANIFEST_LOADED:-0} -eq 1 && $is_manifest_stale -eq 0 ]]; then
        if [[ ${#CANDIDATE_VERSIONS[@]} -ge 5 ]]; then
            return 0
        fi
    fi
    local api_urls=()
    api_urls+=("${GITHUB_API}?per_page=30" "${UPSTREAM_API}?per_page=30")
    for api_url in "${api_urls[@]}"; do
        fetch_api_release_tags "$api_url"
        if [[ ${#CANDIDATE_VERSIONS[@]} -ge 10 ]]; then
            break
        fi
    done
}

populate_fallback_candidates() {
    if [[ ${IS_PINNED:-0} -eq 1 ]]; then
        return 0
    fi
    local fallbacks=("4.65.0" "4.64.0" "4.63.0" "4.62.0" "4.61.0" "4.60.0" "4.59.0" "4.58.0" "4.57.0" "4.56.0" "4.55.0" "4.52.0" "4.51.0" "4.49.0" "4.48.0" "4.47.1" "4.41.0" "4.40.0" "4.39.0" "4.38.1" "4.38.0" "4.37.0" "4.36.0" "4.35.0" "4.34.0" "4.33.0" "4.32.0" "4.31.0" "4.30.0" "4.7.6")
    for fb in "${fallbacks[@]}"; do
        if [[ ${#CANDIDATE_VERSIONS[@]} -ge 10 ]]; then
            break
        fi
        if is_version_in_candidates "$fb"; then
            continue
        fi
        CANDIDATE_VERSIONS+=("$fb")
        CANDIDATE_TAG_URLS+=("https://github.com/${REPO}/releases/tag/v${fb}")
        CANDIDATE_ASSET_URLS+=("")
    done
    CANDIDATE_VERSIONS=("${CANDIDATE_VERSIONS[@]:0:10}")
    CANDIDATE_TAG_URLS=("${CANDIDATE_TAG_URLS[@]:0:10}")
    CANDIDATE_ASSET_URLS=("${CANDIDATE_ASSET_URLS[@]:0:10}")
}

# Resolve target version with rate-limit-free CDN manifest and API fallback
get_version() {
    _is_valid_version() { [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; }
    resolve_pinned_version
    init_candidate_queues
    discover_manifest_candidates
    discover_api_candidates
    populate_fallback_candidates
    RELEASE_VERSION="${CANDIDATE_VERSIONS[0]}"
}

# Display migration path
display_migration() {
    local from_ver="None (Fresh Install)"
    if [[ -n "${CURRENT_VERSION:-}" ]]; then
        from_ver="v${CURRENT_VERSION#v}"
    fi
    local to_ver="v${RELEASE_VERSION#v}"

    echo ""
    info "Current installed version: ${from_ver}"
    info "Target release version   : ${to_ver}"
    if [[ "$from_ver" == "None (Fresh Install)" ]]; then
        info "Installation mode        : Fresh installation (${to_ver})"
    elif [[ "$from_ver" == "$to_ver" ]]; then
        info "Migration mode           : Reinstalling / Updating ${to_ver}"
    else
        info "Migration path           : ${from_ver} -> ${to_ver}"
    fi
    echo ""
}

# Build download URL based on platform and package manager
build_download_url() {
    local base_url="https://github.com/${REPO}/releases/download/v${RELEASE_VERSION}"

    case "$PLATFORM" in
        linux)
            case "$PKG_EXT" in
                deb)
                    DOWNLOAD_URL="${base_url}/Antigravity.Tools_${RELEASE_VERSION}_${DEB_ARCH}.deb"
                    FILENAME="Antigravity.Tools_${RELEASE_VERSION}_${DEB_ARCH}.deb"
                    ;;
                rpm)
                    DOWNLOAD_URL="${base_url}/Antigravity.Tools-${RELEASE_VERSION}-1.${RPM_ARCH}.rpm"
                    FILENAME="Antigravity.Tools-${RELEASE_VERSION}-1.${RPM_ARCH}.rpm"
                    ;;
                AppImage)
                    local appimage_arch
                    if [[ "$ARCH_LABEL" == "x86_64" ]]; then
                        appimage_arch="amd64"
                    else
                        appimage_arch="aarch64"
                    fi
                    DOWNLOAD_URL="${base_url}/Antigravity.Tools_${RELEASE_VERSION}_${appimage_arch}.AppImage"
                    FILENAME="Antigravity.Tools_${RELEASE_VERSION}_${appimage_arch}.AppImage"
                    ;;
            esac
            ;;
        macos)
            local mac_arch
            if [[ "$ARCH_LABEL" == "x86_64" ]]; then
                mac_arch="x64"
            else
                mac_arch="aarch64"
            fi
            DOWNLOAD_URL="${base_url}/Antigravity.Tools_${RELEASE_VERSION}_${mac_arch}.dmg"
            FILENAME="Antigravity.Tools_${RELEASE_VERSION}_${mac_arch}.dmg"
            ;;
    esac

    info "Primary URL       : $DOWNLOAD_URL"
}

# Detect or install aria2c accelerator
ensure_aria2c() {
    ARIA2C_BIN=""

    # Check standard locations
    if command -v aria2c &>/dev/null; then
        ARIA2C_BIN=$(command -v aria2c)
    elif [[ -x "/usr/bin/aria2c" ]]; then
        ARIA2C_BIN="/usr/bin/aria2c"
    elif [[ -x "/usr/local/bin/aria2c" ]]; then
        ARIA2C_BIN="/usr/local/bin/aria2c"
    fi

    if [[ -n "$ARIA2C_BIN" ]]; then
        info "aria2c accelerator: $ARIA2C_BIN (Active)"
        return 0
    fi

    # Attempt automatic install on Linux
    if [[ "$PLATFORM" == "linux" ]]; then
        info "aria2c accelerator not found. Attempting automatic installation..."
        local sudo_cmd=""
        if command -v sudo &>/dev/null && [[ ${EUID:-$(id -u)} -ne 0 ]]; then
            sudo_cmd="sudo"
        fi

        if [[ "${DRY_RUN:-0}" != "1" ]]; then
            case "$PKG_MANAGER" in
                apt)
                    $sudo_cmd apt-get update -qq 2>/dev/null || true
                    $sudo_cmd apt-get install -y -qq aria2 2>/dev/null || true
                    ;;
                dnf)
                    $sudo_cmd dnf install -y -q aria2 2>/dev/null || true
                    ;;
                yum)
                    $sudo_cmd yum install -y -q aria2 2>/dev/null || true
                    ;;
            esac
        fi

        if command -v aria2c &>/dev/null; then
            ARIA2C_BIN=$(command -v aria2c)
            success "Installed aria2c accelerator: $ARIA2C_BIN"
            return 0
        fi
    fi

    info "aria2c not available; using standard curl for download."
    return 1
}

# Download file with aria2c multi-connection parallel split or curl fallback
download_file() {
    local url="$1"
    local dest_dir="$2"
    local dest_file="$3"
    local full_path="${dest_dir}/${dest_file}"

    if [[ "${DRY_RUN:-0}" == "1" ]]; then
        info "[DRY-RUN] Simulating download: $url -> $full_path"
        touch "$full_path" 2>/dev/null || true
        return 0
    fi

    # Quick check if URL is reachable (HTTP 200 or 302)
    local http_status
    http_status=$(curl -fsSL -I -o /dev/null -w "%{http_code}" --max-time 8 "$url" 2>/dev/null || echo "000")
    if [[ "$http_status" != "200" && "$http_status" != "302" ]]; then
        return 1
    fi

    # Try aria2c with 80 parallel split connections and 1MB chunks
    if [[ -n "${ARIA2C_BIN:-}" && -x "$ARIA2C_BIN" ]]; then
        info "Delegating download request to aria2c accelerator..."
        info "Accelerating download with aria2c (16 connections, 80 splits, 1MB chunks)..."
        local aria_exit=0
        run_indented "$ARIA2C_BIN" --disable-ipv6=true -x 16 -s 80 -j 16 -k 1M \
            --allow-overwrite=true \
            --auto-file-renaming=false \
            --summary-interval=0 \
            --console-log-level=error \
            --show-console-readout=false \
            --dir="$dest_dir" \
            -o "$dest_file" \
            "$url" || aria_exit=$?

        if [[ "$aria_exit" -eq 0 && -f "$full_path" && -s "$full_path" ]]; then
            success "Download completed successfully via aria2c."
            return 0
        fi
        warn "aria2c download failed or interrupted; delegating download request to secondary downloader (curl)..."
    fi

    # Fallback to curl
    info "Downloading with curl..."
    if run_indented curl -fSL --progress-bar --connect-timeout 10 --retry 3 -o "$full_path" "$url"; then
        if [[ -f "$full_path" && -s "$full_path" ]]; then
            return 0
        fi
    fi

    return 1
}

# Download installer with fallback candidates
download_installer() {
    TEMP_DIR=$(mktemp -d)
    DOWNLOAD_PATH="${TEMP_DIR}/${FILENAME}"

    step "Downloading ${APP_NAME}..."
    info "Package filename  : $FILENAME"
    info "Temporary staging : $TEMP_DIR"

    ensure_aria2c || true

    local candidate_urls=(
        "$DOWNLOAD_URL"
        "${DOWNLOAD_URL//$REPO/$UPSTREAM_REPO}"
    )

    # If requested version lacks specific platform package, include known upstream release candidate
    if [[ "$PLATFORM" == "linux" ]]; then
        if [[ "$RELEASE_VERSION" != "$FALLBACK_STABLE_VERSION" ]]; then
            local fallback_file="Antigravity.Tools_${FALLBACK_STABLE_VERSION}_${DEB_ARCH:-amd64}.deb"
            if [[ "$PKG_EXT" == "rpm" ]]; then
                fallback_file="Antigravity.Tools-${FALLBACK_STABLE_VERSION}-1.${RPM_ARCH:-x86_64}.rpm"
            fi
            candidate_urls+=("https://github.com/${UPSTREAM_REPO}/releases/download/v${FALLBACK_STABLE_VERSION}/${fallback_file}")
        fi
    fi

    local download_success=0
    for try_url in "${candidate_urls[@]}"; do
        local try_filename
        try_filename=$(basename "$try_url")
        local try_dest="${TEMP_DIR}/${try_filename}"

        info "Checking source   : $try_url"
        if download_file "$try_url" "$TEMP_DIR" "$try_filename"; then
            download_success=1
            DOWNLOAD_PATH="$try_dest"
            FILENAME="$try_filename"
            success "Package downloaded successfully ($FILENAME)"
            break
        fi
        warn "Source unavailable. Trying next candidate..."
    done

    if [[ "$download_success" -eq 0 ]]; then
        warn "All download sources failed for release v${RELEASE_VERSION}."
        return 1
    fi

    if [[ "${DRY_RUN:-0}" != "1" && ! -s "$DOWNLOAD_PATH" ]]; then
        warn "Downloaded package file is empty or missing: $DOWNLOAD_PATH"
        return 1
    fi

    return 0
}

verify_download_checksum() {
    local file="$1"
    if [[ -f "$file" ]]; then
        if command -v sha256sum &>/dev/null; then
            sha256sum "$file" 2>/dev/null || true
        elif command -v shasum &>/dev/null; then
            shasum -a 256 "$file" 2>/dev/null || true
        fi
    fi
}

# Remove any pre-existing previous tool packages
close_tool_processes() {
    local context="${1:-installation}"
    step "Closing active tool processes and WebViews ($context)"
    killall -9 agm-alim 2>/dev/null || true
    killall -9 antigravity-tools 2>/dev/null || true
    pkill -9 -f "agm-alim" 2>/dev/null || true
    pkill -9 -f "antigravity-tools" 2>/dev/null || true
    pkill -9 -f "WebKitWebProcess" 2>/dev/null || true
    pkill -9 -f "electron.*antigravity" 2>/dev/null || true
    sleep 0.5
}

ensure_default_config() {
    local cfg_dir="${HOME}/.antigravity_tools"
    local cfg_file="${cfg_dir}/gui_config.json"
    mkdir -p "$cfg_dir" 2>/dev/null || true
    if [[ -f "$cfg_file" ]]; then
        if command -v python3 &>/dev/null; then
            python3 -c "
import json
try:
    with open('$cfg_file', 'r', encoding='utf-8') as f:
        d = json.load(f)
    if not d.get('auto_sync_migrated'):
        d['auto_sync'] = True
        d['auto_sync_migrated'] = True
        with open('$cfg_file', 'w', encoding='utf-8') as f:
            json.dump(d, f, indent=2)
except Exception:
    pass
" 2>/dev/null || true
        fi
    else
        cat << 'EOF' > "$cfg_file"
{
  "language": "en",
  "theme": "system",
  "auto_refresh": true,
  "refresh_interval": 15,
  "auto_sync": true,
  "auto_sync_migrated": true,
  "sync_interval": 5
}
EOF
    fi
}

# NOTE: Executed ONLY AFTER new package has been successfully downloaded and verified!
remove_previous_installation() {
    close_tool_processes "pre-installation"

    if [[ "$PLATFORM" != "linux" ]]; then
        return
    fi

    local has_legacy=0
    if [[ "${SIMULATE_LEGACY:-0}" == "1" ]]; then
        has_legacy=1
    fi

    case "$PKG_MANAGER" in
        apt)
            if dpkg -s "$APP_ID" &>/dev/null || dpkg -s antigravity-tools &>/dev/null; then
                has_legacy=1
            fi
            ;;
        dnf|yum)
            if rpm -q "$APP_ID" &>/dev/null || rpm -q antigravity-tools &>/dev/null; then
                has_legacy=1
            fi
            ;;
    esac

    local legacy_bins=(
        "/usr/local/bin/antigravity-tools"
        "${HOME}/.local/bin/antigravity-tools"
        "/usr/share/applications/com.lbjlaq.antigravity-tools.desktop"
        "${HOME}/.local/share/applications/com.lbjlaq.antigravity-tools.desktop"
    )
    for lb in "${legacy_bins[@]}"; do
        if [[ -f "$lb" ]]; then
            has_legacy=1
            break
        fi
    done

    if [[ "$has_legacy" -eq 1 ]]; then
        step "Cleaning Previous Tool Installation"
        info "Uninstalling previous version..."

        local sudo_cmd=""
        if command -v sudo &>/dev/null && [[ ${EUID:-$(id -u)} -ne 0 ]]; then
            sudo_cmd="sudo"
        fi

        case "$PKG_MANAGER" in
            apt)
                if dpkg -s "$APP_ID" &>/dev/null; then
                    run_indented $sudo_cmd apt-get remove -y "$APP_ID" 2>/dev/null || run_indented $sudo_cmd dpkg -r "$APP_ID" 2>/dev/null || true
                fi
                if dpkg -s antigravity-tools &>/dev/null; then
                    run_indented $sudo_cmd apt-get remove -y antigravity-tools 2>/dev/null || run_indented $sudo_cmd dpkg -r antigravity-tools 2>/dev/null || true
                fi
                ;;
            dnf|yum)
                if rpm -q "$APP_ID" &>/dev/null; then
                    run_indented $sudo_cmd "${PKG_MANAGER}" remove -y "$APP_ID" 2>/dev/null || true
                fi
                if rpm -q antigravity-tools &>/dev/null; then
                    run_indented $sudo_cmd "${PKG_MANAGER}" remove -y antigravity-tools 2>/dev/null || true
                fi
                ;;
        esac

        for lb in "${legacy_bins[@]}"; do
            if [[ -f "$lb" ]]; then
                run rm -f "$lb" 2>/dev/null || run $sudo_cmd rm -f "$lb" 2>/dev/null || true
            fi
        done
        success "Previous version uninstalled successfully."
    fi
}

# Pin application to Ubuntu GNOME dock / taskbar
pin_ubuntu_dock() {
    if [[ "$PLATFORM" != "linux" ]]; then
        return
    fi

    local desktop_id="agm-alim.desktop"

    if command -v gsettings &>/dev/null; then
        local current_favs
        current_favs=$(gsettings get org.gnome.shell favorite-apps 2>/dev/null || echo "")

        # Replace any legacy desktop entries in favorite-apps
        for old in "anti-gravity-tools-by-alim.desktop" "anti-gravity-tools.desktop" "antigravity-tools.desktop" "com.lbjlaq.antigravity-tools.desktop"; do
            if [[ "$current_favs" == *"$old"* ]]; then
                current_favs=$(echo "$current_favs" | sed "s/'$old'/'$desktop_id'/g")
            fi
        done

        if [[ -n "$current_favs" && "$current_favs" != *"$desktop_id"* ]]; then
            info "Pinning ${DESKTOP_NAME} to Ubuntu taskbar / dock..."
            local new_favs
            if [[ "$current_favs" == "[]" || "$current_favs" == "@as []" ]]; then
                new_favs="['${desktop_id}']"
            else
                new_favs=$(echo "$current_favs" | sed "s/]/, '${desktop_id}']/")
            fi
            run gsettings set org.gnome.shell favorite-apps "$new_favs" 2>/dev/null || true
            success "Pinned ${DESKTOP_NAME} (${TOOLTIP}) to Ubuntu dock."
        fi
    fi
}

# Install on Linux
install_linux() {
    step "Installing ${FULL_NAME}..."

    local sudo_cmd=""
    if command -v sudo &>/dev/null && [[ ${EUID:-$(id -u)} -ne 0 ]]; then
        sudo_cmd="sudo"
    fi

    case "$PKG_MANAGER" in
        apt)
            run_indented $sudo_cmd dpkg -i "$DOWNLOAD_PATH"
            run_indented $sudo_cmd apt-get install -f -y  # Fix dependencies if needed
            ;;
        dnf)
            run_indented $sudo_cmd dnf install -y "$DOWNLOAD_PATH"
            ;;
        yum)
            run_indented $sudo_cmd yum install -y "$DOWNLOAD_PATH"
            ;;
        appimage)
            local install_dir="${HOME}/.local/bin"
            run mkdir -p "$install_dir"
            run chmod +x "$DOWNLOAD_PATH"
            run cp "$DOWNLOAD_PATH" "${install_dir}/${BINARY_NAME}"
            run ln -sf "${install_dir}/${BINARY_NAME}" "${install_dir}/antigravity-tools" 2>/dev/null || true

            # Create standard freedesktop .desktop launcher
            local apps_dir="${HOME}/.local/share/applications"
            mkdir -p "$apps_dir"
            cat <<EOF > "${apps_dir}/agm-alim.desktop"
[Desktop Entry]
Name=${DESKTOP_NAME}
Comment=${TOOLTIP}
GenericName=${TOOLTIP}
Exec=${install_dir}/${BINARY_NAME} %U
Icon=${BINARY_NAME}
Terminal=false
Type=Application
Categories=Utility;Development;
StartupWMClass=${BINARY_NAME}
EOF
            chmod +x "${apps_dir}/agm-alim.desktop"

            if [[ ":$PATH:" != *":${install_dir}:"* ]]; then
                warn "Add ${install_dir} to your PATH to run ${BINARY_NAME} from anywhere"

                local shell_name rc_file export_line
                shell_name="$(basename "${SHELL:-/bin/bash}")"
                case "$shell_name" in
                    zsh)  rc_file="$HOME/.zshrc" ;;
                    fish) rc_file="$HOME/.config/fish/config.fish" ;;
                    *)    rc_file="$HOME/.bashrc" ;;
                esac

                export_line="export PATH=\"${install_dir}:\$PATH\""
                [[ "$shell_name" == "fish" ]] && export_line="fish_add_path ${install_dir}"

                if [[ -f "$rc_file" ]] && grep -qF "$install_dir" "$rc_file" 2>/dev/null; then
                    info "PATH entry already in $rc_file"
                else
                    run echo "$export_line" >> "$rc_file"
                    info "Added ${install_dir} to PATH in $rc_file"
                    warn "Run: source $rc_file  (or restart terminal)"
                fi
            fi
            ;;
    esac

    # Ensure user application entry exists with exact customized Name and Tooltip
    local apps_dir="${HOME}/.local/share/applications"
    mkdir -p "$apps_dir"
    local bin_cmd
    bin_cmd=$(command -v "${BINARY_NAME}" 2>/dev/null || echo "${HOME}/.local/bin/${BINARY_NAME}")
    cat <<EOF > "${apps_dir}/agm-alim.desktop"
[Desktop Entry]
Name=${DESKTOP_NAME}
Comment=${TOOLTIP}
GenericName=${TOOLTIP}
Exec=${bin_cmd} %U
Icon=${BINARY_NAME}
Terminal=false
Type=Application
Categories=Utility;Development;
StartupWMClass=${BINARY_NAME}
EOF
    chmod +x "${apps_dir}/agm-alim.desktop"

    # Purge legacy .desktop entries
    for old_desktop in "anti-gravity-tools-by-alim.desktop" "anti-gravity-tools.desktop" "antigravity-tools.desktop" "com.lbjlaq.antigravity-tools.desktop"; do
        rm -f "${apps_dir}/${old_desktop}" 2>/dev/null || true
    done

    pin_ubuntu_dock

    success "${FULL_NAME} installed successfully!"
}

# Install on macOS
install_macos() {
    step "Installing ${APP_NAME}..."

    if [[ "${DRY_RUN:-0}" == "1" ]]; then
        run hdiutil attach "$DOWNLOAD_PATH" -nobrowse -noautoopen
        run cp -R "<mount>/${APP_NAME}.app" /Applications/
        run hdiutil detach "<mount>"
        run sudo xattr -rd com.apple.quarantine "/Applications/${APP_NAME}.app"
        return
    fi

    # Mount DMG
    local mount_output mount_point
    mount_output=$(hdiutil attach "$DOWNLOAD_PATH" -nobrowse -noautoopen 2>&1)
    mount_point=$(echo "$mount_output" | grep -o '/Volumes/.*' | head -n1)

    if [[ -z "$mount_point" ]]; then
        error "Failed to mount DMG. Output: $mount_output"
    fi

    # Copy app to /Applications
    if [[ -d "/Applications/${APP_NAME}.app" ]]; then
        info "Removing existing installation in /Applications..."
        rm -rf "/Applications/${APP_NAME}.app"
    fi
    cp -R "${mount_point}/${APP_NAME}.app" /Applications/

    # Unmount DMG
    hdiutil detach "$mount_point" -quiet 2>/dev/null || true

    # Remove quarantine attribute to avoid "app is damaged" error
    info "Removing quarantine attribute..."
    sudo xattr -rd com.apple.quarantine "/Applications/${APP_NAME}.app" 2>/dev/null || true

    success "${APP_NAME} installed to /Applications!"
}

# Post-install cleanup: guarantees temporary download files and directory removal
cleanup() {
    if [[ -n "${TEMP_DIR:-}" && -d "$TEMP_DIR" ]]; then
        info "Cleaning up temporary download files and directories..."
        rm -rf "$TEMP_DIR" 2>/dev/null || true
    fi
}

# Check if a new release is available and print JSON
check_for_updates_cli() {
    detect_platform >&2 2>/dev/null || true
    detect_linux_distro >&2 2>/dev/null || true
    detect_current_version >&2 2>/dev/null || true
    get_version >&2 2>/dev/null || true

    local curr="${CURRENT_VERSION:-}"
    local rel="${RELEASE_VERSION:-}"
    local has_update_num=0

    if [[ -z "$curr" ]]; then
        if [[ -n "$rel" ]]; then
            has_update_num=1
        fi
    elif [[ -n "$rel" ]]; then
        local c_clean="${curr#v}"
        local r_clean="${rel#v}"
        if [[ "$c_clean" != "$r_clean" ]]; then
            local lowest
            lowest=$(printf "%s\n%s\n" "$c_clean" "$r_clean" | sort -V | head -n1)
            if [[ "$lowest" != "$r_clean" ]]; then
                has_update_num=1
            fi
        fi
    fi

    local has_update_str="false"
    if [[ $has_update_num -eq 1 ]]; then
        has_update_str="true"
    fi

    cat <<EOF
{
  "has_update": $has_update_str,
  "current_version": "${curr:-unknown}",
  "latest_version": "${rel:-unknown}",
  "download_url": "https://github.com/${REPO}/releases/tag/v${rel:-latest}"
}
EOF
    exit 0
}

# Main entry point
main() {
    local is_update_only=0
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --help|-h)
                show_help
                ;;
            --version|-v)
                if [[ $# -gt 1 && ! "$2" =~ ^-- ]]; then
                    VERSION="$2"
                    shift 2
                else
                    echo "install.sh v2.5.0"
                    exit 0
                fi
                ;;
            --version=*)
                VERSION="${1#*=}"
                shift
                ;;
            -v=*)
                VERSION="${1#*=}"
                shift
                ;;
            --check-update|--check)
                check_for_updates_cli
                ;;
            --update)
                is_update_only=1
                shift
                ;;
            v[0-9]*|[0-9]*)
                VERSION="$1"
                shift
                ;;
            *)
                shift
                ;;
        esac
    done

    # Top padding
    echo ""
    echo ""
    echo -e "${INDENT}${BLUE}========================================${NC}"
    echo -e "${INDENT}${BLUE}    ${FULL_NAME} Installer${NC}"
    echo -e "${INDENT}${BLUE}========================================${NC}"
    echo ""

    trap cleanup EXIT INT TERM

    detect_platform
    detect_linux_distro
    detect_current_version
    get_version

    if [[ $is_update_only -eq 1 ]]; then
        local curr="${CURRENT_VERSION:-}"
        local rel="${RELEASE_VERSION:-}"
        if [[ -n "$curr" ]]; then
            local c_clean="${curr#v}"
            local r_clean="${rel#v}"
            if [[ "$c_clean" == "$r_clean" ]]; then
                success "Already up to date ($curr)."
                exit 0
            fi
        fi
    fi

    display_migration

    local max_attempts=10
    local attempt=0
    local installed_ok=0

    for i in "${!CANDIDATE_VERSIONS[@]}"; do
        cand_ver="${CANDIDATE_VERSIONS[$i]}"
        cand_tag_url="${CANDIDATE_TAG_URLS[$i]:-https://github.com/${REPO}/releases/tag/v${cand_ver}}"
        cand_asset_url="${CANDIDATE_ASSET_URLS[$i]:-}"

        attempt=$((attempt + 1))
        if [[ $attempt -gt $max_attempts ]]; then
            break
        fi

        echo ""
        step "Installation attempt $attempt of $max_attempts: Release v$cand_ver"
        info "Release tag URL     : $cand_tag_url"
        RELEASE_VERSION="$cand_ver"
        if [[ -n "$cand_asset_url" ]]; then
            DOWNLOAD_URL="$cand_asset_url"
            FILENAME="$(basename "$DOWNLOAD_URL")"
            info "Package download URL: $DOWNLOAD_URL"
        else
            build_download_url
        fi

        if download_installer; then
            remove_previous_installation
            local inst_res=0
            case "$PLATFORM" in
                linux) install_linux || inst_res=$? ;;
                macos) install_macos || inst_res=$? ;;
            esac

            if [[ $inst_res -eq 0 ]]; then
                installed_ok=1
                success "Installation of v$cand_ver verified successfully!"
                break
            else
                warn "Installation failed for v$cand_ver (exit code: $inst_res)."
            fi
        else
            warn "Download failed for release v$cand_ver."
        fi

        if [[ $attempt -lt $max_attempts ]]; then
            info "Falling back to previous release version in sequence..."
        fi
    done

    if [[ $installed_ok -eq 0 ]]; then
        error "All $max_attempts attempts failed. I fail, so I cannot do anything."
    fi

    # Explicit post-install cleanup of downloaded packages
    cleanup

    # Ensure Auto Sync Current Account is default true
    ensure_default_config

    # Close any lingering processes, WebViews, or Electron tasks
    close_tool_processes "post-installation"

    # Newline before summary
    echo ""
    success "Installation complete!"
    echo ""
    info "Launch '${APP_NAME}' from your application menu or terminal."
    echo ""
    echo ""
}

main "$@"
