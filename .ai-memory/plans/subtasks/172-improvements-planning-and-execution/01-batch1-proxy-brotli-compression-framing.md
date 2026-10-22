# Subtask 01: Batch 1 (Imp 451–456: Proxy Upstream Resilience, Compression & Chunk Framing)

- **Parent Plan**: `.ai-memory/plans/pending/172-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/172-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-451: Stream Brotli Decompression Fallback Filter** (`src-tauri/src/proxy/upstream/brotli_decompressor.rs`)
   - Decompresses `br` encoded upstream responses when client accepts uncompressed payloads, safeguarding stream parser pipelines.
2. **Imp-452: Upstream Connection Maximum Age Sentinel** (`src-tauri/src/proxy/upstream/connection_age_sentinel.rs`)
   - Tracks upstream HTTP/2 connection establishment timestamps and triggers smooth connection rotation after 300s to avoid stale NAT bindings.
3. **Imp-453: SSE Chunk Stream Boundary Validator** (`src-tauri/src/proxy/handlers/common/chunk_boundary_validator.rs`)
   - Validates streaming chunk transitions and byte boundaries, detecting truncated frame markers and malformed data prefixes.
4. **Imp-454: Response Content-Type Header Normalizer** (`src-tauri/src/proxy/handlers/common/response_content_type_matcher.rs`)
   - Matches and normalizes upstream `Content-Type` headers (`text/event-stream; charset=utf-8`, `application/json`) for downstream clients.
5. **Imp-455: Claude Stream Custom Stop Sequence Detector** (`src-tauri/src/proxy/handlers/claude/stop_sequence_matcher.rs`)
   - Detects client-specified custom stop sequences in active Claude streaming deltas and safely terminates generation upon match.
6. **Imp-456: Adaptive Connection Pool Dynamic Sizer** (`src-tauri/src/proxy/upstream/dynamic_pool_sizer.rs`)
   - Adjusts maximum idle connections per upstream host dynamically (5 to 50) based on real-time request concurrency.
