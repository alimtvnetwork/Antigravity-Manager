# Subtask 04: Tag & Changelog Heading Matcher
Status: COMPLETED
Sequence: 201
Owner: Worker-2
Target Module: `src-tauri/src/modules/release_gate/tag_changelog_heading_matcher.rs`
Test Suite: `src-tauri/src/modules/release_gate/tests_tag_changelog_heading_matcher.rs`
Objective: Enforce AGENTS.md commit, tag & push invariant: Push stable releases to `main`, reserving `beta` exclusively for pre-releases. Tags must match `CHANGELOG.md` headings character-for-character (including `v` prefix and pre-release suffix).
