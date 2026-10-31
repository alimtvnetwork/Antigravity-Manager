# Sequence 201 Specification: Implementing Next Five Tasks (Steps 216–220)

## Overview
This specification details the design, constraints, and verification requirements for Batch 201 (Steps 216–220 of 255) in Antigravity-Manager.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-630`)**: `src-tauri/src/modules/release_gate/thinking_cache_invalidation_controller.rs` & `tests_thinking_cache_invalidation_controller.rs`
   - Enforce AGENTS.md thinking cache invalidation control (release cleaning suggestions): Routine releases (no prompt): Keep `SUGGESTION_DELETE_THINKING_STORE = false`. Architecture / schema refactors (prompt users to clean once): Set `SUGGESTION_DELETE_THINKING_STORE = true`, Set `SUGGESTION_TARGET_VERSION = '<version>'`. On upgrade, users with existing cache get a one-time prompt; action state persists in `gui_config.json`.
2. **Task 2 (`Imp-631`)**: `src-tauri/src/modules/release_gate/readme_changelog_synchronizer.rs` & `tests_readme_changelog_synchronizer.rs`
   - Enforce AGENTS.md release workflow: Synchronize README Changelog: For stable releases, you MUST update the release summary in both `README.md` (under '## 📝 changelog') and `README_EN.md` (under '## 📝 Changelog'). Never update only `CHANGELOG.md` while leaving `README.md` / `README_EN.md` with outdated release notes. Pre-release / beta versions remain exclusively in changelogs; stable releases require full synchronization across both README files.
3. **Task 3 (`Imp-632`)**: `src-tauri/src/modules/release_gate/sole_contributor_attribution_guard.rs` & `tests_sole_contributor_attribution_guard.rs`
   - Enforce AGENTS.md documentation & attribution: Audit Git history (`<last-tag>..HEAD`). Attribution must attribute strictly `@aukgit` (`(Thanks to @aukgit)`) in `changelog.md` and `changelog_en.md`. Do NOT include any other GitHub user handles (`@...`) in release changelogs or release notes, ensuring the GitHub release page contributors list only contains `aukgit` and none else.
4. **Task 4 (`Imp-633`)**: `src-tauri/src/modules/release_gate/tag_changelog_heading_matcher.rs` & `tests_tag_changelog_heading_matcher.rs`
   - Enforce AGENTS.md commit, tag & push invariant: Push stable releases to `main` (`git tag vX.Y.Z && git push origin vX.Y.Z`), reserving `beta` exclusively for pre-releases (`git tag vX.Y.Z-beta.N && git push origin vX.Y.Z-beta.N`). The release gate strictly intercepts cross-branch misplacement. Tags must match `CHANGELOG.md` headings character-for-character (including `v` prefix and pre-release suffix).
5. **Task 5 (`Imp-634`)**: `src-tauri/src/proxy/handlers/common/prefix_stability_pipeline_guard.rs` & `tests_prefix_stability_pipeline_guard.rs`
   - Enforce AGENTS.md pipeline-first context structural alignment: The pipeline stage uniformly handles converted Gemini payloads, guaranteeing prefix stability across multi-turn sessions, preserving stable prompt prefixes for upstream cache hits, and normalizing prefix key ordering.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All queries must use `ORDER BY created_at ASC, id ASC`.
