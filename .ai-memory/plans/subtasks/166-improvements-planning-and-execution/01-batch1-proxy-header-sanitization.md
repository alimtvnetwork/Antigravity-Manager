# Subtask 01: Batch 1 (Imp 271–276: Reverse Proxy Header Sanitization, Upstream Fallback & Response Assembly)

- **Parent Plan**: `.ai-memory/plans/pending/166-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/166-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Targeted Improvements
- [x] **Imp-271: Header Allowlist Filter** (`src-tauri/src/proxy/handlers/common/header_allowlist_filter.rs`)
- [x] **Imp-272: Upstream Non-Streaming Response Chunk Assembler** (`src-tauri/src/proxy/upstream/response_chunk_assembler.rs`)
- [x] **Imp-273: User-Agent Header Normalizer** (`src-tauri/src/proxy/handlers/common/user_agent_normalizer.rs`)
- [x] **Imp-274: Upstream Endpoint Weight Matrix** (`src-tauri/src/proxy/upstream/endpoint_weight_matrix.rs`)
- [x] **Imp-275: Compression & Content-Encoding Guard** (`src-tauri/src/proxy/handlers/common/compression_guard.rs`)
- [x] **Imp-276: Upstream Transport Metrics Collector** (`src-tauri/src/proxy/upstream/transport_metrics_collector.rs`)

## Verification Summary
- `header_allowlist_filter.rs`: Permitted header filter preventing credential leakage and internal routing pollution.
- `response_chunk_assembler.rs`: Byte-capping chunk assembler preventing memory exhaustion on oversized upstream responses.
- `user_agent_normalizer.rs`: Standards-compliant User-Agent normalizer preserving client tags while adding Antigravity signatures.
- `endpoint_weight_matrix.rs`: Dynamic load distribution weight matrix rewarding successes and degrading failing hosts.
- `compression_guard.rs`: Handles Content-Encoding negotiation and prevents duplicate decompression.
- `transport_metrics_collector.rs`: Rolling average TTFB and timing collector for upstream health analysis.
