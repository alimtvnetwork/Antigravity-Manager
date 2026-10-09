#!/bin/sh
# ============================================================================
#  install-rust-toolchain.sh v3 — Antigravity-Manager build toolchain CLI
#
#  gitmap-style subcommands:
#    install     Install toolchain items (default; see --profile/--only)
#    check       Verify this machine can build the project (read-only)
#    help        Show help
#
#  Profiles (repeatable, stacked via multiple --profile flags):
#    minimal     rust toolchain + clippy/rustfmt
#    rust-dev    minimal + sccache + cargo-watch
#    frontend    nodejs (fnm) + pnpm + tauri-cli
#    full        rust-dev + frontend + gh
#
#  Examples:
#    ./scripts/install-rust-toolchain.sh install
#    ./scripts/install-rust-toolchain.sh install --profile full
#    ./scripts/install-rust-toolchain.sh install --profile rust-dev --profile frontend
#    ./scripts/install-rust-toolchain.sh install --only nodejs --only pnpm
#    ./scripts/install-rust-toolchain.sh install --profile full --dry-run
#    ./scripts/install-rust-toolchain.sh --list-items
#    ./scripts/install-rust-toolchain.sh --troubleshoot
#    ./scripts/install-rust-toolchain.sh install --ssh dev@buildbox.local
#    ./scripts/install-rust-toolchain.sh check
# ============================================================================
set -eu

SCRIPT_VERSION="3.0.0"
RUSTUP_URL="https://sh.rustup.rs"
TOOLCHAIN_TOML_CANDIDATES="src-tauri/rust-toolchain.toml rust-toolchain.toml"
FNM_INSTALL_URL="https://fnm.vercel.app/install"

# --- item catalog (canonical install order; also used by --list-items) --------
# Format: id|one-line description
ITEM_CATALOG="rust|Rust toolchain via rustup (stable channel) + clippy/rustfmt components
native-deps|OS native build dependencies (Tauri GTK/WebKit on Linux, Xcode CLT + LLVM on macOS)
nodejs|Node.js LTS via fnm (fast node manager)
pnpm|pnpm package manager via npm (requires nodejs)
tauri-cli|Tauri CLI via cargo install (requires rust)
sccache|sccache compilation cache via cargo install (requires rust)
cargo-watch|cargo-watch file watcher via cargo install (requires rust)
gh|GitHub CLI via apt (Linux) / brew (macOS)"

ITEMS_ORDER="native-deps rust nodejs pnpm sccache cargo-watch tauri-cli gh"

# --- flags (defaults) ---------------------------------------------------------
FLAG_FORCE=0
FLAG_QUIET=0
FLAG_SKIP_NATIVE_DEPS=0
FLAG_TOOLCHAIN=""
FLAG_PROFILES=""
FLAG_ONLY=""
FLAG_DRY_RUN=0
FLAG_LIST_ITEMS=0
FLAG_TROUBLESHOOT=0
FLAG_SSH=""
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
  install     Install toolchain items (default command when omitted)
  check       Verify this machine can build the project (read-only)
  help        Show this help (also: <command> --help)

Global flags:
  --profile <name>      Install profile (repeatable, stacked):
                        minimal   = rust toolchain + clippy/rustfmt
                        rust-dev  = minimal + sccache + cargo-watch
                        frontend  = nodejs (fnm) + pnpm + tauri-cli
                        full      = rust-dev + frontend + gh
  --only <item>         Install exactly this item (repeatable).
                        See --list-items for ids.
  --dry-run             Print the resolved item list, install nothing.
  --list-items          Print installable items (id + description), then exit.
  --troubleshoot        Print common issues and fixes, then exit.
  --ssh <user@host>     Run install/check on a remote host via SSH
                        (key auth only, never passwords).
  --force               Reinstall even if already present
  --quiet               Minimal output, non-interactive (CI-friendly)
  --skip-native-deps    Skip OS packages, install the Rust toolchain only
  --toolchain <ver>     Toolchain channel/version (default: stable, or the
                        channel from rust-toolchain.toml when present)
  -h, --help            Show help and exit

