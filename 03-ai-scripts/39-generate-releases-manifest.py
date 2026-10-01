#!/usr/bin/env python3
"""
Generate and synchronize releases-manifest.json containing recent releases, tag URLs, and platform assets.
Consumed by install.ps1 and install.sh for instant, rate-limit-free CDN version resolution.
"""

import argparse
from datetime import datetime, timezone
import json
import os
import subprocess
import sys
import urllib.request
import urllib.error


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate releases manifest JSON for installers.")
    parser.add_argument("--repo", default="alimtvnetwork/Antigravity-Manager", help="Repository slug (owner/repo)")
    parser.add_argument("--output", default="releases-manifest.json", help="Output JSON path")
    parser.add_argument("--target-version", default="", help="Optional explicit target version to inject/prepend")
    parser.add_argument("--limit", type=int, default=15, help="Number of releases to include")
    parser.add_argument("--token", default=os.getenv("GITHUB_TOKEN", ""), help="GitHub Personal Access Token")
    return parser.parse_args()


def resolve_auth_token(explicit_token: str) -> str:
    if explicit_token:
        return explicit_token
    try:
        cmd = ["gh", "auth", "token"]
        res = subprocess.run(cmd, capture_output=True, text=True, check=False)
        tok = res.stdout.strip()
        if tok and not tok.startswith("error"):
            return tok
    except Exception:
        pass
    return ""


def build_request_headers(token: str) -> dict:
    headers = {
        "Accept": "application/vnd.github.v3+json",
        "User-Agent": "Antigravity-Manifest-Generator",
    }
    if token:
        headers["Authorization"] = f"token {token}"
    return headers


def fetch_raw_releases(repo: str, limit: int, headers: dict) -> list:
    url = f"https://api.github.com/repos/{repo}/releases?per_page={limit}"
    req = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=12) as resp:
            if resp.status == 200:
                return json.loads(resp.read().decode("utf-8"))
    except Exception as e:
        print(f"[WARN] Failed to fetch releases from GitHub API ({e}); using local fallback.")
    return []


def match_asset_url(assets: list, patterns: list) -> str:
    for pat in patterns:
        for asset in assets:
            name = asset.get("name", "")
            dl_url = asset.get("browser_download_url", "")
            if pat in name:
                return dl_url
    return ""


def build_platform_assets(assets: list, tag: str) -> dict:
    # If live release assets are attached, match them
    if assets:
        return {
            "windows_x64_setup": match_asset_url(assets, ["agm-alim-setup.exe", "x64-setup.exe", "setup.exe"]),
            "windows_x64_zip": match_asset_url(assets, ["windows_x64.zip", "win_x64.zip", ".zip"]),
            "windows_x64_msi": match_asset_url(assets, ["x64_en-US.msi", ".msi"]),
            "macos_x64_dmg": match_asset_url(assets, ["x64.dmg"]),
            "macos_aarch64_dmg": match_asset_url(assets, ["aarch64.dmg", "arm64.dmg"]),
            "linux_amd64_appimage": match_asset_url(assets, ["amd64.AppImage", "x86_64.AppImage"]),
            "linux_aarch64_appimage": match_asset_url(assets, ["aarch64.AppImage", "arm64.AppImage"]),
            "linux_amd64_deb": match_asset_url(assets, ["amd64.deb"]),
            "linux_aarch64_deb": match_asset_url(assets, ["arm64.deb", "aarch64.deb"]),
            "linux_x64_rpm": match_asset_url(assets, ["x86_64.rpm"]),
            "linux_aarch64_rpm": match_asset_url(assets, ["aarch64.rpm", "arm64.rpm"]),
        }

    # Deterministic fallback URLs for pre-release staging or planned releases
    ver = tag.lstrip("v")
    base_dl = f"https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/{tag}"
    return {
        "windows_x64_setup": f"{base_dl}/agm-alim-setup.exe",
        "windows_x64_zip": f"{base_dl}/agm-alim_{ver}_windows_x64.zip",
        "windows_x64_msi": "",
        "macos_x64_dmg": f"{base_dl}/Antigravity.Manager.Tools_{ver}_x64.dmg",
        "macos_aarch64_dmg": f"{base_dl}/Antigravity.Manager.Tools_{ver}_aarch64.dmg",
        "linux_amd64_appimage": "",
        "linux_aarch64_appimage": "",
        "linux_amd64_deb": f"{base_dl}/Antigravity.Manager.Tools_{ver}_amd64.deb",
        "linux_aarch64_deb": f"{base_dl}/Antigravity.Manager.Tools_{ver}_arm64.deb",
        "linux_x64_rpm": "",
        "linux_aarch64_rpm": "",
    }


