# Subtask 03: Streaming Chunk Boundary Guard (`Imp-682`)
Status: IN_PROGRESS
Parent Plan: 211-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Architecture & Pipeline First:
"The pipeline stage uniformly handles the converted Gemini payloads, including thinking block backfilling, thinking budget filtering and backfilling, unified context structural alignment, prefix stability, and the sanitization of risky prompts and request headers."

Implement `src-tauri/src/proxy/handlers/common/streaming_chunk_boundary_guard.rs` and unit tests in `src-tauri/src/proxy/handlers/common/tests_streaming_chunk_boundary_guard.rs`.

## Requirements
- Enforce maximum chunk boundary size limits (e.g. 1MB cap per raw SSE chunk) to prevent unbounded memory buffer allocation.
- Inspect and validate chunk boundaries for compliant `\n\n` and `\r\n\r\n` SSE frame delimiters.
- Provide structured report tracking processed chunks, total byte volume, and boundary compliance.
- Comprehensive unit tests covering oversized chunk detection, valid chunk slicing, and delimiter validation.
