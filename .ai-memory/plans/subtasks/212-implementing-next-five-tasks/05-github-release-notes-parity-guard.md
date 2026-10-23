# Subtask 05: GitHub Release Notes Parity Guard (`Imp-689`)
Status: IN_PROGRESS
Parent Plan: 212-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Release Workflow, Contributor Attribution & Strictly Relative Paths:
"Documentation & Attribution: Audit Git history (`<last-tag>..HEAD`). Attribution must attribute strictly `@aukgit` (`(Thanks to @aukgit)`) in `changelog.md` and `changelog_en.md`. Do NOT include any other GitHub user handles (`@...`) in release changelogs or release notes, ensuring the GitHub release page contributors list only contains `aukgit` and none else.
Synchronize README Changelog: For stable releases, you MUST update the release summary in both `README.md` and `README_EN.md`.
Strictly Relative Git Paths Invariant: Never output or record absolute filesystem paths (e.g. `D:\...`, `C:\...`, `/home/...`) in release notes or changelogs."

Implement `src-tauri/src/modules/release_gate/github_release_notes_parity_guard.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_github_release_notes_parity_guard.rs`.

## Requirements
- Audit generated GitHub release notes against repository release policies.
- Enforce strict attribution solely to `@aukgit` and reject other handle mentions.
- Scan for and reject absolute filesystem paths (`C:\...`, `D:\...`, `/home/...`, `file:///...`).
- Verify release notes match `CHANGELOG.md` entry for the targeted tag.
- Comprehensive unit tests covering attribution verification, path hygiene, and heading matching.
