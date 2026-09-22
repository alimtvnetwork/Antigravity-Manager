#!/usr/bin/env python3
"""
37-bump-version.py - Autonomous SemVer Version Bumper & Manifest Synchronizer

Bumps version strings across repository manifests, documentation, and changelogs:
  1. Resolves canonical version from version.json or package.json.
  2. Calculates next SemVer (minor default per Rule 0, patch resets to 0).
  3. Updates version.json, package.json, readme.md, and changelog.md.
  4. Triggers npm run sync if defined in package.json to regenerate artifacts.
  5. Adheres strictly to repository-aware configuration and file conventions.

Usage:
  python 03-ai-scripts/37-bump-version.py
  python 03-ai-scripts/37-bump-version.py --tier patch
  python 03-ai-scripts/37-bump-version.py --tier minor --scope "Feature release"
  python 03-ai-scripts/37-bump-version.py --tier major --scope "Breaking change"
  python 03-ai-scripts/37-bump-version.py --version 6.42.0
  python 03-ai-scripts/37-bump-version.py --dry-run
"""

import argparse
import datetime
import json
import os
import re
import subprocess
import sys
from pathlib import Path

# Repository root discovery
REPO_ROOT = Path(__file__).resolve().parent.parent

# Canonical version files
VERSION_JSON = REPO_ROOT / "version.json"
PACKAGE_JSON = REPO_ROOT / "package.json"
README_MD = REPO_ROOT / "readme.md"
CHANGELOG_MD = REPO_ROOT / "changelog.md"
CHANGELOG_ROOT_MD = REPO_ROOT / "CHANGELOG.md"
CHANGELOG_EN_MD = REPO_ROOT / "CHANGELOG_EN.md"
TAURI_CONF = REPO_ROOT / "src-tauri" / "tauri.conf.json"
CARGO_TOML = REPO_ROOT / "src-tauri" / "Cargo.toml"
SPEC19_CHANGELOG = REPO_ROOT / "02-spec" / "19-main-worker-service" / "98-changelog.md"
TEMPLATE_VERSION = REPO_ROOT / "prompt-version.template.json"
HOMEBREW_CASK = REPO_ROOT / "Casks" / "antigravity-tools.rb"


def run_cmd(cmd, cwd=None, check=True, capture_output=True):
    """Executes a command with cross-platform safety."""
    target_cwd = cwd or str(REPO_ROOT)
    result = subprocess.run(
        cmd,
        cwd=target_cwd,
        shell=False,
        check=check,
        capture_output=capture_output,
        text=True,
    )

    return result


def read_canonical_version():
    """Reads current SemVer from version.json or package.json."""
    if VERSION_JSON.is_file():
        try:
            with open(VERSION_JSON, "r", encoding="utf-8") as f:
                data = json.load(f)

            raw_ver = data.get("Version") or data.get("version")
            if raw_ver:
                return str(raw_ver).strip()
        except Exception:
            pass

    if PACKAGE_JSON.is_file():
        try:
            with open(PACKAGE_JSON, "r", encoding="utf-8") as f:
                data = json.load(f)

            raw_ver = data.get("version")
            if raw_ver:
                return str(raw_ver).strip()
        except Exception:
            pass

    raise FileNotFoundError("Could not find canonical version in version.json or package.json.")


def parse_semver(ver_str):
    """Parses X.Y.Z into a tuple of ints (major, minor, patch)."""
    clean_ver = ver_str.lstrip("v")
    match = re.match(r"^(\d+)\.(\d+)\.(\d+)$", clean_ver)
    if not match:
        raise ValueError(f"Invalid SemVer format: '{ver_str}' (expected X.Y.Z)")

    return int(match.group(1)), int(match.group(2)), int(match.group(3))


def calculate_next_version(current_ver, tier):
    """Calculates next SemVer based on tier (Rule 0: default minor, patch resets to 0)."""
    major, minor, patch = parse_semver(current_ver)

    if tier == "patch":
        patch += 1
    elif tier == "minor":
        minor += 1
        patch = 0
    elif tier == "major":
        major += 1
        minor = 0
        patch = 0
    else:
        raise ValueError(f"Unknown bump tier: '{tier}'. Expected patch, minor, or major.")

    return f"{major}.{minor}.{patch}"