def map_single_release(rel: dict, repo: str) -> dict:
    tag = rel.get("tag_name", "")
    version = tag.lstrip("v")
    raw_assets = rel.get("assets", [])
    return {
        "version": version,
        "tag": tag,
        "tag_url": f"https://github.com/{repo}/releases/tag/{tag}",
        "raw_tag_url": f"https://raw.githubusercontent.com/{repo}/{tag}",
        "release_url": rel.get("html_url", f"https://github.com/{repo}/releases/tag/{tag}"),
        "published_at": rel.get("published_at", datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")),
        "prerelease": bool(rel.get("prerelease", False)),
        "assets": build_platform_assets(raw_assets, tag),
    }


def create_release_entry(ver: str, repo: str) -> dict:
    ver_clean = ver.lstrip("v")
    tag = f"v{ver_clean}"
    return {
        "version": ver_clean,
        "tag": tag,
        "tag_url": f"https://github.com/{repo}/releases/tag/{tag}",
        "raw_tag_url": f"https://raw.githubusercontent.com/{repo}/{tag}",
        "release_url": f"https://github.com/{repo}/releases/tag/{tag}",
        "published_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "prerelease": "beta" in ver_clean.lower(),
        "assets": build_platform_assets([], tag),
    }


def compile_manifest_dict(repo: str, releases: list) -> dict:
    latest_ver = releases[0]["version"] if releases else ""
    latest_tag = releases[0]["tag"] if releases else ""
    return {
        "$schema": f"https://raw.githubusercontent.com/{repo}/main/02-spec/schema/releases-manifest.schema.json",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "repo": repo,
        "latest_version": latest_ver,
        "latest_tag": latest_tag,
        "latest_tag_url": f"https://github.com/{repo}/releases/tag/{latest_tag}" if latest_tag else "",
        "latest_raw_tag_url": f"https://raw.githubusercontent.com/{repo}/{latest_tag}" if latest_tag else "",
        "releases": releases,
    }


def main():
    args = parse_arguments()
    token = resolve_auth_token(args.token)
    headers = build_request_headers(token)
    raw_list = fetch_raw_releases(args.repo, args.limit, headers)

    existing_manifest = {}
    if os.path.exists(args.output):
        try:
            with open(args.output, "r", encoding="utf-8") as f:
                existing_manifest = json.load(f)
        except Exception:
            pass

    clean_releases = []
    if raw_list:
        clean_releases = [map_single_release(r, args.repo) for r in raw_list if not r.get("draft", False)]
    elif existing_manifest and "releases" in existing_manifest:
        clean_releases = existing_manifest["releases"]

    # Prepend target version if requested and not already present
    if args.target_version:
        target_clean = args.target_version.lstrip("v")
        existing_versions = [r["version"] for r in clean_releases]
        if target_clean not in existing_versions:
            new_entry = create_release_entry(target_clean, args.repo)
            clean_releases.insert(0, new_entry)
        else:
            # Move existing matching entry to top if it matches target
            idx = existing_versions.index(target_clean)
            if idx > 0:
                entry = clean_releases.pop(idx)
                clean_releases.insert(0, entry)

    # Bound releases to limit
    if len(clean_releases) > args.limit:
        clean_releases = clean_releases[:args.limit]

    manifest = compile_manifest_dict(args.repo, clean_releases)

    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)

    top_ver = manifest.get("latest_version", "unknown")
    print(f"[OK] Successfully wrote {len(clean_releases)} releases to {args.output} (latest: v{top_ver})")


if __name__ == "__main__":
    main()