Examples:
  $(basename "$0") install
  $(basename "$0") install --profile full
  $(basename "$0") install --profile rust-dev --profile frontend --dry-run
  $(basename "$0") install --only nodejs --only pnpm
  $(basename "$0") --list-items
  $(basename "$0") --troubleshoot
  $(basename "$0") install --ssh dev@buildbox.local --profile full
  $(basename "$0") check
EOF
}

print_install_help() {
    cat <<EOF
Usage: $(basename "$0") install [flags]

Installs toolchain items resolved from --profile / --only flags
(default: rust + native-deps).

Profiles (repeatable, stacked):
  minimal     rust toolchain + clippy/rustfmt
  rust-dev    minimal + sccache + cargo-watch
  frontend    nodejs (fnm) + pnpm + tauri-cli
  full        rust-dev + frontend + gh

Flags:
  --profile <name>      Install profile (repeatable)
  --only <item>         Install exactly this item (repeatable)
  --dry-run             Print the resolved item list, install nothing
  --ssh <user@host>     Run on a remote host via SSH (key auth only)
  --force               Reinstall even if already present
  --quiet               Minimal output, non-interactive
  --skip-native-deps    Skip OS packages
  --toolchain <ver>     Toolchain channel/version (default: stable)
  -h, --help            Show this help and exit
EOF
}

print_check_help() {
    cat <<EOF
Usage: $(basename "$0") check [--ssh <user@host>]

Read-only verification that this machine (or the remote host) can build
the project. Probes: rustc, cargo, clippy, rustfmt, native libs
(Linux: gtk+-3.0, webkit2gtk-4.1, libsoup-3.0 via pkg-config;
macOS: Xcode CLT), and whether the installed toolchain matches
rust-toolchain.toml when present.

Exit code: 0 = build-ready, 1 = something missing (hints printed).
EOF
}

print_troubleshoot() {
    cat <<EOF
Troubleshooting — common issues and fixes:

1. "cargo: command not found" (right after install)
   Your shell has not picked up ~/.cargo/bin yet.
   Fix: restart the terminal, or run:  source ~/.cargo/env
   This script also appends the PATH export to ~/.profile and ~/.bashrc.

2. "gdk-sys build failed" / "system library \`gdk-3.0\` was not found"
   Missing Tauri GTK system dependencies.
   Fix: rerun WITHOUT --skip-native-deps so OS packages get installed.

3. rustup download / update fails (offline or proxy)
   Fix: check network connectivity and proxy env vars
   (http_proxy/https_proxy), then retry.

4. "permission denied" during apt install
   Fix: rerun with sudo, or as root.

5. "toolchain mismatch" (check reports want X but have Y)
   Fix: rustup default <ver>, or pass --toolchain <ver> to install.

6. fnm / node not found after installing the frontend profile
   Fix: restart the terminal (fnm init), or run:
   eval "\$(fnm env --use-on-cd)"

7. SSH remote install fails with "Permission denied (publickey)"
   This script never handles passwords — pre-configure SSH key auth:
   ssh-copy-id user@host, then retry.
EOF
}

list_items() {
    # Machine-readable: id<TAB>description (one per line, catalog order)
    printf '%s\n' "$ITEM_CATALOG" | while IFS='|' read -r id desc; do
        printf '%s\t%s\n' "$id" "$desc"
    done
}

