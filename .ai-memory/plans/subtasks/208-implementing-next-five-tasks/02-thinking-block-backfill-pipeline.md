# Subtask 02: Thinking Block Backfill Pipeline Guard (`Imp-666`)
Status: IN_PROGRESS
Parent Plan: 208-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Architecture & Pipeline First:
"The pipeline stage uniformly handles the converted Gemini payloads, including thinking block backfilling, thinking budget filtering and backfilling, unified context structural alignment, prefix stability..."

Implement `src-tauri/src/proxy/handlers/common/thinking_block_backfill_pipeline.rs` and unit tests in `src-tauri/src/proxy/handlers/common/tests_thinking_block_backfill_pipeline.rs`.

## Requirements
- Inspect streaming and non-streaming Gemini responses for reasoning model thought content.
- Ensure thinking blocks are wrapped in `<thought>` tags or flagged with `thought: true` in candidate parts.
- Backfill missing thinking blocks when thinking budget was active and reasoning tokens were generated.
- Provide structured report tracking thinking block insertion, token count validation, and formatting integrity.
