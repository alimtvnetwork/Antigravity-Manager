#!/bin/bash
set -e

# Antigravity Tools - Arch Linux Self-Updating Installer
# This script fetches release versions from GitHub and installs using makepkg with a 4-version fallback ladder.

PINNED_VERSION="__PINNED_VERSION__"
REPO="alimtvnetwork/Antigravity-Manager"
UPSTREAM_REPO="lbjlaq/Antigravity-Manager"

echo "🚀 Resolving release information..."

if [ -z "${VERSION:-}" ]; then
    if [ -n "$PINNED_VERSION" ] && [ "$PINNED_VERSION" != "__PINNED_VERSION__" ]; then
        VERSION="$PINNED_VERSION"
        echo "📌 Respecting pinned version: v$VERSION"
    fi
fi

CANDIDATES=()
if [ -n "${VERSION:-}" ]; then
    CANDIDATES+=("${VERSION#v}")
fi

RELEASES_JSON=$(curl -sSL --max-time 6 "https://api.github.com/repos/$REPO/releases?per_page=10" 2>/dev/null || echo "")
if [ -n "$RELEASES_JSON" ]; then
    TAGS=$(echo "$RELEASES_JSON" | grep '"tag_name"' | sed -E 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v?([^"]+)".*/\1/' | tr -d '[:space:]' || true)
    while IFS= read -r tag; do
        if [ -n "$tag" ]; then
            ALREADY=0
            for c in "${CANDIDATES[@]}"; do
                if [ "$c" = "$tag" ]; then ALREADY=1; break; fi
            done
            if [ $ALREADY -eq 0 ]; then
                CANDIDATES+=("$tag")
            fi
        fi
        if [ ${#CANDIDATES[@]} -ge 4 ]; then break; fi
    done <<< "$TAGS"
fi

FALLBACKS=("4.38.1" "4.38.0" "4.37.0" "4.36.0" "4.7.6")
for fb in "${FALLBACKS[@]}"; do
    if [ ${#CANDIDATES[@]} -ge 4 ]; then break; fi
    ALREADY=0
    for c in "${CANDIDATES[@]}"; do
        if [ "$c" = "$fb" ]; then ALREADY=1; break; fi
    done
    if [ $ALREADY -eq 0 ]; then
        CANDIDATES+=("$fb")
    fi
done

run_indented() {
    echo ""
    local exit_code=0
    set +e
    "$@" 2>&1 | sed $'s/^/\t/'
    exit_code="${PIPESTATUS[0]}"
    set -e
    echo ""
    return "$exit_code"
}

download_asset() {
    local url="$1"
    local output="$2"
    if command -v aria2c &>/dev/null; then
        run_indented aria2c --disable-ipv6=true -x 16 -s 80 -j 16 -k 500K \
            --allow-overwrite=true \
            --auto-file-renaming=false \
            --summary-interval=1 \
            --console-log-level=warn \
            -o "$output" "$url"
    else
        run_indented curl -fSL --progress-bar --connect-timeout 10 --retry 3 -o "$output" "$url"
    fi
}

MAX_ATTEMPTS=4
ATTEMPT=0
INSTALLED_OK=0

for PKGVER in "${CANDIDATES[@]}"; do
    ATTEMPT=$((ATTEMPT + 1))
    if [ $ATTEMPT -gt $MAX_ATTEMPTS ]; then break; fi

    echo "📦 Installation attempt $ATTEMPT of $MAX_ATTEMPTS: evaluating v$PKGVER..."

    REL_INFO=$(curl -sSL --max-time 6 "https://api.github.com/repos/$REPO/releases/tags/v$PKGVER" 2>/dev/null || echo "")
    URL_X86_64=$(echo "$REL_INFO" | grep -oP '"browser_download_url": "\K[^"]*amd64\.deb' | head -n 1)
    URL_AARCH64=$(echo "$REL_INFO" | grep -oP '"browser_download_url": "\K[^"]*arm64\.deb' | head -n 1)

    if [ -z "$URL_X86_64" ]; then
        URL_X86_64="https://github.com/$REPO/releases/download/v${PKGVER}/Antigravity.Tools_${PKGVER}_amd64.deb"
    fi
    if [ -z "$URL_AARCH64" ]; then
        URL_AARCH64="https://github.com/$REPO/releases/download/v${PKGVER}/Antigravity.Tools_${PKGVER}_arm64.deb"
    fi

    TEMP_DIR=$(mktemp -d)
    SUCCESS_STEP=0

    (
        cd "$TEMP_DIR"
        echo "🔍 Downloading assets for v$PKGVER..."
        download_asset "$URL_X86_64" "x86_64.deb" || exit 1
        download_asset "$URL_AARCH64" "aarch64.deb" || exit 1

        if [ ! -s "x86_64.deb" ] || [ ! -s "aarch64.deb" ]; then
            exit 1
        fi

        SHA_X86_64=$(sha256sum x86_64.deb | cut -d' ' -f1)
        SHA_AARCH64=$(sha256sum aarch64.deb | cut -d' ' -f1)

        echo "📝 Generating PKGBUILD..."
        curl -sSL "https://raw.githubusercontent.com/$REPO/main/deploy/arch/PKGBUILD.template" -o PKGBUILD.template || exit 1

        sed -e "s/\${_pkgver}/$PKGVER/g" \
            -e "s|\${_url_x86_64}|$URL_X86_64|g" \
            -e "s|\${_url_aarch64}|$URL_AARCH64|g" \
            -e "s/\${_sha256_x86_64}/$SHA_X86_64/g" \
            -e "s/\${_sha256_aarch64}/$SHA_AARCH64/g" \
            PKGBUILD.template > PKGBUILD

        echo "🛠️ Starting installation via makepkg..."
        run_indented makepkg -si --noconfirm || exit 1
    ) && SUCCESS_STEP=1

    rm -rf "$TEMP_DIR" 2>/dev/null || true

    if [ $SUCCESS_STEP -eq 1 ]; then
        INSTALLED_OK=1
        echo "✅ Installation of v$PKGVER completed successfully!"
        break
    else
        echo "⚠️ Attempt $ATTEMPT failed for v$PKGVER."
        if [ $ATTEMPT -lt $MAX_ATTEMPTS ]; then
            echo "🔄 Falling back to previous release version..."
        fi
    fi
done

if [ $INSTALLED_OK -eq 0 ]; then
    echo "❌ All $MAX_ATTEMPTS attempts failed. I fail, so I cannot do anything."
    exit 1
fi
