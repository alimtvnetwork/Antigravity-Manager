#!/usr/bin/env bash
# ============================================================================
#  Antigravity-Manager: Dedicated Production Build Runner (Linux & macOS)
#  Compiles production-ready Tauri desktop bundles and CLI binaries.
# ============================================================================

set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

write_step()    { echo -e "  ${CYAN}[*] $1${NC}"; }
write_success() { echo -e "  ${GREEN}[OK] $1${NC}"; }
write_warn()    { echo -e "  ${YELLOW}[!] $1${NC}"; }
write_err()     { echo -e "  ${RED}[ERROR] $1${NC}" >&2; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
cd "${SCRIPT_DIR}"

TARGET_MODE="release"
CLI_ONLY=0
FRONTEND_ONLY=0
NO_INSTALL=0
CLEAN=0
VERBOSE=0
DRY_RUN=0

for arg in "$@"; do
    case "$arg" in
        --release|-r)
            TARGET_MODE="release"
            ;;
        --debug|-d)
            TARGET_MODE="debug"
            ;;
        --cli-only)
            CLI_ONLY=1
            ;;
        --frontend-only)
            FRONTEND_ONLY=1
            ;;
        --no-install)
            NO_INSTALL=1
            ;;
        --clean)
            CLEAN=1
            ;;
        --verbose|-v)
            VERBOSE=1
            ;;
        --dry-run|-n)
            DRY_RUN=1
            ;;
        --help|-h)
            echo "Usage: ./build.sh [options]"
            echo "Options:"
            echo "  --release, -r     Build production release bundle (default)"
            echo "  --debug, -d       Build debug binary"
            echo "  --cli-only        Build standalone CLI binary (agm) only"
            echo "  --frontend-only   Build frontend assets (npm run build) only"
            echo "  --no-install      Do not auto-install missing toolchains"
            echo "  --clean           Clean target and dist before building"
            echo "  --verbose, -v     Verbose output"
            echo "  --dry-run, -n     Print commands without executing"
            exit 0
            ;;
    esac
done

echo ""
echo -e "${CYAN}============================================================${NC}"
echo -e "${CYAN}            Antigravity-Manager Build Runner                ${NC}"
echo -e "${CYAN}============================================================${NC}"
echo ""

# Source cargo env if present
if [ -f "${HOME}/.cargo/env" ]; then
    . "${HOME}/.cargo/env"
fi

# 1. Prerequisite Checks
MISSING_PREREQS=()

write_step "Checking Node.js & npm..."
if ! command -v node >/dev/null 2>&1 || ! command -v npm >/dev/null 2>&1; then
    write_warn "Node.js or npm is missing."
    MISSING_PREREQS+=("nodejs")
else
    write_success "Node.js: $(node -v) | npm: v$(npm -v)"
fi

if [ "${FRONTEND_ONLY}" -eq 0 ]; then
    write_step "Checking Rust & Cargo toolchain..."
    if ! command -v rustc >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1 || ! rustc --version >/dev/null 2>&1; then
        write_warn "Rust compiler (rustc) or Cargo is missing or default toolchain is not set."
        MISSING_PREREQS+=("rust")
    else
        write_success "Rust: $(rustc --version 2>/dev/null || echo 'Installed')"
    fi
fi

# 2. Self-Healing Toolchain Ladder
if [ "${#MISSING_PREREQS[@]}" -gt 0 ]; then
    if [ "${NO_INSTALL}" -eq 1 ]; then
        write_err "Missing prerequisites: ${MISSING_PREREQS[*]}"
        write_err "--no-install specified. Aborting compilation."
        exit 1
    fi

    write_warn "Build toolchain is incomplete. Invoking automated installer..."
    if [ -f "${SCRIPT_DIR}/install.sh" ]; then
        if [ "${DRY_RUN}" -eq 0 ]; then
            bash "${SCRIPT_DIR}/install.sh" --build-tools || bash "${SCRIPT_DIR}/scripts/install-rust-toolchain.sh" install --profile build-tools
            if [ -f "${HOME}/.cargo/env" ]; then
                . "${HOME}/.cargo/env"
            fi
        else
            echo "  [DRY-RUN] Would run: ./install.sh --build-tools"
        fi
    else
        write_err "Installer not found at ${SCRIPT_DIR}/install.sh"
        exit 1
    fi
fi

# 3. Handle --clean
if [ "${CLEAN}" -eq 1 ]; then
    write_step "Cleaning build artifacts..."
    if [ "${DRY_RUN}" -eq 0 ]; then
        if [ "${FRONTEND_ONLY}" -eq 0 ]; then
            (cd "${SCRIPT_DIR}/src-tauri" && cargo clean)
        fi
        rm -rf "${SCRIPT_DIR}/dist"
        write_success "Build directories cleaned."
    else
        echo "  [DRY-RUN] Would run: cargo clean in src-tauri, remove dist/"
    fi
fi

# 4. Handle --frontend-only
if [ "${FRONTEND_ONLY}" -eq 1 ]; then
    write_step "Executing frontend-only build (npm run build)..."
    if [ "${DRY_RUN}" -eq 0 ]; then
        npm run build
        exit $?
    else
        echo "  [DRY-RUN] Would execute: npm run build"
        exit 0
    fi
fi

# 5. Ensure frontend dependencies are installed
if [ ! -d "${SCRIPT_DIR}/node_modules" ]; then
    write_step "node_modules missing. Installing dependencies (npm install --legacy-peer-deps)..."
    if [ "${DRY_RUN}" -eq 0 ]; then
        npm install --legacy-peer-deps
        write_success "Frontend dependencies installed."
    else
        echo "  [DRY-RUN] Would execute: npm install --legacy-peer-deps"
    fi
fi

# 6. Configure compiler parallelization & cache optimizations
CORES=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
export CARGO_BUILD_JOBS="${CORES}"
write_step "Configured worker threads: ${CORES}"

if command -v sccache >/dev/null 2>&1; then
    export RUSTC_WRAPPER="sccache"
    write_success "Shared compiler cache (sccache) enabled."
fi

# 7. Execute Compilation
if [ "${CLI_ONLY}" -eq 1 ]; then
    CARGO_ARGS=("build" "--bin" "agm" "--manifest-path" "${SCRIPT_DIR}/src-tauri/Cargo.toml")
    if [ "${TARGET_MODE}" = "release" ]; then
        CARGO_ARGS+=("--release")
    fi
    if [ "${VERBOSE}" -eq 1 ]; then
        CARGO_ARGS+=("--verbose")
    fi
    write_step "Executing CLI binary compilation: cargo ${CARGO_ARGS[*]}"
    if [ "${DRY_RUN}" -eq 0 ]; then
        cargo "${CARGO_ARGS[@]}"
        exit $?
    else
        echo "  [DRY-RUN] Would execute: cargo ${CARGO_ARGS[*]}"
        exit 0
    fi
else
    write_step "Executing Tauri bundle compilation (${TARGET_MODE} mode)..."
    if [ "${DRY_RUN}" -eq 0 ]; then
        if [ "${TARGET_MODE}" = "debug" ]; then
            npm run tauri build -- --debug
        else
            npm run tauri build
        fi
        exit $?
    else
        echo "  [DRY-RUN] Would execute: npm run tauri build $( [ "${TARGET_MODE}" = "debug" ] && echo '-- --debug' )"
        exit 0
    fi
fi
