# Subtask 05: README Changelog Section Parity Guard (`Imp-699`)
Sequence: 214
Step: 285
Status: IN_PROGRESS
Module: `src-tauri/src/modules/release_gate/readme_changelog_section_parity_guard.rs`
Test Suite: `src-tauri/src/modules/release_gate/tests_readme_changelog_section_parity_guard.rs`

## Description
Enforces AGENTS.md rules: "Synchronize README Changelog: For stable releases, you MUST update the release summary in both README.md and README_EN.md... Attribution must attribute strictly @aukgit... Strictly Relative Git Paths Invariant: Never output or record absolute filesystem paths." Validates parity across README files, enforces @aukgit attribution, and blocks third-party handles and absolute paths.
