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

            raw_ver = data.get("version")
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

    # Check 1: Node bump script
    if NODE_BUMP_SCRIPT.is_file():
        print(f"[*] Invoking Node bump script: {NODE_BUMP_SCRIPT.relative_to(REPO_ROOT)}")
        run_cmd(["node", str(NODE_BUMP_SCRIPT), "--version", next_version, "--scope", scope])
        return

    # Check 2: Python bump script
    if PYTHON_BUMP_SCRIPT.is_file():
        print(f"[*] Invoking Python bump script: {PYTHON_BUMP_SCRIPT.relative_to(REPO_ROOT)}")
        run_cmd([sys.executable, str(PYTHON_BUMP_SCRIPT), "--version", next_version, "--scope", scope])
        return

    # Check 3: Bootstrap and execute fallback in-place
    print("[!] No bump script found. Executing standalone autonomous version bump...")
    bootstrap_bump_script_if_needed()

    today_str = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")

    # Update version.json
    if VERSION_JSON.is_file():
        with open(VERSION_JSON, "r", encoding="utf-8") as f:
            v_data = json.load(f)
        v_data["version"] = next_version
        v_data["releaseDate"] = today_str
        with open(VERSION_JSON, "w", encoding="utf-8") as f:
            json.dump(v_data, f, indent=2)
            f.write("\n")

    # Update package.json
    if PACKAGE_JSON.is_file():
        with open(PACKAGE_JSON, "r", encoding="utf-8") as f:
            p_data = json.load(f)
        p_data["version"] = next_version
        with open(PACKAGE_JSON, "w", encoding="utf-8") as f:
            json.dump(p_data, f, indent=2)
            f.write("\n")

    # Update changelog.md
    if CHANGELOG_MD.is_file():
        with open(CHANGELOG_MD, "r", encoding="utf-8") as f:
            cl_content = f.read()

        entry_header = f"## [v{next_version}] {today_str} {scope}\n\n- {scope}\n\n"
        if "# Changelog\n" in cl_content:
            cl_content = cl_content.replace("# Changelog\n", f"# Changelog\n\n{entry_header}", 1)
        else:
            cl_content = f"# Changelog\n\n{entry_header}{cl_content}"

        with open(CHANGELOG_MD, "w", encoding="utf-8") as f:
            f.write(cl_content)


def stage_and_commit_release(next_version, scope, dry_run=False):
    """Stages release files and commits on the current branch."""
    commit_msg = f"release: v{next_version} {scope}"

    if dry_run:
        print(f"[DRY RUN] Would stage changes and commit: '{commit_msg}'")
        return "dryrun_commit_sha"

    # Stage only release-specific files
    for vf in [VERSION_JSON, PACKAGE_JSON, CHANGELOG_MD, README_MD]:
        if vf.is_file():
            run_cmd(["git", "add", str(vf)])

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


def push_release(branch_name, tag_name, dry_run=False):
    """Pushes release branch and tag to remote repository."""
    if dry_run:
        print(f"[DRY RUN] Would push branch '{branch_name}' and tag '{tag_name}' to origin")
        return

    print(f"[*] Pushing branch '{branch_name}' to origin...")
    run_cmd(["git", "push", "origin", branch_name])

    print(f"[*] Pushing tag '{tag_name}' to origin...")
    run_cmd(["git", "push", "origin", tag_name])


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
        # 3. Bump version
        execute_version_bump(next_ver, default_scope, dry_run=dry_run)

        # 4. Commit bump changes
        commit_sha = stage_and_commit_release(next_ver, default_scope, dry_run=dry_run)

        # 5 & 6. Create release branch and tag pointing to the commit
        branch_name, tag_name = create_release_branch_and_tag(next_ver, commit_sha, dry_run=dry_run)

        # 7. Push branch and tag if enabled
        is_push_enabled = push and not dry_run
        if is_push_enabled:
            push_release(branch_name, tag_name, dry_run=dry_run)

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
