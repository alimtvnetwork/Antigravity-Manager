#!/usr/bin/env python3
"""Generate release notes from git history for Antigravity-Manager.

Usage: gen_release_notes.py <version> [--since <tag>] [--write CHANGELOG.md]

Reads conventional commits since <tag> (default: previous version tag) and
produces a structured markdown release-notes section. Credits @aukgit per repo policy.
"""
import argparse
import re
import subprocess
import sys

CREDIT = "@aukgit"

SECTION_TITLES = [
    ("feat", "Features"),
    ("fix", "Bug Fixes"),
    ("perf", "Performance"),
    ("refactor", "Refactoring"),
    ("docs", "Documentation"),
    ("chore", "Chores"),
    ("test", "Tests"),
    ("ci", "CI/CD"),
]

CONVENTIONAL_RE = re.compile(
    r"^(feat|fix|perf|refactor|docs|chore|test|ci|style|build)(\([^)]*\))?(!)?:\s*(.+)$",
    re.IGNORECASE,
)


def git(*args):
    return subprocess.run(
        ["git", *args], capture_output=True, text=True, check=True
    ).stdout.strip()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("version", help="e.g. v4.184.0")
    ap.add_argument("--since", default=None, help="since tag (default: previous v-tag)")
    ap.add_argument("--write", default=None, help="prepend into CHANGELOG.md")
    ap.add_argument("--date", default=None, help="release date (default: today)")
    args = ap.parse_args()

    since = args.since
    if not since:
        tags = git("tag", "--list", "v*.*.*", "--sort=-v:refname").splitlines()
        tags = [t for t in tags if t != args.version]
        since = tags[0] if tags else None
    date = args.date or git("show", "-s", "--format=%cs", "HEAD")

    log_range = f"{since}..HEAD" if since else "HEAD"
    raw = git("log", log_range, "--pretty=format:%H%x00%s%x00%b%x1e")
    commits = []
    for entry in raw.split("\x1e"):
        entry = entry.strip()
        if not entry:
            continue
        sha, subject, body = (entry.split("\x00") + ["", ""])[:3]
        commits.append((sha[:8], subject.strip(), body.strip()))

    sections = {k: [] for k, _ in SECTION_TITLES}
    other = []
    for sha, subject, _ in commits:
        m = CONVENTIONAL_RE.match(subject)
        if m:
            kind = m.group(1).lower()
            breaking = m.group(3)
            msg = m.group(4)
            scope = m.group(2)
            label = f"{scope[1:-1]}: {msg}" if scope else msg
            if breaking:
                label = "⚠️ BREAKING: " + label
            sections.setdefault(kind, []).append((sha, label))
        else:
            other.append((sha, subject))

    lines = [f"## {args.version} ({date})", ""]
    if since:
        lines.append(f"Changes since {since}.")
        lines.append("")
    for kind, title in SECTION_TITLES:
        items = sections.get(kind, [])
        if not items:
            continue
        lines.append(f"### {title}")
        lines.append("")
        for sha, label in items:
            lines.append(f"- {label} ({sha})")
        lines.append("")
    if other:
        lines.append("### Other")
        lines.append("")
        for sha, subject in other:
            lines.append(f"- {subject} ({sha})")
        lines.append("")
    lines.append(f"Credits: {CREDIT}")
    lines.append("")
    notes = "\n".join(lines)

    if args.write:
        with open(args.write, encoding="utf-8") as f:
            existing = f.read()
        # insert after the first heading line
        head, _, tail = existing.partition("\n")
        with open(args.write, "w", encoding="utf-8") as f:
            f.write(head + "\n\n" + notes + tail)
        print(f"Prepended release notes to {args.write}")
    else:
        sys.stdout.write(notes)


if __name__ == "__main__":
    main()
