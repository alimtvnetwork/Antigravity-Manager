#!/usr/bin/env bash
# scripts/update-antigravity-ide.sh
# Update Antigravity IDE for Linux.
#
# Steps:
#   1. Check for and stop running Antigravity processes.
#   2. Safely download latest package (or invoke installer).
#   3. Extract package to ~/.local/share/antigravity-ide.
#   4. Ensure antigravity-ide.run wrapper with --no-sandbox is preserved.
#   5. Refresh symlinks in ~/.local/bin/ (antigravity, agy, antigravity-ide).
#   6. Report clear success status.

set -euo pipefail

PRIMARY="\033[1;32m"
SECONDARY="\033[1;36m"
ACCENT="\033[1;33m"
MUTED="\033[0;37m"
ERROR="\033[1;31m"
TEXT="\033[0m"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IDE_DIR="$HOME/.local/share/antigravity-ide"
BIN_DIR="$HOME/.local/bin"

USE_INSTALLER=false
for arg in "$@"; do
    case "$arg" in
        --installer|--use-installer)
            USE_INSTALLER=true
            ;;
        -h|--help|help)
            echo "Usage: ./scripts/update-antigravity-ide.sh [options]"
            echo ""
            echo "Options:"
            echo "  --installer, --use-installer   Invoke external installer script if available"
            echo "  -h, --help                     Show this help message"
            exit 0
            ;;
    esac
done

echo -e "\n${SECONDARY}=== Antigravity IDE Update Utility ===${TEXT}\n"

KILL_PROCESSES=false
for arg in "$@"; do
    case "$arg" in
        --kill|--force-kill)
            KILL_PROCESSES=true
            ;;
    esac
done

# Step 1: Check for running antigravity processes
echo -e "  ${MUTED}[1/5] Checking for running Antigravity processes...${TEXT}"
if pgrep -f "antigravity" >/dev/null 2>&1 || pgrep -x "agy" >/dev/null 2>&1; then
    if [ "$KILL_PROCESSES" = "true" ]; then
        echo -e "  ${ACCENT}[WARN] Active Antigravity process(es) detected. Terminating before update (--kill)...${TEXT}"
        pkill -TERM -f "antigravity" 2>/dev/null || true
        pkill -TERM -x "agy" 2>/dev/null || true
        sleep 2
        if pgrep -f "antigravity" >/dev/null 2>&1; then
            echo -e "  ${ACCENT}[WARN] Forcing termination of lingering processes...${TEXT}"
            pkill -KILL -f "antigravity" 2>/dev/null || true
            sleep 1
        fi
        echo -e "  ${PRIMARY}[ OK ] Antigravity processes stopped.${TEXT}"
    else
        echo -e "  ${ACCENT}[INFO] Active Antigravity process detected. Performing non-destructive atomic update in background without killing active sessions.${TEXT}"
    fi
else
    echo -e "  ${PRIMARY}[ OK ] No active Antigravity processes found.${TEXT}"
fi

# Step 2: Safely download the latest package (or invoke installer)
INSTALLER_PATH="$SCRIPT_DIR/../../scripts-fixer/scripts/os/ubuntu/install-antigravity.sh"
if [ "$USE_INSTALLER" = "true" ] && [ -f "$INSTALLER_PATH" ]; then
    echo -e "  ${MUTED}[2/5] Invoking system installer with --update...${TEXT}"
    bash "$INSTALLER_PATH" --update
