# Subtask 01: Batch 1 (Imp 421–426: Proxy Upstream TLS, HTTP/2 Flow Control & SSE Framing)

- **Parent Plan**: `.ai-memory/plans/pending/171-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/171-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-421: TLS SNI Hostname Sanitizer & IDN Normalizer** (`src-tauri/src/proxy/upstream/tls_sni_sanitizer.rs`)
   - Sanitizes and normalizes upstream TLS Server Name Indication (SNI) hostnames, stripping port numbers, path fragments, and converting internationalized domain names (IDN/Punycode).
2. **Imp-422: HTTP/2 Window Size & Concurrency Flow Controller** (`src-tauri/src/proxy/upstream/http2_flow_controller.rs`)
   - Configures adaptive HTTP/2 connection window size (64KB to 16MB) and concurrent stream thresholds to prevent upstream flow-control buffer stalls.
3. **Imp-423: SSE Chunk Coalescer & Incomplete Frame Buffer** (`src-tauri/src/proxy/handlers/common/sse_chunk_coalescer.rs`)
   - Buffers and coalesces fragmented SSE byte slices, ensuring downstream clients receive intact `data: {...}\n\n` event boundaries without split UTF-8 runes.
4. **Imp-424: Request Content-Length & Body Size Bounds Validator** (`src-tauri/src/proxy/handlers/common/request_content_length_validator.rs`)
   - Validates `Content-Length` headers against actual stream body sizes, intercepting payload size spoofing and oversized requests (>32MB).
5. **Imp-425: Claude Message Content & Multimodal Block Flattener** (`src-tauri/src/proxy/handlers/claude/message_content_flattener.rs`)
   - Normalizes heterogeneous Claude message contents (strings, nested text blocks, image payloads) into canonical linear content representations.
6. **Imp-426: Weighted Round Robin with Randomized Jitter Load Balancer** (`src-tauri/src/proxy/upstream/round_robin_jitter.rs`)
   - Implements smooth weighted round-robin upstream routing augmented with proportional jitter to eliminate traffic synchronization across identical weight nodes.
