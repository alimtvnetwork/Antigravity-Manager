#!/usr/bin/env bash
# ============================================================================
#  Antigravity-Manager: Unix/macOS Application Runner
#  Verifies toolchains, installs dependencies, and runs the application
# ============================================================================

set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

write_step() { echo -e "  ${CYAN}[*] $1${NC}"; }
write_success() { echo -e "  ${GREEN}[OK] $1${NC}"; }
write_warn() { echo -e "  ${YELLOW}[!] $1${NC}"; }
write_err() { echo -e "  ${RED}[ERROR] $1${NC}"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

# Intercept CLI subcommands (agy, ccko, cckf, undo, cache-clear, help)
if [ "$#" -gt 0 ]; then
    case "$1" in
        agy|agi|ccko|cckf|undo|cache-clear|help)
            if [ -f "${HOME}/.cargo/env" ]; then
                . "${HOME}/.cargo/env"
            fi
            cargo run --bin agm-alim --manifest-path "${SCRIPT_DIR}/src-tauri/Cargo.toml" -- "$@"
            exit $?
            ;;
    esac
fi

MODE="dev"
for arg in "$@"; do
    case "$arg" in
        --build|-b)
            MODE="build";;
        --debug|-d)
            MODE="debug";;
        --frontend|-f)
            MODE="frontend";;
        --install-toolchain|-i)
            bash "${SCRIPT_DIR}/scripts/install-rust-toolchain.sh"
            exit 0;;
    esac
done

echo ""
echo -e "${CYAN}============================================================${NC}"
echo -e "${CYAN}                Antigravity-Manager Runner                 ${NC}"
echo -e "${CYAN}============================================================${NC}"
echo ""

# 1. Check Node.js and npm
write_step "Checking Node.js & npm..."
if ! command -v node >/dev/null 2>&1 || ! command -v npm >/dev/null 2>&1; then
    write_err "Node.js and npm are required. Please install Node.js (v20+ LTS)."
    exit 1
fi
write_success "Node.js: $(node -v) | npm: v$(npm -v)"

# 2. Check Rust & Cargo (if running Tauri)
if [ "${MODE}" != "frontend" ]; then
    write_step "Checking Rust & Cargo toolchain..."
    if [ -f "${HOME}/.cargo/env" ]; then
        . "${HOME}/.cargo/env"
    fi

    if ! command -v rustc >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
        write_warn "Rust or Cargo not found. Invoking toolchain installer..."
        bash "${SCRIPT_DIR}/scripts/install-rust-toolchain.sh"
        if [ -f "${HOME}/.cargo/env" ]; then
            . "${HOME}/.cargo/env"
        fi
        if ! command -v rustc >/dev/null 2>&1; then
            write_err "Rust toolchain is still missing. Please install from https://rustup.rs"
            exit 1
        fi
    else
        write_success "Rust: $(rustc --version)"
    fi
fi

# 3. Check frontend dependencies
if [ ! -d "${SCRIPT_DIR}/node_modules" ]; then
    write_step "node_modules missing. Running npm install..."
    npm install --legacy-peer-deps
    write_success "Frontend dependencies installed."
fi

# 4. Dispatch Run Mode
case "${MODE}" in
    build)
        write_step "Executing production build (npm run tauri build)..."
        npm run tauri build;;
    debug)
        write_step "Executing debug mode with verbose logs..."
        RUST_LOG=debug npm run tauri dev;;
    frontend)
        write_step "Launching Vite frontend dev server (npm run dev)..."
        npm run dev;;
    *)
        write_step "Launching Antigravity-Manager development mode (npm run tauri dev)..."
        npm run tauri dev;;
esac
