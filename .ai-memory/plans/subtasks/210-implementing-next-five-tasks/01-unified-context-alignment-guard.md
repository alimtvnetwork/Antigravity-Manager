# Subtask 01: Unified Context Structural Alignment Guard (`Imp-675`)
Status: COMPLETED
Parent Plan: 210-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Architecture & Pipeline First:
"The pipeline stage uniformly handles the converted Gemini payloads, including thinking block backfilling, thinking budget filtering and backfilling, unified context structural alignment, prefix stability, and the sanitization of risky prompts and request headers."

Implement `src-tauri/src/proxy/handlers/common/unified_context_alignment_guard.rs` and unit tests in `src-tauri/src/proxy/handlers/common/tests_unified_context_alignment_guard.rs`.

## Requirements
- Define dialog turn representation (`role`, `content`, `parts`).
- Validate strict Gemini role alternation: `user` -> `model` -> `user`.
- Detect adjacent duplicate roles (e.g., `user` immediately following `user`).
- Merge contiguous identical roles or backfill missing responses to guarantee structural compliance with Gemini protocol requirements.
- Provide structured report tracking input turns, aligned turns, merged turn count, and compliance status.
