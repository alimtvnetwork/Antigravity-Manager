import os
import re
import json
import argparse
import subprocess
import datetime

# Files that need to be bumped
FILES_TO_BUMP = [
    "package.json",
    "prompt-version.template.json",
    "readme.md",
    ".lovable/coding-guidelines.md",
    "linter-scripts/validate-guidelines.go",
    "linter-scripts/validate-guidelines.py",
    "02-spec/14-update/28-worker-push-instruction.md",
    "02-spec/17-consolidated-guidelines/34-compiled-simple-coding-guidelines.md",
    "02-spec/19-main-worker-service/diagrams/seq-push-update.mmd",
    "02-spec/19-main-worker-service/11-worker-bootstrap-protocol.md",
    "02-spec/19-main-worker-service/15-rbac-and-status-seed.md",
    "02-spec/19-main-worker-service/16-tunable-constants.md",
    "02-spec/19-main-worker-service/17-update-channels.md",
    "02-spec/19-main-worker-service/26-inherited-rules.md"
]

def get_current_version():
    with open("version.json", "r", encoding="utf-8") as f:
        data = json.load(f)
        return data.get("version", "1.0.0")

def set_current_version(new_version):
    with open("version.json", "r+", encoding="utf-8") as f:
        data = json.load(f)
        data["version"] = new_version
        if "Version" in data:
            data["Version"] = new_version
        if "releaseDate" in data:
            data["releaseDate"] = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")
        f.seek(0)
        json.dump(data, f, indent=4)
        f.truncate()

def bump_version(current, bump_type):
    major, minor, patch = map(int, current.split("."))
    if bump_type == "major":
        return f"{major + 1}.0.0"
    elif bump_type == "minor":
        return f"{major}.{minor + 1}.0"
    elif bump_type == "patch":
        return f"{major}.{minor}.{patch + 1}"
    return current

def update_files(old_version, new_version):
    escaped_old = re.escape(old_version)
    regex_plain = re.compile(rf'\b{escaped_old}\b')
    regex_prefixed = re.compile(rf'v{escaped_old}\b')

    for file_path in FILES_TO_BUMP:
        if not os.path.exists(file_path):
            print(f"Warning: {file_path} not found. Skipping.")
            continue

        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()

        new_content = regex_plain.sub(new_version, content)
        new_content = regex_prefixed.sub(f"v{new_version}", new_content)

        if new_content != content:
            with open(file_path, "w", encoding="utf-8", newline='\n') as f:
                f.write(new_content)
            print(f"Bumped version in {file_path}")

    # Synchronize all generated manifests and spec trees via npm run sync
    try:
        print("[*] Running npm run sync to update spec trees, manifests, and badges...")
        subprocess.run(["npm", "run", "sync"], check=True)
    except Exception as e:
        print(f"Warning running npm run sync: {e}")

def get_repo_slug():
    """Extracts owner/repo from git remote origin url."""
    try:
        url = subprocess.run(["git", "config", "--get", "remote.origin.url"], capture_output=True, text=True, check=True).stdout.strip()
        m = re.search(r'github\.com[:/]([^/]+/[^/.]+)', url)
        if m:
            return m.group(1)
    except Exception:
        pass
    return "alimtvnetwork/prompt-architect-v2"

def extract_changelog_section(version):
    """Extracts changelog entry for version from changelog.md if present."""
    if not os.path.exists("changelog.md"):
        return ""
    try:
        with open("changelog.md", "r", encoding="utf-8") as f:
            content = f.read()
        pattern = rf'## \[?v?{re.escape(version)}\]?[^\n]*\n(.*?)(?=\n## \[?v?\d|\Z)'
        match = re.search(pattern, content, re.DOTALL)
        if match:
            return match.group(0).strip()
    except Exception as e:
        print(f"Warning extracting changelog: {e}")
    return ""