def update_version_json(next_version, today_str, dry_run=False):
    """Updates version and releaseDate in version.json."""
    if not VERSION_JSON.is_file():
        return

    with open(VERSION_JSON, "r", encoding="utf-8") as f:
        data = json.load(f)

    data["version"] = next_version
    if "Version" in data:
        data["Version"] = next_version
    data["releaseDate"] = today_str

    if dry_run:
        print(f"[DRY RUN] Would update version.json to {next_version} ({today_str})")
        return

    with open(VERSION_JSON, "w", encoding="utf-8", newline="\n") as f:
        json.dump(data, f, indent=2)
        f.write("\n")

    print(f"[*] Updated version.json -> {next_version}")


def update_package_json(next_version, dry_run=False):
    """Updates version in package.json."""
    if not PACKAGE_JSON.is_file():
        return

    with open(PACKAGE_JSON, "r", encoding="utf-8") as f:
        data = json.load(f)

    data["version"] = next_version

    if dry_run:
        print(f"[DRY RUN] Would update package.json to {next_version}")
        return

    with open(PACKAGE_JSON, "w", encoding="utf-8", newline="\n") as f:
        json.dump(data, f, indent=2)
        f.write("\n")

    print(f"[*] Updated package.json -> {next_version}")


def update_tauri_conf(next_version, dry_run=False):
    """Updates version in src-tauri/tauri.conf.json."""
    if not TAURI_CONF.is_file():
        return

    with open(TAURI_CONF, "r", encoding="utf-8") as f:
        data = json.load(f)

    data["version"] = next_version

    if dry_run:
        print(f"[DRY RUN] Would update src-tauri/tauri.conf.json to {next_version}")
        return

    with open(TAURI_CONF, "w", encoding="utf-8", newline="\n") as f:
        json.dump(data, f, indent=2)
        f.write("\n")

    print(f"[*] Updated src-tauri/tauri.conf.json -> {next_version}")


def update_cargo_toml(next_version, dry_run=False):
    """Updates version in src-tauri/Cargo.toml."""
    if not CARGO_TOML.is_file():
        return

    with open(CARGO_TOML, "r", encoding="utf-8") as f:
        content = f.read()

    new_content = re.sub(
        r'(\[package\][^\[]*?version\s*=\s*")[^"]+(")',
        rf'\g<1>{next_version}\g<2>',
        content,
        count=1,
        flags=re.DOTALL
    )

    if new_content == content:
        return

    if dry_run:
        print(f"[DRY RUN] Would update src-tauri/Cargo.toml to {next_version}")
        return

    with open(CARGO_TOML, "w", encoding="utf-8", newline="\n") as f:
        f.write(new_content)

    print(f"[*] Updated src-tauri/Cargo.toml -> {next_version}")


def update_template_version(next_version, dry_run=False):
    """Updates prompt-version.template.json if present."""
    if not TEMPLATE_VERSION.is_file():
        return

    with open(TEMPLATE_VERSION, "r", encoding="utf-8") as f:
        data = json.load(f)

    data["version"] = next_version

    if dry_run:
        print(f"[DRY RUN] Would update prompt-version.template.json to {next_version}")
        return

    with open(TEMPLATE_VERSION, "w", encoding="utf-8", newline="\n") as f:
        json.dump(data, f, indent=2)
        f.write("\n")

    print(f"[*] Updated prompt-version.template.json -> {next_version}")


def update_homebrew_cask(next_version, dry_run=False):
    """Updates version in Casks/antigravity-tools.rb."""
    if not HOMEBREW_CASK.is_file():
        return

    with open(HOMEBREW_CASK, "r", encoding="utf-8") as f:
        content = f.read()

    new_content = re.sub(
        r'(version\s+")[^"]+(")',
        rf'\g<1>{next_version}\g<2>',
        content,
        count=1
    )

    if new_content == content:
        return

    if dry_run:
        print(f"[DRY RUN] Would update Casks/antigravity-tools.rb to {next_version}")
        return

    with open(HOMEBREW_CASK, "w", encoding="utf-8", newline="\n") as f:
        f.write(new_content)

    print(f"[*] Updated Casks/antigravity-tools.rb -> {next_version}")


