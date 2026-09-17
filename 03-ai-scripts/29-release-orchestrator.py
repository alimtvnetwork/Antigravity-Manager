#!/usr/bin/env python3
"""
29-release-orchestrator.py - Standalone Release Orchestrator

Automates the complete release heavy-lifting lifecycle:
  1. Detects and preserves the original starting branch.
  2. Resolves current version and calculates next SemVer (minor default per Rule 0).
  3. Checks for and runs bump-version script (bootstrapping it if missing).
  4. Commits version bump changes to the repository.
  5. Creates/updates the release branch: release/vX.Y.Z pointing to that commit.
  6. Creates the annotated git tag: vX.Y.Z on that commit.
  7. Optionally pushes the release branch and tag to the remote repository.
  8. Reverts working tree back to the original starting branch.

Usage:
  python 03-ai-scripts/29-release-orchestrator.py
  python 03-ai-scripts/29-release-orchestrator.py --tier patch
  python 03-ai-scripts/29-release-orchestrator.py --tier minor --scope "Feature release"
  python 03-ai-scripts/29-release-orchestrator.py --tier major --scope "Breaking change"
  python 03-ai-scripts/29-release-orchestrator.py --version 5.30.0
  python 03-ai-scripts/29-release-orchestrator.py --dry-run
  python 03-ai-scripts/29-release-orchestrator.py --no-push
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

# Known bump scripts
NODE_BUMP_SCRIPT = REPO_ROOT / "scripts" / "bump-version.mjs"
PYTHON_BUMP_SCRIPT = REPO_ROOT / ".lovable" / "release" / "bump_versions.py"


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


def get_git_output(*args):
    """Executes a git command and returns stripped stdout."""
    res = run_cmd(["git", *args])

    return res.stdout.strip()


def get_current_branch():
    """Detects and returns current git branch name."""
    branch = get_git_output("rev-parse", "--abbrev-ref", "HEAD")
    if not branch or branch == "HEAD":
        raise RuntimeError("Detached HEAD or unable to determine current git branch.")

    return branch


def read_canonical_version():
    """Reads current SemVer from version.json or package.json."""
    if VERSION_JSON.is_file():
        try:
            with open(VERSION_JSON, "r", encoding="utf-8") as f:
                data = json.load(f)

            raw_ver = data.get("version") or data.get("Version")
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


def bootstrap_bump_script_if_needed():
    """Creates a basic bump script if none exists in the repository."""
    if NODE_BUMP_SCRIPT.is_file() or PYTHON_BUMP_SCRIPT.is_file():
        return

    scripts_dir = REPO_ROOT / "scripts"
    scripts_dir.mkdir(parents=True, exist_ok=True)

    bootstrap_content = '''#!/usr/bin/env node
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");

const args = process.argv.slice(2);
let version = null;
let scope = "Routine release ceremony";

for (let i = 0; i < args.length; i++) {
  if (args[i] === "--version" || args[i] === "-v") version = args[++i];
  if (args[i] === "--scope" || args[i] === "-s") scope = args[++i];
}

if (!version) {
  console.error("Missing required --version argument");
  process.exit(1);
}

// 1. Update version.json
const verJsonPath = resolve(ROOT, "version.json");
if (existsSync(verJsonPath)) {
  const data = JSON.parse(readFileSync(verJsonPath, "utf8"));
  data.version = version;
  data.releaseDate = new Date().toISOString().split("T")[0];
  writeFileSync(verJsonPath, JSON.stringify(data, null, 2) + "\\n", "utf8");
}

// 2. Update package.json
const pkgPath = resolve(ROOT, "package.json");
if (existsSync(pkgPath)) {
  const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
  pkg.version = version;
  writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\\n", "utf8");
}

console.log(`Successfully bumped to ${version}`);
'''
    with open(NODE_BUMP_SCRIPT, "w", encoding="utf-8") as f:
        f.write(bootstrap_content)

    print(f"[*] Bootstrapped missing bump script: {NODE_BUMP_SCRIPT.relative_to(REPO_ROOT)}")


def execute_version_bump(next_version, scope, dry_run=False):
    """Executes the version bump via existing scripts or standalone fallback."""
    if dry_run:
        print(f"[DRY RUN] Would bump version to {next_version} (scope: {scope})")
        return

    # Optional helper bump script invocation
    if PYTHON_BUMP_SCRIPT.is_file():
        print(f"[*] Invoking Python bump script: {PYTHON_BUMP_SCRIPT.relative_to(REPO_ROOT)}")
        try:
            run_cmd([sys.executable, str(PYTHON_BUMP_SCRIPT), "--set", next_version])
        except Exception as e:
            print(f"[!] Sub-bump script notice: {e}")

    # Check 3: Standalone autonomous version bump across all manifests and docs
    print("[!] Executing standalone autonomous version bump across repository...")
    today_str = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")

    # 1. Update version.json
    if VERSION_JSON.is_file():
        with open(VERSION_JSON, "r", encoding="utf-8") as f:
            v_data = json.load(f)
        v_data["Version"] = next_version
        v_data["version"] = next_version
        v_data["releaseDate"] = today_str
        with open(VERSION_JSON, "w", encoding="utf-8") as f:
            json.dump(v_data, f, indent=2)
            f.write("\n")

    # 2. Update package.json
    if PACKAGE_JSON.is_file():
        with open(PACKAGE_JSON, "r", encoding="utf-8") as f:
            p_data = json.load(f)
        p_data["version"] = next_version
        with open(PACKAGE_JSON, "w", encoding="utf-8") as f:
            json.dump(p_data, f, indent=2)
            f.write("\n")

    # 3. Update package-lock.json
    pkg_lock_file = REPO_ROOT / "package-lock.json"
    if pkg_lock_file.is_file():
        with open(pkg_lock_file, "r", encoding="utf-8") as f:
            pl_data = json.load(f)
        pl_data["version"] = next_version
        if "packages" in pl_data:
            if "" in pl_data["packages"]:
                pl_data["packages"][""]["version"] = next_version
        with open(pkg_lock_file, "w", encoding="utf-8") as f:
            json.dump(pl_data, f, indent=2)
            f.write("\n")

    # 4. Update src-tauri/Cargo.toml & Cargo.lock
    cargo_file = REPO_ROOT / "src-tauri" / "Cargo.toml"
    if cargo_file.is_file():
        cargo_content = cargo_file.read_text(encoding="utf-8")
        cargo_content = re.sub(r'(?m)^version\s*=\s*"[^"]+"', f'version = "{next_version}"', cargo_content, count=1)
        cargo_file.write_text(cargo_content, encoding="utf-8")

    cargo_lock = REPO_ROOT / "src-tauri" / "Cargo.lock"
    if cargo_lock.is_file():
        lock_content = cargo_lock.read_text(encoding="utf-8")
        lock_content = re.sub(
            r'(\[\[package\]\]\r?\nname\s*=\s*"antigravity-tools"\r?\nversion\s*=\s*)"[^"]+"',
            f'\\g<1>"{next_version}"',
            lock_content,
            count=1,
        )
        cargo_lock.write_text(lock_content, encoding="utf-8")

    # 5. Update src-tauri/tauri.conf.json
    tauri_conf = REPO_ROOT / "src-tauri" / "tauri.conf.json"
    if tauri_conf.is_file():
        with open(tauri_conf, "r", encoding="utf-8") as f:
            tc_data = json.load(f)
        tc_data["version"] = next_version
        with open(tauri_conf, "w", encoding="utf-8") as f:
            json.dump(tc_data, f, indent=2)
            f.write("\n")

    # 6. Update Casks/antigravity-tools.rb
    cask_file = REPO_ROOT / "Casks" / "antigravity-tools.rb"
    if cask_file.is_file():
        cask_content = cask_file.read_text(encoding="utf-8")
        cask_content = re.sub(r'version\s+"[^"]+"', f'version "{next_version}"', cask_content, count=1)
        cask_file.write_text(cask_content, encoding="utf-8")

    # 7. Update UI fallback version strings in React
    settings_tsx = REPO_ROOT / "src" / "pages" / "Settings.tsx"
    if settings_tsx.is_file():
        st_content = settings_tsx.read_text(encoding="utf-8")
        st_content = re.sub(r"useState<string>\('[0-9.]+'\)", f"useState<string>('{next_version}')", st_content)
        settings_tsx.write_text(st_content, encoding="utf-8")

    miniview_tsx = REPO_ROOT / "src" / "components" / "layout" / "MiniView.tsx"
    if miniview_tsx.is_file():
        mv_content = miniview_tsx.read_text(encoding="utf-8")
        mv_content = re.sub(r"setAppVersion\('[0-9.]+'\)", f"setAppVersion('{next_version}')", mv_content)
        miniview_tsx.write_text(mv_content, encoding="utf-8")

    # 8. Update README files badges and version titles
    for readme_path in [REPO_ROOT / "readme.md", REPO_ROOT / "README.md", REPO_ROOT / "README_EN.md"]:
        if readme_path.is_file():
            rm_content = readme_path.read_text(encoding="utf-8")
            rm_content = re.sub(r"\(v[0-9.]+\)", f"(v{next_version})", rm_content)
            rm_content = re.sub(r"badge/Version-[0-9.]+-blue", f"badge/Version-{next_version}-blue", rm_content)
            readme_path.write_text(rm_content, encoding="utf-8")

    # 9. Update fallback version in standalone install scripts
    for installer_path in [REPO_ROOT / "install.ps1", REPO_ROOT / "install.sh"]:
        if installer_path.is_file():
            inst_content = installer_path.read_text(encoding="utf-8")
            inst_content = re.sub(r'(\$TargetVersion\s*=\s*)"[0-9.]+"', f'\\g<1>"{next_version}"', inst_content)
            inst_content = re.sub(r'(RELEASE_VERSION\s*=\s*)"[0-9.]+"', f'\\g<1>"{next_version}"', inst_content)
            inst_content = re.sub(r'falling back to v[0-9.]+', f'falling back to v{next_version}', inst_content)
            installer_path.write_text(inst_content, encoding="utf-8")

    # 10. Update changelog files
    changelog_zh = REPO_ROOT / "CHANGELOG.md"
    if changelog_zh.is_file():
        cl_content = changelog_zh.read_text(encoding="utf-8")
        zh_entry = (
            f"    *   **v{next_version} ({today_str})**:\n"
            f"        -   **[Branding, Quota UI & Instance Management] AGM by Alim Branding, UI Compactness, and Automatic Rotation Guidance**:\n"
            f"            -   **AGM by Alim Branding**: Standardized window title, Navbar title, and page titles to 'AGM by Alim', and executable output to 'Anti-Gravity Tools by Alim'.\n"
            f"            -   **Accounts Quota UI Compactness**: Consolidated multi-badge model quotas into unified Gemini and Claude shared buckets, streamlining vertical and horizontal density.\n"
            f"            -   **Instance Management & Auto-Rotation Guidance**: Fixed instance discovery and runtime interactions, and added clear automatic rotation documentation in Settings.\n"
            f"            -   **Manifest & Attribution Cleanup**: Removed Chinese comments from backend manifests and release tools, giving full attribution to upstream lbjlaq/Antigravity-Manager.\n"
        )
        marker_zh = "[English Changelog](CHANGELOG_EN.md)。"
        if marker_zh in cl_content:
            idx = cl_content.find(marker_zh) + len(marker_zh)
            nl_idx = cl_content.find("\n", idx)
            if nl_idx != -1:
                cl_content = cl_content[:nl_idx] + "\n\n" + zh_entry + cl_content[nl_idx+1:]
        elif "*   **版本演进**:\n" in cl_content:
            cl_content = cl_content.replace("*   **版本演进**:\n", f"*   **版本演进**:\n{zh_entry}", 1)
        changelog_zh.write_text(cl_content, encoding="utf-8")

    changelog_en = REPO_ROOT / "CHANGELOG_EN.md"
    if changelog_en.is_file():
        cl_en_content = changelog_en.read_text(encoding="utf-8")
        en_entry = (
            f"    *   **v{next_version} ({today_str})**:\n"
            f"        -   **[Branding, Quota UI & Instance Management] AGM by Alim Branding, UI Compactness, and Automatic Rotation Guidance**:\n"
            f"            -   **AGM by Alim Branding**: Standardized window title, Navbar title, and page titles to 'AGM by Alim', and executable output to 'Anti-Gravity Tools by Alim'.\n"
            f"            -   **Accounts Quota UI Compactness**: Consolidated multi-badge model quotas into unified Gemini and Claude shared buckets, streamlining vertical and horizontal density.\n"
            f"            -   **Instance Management & Auto-Rotation Guidance**: Fixed instance discovery and runtime interactions, and added clear automatic rotation documentation in Settings.\n"
            f"            -   **Manifest & Attribution Cleanup**: Removed Chinese comments from backend manifests and release tools, giving full attribution to upstream lbjlaq/Antigravity-Manager.\n"
        )
        if "*   **Version History**:\n" in cl_en_content:
            cl_en_content = cl_en_content.replace("*   **Version History**:\n", f"*   **Version History**:\n{en_entry}", 1)
        elif "*   **Version Evolution**:\n" in cl_en_content:
            cl_en_content = cl_en_content.replace("*   **Version Evolution**:\n", f"*   **Version Evolution**:\n{en_entry}", 1)
        changelog_en.write_text(cl_en_content, encoding="utf-8")

    # 11. Generate release notes file with Quick Install one-liners
    release_notes_dir = REPO_ROOT / ".lovable" / "release"
    release_notes_dir.mkdir(parents=True, exist_ok=True)
    notes_file = release_notes_dir / f"release-notes-v{next_version}.md"
    notes_content = (
        f"## Quick Install v{next_version}\n\n"
        f"### Windows (PowerShell 5.1+)\n"
        f"```powershell\n"
        f"irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex\n"
        f"# Or pinned version:\n"
        f"irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v{next_version}/install.ps1 | iex\n"
        f"```\n\n"
        f"### Linux / macOS (Bash)\n"
        f"```bash\n"
        f"curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash\n"
        f"# Or pinned version:\n"
        f"curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v{next_version}/install.sh | bash\n"
        f"```\n\n"
        f"---\n\n"
        f"## What's Changed in v{next_version}\n\n"
        f"- **AGM by Alim Branding**: Standardized window title, Navbar title, and page titles to 'AGM by Alim', and executable output to 'Anti-Gravity Tools by Alim'.\n"
        f"- **Accounts Quota UI Compactness**: Consolidated multi-badge model quotas into unified Gemini and Claude shared buckets, streamlining vertical and horizontal density.\n"
        f"- **Instance Management & Auto-Rotation Guidance**: Fixed instance discovery and runtime interactions, and added clear automatic rotation documentation in Settings.\n"
        f"- **Attribution & Manifest Cleanup**: Removed Chinese comments from backend manifests and release tools, giving full attribution to upstream lbjlaq/Antigravity-Manager.\n"
    )
    notes_file.write_text(notes_content, encoding="utf-8", newline="\n")


def stage_and_commit_release(next_version, scope, dry_run=False):
    """Stages release files and commits on the current branch."""
    commit_msg = f"release: v{next_version} {scope}"

    if dry_run:
        print(f"[DRY RUN] Would stage changes and commit: '{commit_msg}'")
        return "dryrun_commit_sha"

    # Stage all repository changes including specs and memory
    run_cmd(["git", "add", "-A"])

    # Commit
    run_cmd(["git", "commit", "-m", commit_msg])
    commit_sha = get_git_output("rev-parse", "HEAD")
    print(f"[*] Committed release changes: {commit_sha[:8]} ('{commit_msg}')")

    return commit_sha


def create_release_branch_and_tag(next_version, commit_sha, dry_run=False):
    """Creates release branch and tag pointing to commit_sha."""
    branch_name = f"release/v{next_version}"
    tag_name = f"v{next_version}"

    if dry_run:
        print(f"[DRY RUN] Would create branch '{branch_name}' and tag '{tag_name}' at {commit_sha}")
        return branch_name, tag_name

    # Switch to the new release branch pointing to the release commit
    run_cmd(["git", "branch", "-f", branch_name, commit_sha])
    run_cmd(["git", "checkout", branch_name])
    print(f"[*] Checked out release branch: {branch_name} -> {commit_sha[:8]}")

    # Create annotated tag
    run_cmd(["git", "tag", "-a", tag_name, "-m", f"Release {tag_name}", commit_sha])
    print(f"[*] Created tag: {tag_name} -> {commit_sha[:8]}")

    return branch_name, tag_name


def push_release(branch_name, tag_name, original_branch="main", dry_run=False):
    """Pushes release branch, tag, and original branch to remote repository."""
    if dry_run:
        print(f"[DRY RUN] Would push branch '{branch_name}', tag '{tag_name}', and '{original_branch}' to origin")
        return

    print(f"[*] Pushing branch '{branch_name}' to origin...")
    run_cmd(["git", "push", "origin", branch_name])

    print(f"[*] Pushing tag '{tag_name}' to origin...")
    run_cmd(["git", "push", "origin", tag_name])

    if original_branch:
        print(f"[*] Pushing original branch '{original_branch}' to origin...")
        run_cmd(["git", "push", "origin", original_branch])


def revert_to_original_branch(original_branch, dry_run=False):
    """Switches git working tree back to the starting branch."""
    if dry_run:
        print(f"[DRY RUN] Would revert back to original branch: '{original_branch}'")
        return

    current = get_current_branch()
    if current != original_branch:
        print(f"[*] Reverting back to original branch: '{original_branch}' (from '{current}')...")
        run_cmd(["git", "checkout", original_branch])

    restored = get_current_branch()
    if restored != original_branch:
        raise RuntimeError(
            f"Failed to restore original branch! Current branch is '{restored}', expected '{original_branch}'"
        )

    print(f"[OK] Working tree successfully restored to original branch: '{restored}'")


def orchestrate_release(tier="minor", explicit_version=None, scope=None, dry_run=False, push=True):
    """Executes the complete release orchestration flow."""
    # 1. Capture starting branch
    original_branch = get_current_branch()
    print(f"[*] Starting release orchestration on branch: '{original_branch}'")

    # 2. Resolve versions
    current_ver = read_canonical_version()
    if explicit_version:
        next_ver = explicit_version.lstrip("v")
    else:
        next_ver = calculate_next_version(current_ver, tier)

    default_scope = scope or f"Release v{next_ver}"
    print(f"[*] Version Plan: {current_ver} -> {next_ver} (Tier: {tier})")

    try:
        # 3. Bump version across manifests and docs
        execute_version_bump(next_ver, default_scope, dry_run=dry_run)

        # 4. Commit bump changes
        commit_sha = stage_and_commit_release(next_ver, default_scope, dry_run=dry_run)

        # 5 & 6. Create release branch and tag pointing to the commit
        branch_name, tag_name = create_release_branch_and_tag(next_ver, commit_sha, dry_run=dry_run)

        # 7. Push branch and tag if enabled
        if not dry_run:
            if push:
                push_release(branch_name, tag_name, original_branch=original_branch, dry_run=dry_run)

    finally:
        # 8. Always revert back to the exact starting branch
        revert_to_original_branch(original_branch, dry_run=dry_run)

    print("\n" + "=" * 60)
    print("[OK] RELEASE ORCHESTRATION COMPLETE")
    print(f"  - Starting Branch:  {original_branch}")
    print(f"  - Previous Version: {current_ver}")
    print(f"  - Released Version: {next_ver}")
    print(f"  - Release Branch:   release/v{next_ver}")
    print(f"  - Release Tag:      v{next_ver}")
    print(f"  - Active Branch:    {get_current_branch()} [Preserved]")
    print("=" * 60 + "\n")


def parse_arguments():
    """Configures CLI argument parser."""
    parser = argparse.ArgumentParser(
        description="29-release-orchestrator: Autonomous release lifecycle with branch preservation."
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
        help="Simulate the release workflow without modifying files or git",
    )
    parser.add_argument(
        "--no-push",
        action="store_true",
        help="Do not push release branch and tag to remote repository",
    )

    return parser.parse_args()


def main():
    """Main CLI entrypoint."""
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8")

    args = parse_arguments()
    should_push = not args.no_push

    orchestrate_release(
        tier=args.tier,
        explicit_version=args.explicit_version,
        scope=args.scope,
        dry_run=args.dry_run,
        push=should_push,
    )


if __name__ == "__main__":
    main()
