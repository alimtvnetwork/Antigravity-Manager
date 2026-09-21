---
name: agm-thinking-store
description: Specialized skill for managing, debugging, and enhancing the server-side Thinking Store, thought signature cache, reasoning token budgets, and session isolation in Antigravity-Manager.
---

# AGM Thinking Store & Thought Signature Preservation

This skill guides engineering work on the server-side Thinking Store. Upstream Google Gemini models with reasoning capabilities (e.g. Gemini 2.0 Flash Thinking, Pro Thinking) emit cryptographic `thoughtSignature` fields alongside thought blocks. Downstream clients frequently strip or truncate reasoning content, causing upstream `400 Invalid Argument` errors on subsequent turns. The Thinking Store solves this by caching and re-injecting full thoughts and signatures.

## Key Source Files

- `src-tauri/src/proxy/thinking_store.rs` — Core implementation of the full thinking-block store, content-based matching, and session eviction.
- `src-tauri/src/proxy/handlers/thinking.rs` — Thinking budget extraction and reasoning content adapter.
- `src-tauri/src/proxy/config.rs` — Configuration accessors (`is_thinking_store_enabled`, `get_thinking_max_memory_turns`, `get_thinking_retention_days`).
- `src-tauri/src/proxy/handlers/openai.rs` & `handlers/claude.rs` — Integration points where incoming turn payloads are inspected and enriched with cached thought signatures.

## Core Mechanisms

### 1. Session Isolation
- Each thinking context is keyed by:
  `{tenant}:{client_session_id}`
  - `tenant`: SHA-256 hash of the caller's API key or user token.
  - `client_session_id`: Extracted from HTTP header `X-Session-Id` or request body `session_id`.
- Memory is isolated per session with hard caps (`MAX_SESSIONS = 2000`, `MAX_BYTES_PER_SESSION = 64 MB`).

### 2. Content-Based Matching
Rather than relying on fragile turn indices (which break between differing packet shapes in OpenAI, Claude, and Gemini protocols):
- Matches incoming conversational history by fingerprinting visible assistant responses and associated tool call IDs.
- Restores the exact `thoughtSignature` and untruncated thought text from the previous turn into the outgoing Gemini `Content` block.

### 3. Model Force Rules
- Models containing `claude`, `flash`, `pro`, or `agent` automatically trigger server-side thinking and signature tracking.
- Image generation (`imagen`), embedding (`embed`), or lightweight models are explicitly excluded from thinking store interception to prevent invalid configuration errors.

### 4. Memory Retention & Persistence
- Turns per session are governed by `thinking_max_memory_turns`.
- Sessions older than `idle_ttl` (retention days) are pruned during background sweep cycles.
- Persists session access timestamps at periodic intervals (`TOUCH_PERSIST_INTERVAL`) to optimize memory-to-disk synchronization.

## Verification Checklist
- When modifying matching logic, ensure tests in `src-tauri/src/proxy/tests/` pass.
- Verify signature length validation (`MIN_SIGNATURE_LENGTH = 50`) and sentinel bypass rules.
- Test that downstream streaming does not emit duplicate reasoning deltas.