def update_readme_pins(current_ver, next_version, today_str=None, dry_run=False):
    """Pins new version in readme.md badges, table, and text references."""
    if not README_MD.is_file():
        return

    if not today_str:
        today_str = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")

    with open(README_MD, "r", encoding="utf-8") as f:
        content = f.read()

    escaped_curr = re.escape(current_ver)
    new_content = re.sub(rf"Version-{escaped_curr}", f"Version-{next_version}", content)
    new_content = re.sub(rf"alt=\"Version {escaped_curr}\"", f'alt="Version {next_version}"', new_content)
    new_content = re.sub(
        r"releases/download/v[0-9]+\.[0-9]+\.[0-9]+[^/]*/",
        f"releases/download/v{next_version}/",
        new_content,
    )
    new_content = re.sub(
        r"e\.g\.\s*v[0-9]+\.[0-9]+\.[0-9]+[^)]*\)",
        f"e.g. v{next_version})",
        new_content,
    )

    # Maintain Version Pinning Table between anchors
    start_anchor = "<!-- STAMP:VERSION_PIN_TABLE_START -->"
    end_anchor = "<!-- STAMP:VERSION_PIN_TABLE_END -->"
    if start_anchor in new_content and end_anchor in new_content:
        start_idx = new_content.find(start_anchor) + len(start_anchor)
        end_idx = new_content.find(end_anchor)
        table_block = new_content[start_idx:end_idx]

        if f"v{next_version}" not in table_block:
            # Strip '(Latest)' from prior releases
            table_block = table_block.replace(" (Latest)", "")
            table_lines = [line for line in table_block.strip().splitlines() if line.strip()]
            if len(table_lines) >= 2:
                header = table_lines[0]
                separator = table_lines[1]
                existing_rows = table_lines[2:]

                new_row = (
                    f"| **v{next_version}** (Latest) | {today_str} | 🟢 Verified | "
                    f"`irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v{next_version}/install.ps1 \\| iex` | "
                    f"`curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v{next_version}/install.sh \\| bash` | "
                    f"`git checkout v{next_version}` |"
                )
                updated_table = (
                    "\n"
                    + header
                    + "\n"
                    + separator
                    + "\n"
                    + new_row
                    + "\n"
                    + "\n".join(existing_rows)
                    + "\n"
                )
                new_content = new_content[:start_idx] + updated_table + new_content[end_idx:]

    if new_content == content:
        new_content = re.sub(rf"\bv?{escaped_curr}\b", f"v{next_version}", content)

    if new_content == content:
        return

    if dry_run:
        print(f"[DRY RUN] Would update version references and pin matrix in readme.md: {current_ver} -> {next_version}")
        return

    with open(README_MD, "w", encoding="utf-8", newline="\n") as f:
        f.write(new_content)

    print(f"[*] Updated readme.md version pins -> v{next_version}")



def update_changelogs(next_version, scope, today_str, dry_run=False):
    """Prepends release entries to CHANGELOG.md, CHANGELOG_EN.md, and changelog.md if present."""
    formatted_entry = f"""    *   **v{next_version} ({today_str})**:
        -   **[Release v{next_version}] {scope}**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: {scope}.
"""
    standard_entry = f"## [v{next_version}] - {today_str}\n\n### Added\n- {scope}\n\n"

    for cl_path in [CHANGELOG_ROOT_MD, CHANGELOG_EN_MD, CHANGELOG_MD]:
        if not cl_path.is_file():
            continue
        with open(cl_path, "r", encoding="utf-8") as f:
            cl_content = f.read()

        if f"v{next_version}" in cl_content or f"[{next_version}]" in cl_content:
            continue

        if dry_run:
            print(f"[DRY RUN] Would prepend changelog entry to {cl_path.name} for v{next_version}")
            continue

        if "*   **Version History**:\n" in cl_content:
            cl_content = cl_content.replace(
                "*   **Version History**:\n",
                f"*   **Version History**:\n{formatted_entry}",
                1
            )
        elif "> 完整版本历史记录。" in cl_content:
            idx = cl_content.find("\n\n    *   **v")
            if idx != -1:
                cl_content = cl_content[:idx + 2] + formatted_entry + cl_content[idx + 2:]
            else:
                cl_content = f"{cl_content}\n\n{formatted_entry}"
        elif "# Changelog\n" in cl_content:
            cl_content = cl_content.replace("# Changelog\n", f"# Changelog\n\n{standard_entry}", 1)
        else:
            cl_content = f"{standard_entry}{cl_content}"

        with open(cl_path, "w", encoding="utf-8", newline="\n") as f:
            f.write(cl_content)

        print(f"[*] Prepended changelog entry in {cl_path.name} -> v{next_version}")

    if SPEC19_CHANGELOG.is_file():
        with open(SPEC19_CHANGELOG, "r", encoding="utf-8") as f:
            s19_content = f.read()

        if f"v{next_version}" not in s19_content:
            s19_entry = f"## v{next_version} — {today_str} ({scope})\n\n**Scope:** Version bump. {scope}.\n\n---\n\n"
            if dry_run:
                print(f"[DRY RUN] Would prepend entry to {SPEC19_CHANGELOG.name}")
            else:
                s19_content = f"{s19_entry}{s19_content}"
                with open(SPEC19_CHANGELOG, "w", encoding="utf-8", newline="\n") as f:
                    f.write(s19_content)

                print(f"[*] Prepended entry in {SPEC19_CHANGELOG.relative_to(REPO_ROOT)} -> v{next_version}")


