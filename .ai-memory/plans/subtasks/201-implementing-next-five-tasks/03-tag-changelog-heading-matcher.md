# Subtask 03: Tag Changelog Heading Exact Matcher Gate
Status: IN PROGRESS
Sequence: 201
Owner: Worker-2
Target Module: `src-tauri/src/modules/release_gate/tag_changelog_heading_matcher_gate.rs`
Test Suite: `src-tauri/src/modules/release_gate/tests_tag_changelog_heading_matcher_gate.rs`
Objective: Enforce AGENTS.md rule: git tags must match CHANGELOG.md headings character-for-character including 'v' prefix, and cross-branch tag gate must restrict stable tags to main and beta tags to beta.
