# Subtask 03: Changelog Heading & Tag Monotonic Progression Gate (`Imp-662`)
Status: IN_PROGRESS
Parent Plan: 207-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Release Workflow & Commit, Tag & Push Invariant:
"Push stable releases to `main` (`git tag vX.Y.Z && git push origin vX.Y.Z`), reserving `beta` exclusively for pre-releases (`git tag vX.Y.Z-beta.N && git push origin vX.Y.Z-beta.N`). The release gate strictly intercepts cross-branch misplacement. Tags must match `CHANGELOG.md` headings character-for-character (including `v` prefix and pre-release suffix)."

Implement `src-tauri/src/modules/release_gate/changelog_tag_monotonic_gate.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_changelog_tag_monotonic_gate.rs`.

## Requirements
- Extract SemVer components from release tags and changelog headings.
- Verify release tag strictly matches heading character-for-character in `CHANGELOG.md`.
- Verify monotonic SemVer progression (patch, minor, or major increase) over historical tags.
- Intercept cross-branch misplacement (pre-release tag on main branch or stable tag on beta branch).
- Return structured `TagMonotonicValidationReport` with validation status and violation diagnostics.
