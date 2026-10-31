# Subtask 01: Batch 1 (Imp 481–486: Proxy SSE Framing, Chunk Buffering & Upstream Health)

- **Parent Plan**: `.ai-memory/plans/pending/173-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/173-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-481: Upstream TCP Keep-Alive Probe Configuration** (`src-tauri/src/proxy/upstream/tcp_keepalive_probe.rs`)
   - Configures platform-level TCP keepalive probes (idle 30s, interval 10s, probe count 3) to detect silent upstream socket drops.
2. **Imp-482: SSE Stream Preamble Buffer Padder** (`src-tauri/src/proxy/handlers/common/sse_stream_padder.rs`)
   - Generates initial SSE comment byte buffers (`: padding to bypass intermediate proxy buffers\n\n`) to ensure immediate client streaming.
3. **Imp-483: Request Content-Encoding Negotiator** (`src-tauri/src/proxy/handlers/common/request_content_encoding_negotiator.rs`)
   - Negotiates Content-Encoding compatibility between downstream client capabilities and upstream endpoint headers.
4. **Imp-484: Upstream Stream Drain Cancellation Guard** (`src-tauri/src/proxy/handlers/common/stream_drain_guard.rs`)
   - Consumes residual HTTP body bytes upon downstream client abort, returning upstream connections to the pool cleanly.
5. **Imp-485: Claude Message Alternating Role Prefix Sanitizer** (`src-tauri/src/proxy/handlers/claude/role_prefix_sanitizer.rs`)
   - Validates and sanitizes message role alternation (user/assistant) in Claude payloads, inserting empty padding turns when needed.
6. **Imp-486: Multi-Endpoint Dynamic Route Balancer** (`src-tauri/src/proxy/upstream/dynamic_route_balancer.rs`)
   - Dynamically adjusts upstream route weights based on rolling error-rate scores and response latencies.
