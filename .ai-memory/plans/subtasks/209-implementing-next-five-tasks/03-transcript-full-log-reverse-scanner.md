# Subtask 03: Transcript Full Log Reverse Scanner (`Imp-672`)
Status: IN_PROGRESS
Parent Plan: 209-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Deep Transcript Inspection & Full Log Precedence:
"When inspecting conversation transcripts, always prioritize un-truncated `transcript_full.jsonl` over `transcript.jsonl`. Reverse-scan across at least 10 lines, bypassing trailing `TOKEN_USAGE`, `HEARTBEAT`, and `TELEMETRY` records, to extract genuine tool executions (`tool_calls`) and assistant thoughts (`thinking`)."

Implement `src-tauri/src/modules/release_gate/transcript_full_log_reverse_scanner.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_transcript_full_log_reverse_scanner.rs`.

## Requirements
- Enforce precedence of `transcript_full.jsonl` over compact `transcript.jsonl`.
- Reverse-scan lines from the bottom up across at least 10 records.
- Skip trailing metadata noise (`TOKEN_USAGE`, `HEARTBEAT`, `TELEMETRY`).
- Extract genuine assistant thoughts (`thinking`) and tool executions (`tool_calls`).
- Provide structured report tracking scanned records count, skipped noise count, and extracted genuine steps.
