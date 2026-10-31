# Subtask 01: Pipeline Request Header Sanitizer & Gemini Header Protocol (`Imp-645`)
Status: COMPLETED
Parent Plan: 204-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Architecture & Pipeline First:
"The pipeline stage uniformly handles the converted Gemini payloads, including thinking block backfilling, thinking budget filtering and backfilling, unified context structural alignment, prefix stability, and the sanitization of risky prompts and request headers."

Implement `src-tauri/src/proxy/handlers/common/pipeline_request_header_sanitizer.rs` and unit tests in `src-tauri/src/proxy/handlers/common/tests_pipeline_request_header_sanitizer.rs`.

## Requirements
- Enforce sanitization of headers before upstream Gemini protocol dispatch.
- Filter hop-by-hop headers (`connection`, `keep-alive`, `transfer-encoding`, `upgrade`, `proxy-authorization`).
- Strip risky internal headers or leaked auth headers (`x-goog-user-project`, `x-antigravity-internal`, `cf-connecting-ip`, `x-forwarded-for`).
- Retain authorized protocol headers (`x-goog-api-key`, `content-type`, `authorization`, `accept`, `user-agent`).
- Provide header verification report detailing stripped headers, retained headers, and violations.