# --- arg parsing --------------------------------------------------------------
parse_args() {
    SSH_FWD=""
    STRIP_NEXT=0
    # Pre-scan: extract --ssh (never forwarded; prevents remote recursion)
    for a in "$@"; do
        if [ "$STRIP_NEXT" = "1" ]; then FLAG_SSH="$a"; STRIP_NEXT=0; continue; fi
        case "$a" in
            --ssh) STRIP_NEXT=1 ;;
            --ssh=*) FLAG_SSH="${a#--ssh=}" ;;
            *) SSH_FWD="$SSH_FWD $a" ;;
        esac
    done
    if [ "$STRIP_NEXT" = "1" ]; then say_err "--ssh needs user@host"; exit 2; fi

    while [ $# -gt 0 ]; do
        case "$1" in
            install|check|help)
                if [ -n "$COMMAND" ]; then say_err "only one command allowed"; exit 2; fi
                COMMAND="$1"; shift ;;
            --profile)
                if [ $# -lt 2 ]; then say_err "--profile needs a value"; exit 2; fi
                FLAG_PROFILES="$FLAG_PROFILES $2"; shift 2 ;;
            --profile=*)      FLAG_PROFILES="$FLAG_PROFILES ${1#--profile=}"; shift ;;
            --only)
                if [ $# -lt 2 ]; then say_err "--only needs an item id"; exit 2; fi
                FLAG_ONLY="$FLAG_ONLY $2"; shift 2 ;;
            --only=*)         FLAG_ONLY="$FLAG_ONLY ${1#--only=}"; shift ;;
            --item)           # alias for --only (backend compatibility)
                if [ $# -lt 2 ]; then say_err "--item needs an item id"; exit 2; fi
                FLAG_ONLY="$FLAG_ONLY $2"; shift 2 ;;
            --item=*)         FLAG_ONLY="$FLAG_ONLY ${1#--item=}"; shift ;;
            --dry-run)        FLAG_DRY_RUN=1; shift ;;
            --list-items)     FLAG_LIST_ITEMS=1; shift ;;
            --troubleshoot)   FLAG_TROUBLESHOOT=1; shift ;;
            --ssh)            shift 2 ;;   # consumed in pre-scan
            --ssh=*)          shift ;;
            --force)          FLAG_FORCE=1; shift ;;
            --quiet)          FLAG_QUIET=1; shift ;;
            --skip-native-deps) FLAG_SKIP_NATIVE_DEPS=1; shift ;;
            --toolchain)
                if [ $# -lt 2 ]; then say_err "--toolchain needs a value"; exit 2; fi
                FLAG_TOOLCHAIN="$2"; shift 2 ;;
            --toolchain=*)    FLAG_TOOLCHAIN="${1#--toolchain=}"; shift ;;
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

# --- item resolution ----------------------------------------------------------
profile_items() {
    case "$1" in
        minimal)  printf 'rust' ;;
        rust-dev) printf 'rust sccache cargo-watch' ;;
        frontend) printf 'nodejs pnpm tauri-cli' ;;
        full)     printf 'rust sccache cargo-watch nodejs pnpm tauri-cli gh' ;;
        *) say_err "unknown profile: '$1' (minimal|rust-dev|frontend|full)"; exit 2 ;;
    esac
}

item_known() {
    case " $ITEMS_ORDER " in
        *" $1 "*) return 0 ;;
        *) return 1 ;;
    esac
}

resolve_items() {
    # Sets RESOLVED_ITEMS (space-separated, canonical install order).
    items=""
    if [ -n "$FLAG_ONLY" ]; then
        items="$FLAG_ONLY"                       # exact set
    elif [ -n "$FLAG_PROFILES" ]; then
        for p in $FLAG_PROFILES; do
            items="$items $(profile_items "$p")"
        done
    else
        items="rust"                             # default
    fi
    # native-deps rides along by default (backward compat), unless the user
    # asked for an exact set (--only) or opted out (--skip-native-deps).
    if [ -z "$FLAG_ONLY" ] && [ "$FLAG_SKIP_NATIVE_DEPS" -eq 0 ]; then
        items="native-deps $items"
    fi
    for it in $items; do
        if ! item_known "$it"; then
            say_err "unknown item: '$it' (see --list-items)"; exit 2
        fi
    done
    ordered=""
    for want in $ITEMS_ORDER; do
        case " $items " in
            *" $want "*) ordered="$ordered $want" ;;
        esac
    done
    RESOLVED_ITEMS="$(printf '%s' "$ordered" | sed 's/^ //')"
}

