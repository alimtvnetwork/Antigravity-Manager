#!/usr/bin/env sh
# ============================================================================
#  clear-downloads.sh — Clear Antigravity-Manager build toolchain caches
# ============================================================================
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TOOLCHAIN_SCRIPT="${SCRIPT_DIR}/scripts/install-rust-toolchain.sh"

if [ -f "$TOOLCHAIN_SCRIPT" ]; then
    sh "$TOOLCHAIN_SCRIPT" --clear-downloads "$@"
else
    echo "  [ERROR] Toolchain script not found: $TOOLCHAIN_SCRIPT" >&2
    exit 1
fi
