# Master E2E Test Plan: Batch 205 (Steps 236–240)

## Overview
This master verification plan outlines the deferred end-to-end testing criteria for all components introduced in Batch 205.
All tests are implemented as unit tests with mocked environments, preserving the Zero-Build / Zero-Test Invariant during normal coding runs.

## Suite Matrix
| Step | Implementation ID | Module Target | Test Suite File |
| :--- | :--- | :--- | :--- |
| Step 236 | `Imp-650` | `proxy::handlers::common` | `src-tauri/src/proxy/handlers/common/tests_thinking_budget_headroom_evaluator.rs` |
| Step 237 | `Imp-651` | `modules::release_gate` | `src-tauri/src/modules/release_gate/tests_preflight_clippy_gate.rs` |
| Step 238 | `Imp-652` | `modules::release_gate` | `src-tauri/src/modules/release_gate/tests_pr_unverified_paths_linter.rs` |
| Step 239 | `Imp-653` | `modules::instance` | `src-tauri/src/modules/instance/tests_multi_stage_launcher_engine.rs` |
| Step 240 | `Imp-654` | `modules::db_orm` | `src-tauri/src/modules/db_orm/tests_fifo_prompt_order_enforcer.rs` |

## Acceptance Criteria
1. `thinking_budget_headroom_evaluator`:
   - Validates that when requested thinking budget meets or exceeds requested max output tokens, headroom is preserved by clamping thinking budget or expanding max output.
   - Guarantees minimum text headroom (at least 512 tokens or proportional floor) across thinking-capable models.
   - Disables thinking budget entirely when models lack thinking support.
2. `preflight_clippy_gate`:
   - Scans cargo clippy diagnostic outputs.
   - Detects and flags warnings, clippy error lints, and missing mandatory flags (`--all-targets`, `--all-features`).
   - Rejects release gate progression if any clippy warning or error is detected.
3. `pr_unverified_paths_linter`:
   - Analyzes PR markdown descriptions against list of touched subsystems.
   - When high-risk subsystems (reverse proxy, database ORM, instance lifecycle) are modified, rejects generic "N/A" or empty disclosures in "Unverified Paths".
   - Confirms that substantive unverified paths and testing limitations are detailed.
4. `multi_stage_launcher_engine`:
   - Executes multi-stage fallback ladder: Target Binary -> Base Binary -> Windows Shell dispatch (`cmd.exe /c start`).
   - Simulates and intercepts Windows Job Object breakaway denials (`os error 5: Access is denied`), safely stepping down to in-job execution.
   - Disallows direct `.cmd` and `.bat` execution via process commands.
5. `fifo_prompt_order_enforcer`:
   - Intercepts raw SQL queries for prompt tables (`active_prompts`, `prompt_backups`, `repo_prompts`).
   - Detects inverted LIFO ordering (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`).
   - Replaces non-FIFO clauses with canonical `ORDER BY created_at ASC, id ASC`.
   - Leaves already-compliant queries intact and logs transformation metrics.