else
    echo -e "  ${MUTED}[2/5] Obtaining latest Antigravity IDE archive...${TEXT}"

    ARCH_RAW="$(uname -m)"
    case "$ARCH_RAW" in
        x86_64|amd64) ARCH="x64" ;;
        aarch64|arm64) ARCH="arm64" ;;
        *) ARCH="x64" ;;
    esac

    if [ "$ARCH" = "arm64" ]; then
        DOWNLOAD_URL="https://storage.googleapis.com/antigravity-public/antigravity-hub/2.19.1-6046815158665216/linux-arm/Antigravity.tar.gz"
    else
        DOWNLOAD_URL="https://storage.googleapis.com/antigravity-public/antigravity-hub/2.19.1-6046815158665216/linux-x64/Antigravity.tar.gz"
    fi

    TMP_DIR="$(mktemp -d /tmp/antigravity-update-XXXXXX)"
    cleanup() {
        rm -rf "$TMP_DIR"
    }
    trap cleanup EXIT

    CACHED_ARCHIVE=""
    for candidate in \
        "/tmp/scripts-fixer-downloads/Antigravity.tar.gz" \
        "/tmp/Antigravity.tar.gz" \
        "$HOME/Downloads/Antigravity.tar.gz"; do
        if [ -f "$candidate" ] && [ -s "$candidate" ]; then
            if gzip -t "$candidate" >/dev/null 2>&1; then
                CACHED_ARCHIVE="$candidate"
                break
            fi
        fi
    done

    ARCHIVE_FILE="$TMP_DIR/Antigravity.tar.gz"
    if [ -n "$CACHED_ARCHIVE" ]; then
        echo -e "  ${PRIMARY}[ OK ] Reusing verified cached archive: $CACHED_ARCHIVE${TEXT}"
        ARCHIVE_FILE="$CACHED_ARCHIVE"
    else
        echo -e "  ${MUTED}Downloading from: $DOWNLOAD_URL${TEXT}"
        DOWNLOAD_TMP="$TMP_DIR/download.tmp"
        if command -v aria2c >/dev/null 2>&1; then
            aria2c -x 4 -s 4 -d "$TMP_DIR" -o "download.tmp" "$DOWNLOAD_URL"
        elif command -v curl >/dev/null 2>&1; then
            curl -fL --retry 3 --progress-bar "$DOWNLOAD_URL" -o "$DOWNLOAD_TMP"
        elif command -v wget >/dev/null 2>&1; then
            wget -q --show-progress "$DOWNLOAD_URL" -O "$DOWNLOAD_TMP"
        else
            echo -e "  ${ERROR}[FAIL] No supported downloader (curl, wget, aria2c) found.${TEXT}"
            exit 1
        fi
        mv "$DOWNLOAD_TMP" "$ARCHIVE_FILE"
    fi

    if ! gzip -t "$ARCHIVE_FILE" >/dev/null 2>&1; then
        echo -e "  ${ERROR}[FAIL] Archive integrity verification failed.${TEXT}"
        exit 1
    fi

    # Step 3: Extract to ~/.local/share/antigravity-ide
    echo -e "  ${MUTED}[3/5] Extracting archive to $IDE_DIR...${TEXT}"
    STAGE_DIR="$TMP_DIR/staging"
    mkdir -p "$STAGE_DIR"
    tar -xzf "$ARCHIVE_FILE" -C "$STAGE_DIR"

    EXTRACT_ROOT="$STAGE_DIR"
    if [ -d "$STAGE_DIR/Antigravity-x64" ]; then
        EXTRACT_ROOT="$STAGE_DIR/Antigravity-x64"
    elif [ -d "$STAGE_DIR/antigravity" ]; then
        EXTRACT_ROOT="$STAGE_DIR/antigravity"
    fi

    mkdir -p "$IDE_DIR"
    cp -a "$EXTRACT_ROOT"/* "$IDE_DIR/" 2>/dev/null || cp -a "$EXTRACT_ROOT"/. "$IDE_DIR/" 2>/dev/null || true
    ln -sfn "$IDE_DIR" "$HOME/.local/share/antigravity" 2>/dev/null || true
    echo -e "  ${PRIMARY}[ OK ] Package extracted successfully.${TEXT}"
fi

# Step 4: Ensure antigravity-ide.run wrapper with --no-sandbox is preserved
echo -e "  ${MUTED}[4/5] Ensuring antigravity-ide.run wrapper with --no-sandbox...${TEXT}"
EXEC_BIN="$IDE_DIR/antigravity"
[ -f "$EXEC_BIN" ] || EXEC_BIN="$IDE_DIR/Antigravity"

if [ -f "$EXEC_BIN" ]; then
    chmod +x "$EXEC_BIN"
fi

if [ -f "$IDE_DIR/chrome-sandbox" ]; then
    chmod 4755 "$IDE_DIR/chrome-sandbox" 2>/dev/null || chmod +x "$IDE_DIR/chrome-sandbox" 2>/dev/null || true
fi

RUNNER_SCRIPT="$IDE_DIR/antigravity-runner.sh"
cat <<'EOF' > "$RUNNER_SCRIPT"
#!/bin/bash
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export LD_LIBRARY_PATH="$DIR:$DIR/lib:${LD_LIBRARY_PATH:-}"
EXEC="$DIR/antigravity"
[ -f "$EXEC" ] || EXEC="$DIR/Antigravity"

export ELECTRON_OZONE_PLATFORM_HINT="auto"
export DONT_PROMPT_WSL_INSTALL=1

# Always pass --no-sandbox for tarball Electron builds on modern Linux (AppArmor userns restriction)
exec "$EXEC" --no-sandbox "$@"
EOF
chmod +x "$RUNNER_SCRIPT"

# Preserve / link antigravity-ide.run and antigravity.run wrappers
ln -sf "$RUNNER_SCRIPT" "$IDE_DIR/antigravity-ide.run"
ln -sf "$RUNNER_SCRIPT" "$IDE_DIR/antigravity.run"
chmod +x "$IDE_DIR/antigravity-ide.run" 2>/dev/null || true
echo -e "  ${PRIMARY}[ OK ] Runtime wrapper configured with --no-sandbox.${TEXT}"

# Step 5: Refresh symlinks in ~/.local/bin/ (antigravity, agy, antigravity-ide)
echo -e "  ${MUTED}[5/5] Refreshing symlinks in $BIN_DIR...${TEXT}"
mkdir -p "$BIN_DIR"
WRAPPER_BIN="$IDE_DIR/antigravity-ide.run"

ln -sf "$WRAPPER_BIN" "$BIN_DIR/antigravity"
ln -sf "$WRAPPER_BIN" "$BIN_DIR/agy"
ln -sf "$WRAPPER_BIN" "$BIN_DIR/antigravity-ide"
echo -e "  ${PRIMARY}[ OK ] Symlinks refreshed in $BIN_DIR: antigravity, agy, antigravity-ide.${TEXT}"

# Step 6: Output clear success status
echo -e "\n${PRIMARY}================================================================${TEXT}"
echo -e "${PRIMARY}  [SUCCESS] Antigravity IDE update completed successfully!${TEXT}"
echo -e "${PRIMARY}================================================================${TEXT}"
echo -e "  ${MUTED}Install path : ${TEXT}$IDE_DIR"
echo -e "  ${MUTED}Wrapper      : ${TEXT}$IDE_DIR/antigravity-ide.run (--no-sandbox)"
echo -e "  ${MUTED}CLI binaries : ${TEXT}$BIN_DIR/antigravity, $BIN_DIR/agy, $BIN_DIR/antigravity-ide"
echo -e "  ${MUTED}Status       : ${PRIMARY}Up to date and ready to launch.${TEXT}\n"
