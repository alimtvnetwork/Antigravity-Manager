# Subtask 01: Reverse Engineer Proxy Core, Protocol Handlers & IPC Commands

> **Status:** in_progress
> **Agent:** agent_1
> **Total Files:** 116

## Scope
- `src-tauri/src/proxy/handlers/` (OpenAI, Claude, Gemini, Common)
- `src-tauri/src/proxy/mappers/` (Request/Response transformations)
- `src-tauri/src/proxy/common/` (Session tracking, fingerprinting, accumulation guards)
- `src-tauri/src/commands/` (Tauri IPC commands)

## Objectives
1. Reverse-engineer protocol routing, SSE streaming, and upstream provider translation.
2. Analyze conversation session scoping and 1M token accumulation prevention.
3. Document circuit breaker mechanics, rate limiting, and Retry-After backoff.
4. Synthesize findings into `02-spec/21-app/02-proxy-core-and-protocols.md`.
