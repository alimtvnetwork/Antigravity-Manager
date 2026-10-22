# Subtask 01: Batch 1 (Imp 241–246: Reverse Proxy Protocol Translation & IR Payload Normalization)

- **Parent Plan**: `.ai-memory/plans/pending/165-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/165-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Targeted Improvements
- [x] **Imp-241: Canonical Role Token Mapper** (`src-tauri/src/proxy/handlers/common/canonical_role_mapper.rs`)
- [x] **Imp-242: Upstream Dynamic DNS Resolver & Cache** (`src-tauri/src/proxy/upstream/dynamic_dns_resolver.rs`)
- [x] **Imp-243: Deep JSON Payload Sanitizer** (`src-tauri/src/proxy/handlers/common/json_payload_sanitizer.rs`)
- [x] **Imp-244: Upstream Circuit Breaker Statistics Recorder** (`src-tauri/src/proxy/upstream/circuit_breaker_stats.rs`)
- [x] **Imp-245: UTF-8 Stream Boundary Guard** (`src-tauri/src/proxy/handlers/common/stream_boundary_guard.rs`)
- [x] **Imp-246: TLS Client Connector Configuration Helper** (`src-tauri/src/proxy/upstream/tls_fingerprint_config.rs`)

## Verification Summary
- `canonical_role_mapper.rs`: Maps protocol tokens (`developer`, `assistant`, `model`, `system`, `user`, `tool`) to unified `CanonicalRole` and reverse-maps to OpenAI, Claude, and Gemini formats.
- `dynamic_dns_resolver.rs`: Thread-safe cached DNS resolution with fallback to stale cache during transient DNS outages.
- `json_payload_sanitizer.rs`: Replaces raw ASCII control characters with unicode escape sequences and removes trailing commas.
- `circuit_breaker_stats.rs`: State machine (`Closed` -> `Open` -> `HalfOpen`) tracking consecutive failures and recovery backoff timeouts.
- `stream_boundary_guard.rs`: Multibyte UTF-8 boundary buffer holding incomplete trailing bytes across frames.
- `tls_fingerprint_config.rs`: Native client TLS and connection pool configuration for edge proxy compatibility.
