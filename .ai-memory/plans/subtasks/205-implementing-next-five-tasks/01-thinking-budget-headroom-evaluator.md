# Subtask 01: Pipeline Thinking Budget Headroom Evaluator (`Imp-650`)
Status: COMPLETED
Parent Plan: 205-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Architecture & Pipeline First:
"The pipeline stage uniformly handles the converted Gemini payloads, including thinking block backfilling, thinking budget filtering and backfilling, unified context structural alignment, prefix stability..."

Implement `src-tauri/src/proxy/handlers/common/thinking_budget_headroom_evaluator.rs` and unit tests in `src-tauri/src/proxy/handlers/common/tests_thinking_budget_headroom_evaluator.rs`.

## Requirements
- Dynamically evaluate allocated thinking budget vs max output tokens.
- Enforce that `max_output_tokens` exceeds `thinking_budget` by at least minimum text headroom (default 512 tokens).
- If thinking budget exceeds or equals max output, automatically adjust headroom by capping budget or scaling max output tokens within model limits.
- Return structured `HeadroomAllocationReport` with final thinking budget, max output tokens, and available headroom.
