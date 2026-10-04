#!/usr/bin/env bash
# Antigravity Tools Install Script (Linux + macOS)
# Usage: curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
#
# Environment variables:
#   VERSION     - Install specific version (e.g., "4.7.6"), default: latest
#   DRY_RUN     - Set to "1" to print commands without executing

set -euo pipefail
set -E

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
APP_NAME="Antigravity Manager Tools"
FULL_NAME="Antigravity Manager Tools"
PUBLISHER="Maintained by Alim, Sponsored by RISEUP ASIA LLC"
APP_ID="com.lbjlaq.antigravity-tools"
BINARY_NAME="agm-alim"
DESKTOP_NAME="Antigravity Manager Tools"
TOOLTIP="Antigravity Manager Tools"
GITHUB_API="https://api.github.com/repos/${REPO}/releases"
FALLBACK_STABLE_VERSION="4.7.6"
PINNED_VERSION="__PINNED_VERSION__"

# Candidate queues — must be declared at global scope so that set -u never
# sees them as unbound when helper functions like is_version_in_candidates()
# expand "${CANDIDATE_VERSIONS[@]}" before init_candidate_queues() runs.
CANDIDATE_VERSIONS=()
CANDIDATE_TAG_URLS=()
CANDIDATE_ASSET_URLS=()
IS_PINNED=0
CLEAN_PINNED=""
MANIFEST_LOADED=0

# Helper functions with left indentation
info()    { echo -e "${INDENT}${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${INDENT}${GREEN}[OK]${NC} $1"; }
warn()    { echo -e "${INDENT}${YELLOW}[WARN]${NC} $1"; }
error()   {
    local msg="$1"
    local line_no="${2:-$LINENO}"
    echo -e "${INDENT}${RED}[ERROR]${NC} ${msg}" >&2
    report_error_stack "$line_no" "error(\"${msg}\")" 1
    exit 1
}
step()    { echo -e "\n${INDENT}${CYAN}==>${NC} ${BOLD}$1${NC}"; }

