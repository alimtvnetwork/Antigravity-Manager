# Subtask 03: Streaming Chunk Boundary Guard (`Imp-682`)
Status: COMPLETED
Parent Plan: 211-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Architecture & Pipeline First:
"The pipeline stage uniformly handles the converted Gemini payloads, including thinking block backfilling, thinking budget filtering and backfilling, unified context structural alignment, prefix stability, and the sanitization of risky prompts and request headers."

Implemented `src-tauri/src/proxy/handlers/common/streaming_chunk_boundary_guard.rs` and unit tests in `src-tauri/src/proxy/handlers/common/tests_streaming_chunk_boundary_guard.rs`.
Registered in `src-tauri/src/proxy/handlers/common.rs`.

## Verification Evidence
- Defined `DEFAULT_MAX_CHUNK_BYTES` (1MB ceiling) and `ChunkBoundaryConfig`.
- Implemented `validate_streaming_chunk`, `contains_sse_frame_delimiter`, and `audit_streaming_stream`.
- Detected and rejected oversized chunks, zero-byte chunks, and unclosed stream frames.
- Added comprehensive unit tests covering standard chunks, capacity violations, and tail frame boundaries.
- Deferred E2E command documented in `02-spec/21-app/211-implementing-next-five-tasks/e2e_master_plan.md`.