def build_release_notes_file(new_version):
    """Assembles structured release notes with mandatory Quick Install one-liners and changelog."""
    v_string = f"v{new_version}"
    repo_slug = get_repo_slug()
    repo_name = repo_slug.split('/')[-1] if '/' in repo_slug else repo_slug

    is_binary_repo = os.path.exists("linter-scripts/installer-templates") or os.path.exists("02-spec/16-generic-release")

    lines = [f"# {repo_name} {v_string}\n"]
    lines.append("## Quick Install (One-Liners)\n")

    if is_binary_repo:
        lines.append("### Windows (PowerShell 5.1+)\n```powershell\n"
                     f"irm https://github.com/{repo_slug}/releases/download/{v_string}/install.ps1 | iex\n```\n")
        lines.append("### Linux / macOS (Bash)\n```bash\n"
                     f"curl -fsSL https://github.com/{repo_slug}/releases/download/{v_string}/install.sh | bash\n```\n")
    else:
        lines.append("### Windows (PowerShell)\n```powershell\n"
                     f"Invoke-WebRequest -Uri https://raw.githubusercontent.com/{repo_slug}/{v_string}/install.ps1 -OutFile install.ps1; .\\install.ps1 -TargetDir \".lovable/prompts\" -Version \"{v_string}\"\n```\n")
        lines.append("### Unix / Bash\n```bash\n"
                     f"curl -sL https://raw.githubusercontent.com/{repo_slug}/{v_string}/install.sh | bash -s -- \".lovable/prompts\" \"{v_string}\"\n```\n")

    changelog_entry = extract_changelog_section(new_version)
    if changelog_entry:
        lines.append(f"\n{changelog_entry}\n")

    notes_path = os.path.join(".lovable", "release", f"release-notes-{v_string}.md")
    os.makedirs(os.path.dirname(notes_path), exist_ok=True)
    with open(notes_path, "w", encoding="utf-8", newline='\n') as f:
        f.write("\n".join(lines))

    print(f"Generated release notes at {notes_path}")
    return notes_path

def handle_git_release(new_version):
    v_string = f"v{new_version}"
    branch_name = f"release/{v_string}"

    print(f"\n--- Creating Full Release: {v_string} ---")

    try:
        # 1. Capture current branch to return to it later
        current_branch = subprocess.run(["git", "branch", "--show-current"], capture_output=True, text=True, check=True).stdout.strip()
        if not current_branch:
            current_branch = "main" # Fallback if detached head

        print(f"Current branch is {current_branch}. Creating release branch: {branch_name}")
        subprocess.run(["git", "checkout", "-b", branch_name], check=True)

        # Build release notes with Quick Install one-liners before commit so it is tracked
        notes_path = build_release_notes_file(new_version)

        print("Committing version bump and release notes...")
        subprocess.run(["git", "add", "."], check=True)
        subprocess.run(["git", "commit", "-m", f"chore(release): bump version to {new_version}"], check=True)

        print(f"Tagging release: {v_string}")
        subprocess.run(["git", "tag", v_string], check=True)

        print("Pushing branch and tags...")
        subprocess.run(["git", "push", "-u", "origin", branch_name], check=True)
        subprocess.run(["git", "push", "origin", v_string], check=True)

        # Detect CLI for platform release
        try:
            subprocess.run(["gh", "--version"], capture_output=True, check=True)
            print(f"GitHub CLI detected. Creating GitHub Release with {notes_path}...")
            subprocess.run(["gh", "release", "create", v_string, "--title", v_string, "--notes-file", notes_path, "--generate-notes"], check=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            try:
                subprocess.run(["glab", "--version"], capture_output=True, check=True)
                print(f"GitLab CLI detected. Creating GitLab Release with {notes_path}...")
                subprocess.run(["glab", "release", "create", v_string, "--name", v_string, "--notes-file", notes_path], check=True)
            except (subprocess.CalledProcessError, FileNotFoundError):
                print("No gh or glab CLI detected. Skipping platform release creation.")

        # 2. Return to original branch, merge, and push
        print(f"Returning to {current_branch} and merging {branch_name}...")
        subprocess.run(["git", "checkout", current_branch], check=True)
        subprocess.run(["git", "merge", branch_name], check=True)
        subprocess.run(["git", "push", "origin", current_branch], check=True)
        print("Release loop successfully completed and synced with main branch!")

    except subprocess.CalledProcessError as e:
        print(f"Error during git operations: {e}")
        print("Release automation failed.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Bump project versions and optionally create a Git release.")
    parser.add_argument("--type", choices=["major", "minor", "patch"], help="Type of bump")
    parser.add_argument("--set", type=str, help="Explicitly set a specific version")
    parser.add_argument("--create-release", action="store_true", help="Create a git branch, tag, and push a full release via gh/glab CLI")
    args = parser.parse_args()

    current_version = get_current_version()

    if args.set:
        new_version = args.set
    elif args.type:
        new_version = bump_version(current_version, args.type)
    else:
        print("Error: Must specify --type or --set")
        exit(1)

    print(f"Bumping from {current_version} to {new_version}...")

    set_current_version(new_version)
    update_files(current_version, new_version)

    if args.create_release:
        handle_git_release(new_version)
    else:
        print(f"Successfully bumped to {new_version} (Standard Mode - No git operations performed).")
