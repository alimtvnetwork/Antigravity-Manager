---
name: agm-thinking-store
description: Specialized skill for managing the Antigravity-Manager thinking store, thought block positioning (Invariant I4), L1/L2 dual-layer cache, content-based fingerprinting, and signature re-injection.
---

# AGM Thinking Store & Signature Preservation Architecture

This skill provides comprehensive architectural guidance, thought block handling invariants, caching layers, and signature hydration procedures for the Thinking Store subsystem in Antigravity-Manager.

---

## 1. Subsystem Architecture Overview

The Thinking Store ensures that AI thinking/reasoning blocks and opaque tool call cryptographic signatures are preserved across multi-turn conversations, even when client applications strip or truncate them:

```
+-----------------------------------------------------------------------------------------+
|                                  Client Inbound Turn                                    |
|   (Client sends history; client may have dropped thoughts or signatures)               |
+--------------------------------------------+--------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                              Inbound Thinking Pipeline                                  |
|                         src-tauri/src/proxy/pipeline/inbound.rs                         |
|  1. Normalize Tool Call IDs: force call_... canonical format                            |
|  2. Thought Positioning: enforce thought: true strictly at index 0 of parts             |
|  3. Drop Placeholders (Invariant I4): drop dummy thoughts ("...", "·")                  |
|  4. Signature Migration: migrate valid signatures from placeholders to anchor parts     |
|  5. Causal Anchor Matching: compute causal anchor from preceding user & system turns    |
+--------------------------------------------+--------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                             Thinking Store (Dual-Layer)                                 |
|                       src-tauri/src/proxy/thinking_store.rs                             |
|  +-----------------------------------------------------------------------------------+  |
|  |  L1 Memory Cache: DashMap<String, SessionEntry> (capped at 2000 sessions, 64MB)   |  |
|  +-----------------------------------------+-----------------------------------------+  |
|                                            | Cache Miss                                 |
|                                            v                                            |
|  +-----------------------------------------------------------------------------------+  |
|  |  L2 Persistent Storage: SQLite WAL mode in proxy.db (thinking_sessions table)    |  |
|  +-----------------------------------------------------------------------------------+  |
+--------------------------------------------+--------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                            Signature & Thought Hydration                                |
|  - Re-inject cached thought blocks and tool call signatures into Canonical IR payload   |
|  - Pass fully hydrated payload to Google Cloud Code upstream                            |
+--------------------------------------------+--------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                                Outbound Capture Pipe                                    |
|  - Stream upstream response chunks to client                                            |
|  - Intercept model thought blocks & signatures; save to L1 Memory + L2 SQLite Storage   |
+-----------------------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/proxy/thinking_store.rs` | L1 DashMap cache, L2 SQLite persistence (`proxy.db`), session cleanup, causal anchor computation, and fingerprint matching. |
| `src-tauri/src/proxy/pipeline/inbound.rs` | Multi-stage inbound thinking pipeline: tool call ID normalization, thought sorting, placeholder dropping, signature migration, and history hydration. |
| `src-tauri/src/proxy/handlers/thinking.rs` | Thinking parameters normalization, thinking budget calculations (`maxOutputTokens = budget + 8192`), and model tier mapping. |
| `src-tauri/src/proxy/proxy_db.rs` | SQLite schema and operations for `thinking_sessions` and request logs in `proxy.db`. |

---

## 3. Critical Invariants for Thinking Blocks

### Invariant I1: Thought Block Positioning
- In any model turn `contents`, parts with `thought: true` **must reside strictly at index 0** of `parts`.
- If an incoming payload contains thoughts scattered across parts, `sort_thought_to_index_zero` moves them to the head before prefix alignment.

### Invariant I2: Gemini Thought Part Signatures
- Gemini thought parts **must never carry signatures**.
- If a client or upstream sends a thought block with a signature, the signature must be extracted and migrated to the first non-thought anchor part (text or functionCall).

### Invariant I3: Foreign Signature Stripping
- Claude signatures must not be sent to Gemini models, and Gemini signatures must not be sent to Claude models. Cross-protocol adapters must strip foreign signatures prior to upstream transmission.

### Invariant I4: Placeholder Dropping & Signature Preservation
- Placeholder thought blocks containing trivial placeholders (e.g. `...`, `·`, `.`, `[undefined]`, whitespace-only) are dropped from the upstream request.
- **Critical Caveat**: If a dropped placeholder block carries a valid cryptographic signature, that signature **must not be discarded**; it must be migrated to the first non-thought part in that turn.

---

## 4. Content-Based Fingerprinting & Causal Anchors

Rather than relying on fragile conversation turn indices (which break when clients summarize or prune history), the Thinking Store uses content-based hashing:

1. **Part Fingerprint (`ThinkingRecord.fingerprint`)**:
   - SHA-256 hash computed over visible text and tool call identifiers.
2. **Causal Anchor (`compute_causal_anchor`)**:
   - Computed from the hash of the preceding user turn combined with system instructions.
   - Allows exact retrieval of thoughts and signatures even if intermediate assistant turns were modified by client UI layers.

---

## 5. Model Tier Routing & Budget Configuration

- **Thinking Budget Calculation**: When a client requests an explicit thinking budget, `configure_inbound_thinking` sets `thinkingBudget` and dynamically sizes `maxOutputTokens = budget + 8192`.
- **Gemini < 3 Stripping**: For legacy Gemini models (e.g. Gemini 1.5 Pro / Flash), thinking parameters are unsupported upstream; the pipeline automatically strips all thinking configuration.
- **Model Tiers**: Maps `flash_low`, `flash_medium`, `flash_high` to appropriate budget bounds while protecting against false 429 quota exhaustion.
