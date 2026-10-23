# Master E2E Test Plan: Batch 207 (Steps 246–250)

## Overview
This master verification plan outlines the deferred end-to-end testing criteria for all components introduced in Batch 207.
All tests are implemented as unit tests with mocked environments, preserving the Zero-Build / Zero-Test Invariant during normal coding runs.

## Suite Matrix
| Step | Implementation ID | Module Target | Test Suite File |
| :--- | :--- | :--- | :--- |
| Step 246 | `Imp-660` | `proxy::handlers::common` | `src-tauri/src/proxy/handlers/common/tests_agent_prompt_sanitization_regex_engine.rs` |
| Step 247 | `Imp-661` | `modules::instance` | `src-tauri/src/modules/instance/tests_caller_ide_heartbeat_watchdog.rs` |
| Step 248 | `Imp-662` | `modules::release_gate` | `src-tauri/src/modules/release_gate/tests_changelog_tag_monotonic_gate.rs` |
| Step 249 | `Imp-663` | `modules::db_orm` | `src-tauri/src/modules/db_orm/tests_fifo_queue_index_analyzer.rs` |
| Step 250 | `Imp-664` | `modules::release_gate` | `src-tauri/src/modules/release_gate/tests_bounded_matcher_timeout_verifier.rs` |

## Acceptance Criteria
1. `agent_prompt_sanitization_regex_engine`:
   - Masks dangerous prompt injection attempts, system prompt overrides, and unescaped instruction markers.
   - Preserves legitimate Gemini pipeline prompts, system instructions, and multi-turn user dialog queries.
   - Forbids static keyword replacements in favor of regex pattern matching.
2. `caller_ide_heartbeat_watchdog`:
   - Observes caller IDE process heartbeat ticks.
   - Detects active caller PID and marks caller instance as non-evictable.
   - Protects caller instance during automatic quota switches or daemon pruning cycles.
3. `changelog_tag_monotonic_gate`:
   - Validates that release tag strictly matches heading in `CHANGELOG.md` and `CHANGELOG_EN.md`.
   - Confirms monotonic version progression over historical tags.
   - Intercepts cross-branch misplacement (beta tags on main or stable tags on beta).
4. `fifo_queue_index_analyzer`:
   - Analyzes SQLite index schema for `active_prompts`, `prompt_backups`, and `repo_prompts`.
   - Validates presence of composite index on `(created_at, id)`.
   - Reports missing indexes and provides automated `CREATE INDEX` remediation statements.
5. `bounded_matcher_timeout_verifier`:
   - Validates regex matchers for bounded spans (disallowing unconstrained multiline `.*` spans crossing tags or newlines).
   - Validates that wait loops and timeouts are non-zero and bounded by safe maximum limits.
