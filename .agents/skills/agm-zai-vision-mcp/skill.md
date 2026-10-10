---
name: agm-zai-vision-mcp
description: Specialized skill for managing Model Context Protocol (MCP) server endpoints, ZAI vision tools (understand_image, analyze_video), session lifecycles, and tool call routing in src-tauri/src/proxy/zai_vision_mcp.rs, zai_vision_tools.rs, and handlers/mcp.rs.
---

# AGM ZAI Vision & MCP Subsystem

This skill provides comprehensive architectural guidance, schema definitions, session lifecycles, and tool execution procedures for the **Model Context Protocol (MCP)** and **ZAI Vision** integrations in **Antigravity-Manager (AGM)**.

---

## 1. Subsystem Architecture Overview

AGM embeds native MCP server capabilities and specialized ZAI Vision tools to give client agents (Claude Code, Cursor, OpenCode, Gemini) vision inspection and video understanding via external ZAI PaaS APIs:

```
+----------------------------------------------------------------------------------------------------+
|                                      Client Agent Tool Calls                                       |
|    - MCP SSE Stream: GET /mcp/sse                                                                  |
|    - MCP Message Post: POST /mcp/messages?sessionId=<uuid>                                         |
|    - Native Tool Interception: call_understand_image, call_analyze_video                           |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                    MCP Server Handler (Axum)                                       |
|                              src-tauri/src/proxy/handlers/mcp.rs                                   |
|    - Session Store: ZaiVisionMcpState (tracks active UUID session tokens)                          |
|    - SSE Keepalive: 15s interval ping stream                                                       |
|    - Upstream Dispatch: Forwards tool calls to ZAI endpoint or internal tool executor              |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                       ZAI Vision Tools Engine                                      |
|                            src-tauri/src/proxy/zai_vision_tools.rs                                 |
|    - Tools Definition: understand_image, analyze_video schemas                                     |
|    - Media Ingestion: Dispatches HTTP URLs, local files, and data URLs                             |
|    - ZAI PaaS Client: Calls https://api.z.ai/api/paas/v4/chat/completions                          |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/proxy/zai_vision_mcp.rs` | `ZaiVisionMcpState` session store, generating UUID v4 session IDs, checking session validity, and cleaning closed sessions. |
| `src-tauri/src/proxy/zai_vision_tools.rs` | ZAI Vision schemas (`understand_image`, `analyze_video`), media source resolution (`image_source_to_content`, `video_source_to_content`), client builder with upstream proxy support, and execution via ZAI PaaS chat completions. |
| `src-tauri/src/proxy/handlers/mcp.rs` | Axum routes for `/mcp/sse` and `/mcp/messages`. SSE transport handshake, keep-alive heartbeats, and message forwarding. |
| `src-tauri/src/proxy/config.rs` | `ZaiConfig`, `ZaiMcpConfig`, API key resolution, and upstream proxy integration. |

---

## 3. Core Invariants & Rules

1. **Config Guard & Graceful Deactivation**:
   - If `zai.enabled` is false or `zai.api_key` is empty, MCP endpoints must reject requests with `400 BAD_REQUEST ("z.ai is not configured")` or `404 NOT_FOUND` when `mcp.enabled` is false.
   - Never initiate network calls to ZAI endpoints when unconfigured.

2. **Session Lifecycle & SSE Heartbeat**:
   - `/mcp/sse` must establish a long-lived `text/event-stream` with a unique `sessionId` query parameter and send periodic keepalive events every 15 seconds.
   - All subsequent message invocations via `/mcp/messages` must supply a valid `sessionId` registered in `ZaiVisionMcpState`.

3. **Safe Media Resolution**:
   - In `zai_vision_tools.rs`, local file paths must be validated and converted to base64 `data:` URLs before dispatching to external APIs.
   - Size limits must be checked against `max_size_mb` configured in `ZaiConfig`.

4. **Transparent Tool Call Interception**:
   - When a client agent invokes `understand_image` or `analyze_video`, AGM intercepts the call, executes it against the ZAI vision endpoint, and formats the output into standard MCP tool responses.

---

## 4. Verification & Testing

```bash
# 1. Quality pre-flight checks
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features

# 2. Check MCP SSE handshake
curl -N http://127.0.0.1:8045/mcp/sse \
  -H "Authorization: Bearer test"
```
