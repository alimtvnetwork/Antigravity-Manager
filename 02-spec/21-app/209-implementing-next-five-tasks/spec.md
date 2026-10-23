# Sequence 209 Specification: Implementing Next Five Tasks (Steps 256–260)

## Overview
This specification details the design, constraints, and verification requirements for Batch 209 (Steps 256–260) in Antigravity-Manager.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-670`)**: `src-tauri/src/proxy/handlers/common/protocol_adapter_divergence_auditor.rs` & `tests_protocol_adapter_divergence_auditor.rs`
   - Enforce AGENTS.md Pipeline First & Protocol Adapter Decoupling: audit protocol adapters to guarantee that adapters remain strictly decoupled from pipeline concerns (thinking budget filtering, prefix stability, unified context structural alignment) and only perform parameter normalization and payload transformation.
2. **Task 2 (`Imp-671`)**: `src-tauri/src/modules/instance/prompt_tree_branch_hierarchy_formatter.rs` & `tests_prompt_tree_branch_hierarchy_formatter.rs`
   - Enforce AGENTS.md 4-Tier Origin Classification & Hierarchy: format and validate prompt tree hierarchies, guaranteeing that child subagent tasks nest hierarchically beneath parent user prompts with branch glyphs (`↳`, `└──`) and distinct origin tiers (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, `TOOL_OUTPUT`).
3. **Task 3 (`Imp-672`)**: `src-tauri/src/modules/release_gate/transcript_full_log_reverse_scanner.rs` & `tests_transcript_full_log_reverse_scanner.rs`
   - Enforce AGENTS.md Deep Transcript Inspection & Full Log Precedence: prioritize un-truncated `transcript_full.jsonl` over `transcript.jsonl`, reverse-scan across at least 10 lines, bypass trailing telemetry/heartbeat/token-usage records, and extract genuine tool executions and assistant thoughts.
4. **Task 4 (`Imp-673`)**: `src-tauri/src/modules/db_orm/fifo_queue_batch_drain_guard.rs` & `tests_fifo_queue_batch_drain_guard.rs`
   - Enforce AGENTS.md Strict FIFO Queue Invariant in Batch Draining: ensure concurrent and batched prompt queue dequeueing across SQLite tables (`active_prompts`, `prompt_backups`, `repo_prompts.db`) strictly adheres to `ORDER BY created_at ASC, id ASC` without starvation or sequence inversion.
5. **Task 5 (`Imp-674`)**: `src-tauri/src/modules/release_gate/no_silent_errors_justification_auditor.rs` & `tests_no_silent_errors_justification_auditor.rs`
   - Enforce AGENTS.md No Silent Errors Invariant: audit error-handling sites to ensure every fallible operation routes into `AppError` and every `record_ignored` has an explicit `// Justification:` comment explaining safety, rejecting bare ignores or unrouted errors.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All prompt queries must enforce `ORDER BY created_at ASC, id ASC`.
