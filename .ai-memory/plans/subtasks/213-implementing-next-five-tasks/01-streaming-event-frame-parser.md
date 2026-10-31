# Subtask 01: Streaming Event Frame Parser (`Imp-690`)
Sequence: 213
Step: 276
Status: COMPLETED
Module: `src-tauri/src/proxy/handlers/common/streaming_event_frame_parser.rs`
Test Suite: `src-tauri/src/proxy/handlers/common/tests_streaming_event_frame_parser.rs`

## Description
Parses and validates raw SSE and HTTP event streams into discrete Gemini protocol chunks, verifying boundary frames, detecting unclosed data payloads, parsing multi-line `data: ` blocks according to W3C SSE standard, and guaranteeing zero event loss or buffer truncation during high-throughput proxy streaming.

## Verification Details
- Implemented `ParsedEventFrame` struct with `event_type`, `data`, `id`, `retry_ms`, and `is_comment` fields.
- Implemented `StreamingFrameParser` with `push_chunk`, `finish`, `reset`, and offset/metric inspection helpers.
- Concatenates multi-line `data:` lines with `\n`.
- Flushes complete frames on empty line dispatch boundaries (`\n\n` and `\r\n\r\n`).
- Accurately parses SSE comments (`: comment`) while preserving subsequent data payloads.
- Handles byte-fragmented chunk streaming across buffer boundaries.
- Flushes trailing partial frames without data loss when `finish()` is invoked.
- Registered and re-exported in `src-tauri/src/proxy/handlers/common.rs`.
- Comprehensive unit test suite in `src-tauri/src/proxy/handlers/common/tests_streaming_event_frame_parser.rs`.
