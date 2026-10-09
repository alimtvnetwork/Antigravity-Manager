#!/bin/sh
# ============================================================================
#  install-rust-toolchain.sh v2 — Antigravity-Manager build toolchain CLI
#
#  gitmap-style subcommands:
#    install     Install the Rust toolchain + native build deps (default)
#    check       Verify this machine can build (read-only, no changes)
#    help        Show help
#
#  Examples:
#    ./scripts/install-rust-toolchain.sh install
#    ./scripts/install-rust-toolchain.sh install --force --toolchain 1.89.0
#    ./scripts/install-rust-toolchain.sh install --skip-native-deps --quiet
#    ./scripts/install-rust-toolchain.sh check
# ============================================================================
set -eu

SCRIPT_VERSION="2.0.0"
RUSTUP_URL="https://sh.rustup.rs"
TOOLCHAIN_TOML_CANDIDATES="src-tauri/rust-toolchain.toml rust-toolchain.toml"

# --- flags (defaults) ---------------------------------------------------------
FLAG_FORCE=0
FLAG_QUIET=0
FLAG_SKIP_NATIVE_DEPS=0
FLAG_TOOLCHAIN=""
COMMAND=""

# --- output helpers -----------------------------------------------------------
if [ -t 1 ]; then
    CYAN='\033[0;36m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
    RED='\033[0;31m'; NC='\033[0m'
else
    CYAN=''; GREEN=''; YELLOW=''; RED=''; NC=''
fi

say_step()    { if [ "$FLAG_QUIET" -eq 0 ]; then printf '  %s[*]%s %s\n' "$CYAN" "$NC" "$1"; fi; }
say_ok()      { printf '  %s[OK]%s %s\n' "$GREEN" "$NC" "$1"; }
say_warn()    { printf '  %s[!]%s %s\n' "$YELLOW" "$NC" "$1" >&2; }
say_err()     { printf '  %s[ERROR]%s %s\n' "$RED" "$NC" "$1" >&2; }
say_check_ok()   { printf '  %s[OK]%s      %s\n' "$GREEN" "$NC" "$1"; }
say_check_bad()  { printf '  %s[MISSING]%s %s\n' "$RED" "$NC" "$1"; }

have() { command -v "$1" >/dev/null 2>&1; }

# --- help ---------------------------------------------------------------------
print_top_help() {
    cat <<EOF
Antigravity-Manager toolchain installer v$SCRIPT_VERSION

Usage:
  $(basename "$0") [global flags] <command> [command flags]

Commands:
  install     Install the Rust toolchain + native build dependencies
              (default command when omitted)
  check       Verify this machine can build the project (read-only)
  help        Show this help (also: <command> --help)

Global flags:
  --force               Reinstall even if already present
  --quiet               Minimal output, non-interactive (CI-friendly)
  --skip-native-deps    Skip OS packages, install the Rust toolchain only
  --toolchain <ver>     Toolchain channel/version (default: stable, or the
                        channel from rust-toolchain.toml when present)
  -h, --help            Show help and exit

Examples:
  $(basename "$0") install
  $(basename "$0") install --force --toolchain 1.89.0
  $(basename "$0") install --skip-native-deps --quiet
  $(basename "$0") check
EOF
}

print_install_help() {
    cat <<EOF
Usage: $(basename "$0") install [flags]

Installs rustup + the Rust toolchain (clippy, rustfmt) and the native
build dependencies for this project (Tauri/GTK/WebKit on Linux,
Xcode CLT + LLVM on macOS).

Flags:
  --force               Reinstall even if already present
  --quiet               Minimal output, non-interactive
  --skip-native-deps    Skip OS packages, install the Rust toolchain only
  --toolchain <ver>     Toolchain channel/version (default: stable)
  -h, --help            Show this help and exit
EOF
}

print_check_help() {
    cat <<EOF
Usage: $(basename "$0") check

Read-only verification that this machine can build the project.
Probes: rustc, cargo, clippy, rustfmt, native libs (Linux: gtk+-3.0,
webkit2gtk-4.1, libsoup-3.0 via pkg-config; macOS: Xcode CLT), and
whether the installed toolchain matches rust-toolchain.toml when present.

Exit code: 0 = build-ready, 1 = something missing (hints printed).
EOF
}

# --- arg parsing --------------------------------------------------------------
parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            install|check|help)
                if [ -n "$COMMAND" ]; then say_err "only one command allowed"; exit 2; fi
                COMMAND="$1"; shift ;;
            --force)              FLAG_FORCE=1; shift ;;
            --quiet)              FLAG_QUIET=1; shift ;;
            --skip-native-deps)   FLAG_SKIP_NATIVE_DEPS=1; shift ;;
            --toolchain)
                if [ $# -lt 2 ]; then say_err "--toolchain needs a value"; exit 2; fi
                FLAG_TOOLCHAIN="$2"; shift 2 ;;
            --toolchain=*)        FLAG_TOOLCHAIN="${1#--toolchain=}"; shift ;;
            -h|--help)
                case "$COMMAND" in
                    install) print_install_help ;;
                    check)   print_check_help ;;
                    *)       print_top_help ;;
                esac
                exit 0 ;;
            --) shift; break ;;
            -*) say_err "unknown flag: $1 (see '$(basename "$0") help')"; exit 2 ;;
            *)  say_err "unknown command: $1 (see '$(basename "$0") help')"; exit 2 ;;
        esac
    done
    if [ -z "$COMMAND" ]; then COMMAND="install"; fi  # backward compat
}

