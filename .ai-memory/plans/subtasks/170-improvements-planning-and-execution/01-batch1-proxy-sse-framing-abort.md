# Subtask 01: Batch 1 (Imp 391–396: Proxy SSE Framing, Abort Sentinel & Payload Sanitization)

- **Parent Plan**: `.ai-memory/plans/pending/170-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/170-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-391: Upstream Connection Idle Reaper** (`src-tauri/src/proxy/upstream/idle_connection_reaper.rs`)
   - Identifies and terminates idle upstream HTTP/2 and TCP connections that exceeded idle timeout (60s).
2. **Imp-392: SSE Boundary Delimiter Normalizer** (`src-tauri/src/proxy/handlers/common/sse_boundary_normalizer.rs`)
   - Standardizes irregular SSE delimiters (`\r\n\r\n`, trailing solitary `\n`) into RFC-compliant double newline `\n\n`.
3. **Imp-393: Stream Early Abort Sentinel** (`src-tauri/src/proxy/handlers/common/stream_early_abort_sentinel.rs`)
   - Detects client-side TCP reset (RST) and disconnect events mid-stream, immediately propagating cancellation upstream.
4. **Imp-394: Request Header Sanitization Filter** (`src-tauri/src/proxy/handlers/common/request_header_sanitizer.rs`)
   - Strips hop-by-hop, internal host, and sensitive credential headers (`x-forwarded-for`, `cookie`, `proxy-authorization`).
5. **Imp-395: Claude Tool Use Stream Frame Assembler** (`src-tauri/src/proxy/handlers/claude/tool_use_stream_assembler.rs`)
   - Reconstructs fragmented `content_block_start` and `content_block_delta` JSON chunks for Claude tool uses into complete function arguments.
6. **Imp-396: Upstream Route Latency Exponential Smoothing** (`src-tauri/src/proxy/upstream/latency_exponential_smoother.rs`)
   - Applies exponential moving average (EMA with alpha = 0.2) to smooth jittery route latency measurements.
