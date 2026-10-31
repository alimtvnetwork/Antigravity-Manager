# Subtask 02: Branch Release Channel Separation Gate (`Imp-656`)
Status: COMPLETED
Parent Plan: 206-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Release Channels & Discipline:
"Release Channel Separation:
- Stable Releases: Exclusively on `main`. Deploys official production packages, updates Docker/GitHub latest tags.
- Preview Releases (Beta): Exclusively on `beta`. Independently builds and publishes pre-releases (`makeLatest: false`, `prerelease: true`).
Tags must match `CHANGELOG.md` headings character-for-character (including `v` prefix and pre-release suffix). The release gate strictly intercepts cross-branch misplacement."

Implement `src-tauri/src/modules/release_gate/branch_release_channel_gate.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_branch_release_channel_gate.rs`.

## Requirements
- [x] Validate that stable version tags (`vX.Y.Z`) are rejected on `beta` or feature branches.
- [x] Validate that pre-release version tags (`vX.Y.Z-beta.N`) are rejected on `main`.
- [x] Validate exact character-for-character match with `CHANGELOG.md` headings.
- [x] Emit structured `ReleaseChannelValidationReport`.
- [x] Comprehensive unit tests implemented and registered in `src-tauri/src/modules/release_gate/mod.rs`.
