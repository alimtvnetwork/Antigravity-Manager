# Testing, Verification, and Fixture Specifications

> **Specification:** `02-spec/21-app/07-testing-and-verification.md`
> **Status:** Production-Ready
> **Source Directories:** `src-tauri/src/proxy/tests/`, `src-tauri/src/`, `02-spec/21-app/fixtures/`

---

## 1. Overview & Test Architecture

The Antigravity-Manager verification suite provides multi-layered validation across the reverse proxy gateway, database persistence, protocol mappers, and Tauri IPC interfaces. Over 520 automated unit and integration tests are maintained within `src-tauri/`.

```mermaid
graph TD
    TestRunner["Test Runner (cargo test)"] --> UnitTests["Unit Tests"]
    TestRunner --> IntegrationTests["Integration Tests"]
    TestRunner --> ContractTests["Contract & Fixture Tests"]

    UnitTests --> Mappers["Protocol Translation (OpenAI/Claude/Gemini)"]
    UnitTests --> Hashing["FNV-1a Session & Fingerprint"]
    UnitTests --> CircuitBreaker["Rate Limit & Circuit Breakers"]

    IntegrationTests --> Sqlite["SQLite WAL Concurrency (Rusqlite)"]
    IntegrationTests --> Auth["OAuth Loopback Server & Tokens"]

    ContractTests --> Fixtures["Fixtures (fixtures/)"]
```

---

## 2. Test Fixtures Directory (`fixtures/`)

Representative wire payloads are stored under `02-spec/21-app/fixtures/`:

| Fixture File | Protocol / Purpose | Verification Target |
|---|---|---|
| [`openai-chat-request.json`](fixtures/openai-chat-request.json) | OpenAI `/v1/chat/completions` request | Validates JSON deserialization into `OpenAIRequest` |
| [`claude-message-request.json`](fixtures/claude-message-request.json) | Claude `/v1/messages` request | Validates conversion to upstream Gemini format |
| [`gemini-rpc-envelope.json`](fixtures/gemini-rpc-envelope.json) | Upstream Antigravity RPC payload | Validates signed 64-bit integer `sessionId` & envelope |
| [`proxy-request-log.json`](fixtures/proxy-request-log.json) | 18-column SQLite log record | Validates `ProxyRequestLog` persistence schema |

---

## 3. Core Test Suites & Coverage Requirements

### 3.1 Session ID & Fingerprinting (`proxy/common/session.rs`, `proxy/session_manager.rs`)
- **Deterministic FNV-1a Hashing:** Verifies that given identical `account_id`, `fingerprint`, and `generation`, `derive_session_scoped` returns an exact signed 64-bit decimal string.
- **Upstream Session Scoping:** Verifies that session generation bumps correctly invalidate accumulated tokens while preserving conversation context.

### 3.2 Protocol Translation Mappers (`proxy/mappers/`)
- **OpenAI Streaming:** Verifies SSE chunks emit `data: {"object":"chat.completion.chunk",...}` with accurate deltas and terminating `data: [DONE]`.
- **Claude Tool Rewriting:** Verifies transformation of Claude tool calls (`EnterPlanMode`, `grep`, `read`) to upstream Gemini equivalents.
- **Thinking Blocks:** Verifies stripping of `<think>` tags and preservation of thought signatures in `SignatureCache`.

### 3.3 SQLite WAL Concurrency & Migrations (`modules/proxy_db.rs`, `security_db.rs`)
- **WAL Pragma Conformance:** Verifies connections execute with `PRAGMA journal_mode = WAL` and `PRAGMA busy_timeout = 5000`.
- **18-Column Schema Persistence:** Verifies insertion, indexing, and pagination over `request_logs`.

---

## 4. CI/CD Automated Test Execution

Automated test gates must be executed in local workflows and GitHub Actions:

### 4.1 Rust Backend Test Suites

```bash
# Run full Rust test suite
cargo test --manifest-path src-tauri/Cargo.toml -- --nocapture

# Run specific unit test modules
cargo test --manifest-path src-tauri/Cargo.toml --package antigravity-tools --lib proxy::common::session
cargo test --manifest-path src-tauri/Cargo.toml --package antigravity-tools --lib modules::security
```

### 4.2 Frontend Verification & Quality Gates

```bash
# Run TypeScript compilation check
npx tsc --noEmit

# Run ESLint validation
npm run lint

# Run production build validation
npm run build
```

---

## 5. Verification & Acceptance Criteria

### AC-TST-001: Cargo Unit Test Suite
- **Executable Test:** `tests::test_cargo_unit_test_suite`
- **Given** The Rust backend workspace located in `src-tauri/`.
- **When** `cargo test --manifest-path src-tauri/Cargo.toml` executes across all unit modules (`src-tauri/src/proxy/common/session.rs`, `src-tauri/src/modules/security.rs`, `src-tauri/src/proxy/tests/`).
- **Then** All unit test suites pass with zero failures and zero compile warnings.

### AC-TST-002: Frontend TypeScript Build & Lint Quality Gate
- **Executable Test:** `tests::test_frontend_typescript_build`
- **Given** The React + TypeScript frontend codebase in `src/`.
- **When** Quality gates execute via `npm run lint`, `tsc --noEmit`, and `npm run build`.
- **Then** Static analysis and Vite production bundling succeed with exit code 0 and zero TypeScript diagnostic errors.

### AC-TST-003: Wire Protocol Fixture Parity
- **Executable Test:** `tests::test_wire_protocol_fixture_parity`
- **Given** Sample wire payloads stored in `02-spec/21-app/fixtures/` (`openai-chat-request.json`, `claude-message-request.json`, `gemini-rpc-envelope.json`, `proxy-request-log.json`).
- **When** Payloads are deserialized by the protocol mapping engines.
- **Then** All JSON fixtures parse cleanly into their target Rust structs with 100% field fidelity.
