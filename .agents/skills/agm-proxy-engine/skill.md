---
name: agm-proxy-engine
description: Specialized skill for managing the Antigravity-Manager reverse proxy gateway, 4-protocol translation (OpenAI, Claude, Gemini) to Canonical IR, streaming SSE adapters, token management, and upstream endpoint fallback.
---

# AGM Reverse Proxy Engine & Multi-Protocol Gateway

This skill provides comprehensive architectural guidance, protocol translation rules, upstream fallback strategies, and token management procedures for the Reverse Proxy Engine in Antigravity-Manager.

---

## 1. Subsystem Architecture Overview

The Reverse Proxy Gateway aggregates 4 inbound AI protocols into Google Gemini Canonical Intermediate Representation (Canonical IR) and streams back protocol-compliant responses:

```
+-----------------------------------------------------------------------------------------+
|                                    Client Requests                                      |
|    - OpenAI Chat (/v1/chat/completions)         - OpenAI Responses (/v1/responses)      |
|    - Anthropic Claude (/v1/messages)           - Google Gemini (/v1beta/models/*)       |
+--------------------------------------------+--------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                                  Axum Proxy Server                                      |
|                            src-tauri/src/proxy/server.rs                                |
|  - Unified Port 8045 (LAN & Localhost)                                                  |
|  - Downstream Auth Guard: Bearer Token / User Token DB verification                     |
|  - IP Blacklist & CIDR Filter (security.db)                                            |
+--------------------------------------------+--------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                                Protocol Adapters                                        |
|        src-tauri/src/proxy/handlers/ & src-tauri/src/proxy/mappers/                     |
|  - Normalize incoming payloads to Gemini Canonical IR (contents + generationConfig)     |
|  - Map tool definitions, function calls, system prompts, and thinking parameters        |
+--------------------------------------------+--------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                              Inbound Thinking Pipeline                                  |
|                         src-tauri/src/proxy/pipeline/inbound.rs                         |
|  - Normalize tool call IDs to canonical call_... format                                 |
|  - Position thought blocks strictly at index 0 of parts                                 |
|  - Drop placeholder thought blocks; migrate valid signatures to anchor parts            |
|  - Hydrate previous turn history & thought signatures from ThinkingStore                |
|  - Normalize functionResponse turns to role: "model"                                    |
|  - Configure thinking budget & levels; strip thinking configs for Gemini < 3 models     |
|  - Align byte-level identical request prefix topology                                   |
+--------------------------------------------+--------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                               Prompt Sanitizer Pipeline                                 |
|                     src-tauri/src/proxy/mappers/prompt_sanitizer.rs                     |
|  - Header Track: Regex-strip billing headers & agent fingerprint introductions          |
|  - System Track: Neutralize vendor declarations in first 4 sentences (WAF defense)      |
|  - Invariant: Fenced code blocks and inline code are 100% immutable                     |
+--------------------------------------------+--------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                               Upstream HTTP Client                                      |
|                         src-tauri/src/proxy/upstream/client.rs                          |
|  - Chrome 123 TLS fingerprint emulation via rquest                                      |
|  - Multi-Endpoint Fallback Ladder: Daily (P1) -> Sandbox (P2) -> Prod (P3)              |
|  - Strict Trigger: Fallback only on network error, HTTP 408, 404, or 5xx (never 400)    |
|  - Stream chunking: rquest::Body::wrap_stream exclusively for streaming                 |
+-----------------------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/proxy/server.rs` | Axum HTTP server setup, route registration, downstream token authentication, CORS, and connection lifecycle. |
| `src-tauri/src/proxy/handlers/openai.rs` | Handler for `/v1/chat/completions` and `/v1/responses`. Converts OpenAI wire format to Gemini Canonical IR. |
| `src-tauri/src/proxy/handlers/claude.rs` | Handler for `/v1/messages`. Maps Anthropic system blocks, tool use, thinking tags, and streaming events. |
| `src-tauri/src/proxy/handlers/gemini.rs` | Handler for `/v1beta/models/*`. Studio protocol passthrough and parameter normalization. |
| `src-tauri/src/proxy/pipeline/inbound.rs` | Unified multi-stage inbound pipeline: tool normalization, thought sorting, history hydration, and prefix alignment. |
| `src-tauri/src/proxy/mappers/prompt_sanitizer.rs` | WAF false-positive mitigation, billing header stripping, and system identity neutralization. |
| `src-tauri/src/proxy/upstream/client.rs` | Upstream HTTP client, Chrome TLS emulation, and 3-stage endpoint fallback ladder. |
| `src-tauri/src/proxy/token_manager.rs` | Account credential caching, double-checked locking for OAuth refresh, single-flight project ID discovery, and rate limit circuit breakers. |
| `src-tauri/src/proxy/proxy_pool.rs` | Outbound proxy pool management (RoundRobin, LeastUsed), health probes, and per-account proxy routing. |

---

## 3. Upstream Endpoint Fallback Ladder

The upstream client queries endpoints in strict descending priority order:

1. **Priority 1 (Native IDE)**: `https://daily-cloudcode-pa.googleapis.com/v1internal`
2. **Priority 2 (Sandbox)**: `https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal`
3. **Priority 3 (Production Fallback)**: `https://cloudcode-pa.googleapis.com/v1internal`

### Fallback Invariant (PR #3525 / #3526):
- Upstream fallback is triggered **strictly** when encountering a transport error, connection timeout, HTTP 408, HTTP 404, or HTTP 5xx.
- Fallback is **never** triggered on generic HTTP 400 bad requests (e.g. invalid arguments or region blocks) to prevent cyclical retry storms across accounts.

---

## 4. Token Management & Rate-Limit Shielding

- **Double-Checked Locking**: Each account refresh acquires an isolated per-account mutex (`refresh_locks: DashMap<String, Arc<Mutex<()>>>`), preventing duplicate concurrent OAuth refreshes.
- **SingleFlight Project Discovery**: Concurrent requests awaiting project ID discovery share a broadcast channel (`load_code_assist_inflight`), minimizing round-trip latency.
- **Session Scoping & 1M Token Guard (`common/session.rs`)**: Upstream Google servers accumulate conversation tokens per `sessionId`. To prevent HTTP 400 token overflows at 1,048,576 tokens, the proxy derives stable negative 64-bit FNV-1a integer session IDs and bumps generation counters on token limit overflow.
- **Circuit Breakers**: When an account receives an unrecoverable 429 quota exhaustion, it is locked until the upstream `reset_time` boundary arrives. Transient 429 spikes ($\le 2000\text{ ms}$) execute an in-place retry with a +100ms safety buffer.

---

## 5. Architectural Invariants for Proxy Modifications

1. **Decoupled Adapters**: Never insert protocol-specific business logic into the core pipeline (`pipeline/inbound.rs`). Adapters must only normalize into Canonical IR.
2. **Code Block Protection**: Never alter code fences (```) or inline code (`). `RE_CODE_BLOCK` must always remain active during text sanitization.
3. **Session Header Integrity**: Never delete or rewrite client session tracking headers (`x-*-session-id`).
4. **Pre-flight Checks**: Before committing any proxy modifications, always execute `cd src-tauri && cargo fmt -- --check` and `cd src-tauri && cargo clippy --all-targets --all-features`.
