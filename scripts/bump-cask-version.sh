#!/usr/bin/env bash
# bump-cask-version.sh — sync Casks/antigravity-tools.rb with package.json version.
#
# Usage:
#   ./scripts/bump-cask-version.sh           Update cask version to match package.json
#   ./scripts/bump-cask-version.sh --check    Report only; do not modify the cask
#   ./scripts/bump-cask-version.sh --help     Show this help
#
# After updating, the script probes the macOS DMG release URLs for both
# architectures. A non-200 result is a warning only (the release may not be
# published yet) — the cask is still updated.
set -u

REPO="alimtvnetwork/Antigravity-Manager"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CASK_FILE="$REPO_ROOT/Casks/antigravity-tools.rb"
PKG_JSON="$REPO_ROOT/package.json"

show_help() {
    sed -n '2,11p' "${BASH_SOURCE[0]}" | sed 's/^# \?//'
    exit 0
}

CHECK_ONLY=0
for arg in "$@"; do
    case "$arg" in
        --help|-h) show_help ;;
        --check)  CHECK_ONLY=1 ;;
        *)
            echo "Error: unknown argument '$arg' (try --help)" >&2
            exit 2
            ;;
    esac
done

[ -f "$PKG_JSON" ] || { echo "Error: package.json not found at $PKG_JSON" >&2; exit 1; }
[ -f "$CASK_FILE" ] || { echo "Error: cask file not found at $CASK_FILE" >&2; exit 1; }

NEW_VER="$(grep -E '"version"[[:space:]]*:' "$PKG_JSON" | head -n1 \
    | sed -E 's/.*"version"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/')"
if [ -z "$NEW_VER" ]; then
    echo "Error: could not parse version from $PKG_JSON" >&2
    exit 1
fi
if ! [[ "$NEW_VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
    echo "Error: parsed version '$NEW_VER' does not look like x.y.z" >&2
    exit 1
fi

OLD_VER="$(grep -E '^[[:space:]]*version "' "$CASK_FILE" | head -n1 \
    | sed -E 's/^[[:space:]]*version "([^"]+)".*/\1/')"
if [ -z "$OLD_VER" ]; then
    echo "Error: could not parse version line from $CASK_FILE" >&2
    exit 1
fi

if [ "$OLD_VER" = "$NEW_VER" ]; then
    echo "Cask version already in sync with package.json ($NEW_VER)."
    exit 0
fi

echo "package.json version: $NEW_VER"
echo "cask version:         $OLD_VER"

if [ "$CHECK_ONLY" -eq 1 ]; then
    echo "CHECK: cask would be updated $OLD_VER -> $NEW_VER (no changes made)."
    exit 0
fi

# Replace only the top-level `version "..."` line (leading whitespace tolerated).
tmp="$(mktemp)" || { echo "Error: could not create temp file" >&2; exit 1; }
if ! sed -E "s/^([[:space:]]*version \")[^\"]+(\".*)$/\1${NEW_VER}\2/" "$CASK_FILE" > "$tmp"; then
    echo "Error: sed failed on $CASK_FILE" >&2
    rm -f "$tmp"
    exit 1
fi
# Sanity: exactly one version line must remain, and it must carry the new version.
if [ "$(grep -cE "^[[:space:]]*version " "$tmp")" -ne 1 ] \
    || ! grep -qE "^[[:space:]]*version \"${NEW_VER}\"" "$tmp"; then
    echo "Error: unexpected result after version substitution; cask left untouched." >&2
    rm -f "$tmp"
    exit 1
fi
mv "$tmp" "$CASK_FILE"
echo "Updated cask version: $OLD_VER -> $NEW_VER"

# Verify the macOS DMG release URLs for both architectures.
fail=0
for arch in aarch64 x64; do
    url="https://github.com/${REPO}/releases/download/v${NEW_VER}/Antigravity.Manager.Tools_${NEW_VER}_${arch}.dmg"
    code="$(curl -sSLI -o /dev/null -w '%{http_code}' --max-time 10 "$url" 2>/dev/null)" || code="000"
    if [ "$code" = "200" ]; then
        echo "OK (HTTP 200): $url"
    else
        echo "WARNING: $url returned HTTP $code — the v${NEW_VER} release may not be published yet." >&2
        fail=1
    fi
done

if [ "$fail" -eq 0 ]; then
    echo "Done: cask synced to $NEW_VER and both DMG URLs are reachable."
else
    echo "Done: cask synced to $NEW_VER, but one or more DMG URLs are not reachable yet."
fi
exit 0
