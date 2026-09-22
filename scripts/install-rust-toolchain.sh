#!/usr/bin/env bash
# ============================================================================
#  Antigravity-Manager: Unix/Linux/macOS Build Toolchain Installer
#  Installs Rust, Cargo, LLVM/Clang, and native build dependencies
# ============================================================================

set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

write_step() {
    echo -e "  ${CYAN}[*] $1${NC}"
}

write_success() {
    echo -e "  ${GREEN}[OK] $1${NC}"
}

write_warn() {
    echo -e "  ${YELLOW}[!] $1${NC}"
}

write_err() {
    echo -e "  ${RED}[ERROR] $1${NC}"
}

echo ""
echo -e "${CYAN}============================================================${NC}"
echo -e "${CYAN}  Antigravity-Manager: Unix/macOS Build Toolchain Installer ${NC}"
echo -e "${CYAN}============================================================${NC}"
echo ""

# 1. Detect OS
OS_NAME="$(uname -s)"
case "${OS_NAME}" in
    Linux*)     PLATFORM="linux";;
    Darwin*)    PLATFORM="macos";;
    *)          PLATFORM="unknown";;
esac

write_step "Detected platform: ${PLATFORM} (${OS_NAME})"

# 2. Install Native System Dependencies
if [ "${PLATFORM}" = "linux" ]; then
    if command -v apt-get >/dev/null 2>&1; then
        write_step "Installing native Linux build dependencies (apt)..."
        SUDO_CMD=""
        if [ "$(id -u)" -ne 0 ]; then
            if command -v sudo >/dev/null 2>&1; then
                SUDO_CMD="sudo"
            fi
        fi

        ${SUDO_CMD} apt-get update -qq || true
        ${SUDO_CMD} apt-get install -y --no-install-recommends \
            build-essential \
            curl \
            wget \
            file \
            libssl-dev \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev \
            patchelf \
            pkg-config \
            libsoup-3.0-dev \
            javascriptcoregtk-4.1 \
            libjavascriptcoregtk-4.1-dev \
            libwebkit2gtk-4.1-dev \
            libnm-dev \
            xdg-utils \
            clang \
            llvm \
            mold || write_warn "Some apt packages failed to install, proceeding with toolchain check..."
        write_success "Linux native libraries installation complete."
    else
        write_warn "Non-apt package manager detected. Please ensure webkit2gtk, gtk3, and build tools are installed."
    fi
elif [ "${PLATFORM}" = "macos" ]; then
    write_step "Checking macOS developer tools..."
    if ! xcode-select -p >/dev/null 2>&1; then
        write_step "Installing Xcode Command Line Tools..."
        xcode-select --install || true
    else
        write_success "Xcode Command Line Tools already installed."
    fi

    if command -v brew >/dev/null 2>&1; then
        write_step "Checking LLVM via Homebrew..."
        if ! brew list llvm >/dev/null 2>&1; then
            brew install llvm || write_warn "Homebrew llvm installation warning."
        else
            write_success "Homebrew LLVM already installed."
        fi
    fi
fi

# 3. Inspect Rustup & Cargo
if [ -f "${HOME}/.cargo/env" ]; then
    . "${HOME}/.cargo/env"
fi

if command -v rustc >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1; then
    write_success "Rust toolchain already installed: $(rustc --version) | $(cargo --version)"
else
    write_step "Installing Rustup with stable toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    if [ -f "${HOME}/.cargo/env" ]; then
        . "${HOME}/.cargo/env"
    fi
    write_success "Rustup installed successfully."
fi

# 4. Final Toolchain Verification
echo ""
echo -e "${CYAN}--- Toolchain Verification ---${NC}"
if command -v rustc >/dev/null 2>&1; then
    echo -e "  ${GREEN}Rust Compiler : $(rustc --version)${NC}"
else
    echo -e "  ${RED}Rust Compiler : Not Found${NC}"
fi

if command -v cargo >/dev/null 2>&1; then
    echo -e "  ${GREEN}Cargo Package : $(cargo --version)${NC}"
else
    echo -e "  ${RED}Cargo Package : Not Found${NC}"
fi

if command -v clang >/dev/null 2>&1; then
    echo -e "  ${GREEN}LLVM / Clang  : $(clang --version | head -n 1)${NC}"
else
    echo -e "  ${YELLOW}LLVM / Clang  : Not Found (Optional for standard builds)${NC}"
fi

echo ""
write_success "Toolchain setup complete! Run ./run.sh to start development."