print_dry_run() {
    resolve_items
    n="$(printf '%s' "$RESOLVED_ITEMS" | wc -w)"
    profiles_label="${FLAG_PROFILES:-<none>}"
    printf 'Dry run — would install %s item(s) [profiles:%s]:\n' "$n" "$profiles_label"
    for it in $RESOLVED_ITEMS; do
        desc="$(printf '%s\n' "$ITEM_CATALOG" | grep "^$it|" | cut -d'|' -f2)"
        printf '  - %s: %s\n' "$it" "$desc"
    done
    printf '(nothing installed)\n'
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

detect_platform() {
    case "$(uname -s)" in
        Linux*)  printf 'linux' ;;
        Darwin*) printf 'macos' ;;
        *)       printf 'unknown' ;;
    esac
}

# --- item installers ----------------------------------------------------------
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
    # clippy + rustfmt are project gates
    say_step "Ensuring clippy + rustfmt components..."
    if rustup component add clippy rustfmt 2>/dev/null; then
        say_ok "clippy + rustfmt ready"
    else
        say_warn "could not add clippy/rustfmt (offline?)"
    fi
    persist_path
}

install_nodejs() {
    if have node && [ "$FLAG_FORCE" -eq 0 ]; then
        say_ok "node already installed: $(node --version)"
        return 0
    fi
    if ! have fnm; then
        say_step "Installing fnm (fast node manager)..."
        tmp_fnm="$(mktemp /tmp/fnm-install.XXXXXX.sh)"
        if have curl; then
            curl -fsSL -o "$tmp_fnm" "$FNM_INSTALL_URL"
        elif have wget; then
            wget -qO "$tmp_fnm" "$FNM_INSTALL_URL"
        else
            say_err "need curl or wget to download fnm"; exit 1
        fi
        sh "$tmp_fnm"; rm -f "$tmp_fnm"
    fi
    FNM_DIR="${FNM_DIR:-$HOME/.local/share/fnm}"
    export PATH="$FNM_DIR:$PATH"
    # fnm env prints POSIX-compatible exports for bash/sh
    eval "$(fnm env --use-on-cd --shell bash 2>/dev/null)" || true
    say_step "Installing Node.js LTS via fnm..."
    fnm install --lts
    fnm default lts-latest
    if have node; then say_ok "node ready: $(node --version)"
    else say_warn "node installed but not on PATH; restart the shell"; fi
}

install_pnpm() {
    if have pnpm && [ "$FLAG_FORCE" -eq 0 ]; then
        say_ok "pnpm already installed: $(pnpm --version)"
        return 0
    fi
    if ! have npm; then
        say_err "pnpm needs the nodejs item first (npm not found)"; exit 1
    fi
    say_step "Installing pnpm..."
    npm install -g pnpm
    say_ok "pnpm ready: $(pnpm --version 2>/dev/null || echo installed)"
}

install_cargo_pkg() {
    # $1 = crate, $2 = binary to probe
    if have "$2" && [ "$FLAG_FORCE" -eq 0 ]; then
        say_ok "$2 already installed"
        return 0
    fi
    ensure_cargo_on_path
    if ! have cargo; then
        say_err "$2 needs the rust item first (cargo not found)"; exit 1
    fi
    say_step "Installing $1 via cargo install (this may take a while)..."
    if cargo install "$1" --locked; then
        say_ok "$2 ready"
    else
        say_warn "cargo install $1 failed"
    fi
}

