---
name: agm-proxy-engine
description: Specialized skill for developing, debugging, and maintaining the Axum-based reverse proxy engine, multi-protocol translation handlers (Claude, OpenAI, Gemini), streaming adapters, and token management in Antigravity-Manager.
---

# AGM Reverse Proxy Engine & Multi-Protocol Translation

This skill guides engineering work on the high-performance reverse proxy powering Antigravity-Manager. The proxy intercepts downstream requests from developer tools (Cursor, Claude Code, Cline, OpenAI-compatible SDKs) and translates them into upstream Google Antigravity / Gemini RPC protocols.

## Key Source Files

- `src-tauri/src/proxy/server.rs` — Axum router initialization, `AppState` management, server lifecycle, and global request middleware.
- `src-tauri/src/proxy/handlers/openai.rs` — Translates OpenAI `/v1/chat/completions`, `/v1/models`, and embeddings to Gemini API formats.
- `src-tauri/src/proxy/handlers/claude.rs` — Translates Anthropic Claude `/v1/messages` format, handling tool calls, reasoning blocks, and streaming SSE.
- `src-tauri/src/proxy/handlers/gemini.rs` — Native Google Gemini protocol pass-through (`/v1beta/models/*:generateContent` and `streamGenerateContent`).
- `src-tauri/src/proxy/token_manager.rs` — Account rotation, OAuth token refresh, upstream session binding, and rate limiting.
- `src-tauri/src/proxy/proxy_pool.rs` — Upstream proxy pool load balancing (HTTP, SOCKS5).
- `src-tauri/src/proxy/rate_limit.rs` — Per-token and per-IP rate-limiting policies.

## Architectural Guidelines

### 1. Protocol Translation Lifecycle
When a downstream client dispatches a request:
1. **Authentication & Ingestion:** Axum middleware in `server.rs` validates user tokens (`user_token_db.rs`) or admin API keys.
2. **Account & Upstream Resolution:** `TokenManager` selects an active Google account with valid quota and resolves upstream endpoints.
3. **Payload Transformation:**
   - OpenAI format requests are parsed in `handlers/openai.rs` into internal Gemini request structures.
   - Anthropic Claude format requests are parsed in `handlers/claude.rs`, converting `system` strings and `tools` schemas into Gemini `FunctionDeclaration` structures.
4. **Upstream Dispatch:** Dispatched via `upstream::client::UpstreamClient` with Keep-Alive and configurable connection warmup.
5. **Streaming Response Adapter:** Server-Sent Events (SSE) chunks from upstream Gemini are transformed into downstream client formats (`data: {"choices": [...]}` for OpenAI or `content_block_delta` for Claude).

### 2. Adding or Modifying Endpoints
- Register new routes in `src-tauri/src/proxy/server.rs` within `build_router()`.
- Place handler implementations in `src-tauri/src/proxy/handlers/<protocol>.rs`.
- Always propagate `AppState` and handle connection drops gracefully.
- Ensure error responses map to standardized JSON error structures compatible with downstream client expectations.

### 3. Verification & Testing
- Run Rust format checks: `cargo fmt --check` inside `src-tauri/`.
- Run proxy unit and integration tests: `cargo test --bin antigravity_tools_lib proxy`.
- Verify no regressions against local CI runner: `python 03-ai-scripts/06-cicd-local-runner.py`.
