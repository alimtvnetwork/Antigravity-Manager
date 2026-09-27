#!/usr/bin/env python3
"""Generate releases-manifest.json containing the last 10 releases, tag URLs, and platform asset links."""

import argparse
from datetime import datetime, timezone
import json
import os
import subprocess
import sys
import urllib.request


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate releases manifest JSON for installers.")
    parser.add_argument("--repo", default="alimtvnetwork/Antigravity-Manager", help="Repository slug (owner/repo)")
    parser.add_argument("--output", default="releases-manifest.json", help="Output JSON path")
    parser.add_argument("--limit", type=int, default=10, help="Number of releases to include")
    parser.add_argument("--token", default=os.getenv("GITHUB_TOKEN", ""), help="GitHub Personal Access Token")
    return parser.parse_args()


def resolve_auth_token(explicit_token: str) -> str:
    if explicit_token:
        return explicit_token
    try:
        cmd = ["gh", "auth", "token"]
        res = subprocess.run(cmd, capture_output=True, text=True, check=False)
        return res.stdout.strip()
    except Exception:
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
    with urllib.request.urlopen(req, timeout=15) as resp:
        if resp.status == 200:
            return json.loads(resp.read().decode("utf-8"))
    return []


def match_asset_url(assets: list, patterns: list) -> str:
    for pat in patterns:
        for asset in assets:
            name = asset.get("name", "")
            dl_url = asset.get("browser_download_url", "")
            if pat in name:
                return dl_url
    return ""


def build_platform_assets(assets: list) -> dict:
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
        "published_at": rel.get("published_at", ""),
        "prerelease": bool(rel.get("prerelease", False)),
        "assets": build_platform_assets(raw_assets),
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
    
    clean_releases = [map_single_release(r, args.repo) for r in raw_list if not r.get("draft", False)]
    manifest = compile_manifest_dict(args.repo, clean_releases)

    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)

    print(f"[OK] Successfully wrote {len(clean_releases)} releases to {args.output}")


if __name__ == "__main__":
    main()
