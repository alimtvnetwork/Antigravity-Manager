# Subtask 01: Batch 1 (Imp 211–216: Reverse Proxy Streaming Protocol Resilience & SSE Formatting)

- **Parent Plan**: `.ai-memory/plans/pending/164-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/164-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- **Imp-211 (`sse_comment_filter.rs`)**: Implemented SSE comment and event stream stripper removing diagnostic comments while preserving required data events.
- **Imp-212 (`h2_window_recovery.rs`)**: Implemented upstream HTTP/2 stream flow-control window recovery tracking stalls and advising HTTP/1.1 downgrade.
- **Imp-213 (`frame_clamper.rs`)**: Implemented stream frame size clamping sentinel chunking oversized payloads into safe 64KB buffers.
- **Imp-214 (`timeout_matrix.rs`)**: Implemented model-specific streaming connection and TTFT timeout matrix (Flash 60s, Pro 120s, Thinking 180s).
- **Imp-215 (`sse_framing_validator.rs`)**: Implemented downstream SSE framing validator ensuring proper `\n\n` delimiters.
- **Imp-216 (`health_decay.rs`)**: Implemented decaying health scorer calculating composite 0.0-1.0 scores for upstream endpoints.
- Registered and re-exported in `src-tauri/src/proxy/handlers/common.rs` and `src-tauri/src/proxy/upstream/mod.rs`.

## Targeted Improvements
- **Imp-211: SSE Comment & Event Stream Stripper** (`src-tauri/src/proxy/handlers/common/sse_comment_filter.rs`)
- **Imp-212: Upstream HTTP/2 Stream Window Recovery** (`src-tauri/src/proxy/upstream/h2_window_recovery.rs`)
- **Imp-213: Stream Frame Size Clamping Sentinel** (`src-tauri/src/proxy/handlers/common/frame_clamper.rs`)
- **Imp-214: Multi-Model Upstream Timeout Matrix** (`src-tauri/src/proxy/upstream/timeout_matrix.rs`)
- **Imp-215: Downstream SSE Framing Validator** (`src-tauri/src/proxy/handlers/common/sse_framing_validator.rs`)
- **Imp-216: Upstream Endpoint Health Decay Scorer** (`src-tauri/src/proxy/upstream/health_decay.rs`)