# --- toolchain resolution -----------------------------------------------------
resolve_toolchain() {
    if [ -n "$FLAG_TOOLCHAIN" ]; then printf '%s' "$FLAG_TOOLCHAIN"; return; fi
    for cand in $TOOLCHAIN_TOML_CANDIDATES; do
        if [ -f "$cand" ]; then
            chan="$(sed -n 's/^channel[[:space:]]*=[[:space:]]*"\(.*\)"/\1/p' "$cand" | head -1)"
            if [ -n "$chan" ]; then printf '%s' "$chan"; return; fi
        fi
    done
    printf 'stable'
}

ensure_cargo_on_path() {
    case ":$PATH:" in
        *":$HOME/.cargo/bin:"*) ;;
        *) export PATH="$HOME/.cargo/bin:$PATH" ;;
    esac
}

# --- install ------------------------------------------------------------------
detect_platform() {
    case "$(uname -s)" in
        Linux*)  printf 'linux' ;;
        Darwin*) printf 'macos' ;;
        *)       printf 'unknown' ;;
    esac
}

install_native_deps() {
    platform="$1"
    if [ "$platform" = "linux" ]; then
        if ! have apt-get; then
            say_warn "non-apt system: install gtk3, webkit2gtk, libssl, patchelf manually"
            return 0
        fi
        say_step "Installing native Linux build dependencies (apt)..."
        SUDO=""
        if [ "$(id -u)" -ne 0 ] && have sudo; then SUDO="sudo"; fi
        # Essential first (build must-have), optional second (nice-to-have).
        # shellcheck disable=SC2086
        $SUDO apt-get update -qq || true
        # shellcheck disable=SC2086
        $SUDO apt-get install -y --no-install-recommends \
            build-essential curl wget file pkg-config \
            libssl-dev libgtk-3-dev librsvg2-dev patchelf \
            || say_warn "some essential packages failed; continuing"
        # shellcheck disable=SC2086
        $SUDO apt-get install -y --no-install-recommends \
            libayatana-appindicator3-dev libsoup-3.0-dev \
            javascriptcoregtk-4.1 libjavascriptcoregtk-4.1-dev \
            libwebkit2gtk-4.1-dev libnm-dev xdg-utils \
            clang llvm mold \
            || say_warn "some optional packages failed; continuing"
        say_ok "native libraries step done"
    elif [ "$platform" = "macos" ]; then
        say_step "Checking macOS developer tools..."
        if xcode-select -p >/dev/null 2>&1; then
            say_ok "Xcode Command Line Tools present"
        else
            say_step "Installing Xcode Command Line Tools..."
            xcode-select --install || true
        fi
        if have brew; then
            if brew list llvm >/dev/null 2>&1; then
                say_ok "Homebrew LLVM present"
            else
                say_step "Installing LLVM via Homebrew..."
                brew install llvm || say_warn "brew llvm install had warnings"
            fi
            # brew LLVM is keg-only: export so the compiler can find it
            llvm_prefix="$(brew --prefix llvm 2>/dev/null || true)"
            if [ -n "$llvm_prefix" ]; then
                export PATH="$llvm_prefix/bin:$PATH"
                export LDFLAGS="-L$llvm_prefix/lib ${LDFLAGS:-}"
                export CPPFLAGS="-I$llvm_prefix/include ${CPPFLAGS:-}"
            fi
        else
            say_warn "Homebrew not found; skipping LLVM"
        fi
    else
        say_err "unsupported platform: $(uname -s)"; exit 1
    fi
}

install_rustup() {
    toolchain="$1"
    if have rustup && [ "$FLAG_FORCE" -eq 0 ]; then
        say_ok "rustup already installed: $(rustup --version)"
    else
        if have rustup && [ "$FLAG_FORCE" -eq 1 ]; then
            say_step "--force: reinstalling rustup..."
        else
            say_step "Downloading rustup installer..."
        fi
        # Download first, then execute (never pipe curl straight into sh).
        tmp_init="$(mktemp /tmp/rustup-init.XXXXXX.sh)"
        if have curl; then
            curl --proto '=https' --tlsv1.2 -sSf -o "$tmp_init" "$RUSTUP_URL"
        elif have wget; then
            wget -qO "$tmp_init" "$RUSTUP_URL"
        else
            say_err "need curl or wget to download rustup"; exit 1
        fi
        say_step "Running rustup installer (non-interactive)..."
        sh "$tmp_init" -y --no-modify-path --profile minimal \
            --default-toolchain "$toolchain"
        rm -f "$tmp_init"
        say_ok "rustup installed"
    fi
    ensure_cargo_on_path
    if ! rustup toolchain list 2>/dev/null | grep -q "^$toolchain"; then
        say_step "Installing toolchain $toolchain..."
        rustup toolchain install "$toolchain" --profile minimal
    fi
    rustup default "$toolchain" >/dev/null 2>&1 || true
    say_ok "toolchain ready: $(rustc --version 2>/dev/null || echo "$toolchain")"
}

