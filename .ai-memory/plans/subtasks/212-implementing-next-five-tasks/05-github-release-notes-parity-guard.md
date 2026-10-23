# Subtask 05: GitHub Release Notes Parity Guard (`Imp-689`)
Status: COMPLETED
Parent Plan: 212-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Release Discipline & Attribution:
"Audit Git history (`<last-tag>..HEAD`). Attribution must attribute strictly `@aukgit` (`(Thanks to @aukgit)`) in `changelog.md` and `changelog_en.md`. Do NOT include any other GitHub user handles (`@...`) in release changelogs or release notes, ensuring the GitHub release page contributors list only contains `aukgit` and none else."
"Strictly Relative Git Paths Invariant: Never output or record absolute filesystem paths (e.g. `D:\...`, `C:\...`, `/home/...`) in specifications, plans, commit messages, `changelog.md`, `changelog_en.md`, `README.md`, `README_EN.md`, or GitHub release notes. All file references MUST be repository-relative Git paths (e.g., `02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`)."

Implemented `src-tauri/src/modules/release_gate/github_release_notes_parity_guard.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_github_release_notes_parity_guard.rs`.
Registered in `src-tauri/src/modules/release_gate/mod.rs`.

## Verification Evidence
- Defined `SOLE_PERMITTED_AUTHOR_HANDLE` (`@aukgit`), `FORBIDDEN_ABSOLUTE_PREFIXES`, and `ReleaseNotesViolation`.
- Implemented `audit_release_notes_parity` enforcing zero absolute filesystem paths, exact tag heading match, and single attribution strictly to `@aukgit`.
- Detected and flagged foreign GitHub handles (`@collaborator`, `@dependabot`).
- Added comprehensive unit tests covering compliant release notes, absolute path detection, foreign handle rejection, and tag heading mismatch.
- Deferred E2E command documented in `02-spec/21-app/212-implementing-next-five-tasks/e2e_master_plan.md`.