def run_repo_sync_if_available(dry_run=False):
    """Executes `npm run sync` if defined in package.json to regenerate spec trees and manifests."""
    if not PACKAGE_JSON.is_file():
        return

    try:
        with open(PACKAGE_JSON, "r", encoding="utf-8") as f:
            pkg = json.load(f)

        scripts = pkg.get("scripts", {})
        if "sync" in scripts:
            if dry_run:
                print("[DRY RUN] Would run: npm run sync")
                return

            print("[*] Running npm run sync to regenerate spec trees and manifests...")
            is_win = sys.platform == "win32"
            run_cmd(["npm", "run", "sync"], check=False)
            print("[*] Completed npm run sync.")
    except Exception as e:
        print(f"[!] Warning running npm run sync: {e}")


def execute_bump(tier="minor", explicit_version=None, scope=None, dry_run=False):
    """Main bump execution logic."""
    current_ver = read_canonical_version()

    if explicit_version:
        next_ver = explicit_version.lstrip("v")
    else:
        next_ver = calculate_next_version(current_ver, tier)

    today_str = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")
    bump_scope = scope or f"Routine release v{next_ver}"

    print(f"[*] Bumping version: {current_ver} -> {next_ver} (Tier: {tier})")

    update_version_json(next_ver, today_str, dry_run=dry_run)
    update_package_json(next_ver, dry_run=dry_run)
    update_tauri_conf(next_ver, dry_run=dry_run)
    update_cargo_toml(next_ver, dry_run=dry_run)
    update_template_version(next_ver, dry_run=dry_run)
    update_homebrew_cask(next_ver, dry_run=dry_run)
    update_readme_pins(current_ver, next_ver, today_str=today_str, dry_run=dry_run)
    update_changelogs(next_ver, bump_scope, today_str, dry_run=dry_run)
    run_repo_sync_if_available(dry_run=dry_run)

    print(f"[OK] Successfully bumped version to {next_ver}")
    return next_ver


def parse_arguments():
    """Configures CLI argument parser."""
    parser = argparse.ArgumentParser(
        description="37-bump-version: Repository-aware SemVer version bumper & manifest synchronizer."
    )
    parser.add_argument(
        "-t",
        "--tier",
        choices=["patch", "minor", "major"],
        default="minor",
        help="SemVer bump tier (default: minor per Rule 0)",
    )
    parser.add_argument(
        "-v",
        "--version",
        dest="explicit_version",
        default=None,
        help="Explicit SemVer string (overrides --tier)",
    )
    parser.add_argument(
        "-s",
        "--scope",
        default=None,
        help="One-line description/scope of the release",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Simulate the bump without modifying files",
    )

    return parser.parse_args()


def main():
    """CLI entrypoint."""
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8")

    args = parse_arguments()
    execute_bump(
        tier=args.tier,
        explicit_version=args.explicit_version,
        scope=args.scope,
        dry_run=args.dry_run,
    )


if __name__ == "__main__":
    main()
