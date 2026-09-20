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
APP_NAME="Agm Tool By Alim"
FULL_NAME="Antigravity Manager Tools By Alim"
APP_ID="com.lbjlaq.antigravity-tools"
BINARY_NAME="agm-alim"
DESKTOP_NAME="Agm - Alim"
TOOLTIP="Antigravity Manager Tool By Alim"
GITHUB_API="https://api.github.com/repos/${REPO}/releases"
UPSTREAM_API="https://api.github.com/repos/${UPSTREAM_REPO}/releases"
FALLBACK_STABLE_VERSION="4.7.6"

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

# Resolve target version
get_version() {
    _is_valid_version() { [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; }

    if [[ -n "${VERSION:-}" ]]; then
        RELEASE_VERSION="${VERSION#v}"
        info "Target version (user specified): v$RELEASE_VERSION"
        return
    fi

    info "Discovering latest release version from GitHub..."

    # Method 1: GitHub API on primary repo
    local response
    if response=$(curl -fsSL --max-time 8 -H "User-Agent: Antigravity-Installer" "${GITHUB_API}/latest" 2>/dev/null); then
        RELEASE_VERSION=$(echo "$response" | grep '"tag_name"' | head -n1 | sed -E 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v?([^"]+)".*/\1/' | tr -d '[:space:]\r\n')
        if _is_valid_version "${RELEASE_VERSION:-}"; then
            info "Latest version (primary repo): v$RELEASE_VERSION"
            return
        fi
    fi

    # Method 2: Upstream GitHub API
    if response=$(curl -fsSL --max-time 8 -H "User-Agent: Antigravity-Installer" "${UPSTREAM_API}/latest" 2>/dev/null); then
        RELEASE_VERSION=$(echo "$response" | grep '"tag_name"' | head -n1 | sed -E 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v?([^"]+)".*/\1/' | tr -d '[:space:]\r\n')
        if _is_valid_version "${RELEASE_VERSION:-}"; then
            info "Latest version (upstream repo): v$RELEASE_VERSION"
            return
        fi
    fi

    # Method 3: Primary GitHub Releases redirect
    local final_url
    final_url=$(curl -fsSL --max-time 8 -o /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest" 2>/dev/null | tr -d '[:space:]\r\n')
    if [[ -n "$final_url" && "$final_url" =~ /tag/v?([0-9]+\.[0-9]+\.[0-9]+) ]]; then
        RELEASE_VERSION="${BASH_REMATCH[1]}"
        info "Latest version (from redirect): v$RELEASE_VERSION"
        return
    fi

    # Method 4: Upstream GitHub Releases redirect
    final_url=$(curl -fsSL --max-time 8 -o /dev/null -w '%{url_effective}' "https://github.com/${UPSTREAM_REPO}/releases/latest" 2>/dev/null | tr -d '[:space:]\r\n')
    if [[ -n "$final_url" && "$final_url" =~ /tag/v?([0-9]+\.[0-9]+\.[0-9]+) ]]; then
        RELEASE_VERSION="${BASH_REMATCH[1]}"
        info "Latest version (from upstream redirect): v$RELEASE_VERSION"
        return
    fi

    # Method 5: Upstream updater.json
    local updater_ver
    updater_ver=$(curl -fsSL --max-time 6 "https://github.com/${UPSTREAM_REPO}/releases/latest/download/updater.json" 2>/dev/null | grep '"version"' | head -n1 | sed -E 's/.*"version"[[:space:]]*:[[:space:]]*"v?([^"]+)".*/\1/' | tr -d '[:space:]\r\n')
    if _is_valid_version "${updater_ver:-}"; then
        RELEASE_VERSION="$updater_ver"
        info "Latest version (from updater.json): v$RELEASE_VERSION"
        return
    fi

    # Fallback to known stable release with assets
    RELEASE_VERSION="$FALLBACK_STABLE_VERSION"
    warn "Could not resolve latest release dynamically, falling back to v${RELEASE_VERSION}"
}

# Display migration path
display_migration() {
    local from_ver="None (Fresh Install)"
    if [[ -n "${CURRENT_VERSION:-}" ]]; then
        from_ver="v${CURRENT_VERSION#v}"
    fi
    local to_ver="v${RELEASE_VERSION#v}"

    echo ""
    info "Current Version   : ${from_ver}"
    info "Target Version    : ${to_ver}"
    info "Migration Path    : ${from_ver} -> ${to_ver}"
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

    # Try aria2c with 16 split parallel connections
    if [[ -n "${ARIA2C_BIN:-}" && -x "$ARIA2C_BIN" ]]; then
        info "Downloading with aria2c (16 parallel split connections)..."
        if "$ARIA2C_BIN" -x 16 -s 16 -j 16 -k 1M \
            --allow-overwrite=true \
            --auto-file-renaming=false \
            --summary-interval=1 \
            --console-log-level=warn \
            --dir="$dest_dir" \
            -o "$dest_file" \
            "$url"; then
            if [[ -f "$full_path" && -s "$full_path" ]]; then
                return 0
            fi
        fi
        warn "aria2c download failed or interrupted, falling back to curl..."
    fi

    # Fallback to curl
    info "Downloading with curl..."
    if curl -fSL --progress-bar -o "$full_path" "$url"; then
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
    if [[ "$RELEASE_VERSION" != "$FALLBACK_STABLE_VERSION" && "$PLATFORM" == "linux" ]]; then
        local fallback_file="Antigravity.Tools_${FALLBACK_STABLE_VERSION}_${DEB_ARCH:-amd64}.deb"
        if [[ "$PKG_EXT" == "rpm" ]]; then
            fallback_file="Antigravity.Tools-${FALLBACK_STABLE_VERSION}-1.${RPM_ARCH:-x86_64}.rpm"
        fi
        candidate_urls+=("https://github.com/${UPSTREAM_REPO}/releases/download/v${FALLBACK_STABLE_VERSION}/${fallback_file}")
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
        error "All download candidates failed. Please verify your network connection or specify a version: VERSION=4.7.6 curl -fsSL ... | bash"
    fi

    if [[ "${DRY_RUN:-0}" != "1" && ! -s "$DOWNLOAD_PATH" ]]; then
        error "Downloaded package file is empty or missing: $DOWNLOAD_PATH"
    fi
}

# Remove any pre-existing lbjlaq/Antigravity-Manager or legacy packages
# NOTE: Executed ONLY AFTER new package has been successfully downloaded and verified!
remove_legacy_upstream_installation() {
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
        step "Handling Legacy Installation"
        info "Uninstalling the old tool (by original developer)..."

        local sudo_cmd=""
        if command -v sudo &>/dev/null && [[ ${EUID:-$(id -u)} -ne 0 ]]; then
            sudo_cmd="sudo"
        fi

        case "$PKG_MANAGER" in
            apt)
                if dpkg -s "$APP_ID" &>/dev/null; then
                    run $sudo_cmd apt-get remove -y "$APP_ID" 2>/dev/null || run $sudo_cmd dpkg -r "$APP_ID" 2>/dev/null || true
                fi
                if dpkg -s antigravity-tools &>/dev/null; then
                    run $sudo_cmd apt-get remove -y antigravity-tools 2>/dev/null || run $sudo_cmd dpkg -r antigravity-tools 2>/dev/null || true
                fi
                ;;
            dnf|yum)
                if rpm -q "$APP_ID" &>/dev/null; then
                    run $sudo_cmd "${PKG_MANAGER}" remove -y "$APP_ID" 2>/dev/null || true
                fi
                if rpm -q antigravity-tools &>/dev/null; then
                    run $sudo_cmd "${PKG_MANAGER}" remove -y antigravity-tools 2>/dev/null || true
                fi
                ;;
        esac

        for lb in "${legacy_bins[@]}"; do
            if [[ -f "$lb" ]]; then
                run rm -f "$lb" 2>/dev/null || run $sudo_cmd rm -f "$lb" 2>/dev/null || true
            fi
        done
        success "Old tool uninstalled successfully."
    fi
}

# Pin application to Ubuntu GNOME dock / taskbar
pin_ubuntu_dock() {
    if [[ "$PLATFORM" != "linux" ]]; then
        return
    fi

    local desktop_id=""
    local candidates=(
        "agm-alim.desktop"
        "anti-gravity-tools-by-alim.desktop"
        "anti-gravity-tools.desktop"
        "antigravity-tools.desktop"
        "com.lbjlaq.antigravity-tools.desktop"
    )

    for c in "${candidates[@]}"; do
        if [[ -f "/usr/share/applications/$c" || -f "${HOME}/.local/share/applications/$c" ]]; then
            desktop_id="$c"
            break
        fi
    done

    if [[ -z "$desktop_id" ]]; then
        desktop_id="agm-alim.desktop"
    fi

    if command -v gsettings &>/dev/null; then
        local current_favs
        current_favs=$(gsettings get org.gnome.shell favorite-apps 2>/dev/null || echo "")
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
            run $sudo_cmd dpkg -i "$DOWNLOAD_PATH"
            run $sudo_cmd apt-get install -f -y  # Fix dependencies if needed
            ;;
        dnf)
            run $sudo_cmd dnf install -y "$DOWNLOAD_PATH"
            ;;
        yum)
            run $sudo_cmd yum install -y "$DOWNLOAD_PATH"
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
    for arg in "$@"; do
        case "$arg" in
            --help|-h)              show_help ;;
            --version|-v)           echo "install.sh v2.0.0"; exit 0 ;;
            --check-update|--check) check_for_updates_cli ;;
            --update)               is_update_only=1 ;;
        esac
    done

    # Top padding
    echo ""
    echo ""
    echo -e "${INDENT}${BLUE}========================================${NC}"
    echo -e "${INDENT}${BLUE}    ${APP_NAME} Installer${NC}"
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
    build_download_url
    download_installer

    # Old tool by original developer is removed ONLY AFTER new package download succeeds
    remove_legacy_upstream_installation

    case "$PLATFORM" in
        linux) install_linux ;;
        macos) install_macos ;;
    esac

    # Explicit post-install cleanup of downloaded packages
    cleanup

    # Newline before summary
    echo ""
    success "Installation complete!"
    echo ""
    info "Launch '${APP_NAME}' from your application menu or terminal."
    echo ""
}

main "$@"
