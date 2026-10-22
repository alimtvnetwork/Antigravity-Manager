# Subtask 01: Batch 1 (Imp 361–366: Proxy Rate Limiting, Framing & Heartbeat)

- **Parent Plan**: `.ai-memory/plans/pending/169-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/169-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-361: Stream Burst Rate Limiter** (`src-tauri/src/proxy/rate_limit/stream_burst_limiter.rs`)
   - Token bucket rate limiter tuned for streaming chunks with configurable burst size and token refill rate.
2. **Imp-362: SSE Comment Heartbeat Ping** (`src-tauri/src/proxy/handlers/common/sse_ping_heartbeat.rs`)
   - Generates periodic SSE comment frames (`: keepalive <timestamp>\n\n`) when reasoning/thinking delays exceed the ping threshold.
3. **Imp-363: Client Compression Negotiator** (`src-tauri/src/proxy/handlers/common/compression_negotiator.rs`)
   - Evaluates client `Accept-Encoding` against upstream response encodings, deciding whether streaming body should be passed raw or compressed.
4. **Imp-364: Tool Call Response Formatter** (`src-tauri/src/proxy/handlers/common/tool_response_formatter.rs`)
   - Normalizes tool call execution outputs into unified Canonical IR responses across Claude, OpenAI, and Gemini protocols.
5. **Imp-365: Upstream Endpoint Health Decay** (`src-tauri/src/proxy/upstream/endpoint_health_decay.rs`)
   - Implements half-life decay of upstream error weights so recovering endpoints regain traffic gradually without thunder-herd effects.
6. **Imp-366: Claude Stop Reason Normalizer** (`src-tauri/src/proxy/handlers/claude/stop_reason_normalizer.rs`)
   - Maps Anthropic Claude finish reasons (`end_turn`, `max_tokens`, `stop_sequence`, `tool_use`) to Canonical IR and OpenAI-compatible finish reasons.