install_components() {
    say_step "Ensuring clippy + rustfmt components..."
    if rustup component add clippy rustfmt 2>/dev/null; then
        say_ok "clippy + rustfmt ready"
    else
        say_warn "could not add clippy/rustfmt (offline?)"
    fi
}

persist_path() {
    line='export PATH="$HOME/.cargo/bin:$PATH"'
    for profile in "$HOME/.profile" "$HOME/.bashrc"; do
        if [ -f "$profile" ] && ! grep -q '\.cargo/bin' "$profile" 2>/dev/null; then
            printf '\n# Rust toolchain (rustup)\n%s\n' "$line" >> "$profile"
            say_ok "added cargo to PATH in $profile"
        fi
    done
    if [ -f "$HOME/.zshrc" ] && ! grep -q '\.cargo/bin' "$HOME/.zshrc" 2>/dev/null; then
        printf '\n# Rust toolchain (rustup)\n%s\n' "$line" >> "$HOME/.zshrc"
        say_ok "added cargo to PATH in $HOME/.zshrc"
    fi
}

cmd_install() {
    platform="$(detect_platform)"
    say_step "Target toolchain: $(resolve_toolchain) (platform: $platform)"
    if [ "$FLAG_SKIP_NATIVE_DEPS" -eq 0 ]; then
        install_native_deps "$platform"
    else
        say_step "--skip-native-deps: OS packages skipped"
    fi
    if [ -f "$HOME/.cargo/env" ]; then . "$HOME/.cargo/env"; fi
    install_rustup "$(resolve_toolchain)"
    install_components
    persist_path
    say_ok "Toolchain setup complete."
}

# --- check --------------------------------------------------------------------
cmd_check() {
    ensure_cargo_on_path
    failures=0
    platform="$(detect_platform)"

    if have rustc; then say_check_ok "rustc $(rustc --version | cut -d' ' -f2)"
    else say_check_bad "rustc (run '$(basename "$0") install')"; failures=$((failures+1)); fi

    if have cargo; then say_check_ok "cargo $(cargo --version | cut -d' ' -f2)"
    else say_check_bad "cargo (run '$(basename "$0") install')"; failures=$((failures+1)); fi

    if have rustup; then
        installed="$(rustup component list --installed 2>/dev/null || true)"
        case "$installed" in
            *clippy*)  say_check_ok "clippy component" ;;
            *) say_check_bad "clippy component (run '$(basename "$0") install')"; failures=$((failures+1)) ;;
        esac
        case "$installed" in
            *rustfmt*) say_check_ok "rustfmt component" ;;
            *) say_check_bad "rustfmt component (run '$(basename "$0") install')"; failures=$((failures+1)) ;;
        esac
        # toolchain pin check
        for cand in $TOOLCHAIN_TOML_CANDIDATES; do
            if [ -f "$cand" ]; then
                want="$(resolve_toolchain)"
                got="$(rustup show active-toolchain 2>/dev/null | cut -d' ' -f1 || true)"
                if [ "$got" = "$want" ]; then
                    say_check_ok "toolchain matches $cand ($want)"
                else
                    say_check_bad "toolchain mismatch: want $want, have ${got:-none}"
                    failures=$((failures+1))
                fi
                break
            fi
        done
    else
        say_check_bad "rustup (run '$(basename "$0") install')"
        failures=$((failures+1))
    fi

    if [ "$platform" = "linux" ]; then
        if have pkg-config; then
            for lib in gtk+-3.0 webkit2gtk-4.1 libsoup-3.0; do
                if pkg-config --exists "$lib" 2>/dev/null; then
                    say_check_ok "native lib $lib"
                else
                    say_check_bad "native lib $lib (run '$(basename "$0") install')"
                    failures=$((failures+1))
                fi
            done
        else
            say_check_bad "pkg-config (run '$(basename "$0") install')"
            failures=$((failures+1))
        fi
    elif [ "$platform" = "macos" ]; then
        if xcode-select -p >/dev/null 2>&1; then say_check_ok "Xcode Command Line Tools"
        else say_check_bad "Xcode CLT (run '$(basename "$0") install')"; failures=$((failures+1)); fi
    fi

    if [ "$failures" -eq 0 ]; then
        say_ok "this machine is build-ready"
        return 0
    fi
    say_err "$failures check(s) failed"
    return 1
}

# --- main ---------------------------------------------------------------------
parse_args "$@"
case "$COMMAND" in
    install) cmd_install ;;
    check)   cmd_check ;;
    help)    print_top_help ;;
    *)       say_err "unknown command: $COMMAND"; exit 2 ;;
esac