install_gh() {
    if have gh && [ "$FLAG_FORCE" -eq 0 ]; then
        say_ok "gh already installed: $(gh --version 2>/dev/null | head -1)"
        return 0
    fi
    platform="$(detect_platform)"
    if [ "$platform" = "linux" ] && have apt-get; then
        SUDO=""
        if [ "$(id -u)" -ne 0 ] && have sudo; then SUDO="sudo"; fi
        say_step "Installing gh via apt..."
        # shellcheck disable=SC2086
        $SUDO apt-get install -y --no-install-recommends gh \
            || say_warn "gh apt install failed"
    elif [ "$platform" = "macos" ] && have brew; then
        say_step "Installing gh via brew..."
        brew install gh || say_warn "gh brew install failed"
    else
        say_warn "cannot install gh automatically on this platform"
        return 0
    fi
    if have gh; then say_ok "gh ready"; fi
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

# --- commands -----------------------------------------------------------------
cmd_install() {
    resolve_items
    if [ "$FLAG_DRY_RUN" -eq 1 ]; then
        print_dry_run
        return 0
    fi
    platform="$(detect_platform)"
    say_step "Target toolchain: $(resolve_toolchain) (platform: $platform)"
    say_step "Items: $RESOLVED_ITEMS"
    if [ -f "$HOME/.cargo/env" ]; then . "$HOME/.cargo/env"; fi
    for item in $RESOLVED_ITEMS; do
        case "$item" in
            native-deps) install_native_deps "$platform" ;;
            rust)        install_rustup "$(resolve_toolchain)" ;;
            nodejs)      install_nodejs ;;
            pnpm)        install_pnpm ;;
            tauri-cli)   install_cargo_pkg tauri-cli cargo-tauri ;;
            sccache)     install_cargo_pkg sccache sccache ;;
            cargo-watch) install_cargo_pkg cargo-watch cargo-watch ;;
            gh)          install_gh ;;
        esac
    done
    say_ok "Install complete."
}

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

# --- ssh ----------------------------------------------------------------------
validate_ssh_target() {
    # $1 = target; must look like user@host, keys only (BatchMode).
    case "$1" in
        *@*) : ;;
        *) say_err "--ssh needs user@host (got '$1')"; exit 2 ;;
    esac
    user="${1%%@*}"; host="${1#*@}"
    if [ -z "$user" ] || [ -z "$host" ]; then
        say_err "--ssh needs user@host (got '$1')"; exit 2
    fi
    case "$host" in
        *[!a-zA-Z0-9.:-]*)
            say_err "--ssh host looks invalid: '$host'"; exit 2 ;;
    esac
}

run_remote() {
    # Re-invoke this script on the remote over stdin. Key auth only:
    # BatchMode=yes disables password prompts entirely.
    validate_ssh_target "$FLAG_SSH"
    if ! have ssh; then say_err "ssh client not found"; exit 1; fi
    say_step "Running remotely on $FLAG_SSH via SSH (key auth only)..."
    # shellcheck disable=SC2086
    ssh -o BatchMode=yes -o ConnectTimeout=15 "$FLAG_SSH" "sh -s -- $SSH_FWD" < "$0"
    rc=$?
    if [ "$rc" -eq 0 ]; then say_ok "remote run finished on $FLAG_SSH"
    else say_err "remote run failed with exit $rc"; fi
    return $rc
}

# --- main ---------------------------------------------------------------------
parse_args "$@"

if [ "$FLAG_LIST_ITEMS" -eq 1 ]; then
    list_items
    exit 0
fi

if [ "$FLAG_TROUBLESHOOT" -eq 1 ]; then
    print_troubleshoot
    exit 0
fi

if [ -n "$FLAG_SSH" ]; then
    run_remote
    exit $?
fi

case "$COMMAND" in
    install) cmd_install ;;
    check)   cmd_check ;;
    help)    print_top_help ;;
    *)       say_err "unknown command: $COMMAND"; exit 2 ;;
esac
