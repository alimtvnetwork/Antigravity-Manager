# Subtask 01: Batch 1 (Imp 511–516: Proxy SSE Protocol, Chunk Multiplexing & Connection Keepalive)

- **Parent Plan**: `.ai-memory/plans/pending/174-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/174-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-511: Raw SSE Stream Frame Splitter** (`src-tauri/src/proxy/handlers/common/sse_stream_splitter.rs`)
   - Splits coalesced raw SSE network frames on double newline `\n\n` boundaries while preserving partial chunk tails across incoming buffers.
2. **Imp-512: Upstream Connection Exponential Backoff Matrix** (`src-tauri/src/proxy/upstream/connection_backoff_matrix.rs`)
   - Computes exponential backoff intervals with jitter matrix for transient upstream network disconnects and HTTP 503/502 faults.
3. **Imp-513: Request Gzip Compression Sniffer** (`src-tauri/src/proxy/handlers/common/request_gzip_sniffer.rs`)
   - Detects and validates gzip/deflate magic bytes in raw HTTP request payloads before invoking decompressors.
4. **Imp-514: Upstream TLS ALPN Protocol Negotiator** (`src-tauri/src/proxy/upstream/tls_alpn_negotiator.rs`)
   - Configures TLS ALPN preferences (h2, http/1.1) and enforces protocol fallback ladders during upstream socket handshakes.
5. **Imp-515: Claude Message Turn Structure Validator** (`src-tauri/src/proxy/handlers/claude/message_turn_validator.rs`)
   - Validates Claude messages payload structure, ensuring non-empty role arrays and stripping orphaned trailing whitespace blocks.
6. **Imp-516: In-Memory DNS Failover Resolver** (`src-tauri/src/proxy/upstream/dns_failover_resolver.rs`)
   - Provides in-memory cached DNS resolution with dual-stack IPv4/IPv6 fallback for upstream hosts.