# Error stack trace trap for maximum transparency and failure diagnostics
report_error_stack() {
    local exit_code="${3:-$?}"
    if [[ "$exit_code" -eq 0 ]]; then
        exit_code=1
    fi
    local line_no="${1:-$LINENO}"
    local bash_cmd="${2:-${BASH_COMMAND:-unknown}}"
    echo -e "\n${INDENT}${RED}[ERROR] Command: ${bash_cmd} | Exit: ${exit_code} | Line: ${line_no}${NC}" >&2
    local sys_info
    sys_info="$(uname -srm 2>/dev/null || echo "unknown")"
    if command -v sw_vers &>/dev/null; then
        local mac_ver
        mac_ver="$(sw_vers -productVersion 2>/dev/null || true)"
        sys_info="macOS ${mac_ver} (${sys_info})"
    fi
    echo -e "${INDENT}${YELLOW}[SYSTEM CONTEXT] OS: ${sys_info} | Shell: ${SHELL:-bash} | Date: $(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date)${NC}" >&2
    if [[ ${#FUNCNAME[@]} -gt 1 ]]; then
        echo -e "${INDENT}${YELLOW}[STACK TRACE]${NC}" >&2
        for ((i = 1; i < ${#FUNCNAME[@]}; i++)); do
            local fn="${FUNCNAME[$i]}"
            local src="${BASH_SOURCE[$i]:-install.sh}"
            local ln="${BASH_LINENO[$((i - 1))]}"
            echo -e "${INDENT}  -> at ${fn}() in ${src}:${ln}" >&2
        done
    fi
}
trap 'report_error_stack "$LINENO" "$BASH_COMMAND"' ERR

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
            "${HOME}/Applications/${APP_NAME}.app/Contents/Info.plist"
            "/Applications/Antigravity Tools.app/Contents/Info.plist"
            "${HOME}/Applications/Antigravity Tools.app/Contents/Info.plist"
            "/Applications/Anti-Gravity Tools.app/Contents/Info.plist"
            "${HOME}/Applications/Anti-Gravity Tools.app/Contents/Info.plist"
            "/Applications/agm-alim.app/Contents/Info.plist"
            "${HOME}/Applications/agm-alim.app/Contents/Info.plist"
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

test_url_reachable() {
    local url="$1"
    local timeout="${2:-5}"
    if [[ -z "$url" ]]; then
        return 1
    fi
    local http_code
    http_code=$(curl -fsSL -I -o /dev/null -w "%{http_code}" --connect-timeout "$timeout" --max-time $((timeout + 2)) "$url" 2>/dev/null || echo "000")
    if [[ "$http_code" == "200" || "$http_code" == "302" || "$http_code" == "301" ]]; then
        return 0
    fi
    if [[ "$http_code" == "404" ]]; then
        return 1
    fi
    return 0
}

parse_github_releases_py() {
    local py_bin="$1"
    local platform="$2"
    local pkg_ext="$3"
    local arch="$4"
    "$py_bin" -c '
import json, sys, fnmatch

platform = sys.argv[1]
pkg_ext = sys.argv[2]
arch = sys.argv[3]

patterns = []
if platform == "macos":
    if arch in ("aarch64", "arm64"):
        patterns = ["*aarch64*.dmg", "*arm64*.dmg", "*universal*.dmg", "*.dmg", "*universal*.app.tar.gz"]
    else:
        patterns = ["*x64*.dmg", "*x86_64*.dmg", "*universal*.dmg", "*.dmg", "*universal*.app.tar.gz"]
elif platform == "linux":
    if pkg_ext == "deb":
        if arch in ("arm64", "aarch64"):
            patterns = ["*arm64*.deb", "*aarch64*.deb", "*.deb"]
        else:
            patterns = ["*amd64*.deb", "*x86_64*.deb", "*.deb"]
    elif pkg_ext == "rpm":
        if arch == "aarch64":
            patterns = ["*aarch64*.rpm", "*.rpm"]
        else:
            patterns = ["*x86_64*.rpm", "*x64*.rpm", "*.rpm"]
    else:
        if arch == "aarch64":
            patterns = ["*aarch64*.AppImage", "*arm64*.AppImage", "*.AppImage"]
        else:
            patterns = ["*amd64*.AppImage", "*x86_64*.AppImage", "*.AppImage"]

try:
    data = json.load(sys.stdin)
    items = data if isinstance(data, list) else [data]
    for rel in items:
        tag = rel.get("tag_name", "").lstrip("v")
        if not tag:
            continue
        html_url = rel.get("html_url", "")
        assets = rel.get("assets", [])
        matched_url = ""
        for pat in patterns:
            for a in assets:
                name = a.get("name", "")
                if fnmatch.fnmatch(name, pat):
                    matched_url = a.get("browser_download_url", "")
                    break
            if matched_url:
                break
        if matched_url:
            print(f"{tag}\t{html_url}\t{matched_url}")
except Exception:
    pass
' "$platform" "$pkg_ext" "$arch" 2>/dev/null
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
    local script_dir=""
    if [[ -n "${BASH_SOURCE[0]:-}" ]]; then
        script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd 2>/dev/null || true)"
    fi
    local urls=()
    if [[ -n "$script_dir" && -f "${script_dir}/releases-manifest.json" ]]; then
        urls+=("file://${script_dir}/releases-manifest.json")
    fi
    urls+=(
        "https://raw.githubusercontent.com/${REPO}/main/releases-manifest.json"
        "https://github.com/${REPO}/releases/latest/download/releases-manifest.json"
    )
    for u in "${urls[@]}"; do
        local resp=""
        if [[ "$u" == file://* ]]; then
            resp=$(cat "${u#file://}" 2>/dev/null || true)
        else
            resp=$(curl -fsSL --max-time 4 "$u" 2>/dev/null || true)
        fi
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
    if [[ -z "$asset" ]]; then
        return 0
    fi
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
    if [[ "$lowest" == "$ver" && -n "$asset" ]]; then
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
        if _is_valid_version "$m_ver" && [[ -n "$m_asset" ]]; then
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
    local tag_url="${2:-}"
    local asset_url="${3:-}"
    if is_version_in_candidates "$tag"; then
        return 0
    fi
    CANDIDATE_VERSIONS+=("$tag")
    CANDIDATE_TAG_URLS+=("${tag_url:-https://github.com/${REPO}/releases/tag/v${tag}}")
    CANDIDATE_ASSET_URLS+=("${asset_url:-}")
}

fetch_api_release_tags() {
    local api_url="$1"
    local resp
    resp=$(curl -fsSL --max-time 6 -H "User-Agent: Antigravity-Installer" "$api_url" 2>/dev/null || true)
    if [[ -z "$resp" ]]; then
        return 0
    fi

    local parsed_releases=""
    local arch_target="${ARCH_LABEL:-x86_64}"
    if [[ "$PLATFORM" == "linux" && "$PKG_EXT" == "deb" ]]; then
        arch_target="${DEB_ARCH:-amd64}"
    elif [[ "$PLATFORM" == "linux" && "$PKG_EXT" == "rpm" ]]; then
        arch_target="${RPM_ARCH:-x86_64}"
    fi

    if python3 -c "import sys; sys.exit(0)" 2>/dev/null; then
        parsed_releases=$(echo "$resp" | parse_github_releases_py "python3" "$PLATFORM" "${PKG_EXT:-appimage}" "$arch_target")
    elif python -c "import sys; sys.exit(0)" 2>/dev/null; then
        parsed_releases=$(echo "$resp" | parse_github_releases_py "python" "$PLATFORM" "${PKG_EXT:-appimage}" "$arch_target")
    fi

    if [[ -n "$parsed_releases" ]]; then
        while IFS=$'\t' read -r r_ver r_tag r_asset; do
            if [[ -n "$r_ver" && -n "$r_asset" ]]; then
                append_api_candidate "$r_ver" "$r_tag" "$r_asset"
            fi
            if [[ ${#CANDIDATE_VERSIONS[@]} -ge 10 ]]; then
                break
            fi
        done <<< "$parsed_releases"
        return 0
    fi

    # Fallback if Python is not present: extract tags and record candidate
    local tags
    tags=$(echo "$resp" | grep '"tag_name"' | sed -E 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v?([^"]+)".*/\1/' | tr -d '[:space:]' || true)
    while IFS= read -r tag; do
        if _is_valid_version "$tag"; then
            append_api_candidate "$tag" "" ""
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

    local old_versions=("${CANDIDATE_VERSIONS[@]}")
    local old_tag_urls=("${CANDIDATE_TAG_URLS[@]}")
    local old_asset_urls=("${CANDIDATE_ASSET_URLS[@]}")

    if [[ $is_manifest_stale -eq 1 ]]; then
        CANDIDATE_VERSIONS=()
        CANDIDATE_TAG_URLS=()
        CANDIDATE_ASSET_URLS=()
    fi

    local api_urls=()
    api_urls+=("${GITHUB_API}?per_page=30")
    for api_url in "${api_urls[@]}"; do
        fetch_api_release_tags "$api_url"
        if [[ ${#CANDIDATE_VERSIONS[@]} -ge 10 ]]; then
            break
        fi
    done

    if [[ $is_manifest_stale -eq 1 ]]; then
        for idx in "${!old_versions[@]}"; do
            if [[ ${#CANDIDATE_VERSIONS[@]} -ge 10 ]]; then
                break
            fi
            local ov="${old_versions[$idx]}"
            if ! is_version_in_candidates "$ov"; then
                CANDIDATE_VERSIONS+=("$ov")
                CANDIDATE_TAG_URLS+=("${old_tag_urls[$idx]}")
                CANDIDATE_ASSET_URLS+=("${old_asset_urls[$idx]}")
            fi
        done
    fi
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

build_candidate_download_urls() {
    local ver="$1"
    local base_url="https://github.com/${REPO}/releases/download/v${ver}"
    local urls=()

    case "$PLATFORM" in
        linux)
            case "$PKG_EXT" in
                deb)
                    urls+=(
                        "${base_url}/Antigravity.Manager.Tools_${ver}_${DEB_ARCH}.deb"
                        "${base_url}/Antigravity.Tools_${ver}_${DEB_ARCH}.deb"
                        "${base_url}/agm-alim_${ver}_${DEB_ARCH}.deb"
                        "${base_url}/agm-alim_${DEB_ARCH}.deb"
                    )
                    ;;
                rpm)
                    urls+=(
                        "${base_url}/Antigravity.Manager.Tools-${ver}-1.${RPM_ARCH}.rpm"
                        "${base_url}/Antigravity.Tools-${ver}-1.${RPM_ARCH}.rpm"
                        "${base_url}/agm-alim-${ver}-1.${RPM_ARCH}.rpm"
                        "${base_url}/agm-alim-1.${RPM_ARCH}.rpm"
                    )
                    ;;
                *)
                    local appimage_arch="amd64"
                    [[ "$ARCH_LABEL" == "aarch64" ]] && appimage_arch="aarch64"
                    urls+=(
                        "${base_url}/Antigravity.Manager.Tools_${ver}_${appimage_arch}.AppImage"
                        "${base_url}/Antigravity.Tools_${ver}_${appimage_arch}.AppImage"
                        "${base_url}/agm-alim_${ver}_${appimage_arch}.AppImage"
                        "${base_url}/agm-alim_${appimage_arch}.AppImage"
                    )
                    ;;
            esac
            ;;
        macos)
            local mac_arch="x64"
            [[ "$ARCH_LABEL" == "aarch64" ]] && mac_arch="aarch64"
            urls+=(
                "${base_url}/Antigravity.Manager.Tools_${ver}_${mac_arch}.dmg"
                "${base_url}/Antigravity.Tools_${ver}_${mac_arch}.dmg"
                "${base_url}/agm-alim_${ver}_${mac_arch}.dmg"
                "${base_url}/Antigravity_Manager_Tools_${ver}_ManualFix.dmg"
                "${base_url}/Antigravity.Manager.Tools_universal.app.tar.gz"
            )
            ;;
    esac

    echo "${urls[@]}"
}

# Build download URL based on platform and package manager
build_download_url() {
    local candidate_list=()
    if [[ -n "${cand_asset_url:-}" ]]; then
        candidate_list+=("$cand_asset_url")
    fi

    # Probe release API for tag to get exact asset URL if available
    local tag_url="https://api.github.com/repos/${REPO}/releases/tags/v${RELEASE_VERSION}"
    local api_res
    api_res=$(curl -fsSL --max-time 6 -H "User-Agent: Antigravity-Installer" "$tag_url" 2>/dev/null || true)
    if [[ -n "$api_res" ]]; then
        local arch_target="${ARCH_LABEL:-x86_64}"
        if [[ "$PLATFORM" == "linux" && "$PKG_EXT" == "deb" ]]; then
            arch_target="${DEB_ARCH:-amd64}"
        elif [[ "$PLATFORM" == "linux" && "$PKG_EXT" == "rpm" ]]; then
            arch_target="${RPM_ARCH:-x86_64}"
        fi
        local parsed=""
        if python3 -c "import sys; sys.exit(0)" 2>/dev/null; then
            parsed=$(echo "$api_res" | parse_github_releases_py "python3" "$PLATFORM" "${PKG_EXT:-appimage}" "$arch_target")
        elif python -c "import sys; sys.exit(0)" 2>/dev/null; then
            parsed=$(echo "$api_res" | parse_github_releases_py "python" "$PLATFORM" "${PKG_EXT:-appimage}" "$arch_target")
        fi
        if [[ -n "$parsed" ]]; then
            local p_ver p_tag p_asset
            IFS=$'\t' read -r p_ver p_tag p_asset <<< "$parsed"
            if [[ -n "$p_asset" ]]; then
                candidate_list+=("$p_asset")
            fi
        fi
    fi

    local fallback_urls
    fallback_urls=$(build_candidate_download_urls "$RELEASE_VERSION")
    for fu in $fallback_urls; do
        candidate_list+=("$fu")
    done

    DOWNLOAD_URL=""
    for cu in "${candidate_list[@]}"; do
        if test_url_reachable "$cu" 4; then
            DOWNLOAD_URL="$cu"
            FILENAME="$(basename "$DOWNLOAD_URL")"
            break
        fi
    done

    if [[ -z "$DOWNLOAD_URL" && ${#candidate_list[@]} -gt 0 ]]; then
        DOWNLOAD_URL="${candidate_list[0]}"
        FILENAME="$(basename "$DOWNLOAD_URL")"
    fi

    if [[ -n "$DOWNLOAD_URL" ]]; then
        info "Package download URL: $DOWNLOAD_URL"
    fi
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

download_installer() {
    if [[ -z "$DOWNLOAD_URL" ]]; then
        warn "Release v${RELEASE_VERSION} does not contain an installer package for ${PLATFORM} (${PKG_EXT:-dmg})."
        return 1
    fi

    if ! test_url_reachable "$DOWNLOAD_URL" 4; then
        warn "Release package for v${RELEASE_VERSION} is not found on GitHub ($DOWNLOAD_URL returned 404)."
        return 1
    fi

    TEMP_DIR=$(mktemp -d)
    DOWNLOAD_PATH="${TEMP_DIR}/${FILENAME}"

    step "Downloading ${APP_NAME}..."
    info "Package filename  : $FILENAME"
    info "Temporary staging : $TEMP_DIR"

    ensure_aria2c || true

    local candidate_urls=(
        "$DOWNLOAD_URL"
    )

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
    killall -9 "Antigravity Manager Tools" 2>/dev/null || true
    killall -9 "Antigravity Tools" 2>/dev/null || true
    pkill -9 -f "agm-alim" 2>/dev/null || true
    pkill -9 -f "antigravity-tools" 2>/dev/null || true
    pkill -9 -f "Antigravity Manager Tools" 2>/dev/null || true
    pkill -9 -f "Antigravity Tools" 2>/dev/null || true
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

# Detect Antigravity IDE installation
detect_ide_path() {
    info "Detecting Antigravity IDE installation..."
    local ide_path=""
    if command -v mdfind &>/dev/null; then
        ide_path=$(mdfind 'kMDItemCFBundleIdentifier == "com.google.antigravity"' 2>/dev/null | head -n1)
        if [[ -z "$ide_path" ]]; then
            ide_path=$(mdfind "kMDItemCFBundleIdentifier == 'com.antigravity.ide'" 2>/dev/null | head -n1)
        fi
    fi
    if [[ -z "$ide_path" ]]; then
        local candidates=(
            "/Applications/Antigravity.app"
            "${HOME}/Applications/Antigravity.app"
        )
        for candidate in "${candidates[@]}"; do
            if [[ -d "$candidate" ]]; then
                ide_path="$candidate"
                break
            fi
        done
    fi
    if [[ -n "$ide_path" ]]; then
        success "Detected Antigravity IDE: $ide_path"
    else
        info "Antigravity IDE not found in standard paths (will be detected on first launch)."
    fi
}

# Install on macOS
install_macos() {
    step "Installing ${APP_NAME}..."

    local macos_major=0
    if command -v sw_vers &>/dev/null; then
        macos_major=$(sw_vers -productVersion | cut -d. -f1)
    fi

    if [[ "${DRY_RUN:-0}" == "1" ]]; then
        info "[DRY-RUN] Clearing quarantine attributes from DMG: $DOWNLOAD_PATH"
        run xattr -cr "$DOWNLOAD_PATH"
        run xattr -r -d com.apple.quarantine "$DOWNLOAD_PATH"
        run hdiutil attach "$DOWNLOAD_PATH" -nobrowse -noautoopen
        run cp -R "<mount>/*.app" /Applications/
        run hdiutil detach "<mount>" -force -quiet
        run xattr -cr "/Applications/${APP_NAME}.app"
        run xattr -d com.apple.quarantine "/Applications/${APP_NAME}.app"
        run find "/Applications/${APP_NAME}.app" -exec xattr -d com.apple.quarantine {} +
        if [[ "$macos_major" -ge 13 ]]; then
            run spctl --add "/Applications/${APP_NAME}.app"
        fi
        if [[ "$macos_major" -lt 15 ]]; then
            run codesign --force --deep --sign - "/Applications/${APP_NAME}.app"
        fi
        run ln -sf "/Applications/${APP_NAME}.app/Contents/MacOS/agm" "${HOME}/.local/bin/agm"
        run ln -sf "/Applications/${APP_NAME}.app/Contents/MacOS/agm-alim" "${HOME}/.local/bin/agm-alim"
        return 0
    fi

    # 0. Clean up active processes and stale background LaunchAgents
    info "Terminating active processes and checking for stale LaunchAgents..."
    killall "agm-alim" 2>/dev/null || true
    killall "${APP_NAME}" 2>/dev/null || true
    killall "Antigravity Tools" 2>/dev/null || true
    killall "agm" 2>/dev/null || true

    local launch_agent_plist="${HOME}/Library/LaunchAgents/${APP_ID}.plist"
    if [[ -f "$launch_agent_plist" ]]; then
        info "Unloading stale LaunchAgent: $launch_agent_plist"
        launchctl bootout "gui/$(id -u 2>/dev/null || echo 501)" "$launch_agent_plist" 2>/dev/null || true
        launchctl unload "$launch_agent_plist" 2>/dev/null || true
        rm -f "$launch_agent_plist" 2>/dev/null || true
    fi

    # 0.1 Clean up any trashed application bundles in ~/.Trash/ that poison LaunchServices
    info "Inspecting Trash for stale or conflicting application bundles..."
    local lsregister_bin=""
    local lsregister_candidates=(
        "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
        "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister"
        "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister"
        "/System/Library/Frameworks/CoreServices.framework/Versions/Current/Frameworks/LaunchServices.framework/Versions/Current/Support/lsregister"
    )
    for cand in "${lsregister_candidates[@]}"; do
        if [[ -x "$cand" ]]; then
            lsregister_bin="$cand"
            break
        fi
    done
    if [[ -z "$lsregister_bin" ]]; then
        lsregister_bin=$(find /System/Library/Frameworks/CoreServices.framework -name "lsregister" -type f 2>/dev/null | head -n1 || true)
    fi

    local trashed_apps=()
    while IFS= read -r t_app; do
        if [[ -n "$t_app" && -e "$t_app" ]]; then
            trashed_apps+=("$t_app")
        fi
    done < <(find "${HOME}/.Trash" -maxdepth 1 \( -iname "*antigravity*.app" -o -iname "*agm*.app" -o -iname "*Antigravity Manager Tools*" -o -iname "*Antigravity*" \) 2>/dev/null || true)

    # Check for stale registrations in LaunchServices dump pointing to Trash
    if [[ -n "$lsregister_bin" && -x "$lsregister_bin" ]]; then
        while IFS= read -r ls_path; do
            if [[ -n "$ls_path" && ("$ls_path" == *".Trash"* || "$ls_path" == *".Trashes"*) ]]; then
                "$lsregister_bin" -u "$ls_path" 2>/dev/null || true
                trashed_apps+=("$ls_path")
            fi
        done < <("$lsregister_bin" -dump 2>/dev/null | awk '
            BEGIN { RS = "--------------------------------------------------------------------------------"; FS = "\n" }
            /com\.lbjlaq\.antigravity-tools|[Aa]ntigravity|[Aa]gm/ {
                for (i = 1; i <= NF; i++) {
                    if ($i ~ /^[[:space:]]*path:[[:space:]]*/) {
                        p = $i
                        sub(/^[[:space:]]*path:[[:space:]]*/, "", p)
                        sub(/[[:space:]]*\(0x[0-9a-fA-F]+\)[[:space:]]*$/, "", p)
                        if (length(p) > 0) print p
                    }
                }
            }
        ' || true)
    fi

    # Bypass TCC: use AppleScript to move matching trashed items to /private/tmp and remove
    rm -rf /private/tmp/*[Aa]ntigravity* /private/tmp/*[Aa]gm* /tmp/*[Aa]ntigravity* /tmp/*[Aa]gm* 2>/dev/null || true
    osascript -e '
    tell application "Finder"
        try
            set destFolder to (POSIX file "/private/tmp") as alias
            repeat with anItem in (every item of trash)
                try
                    set n to name of anItem as text
                    if n contains "Antigravity" or n contains "agm" then
                        move anItem to destFolder with replacing
                    end if
                end try
            end repeat
        end try
    end tell' 2>/dev/null || true
    rm -rf /private/tmp/*[Aa]ntigravity* /private/tmp/*[Aa]gm* /tmp/*[Aa]ntigravity* /tmp/*[Aa]gm* 2>/dev/null || true

    if [[ ${#trashed_apps[@]} -gt 0 ]]; then
        info "Found ${#trashed_apps[@]} stale application bundle(s) in Trash poisoning LaunchServices; purging..."
        for ta in "${trashed_apps[@]}"; do
            info "  Purging trashed bundle: $(basename "$ta")"
            if [[ -n "$lsregister_bin" ]]; then
                "$lsregister_bin" -u "$ta" 2>/dev/null || true
            fi
            chflags -R nouchg,noschg "$ta" 2>/dev/null || true
            rm -rf "$ta" 2>/dev/null || true
        done
    fi

    if [[ -n "$lsregister_bin" ]]; then
        "$lsregister_bin" -gc -R -v -apps u,s,l 2>/dev/null || "$lsregister_bin" -gc 2>/dev/null || true
    fi

    # 1. Remove quarantine from downloaded DMG
    info "Clearing quarantine attributes from downloaded package..."
    xattr -cr "$DOWNLOAD_PATH" 2>/dev/null || true
    xattr -r -d com.apple.quarantine "$DOWNLOAD_PATH" 2>/dev/null || true

    # 2. Mount DMG cleanly
    info "Mounting disk image ($DOWNLOAD_PATH)..."
    local mount_output mount_point
    mount_output=$(hdiutil attach "$DOWNLOAD_PATH" -nobrowse -noautoopen 2>&1) || {
        local hdi_exit="$?"
        error "Failed to mount DMG (exit code: $hdi_exit). Output: $mount_output"
    }
    mount_point=$(echo "$mount_output" | awk '/\/Volumes\// { for(i=1;i<=NF;i++) if($i ~ /^\/Volumes\//) { print $i; exit } }')

    if [[ -z "$mount_point" || ! -d "$mount_point" ]]; then
        error "Failed to locate mounted volume from DMG. Output: $mount_output"
    fi

    # 3. Dynamic discovery of .app bundle inside $mount_point
    local source_app
    source_app=$(find "$mount_point" -maxdepth 2 -name "*.app" -type d 2>/dev/null | head -n1)
    if [[ -z "$source_app" || ! -d "$source_app" ]]; then
        hdiutil detach "$mount_point" -force -quiet 2>/dev/null || true
        error "No .app bundle discovered inside mounted DMG at $mount_point"
    fi

    local app_bundle_name
    app_bundle_name="$(basename "$source_app")"
    info "Discovered application bundle: $app_bundle_name"

    # 4. Destination permissions: Try /Applications/, fall back to $HOME/Applications/
    local dest_dir="/Applications"
    if [[ ! -d "$dest_dir" || ! -w "$dest_dir" ]]; then
        info "/Applications is not writable; falling back to ${HOME}/Applications..."
        dest_dir="${HOME}/Applications"
        mkdir -p "$dest_dir" 2>/dev/null || true
    fi

    local target_app="${dest_dir}/${app_bundle_name}"

    # 5. Remove any existing app at target destination before copying
    if [[ -d "$target_app" ]]; then
        info "Removing existing installation at $target_app..."
        rm -rf "$target_app" 2>/dev/null || true
        if [[ -d "$target_app" && "$dest_dir" != "${HOME}/Applications" ]]; then
            warn "Unable to remove existing $target_app in $dest_dir (permission denied). Falling back to ${HOME}/Applications..."
            dest_dir="${HOME}/Applications"
            mkdir -p "$dest_dir" 2>/dev/null || true
            target_app="${dest_dir}/${app_bundle_name}"
            rm -rf "$target_app" 2>/dev/null || true
        fi
    fi

    # Also clean up any legacy application bundle names in target destination
    for legacy_name in "Antigravity Tools.app" "Anti-Gravity Tools.app" "agm-alim.app"; do
        local legacy_path="${dest_dir}/${legacy_name}"
        if [[ -d "$legacy_path" && "$legacy_path" != "$target_app" ]]; then
            info "Removing legacy bundle at $legacy_path..."
            rm -rf "$legacy_path" 2>/dev/null || true
        fi
    done

    # 6. Copy discovered .app bundle to destination directory
    info "Copying $app_bundle_name to $dest_dir..."
    if ! cp -R "$source_app" "$dest_dir/" 2>/dev/null; then
        if [[ "$dest_dir" != "${HOME}/Applications" ]]; then
            warn "Permission denied copying to $dest_dir. Falling back to ${HOME}/Applications..."
            dest_dir="${HOME}/Applications"
            mkdir -p "$dest_dir" 2>/dev/null || true
            target_app="${dest_dir}/${app_bundle_name}"
            rm -rf "$target_app" 2>/dev/null || true
            cp -R "$source_app" "$dest_dir/"
        else
            error "Failed to copy $source_app to $dest_dir"
        fi
    fi

    chmod -R u+rwX "$target_app" 2>/dev/null || true
    if [[ -d "$target_app/Contents/MacOS" ]]; then
        chmod -R +x "$target_app/Contents/MacOS" 2>/dev/null || true
    fi

    # 7. Unmount DMG volume cleanly
    info "Unmounting disk image..."
    hdiutil detach "$mount_point" -force -quiet 2>/dev/null || true

    # 8. Strip Gatekeeper quarantine on target app without requiring sudo
    info "Stripping Gatekeeper quarantine attributes from $target_app..."
    xattr -cr "$target_app" 2>/dev/null || true
    xattr -rd com.apple.quarantine "$target_app" 2>/dev/null || true
    xattr -rd com.apple.provenance "$target_app" 2>/dev/null || true
    find "$target_app" -exec xattr -c {} + 2>/dev/null || true
    find "$target_app" -exec xattr -d com.apple.quarantine {} + 2>/dev/null || true

    # 9. Register with Gatekeeper assessment subsystem (spctl) on macOS 13+ (Ventura+)
    if [[ "$macos_major" -ge 13 ]] && command -v spctl &>/dev/null; then
        info "Registering application with Gatekeeper assessment subsystem (spctl)..."
        spctl --add "$target_app" 2>/dev/null || true
    fi

    # 10. Apply ad-hoc local code signature if codesign is present
    if command -v codesign &>/dev/null; then
        info "Applying ad-hoc code signature..."
        codesign --force --deep --sign - "$target_app" 2>/dev/null || true
    fi

    # 11. Refresh LaunchServices database registration: force re-register and reload daemon across all domains
    if [[ -n "$lsregister_bin" ]]; then
        info "Registering application with LaunchServices..."
        "$lsregister_bin" -u "$target_app" 2>/dev/null || true
        "$lsregister_bin" -gc -R -v -apps u,s,l 2>/dev/null || "$lsregister_bin" -gc 2>/dev/null || true
        "$lsregister_bin" -f -r "$target_app" 2>/dev/null || true
        killall Finder Dock 2>/dev/null || true
        sleep 1
    fi

    # 12. CLI symlink: Check for agm or agm-alim binary inside Contents/MacOS/
    local macos_bin_dir="${target_app}/Contents/MacOS"
    local cli_bin=""
    if [[ -d "$macos_bin_dir" ]]; then
        for candidate_name in "agm" "agm-alim" "${BINARY_NAME}" "antigravity-tools"; do
            if [[ -x "${macos_bin_dir}/${candidate_name}" ]]; then
                cli_bin="${macos_bin_dir}/${candidate_name}"
                break
            fi
        done
        # Fallback to main app binary if specific name not found
        if [[ -z "$cli_bin" ]]; then
            for f in "${macos_bin_dir}"/*; do
                if [[ -f "$f" && -x "$f" ]]; then
                    cli_bin="$f"
                    break
                fi
            done
        fi
    fi

    if [[ -n "$cli_bin" && -f "$cli_bin" ]]; then
        chmod +x "$cli_bin" 2>/dev/null || true
        local user_bin="${HOME}/.local/bin"
        mkdir -p "$user_bin" 2>/dev/null || true
        ln -sf "$cli_bin" "${user_bin}/agm" 2>/dev/null || true
        ln -sf "$cli_bin" "${user_bin}/agm-alim" 2>/dev/null || true
        info "CLI binary symlinked to ${user_bin}/agm and ${user_bin}/agm-alim"

        # Ensure $HOME/.local/bin is in PATH in shell rc files (.zshrc, .bashrc, .bash_profile)
        local shell_rcs=()
        [[ -f "$HOME/.zshrc" ]] && shell_rcs+=("$HOME/.zshrc")
        [[ -f "$HOME/.bashrc" ]] && shell_rcs+=("$HOME/.bashrc")
        [[ -f "$HOME/.bash_profile" ]] && shell_rcs+=("$HOME/.bash_profile")
        if [[ ${#shell_rcs[@]} -eq 0 ]]; then
            shell_rcs+=("$HOME/.zshrc")
        fi

        for rc in "${shell_rcs[@]}"; do
            if [[ -f "$rc" ]] && grep -qF "${user_bin}" "$rc" 2>/dev/null; then
                continue
            fi
            echo "" >> "$rc" 2>/dev/null || true
            echo "export PATH=\"${user_bin}:\$PATH\"" >> "$rc" 2>/dev/null || true
            info "Added ${user_bin} to PATH in $rc"
        done

        if [[ ":$PATH:" != *":${user_bin}:"* ]]; then
            warn "To use 'agm' immediately in this terminal, run: export PATH=\"${user_bin}:\$PATH\""
        fi
    fi

    # 13. Success report with clear next steps
    success "${APP_NAME} installed to ${target_app}!"
    echo ""
    info "Next steps:"
    info "  1. You can launch '${app_bundle_name%.app}' from ${dest_dir}"
    info "  2. Or run 'agm' / 'agm-alim' in terminal"

    detect_ide_path
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
            --dry-run)
                DRY_RUN=1
                shift
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

    trap 'report_error_stack "$LINENO"' ERR
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

        if [[ -z "$DOWNLOAD_URL" ]] || ! test_url_reachable "$DOWNLOAD_URL" 4; then
            warn "Release v$cand_ver does not contain an accessible installer package for ${PLATFORM} (${PKG_EXT:-dmg}). Skipping to next candidate..."
            continue
        fi

        if [[ "${DRY_RUN:-0}" == "1" ]]; then
            warn "[DRY-RUN] Would download $DOWNLOAD_URL"
            warn "[DRY-RUN] Would execute/install for ${PLATFORM} (${PKG_EXT:-dmg})"
            success "[DRY-RUN] Dry run completed successfully."
            exit 0
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

if [[ "${BASH_SOURCE[0]:-$0}" == "$0" ]]; then
    main "$@"
fi
