# Sequence 202 Specification: Implementing Next Five Tasks (Steps 221–225)

## Overview
This specification details the design, constraints, and verification requirements for Batch 202 (Steps 221–225 of 255) in Antigravity-Manager.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-635`)**: `src-tauri/src/modules/db_orm/connection_starvation_watchdog.rs` & `tests_connection_starvation_watchdog.rs`
   - Enforce AGENTS.md bounded wait and timeout rules: Under high concurrency across split SQLite databases (`active_prompts`, `repo_prompts.db`), tracks active connection checkout durations across threads and detects thread pool starvation before query timeout exhaustion. Complies with the AGENTS.md mandate: "Bound every wait with a timeout" and "No Silent Errors".
2. **Task 2 (`Imp-636`)**: `src-tauri/src/modules/release_gate/sole_contributor_attribution_enforcer.rs` & `tests_sole_contributor_attribution_enforcer.rs`
   - Enforce AGENTS.md documentation & attribution discipline: Audit Git history (`<last-tag>..HEAD`). Attribution must attribute strictly `@aukgit` (`(Thanks to @aukgit)`) in `changelog.md` and `changelog_en.md`. Do NOT include any other GitHub user handles (`@...`) in release changelogs or release notes, ensuring the GitHub release page contributors list only contains `aukgit` and none else.
3. **Task 3 (`Imp-637`)**: `src-tauri/src/modules/release_gate/tag_changelog_heading_matcher_gate.rs` & `tests_tag_changelog_heading_matcher_gate.rs`
   - Enforce AGENTS.md commit, tag & push invariant: Push stable releases to `main` (`git tag vX.Y.Z && git push origin vX.Y.Z`), reserving `beta` exclusively for pre-releases (`git tag vX.Y.Z-beta.N && git push origin vX.Y.Z-beta.N`). The release gate strictly intercepts cross-branch misplacement. Tags must match `CHANGELOG.md` headings character-for-character (including `v` prefix and pre-release suffix).
4. **Task 4 (`Imp-638`)**: `src-tauri/src/modules/instance/workspace_storage_continuity_verifier.rs` & `tests_workspace_storage_continuity_verifier.rs`
   - Enforce AGENTS.md headless & CLI parity: Ensure GUI configurations and lifecycle operations maintain functional parity across headless servers, CLI environments, and cross-platform environments. Any profile/instance duplication or settings synchronization must pass through identical underlying service routes (`copy_instance_with_options`), preserving project continuity (`workspaceStorage`) and allowing deep-merge settings synchronization from both GUI and CLI.
5. **Task 5 (`Imp-639`)**: `src-tauri/src/proxy/handlers/common/response_stream_delimiter_normalizer.rs` & `tests_response_stream_delimiter_normalizer.rs`
   - Enforce AGENTS.md pipeline-first context structural alignment & framing: Normalizes SSE response delimiters across converted Gemini payloads, enforcing consistent `\n\n` event boundaries and standard event framing (`data: ...\n\n`) to prevent downstream client socket stalls and parse errors during token bursts.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All queries must use `ORDER BY created_at ASC, id ASC`.
