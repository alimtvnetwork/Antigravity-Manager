# Subtask 01: Pipeline Response Header Sanitizer (`Imp-685`)
Status: IN_PROGRESS
Parent Plan: 212-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Architecture & Pipeline First:
"The pipeline stage uniformly handles the converted Gemini payloads, including thinking block backfilling, thinking budget filtering and backfilling, unified context structural alignment, prefix stability, and the sanitization of risky prompts and request headers."

Implement `src-tauri/src/proxy/handlers/common/pipeline_response_header_sanitizer.rs` and unit tests in `src-tauri/src/proxy/handlers/common/tests_pipeline_response_header_sanitizer.rs`.

## Requirements
- Strip internal and hop-by-hop upstream response headers (`server`, `alt-svc`, `x-goog-safety-ratings`, `x-goog-ratelimit-remaining`, set-cookie).
- Enforce standardized uniform Antigravity response headers (`content-type: text/event-stream; charset=utf-8`, `cache-control: no-cache`, `connection: keep-alive`).
- Comprehensive unit tests covering header filtering, content-type normalization, and preservation of standard metadata.
