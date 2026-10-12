---
plan: supabase-connection-and-ui-ux-enhancement
subtask: "01"
title: Dual Supabase Cluster Architecture, Config Auto-Seeding & Error Stack Trace Propagation
domain: backend/sync/observability
target_files:
  - src-tauri/src/modules/supabase_client/client.rs
  - src-tauri/src/modules/supabase_client/types.rs
  - src-tauri/src/modules/supabase_client/mod.rs
  - src-tauri/src/modules/supabase_sync/config_a.rs
  - src-tauri/src/modules/supabase_sync/config_b.rs
  - src-tauri/src/error.rs
  - src/stores/error-store.ts
status: pending
---

# 01 — Dual Supabase Cluster Architecture, Config Auto-Seeding & Error Stack Trace Propagation

## 1. Objective

Implement the core backend and client-side architecture for the **Dual Supabase Cluster** integration, ensuring automated configuration seeding, resilient multi-cluster routing, universal URL normalization, and end-to-end error stack trace propagation:

1. **Dual Cluster Seeding**: Configure default endpoints for Primary Lovable (`https://[REDACTED_LOVABLE_URL].supabase.co` with key `[REDACTED_LOVABLE_KEY]`) and Secondary Personal (`https://[REDACTED_PERSONAL_URL].supabase.co` with key `[REDACTED_PERSONAL_KEY]`).
2. **Cluster Failover & Router**: Enable seamless request dispatching across Primary and Secondary endpoints with health checking, fallback escalation, and 400 MB prune threshold guards.
3. **URL Normalization**: Enforce recursive stripping of trailing slashes and `/rest/v1` suffixes across all endpoint registration, persistence, and request builders.
4. **Error Observability & Stack Traces**: Implement backend stack trace capture (`std::backtrace::Backtrace`) within `AppError`, propagate traces via `AppErrorPayload` across the Tauri IPC bridge, and render actionable debug frames in `useErrorStore`.
5. **No Silent Errors Compliance**: Replace swallowed errors and bare `.ok()` usages in Supabase operations with contextual `AppError` returns or justified `record_ignored` logging.

---

## 2. Technical Scope & Implementation Specifications

### 2.1 Dual Supabase Cluster Configuration & Auto-Seeding

In `src-tauri/src/modules/supabase_sync/config_a.rs` and `config_b.rs`:
- Define canonical default seed endpoints:
  - **Primary (Lovable Root)**:
    - ID: `ep-lovable-root`
    - Name: `Supabase (Lovable Root)`
    - Raw URL: `https://[REDACTED_LOVABLE_URL].supabase.co/rest/v1/` (normalized to `https://[REDACTED_LOVABLE_URL].supabase.co`)
    - API Key: `[REDACTED_LOVABLE_KEY]`
    - Role: `root`
    - Priority: `1`
    - Prune Threshold: `400` MB
  - **Secondary (Personal Secondary)**:
    - ID: `ep-personal-secondary`
    - Name: `Supabase (Personal Secondary)`
    - Raw URL: `https://[REDACTED_PERSONAL_URL].supabase.co/rest/v1/` (normalized to `https://[REDACTED_PERSONAL_URL].supabase.co`)
    - API Key: `[REDACTED_PERSONAL_KEY]`
    - Role: `secondary`
    - Priority: `2`
    - Prune Threshold: `400` MB
- Enhance `auto_seed_from_repo_secrets()`:
  - When scanning candidate paths (`02-antigravity-manager/vault/supabase_config.json`, `03-supabase/01-own/supabase-credentials.json`, `03-supabase/02-lovable/supabase-credentials.json`), ensure both Primary and Secondary clusters are registered without overwriting user custom endpoints.
  - Automatically order endpoints by priority (`endpoints.sort_by_key(|e| e.priority)`).
  - Activate sync worker if at least one root endpoint is verified.

### 2.2 Client-Side Multi-Cluster Failover Routing

In `src-tauri/src/modules/supabase_client/client.rs`:
- Extend `SupabaseClient` to support cluster awareness:
  ```rust
  pub struct ClusterClient {
      primary: SupabaseClient,
      secondary: Option<SupabaseClient>,
  }
  ```
- Implement `request_with_failover<F, Fut, T>(&self, op: F) -> Result<T, AppError>`:
  1. Execute operation against Primary (Lovable).
  2. If Primary succeeds (HTTP < 400), return result immediately.
  3. If Primary fails due to transport timeout, rate limit (HTTP 429), or server outage (HTTP 502/503), log a warning with context and attempt the request against Secondary (Personal).
  4. If Secondary succeeds, record telemetry regarding the failover event.
  5. If both fail, construct an aggregated `AppError::Network` containing diagnostic details for both endpoints.

### 2.3 URL Normalization Invariants & Test Suite

In `src-tauri/src/modules/supabase_client/types.rs`:
- Ensure `normalize_supabase_url` handles:
  - Trailing slashes (`https://xyz.supabase.co/` -> `https://xyz.supabase.co`)
  - Redundant REST paths (`https://xyz.supabase.co/rest/v1/` -> `https://xyz.supabase.co`)
  - Multiple trailing slashes (`https://xyz.supabase.co/rest/v1///` -> `https://xyz.supabase.co`)
  - Ports and local development hosts (`http://127.0.0.1:54321/rest/v1` -> `http://127.0.0.1:54321`)
- Add unit tests in `src-tauri/src/modules/supabase_client/tests.rs` verifying all permutations.

### 2.4 Stack Trace Propagation in Rust Backend

In `src-tauri/src/error.rs`:
- Add a backtrace capture field to `AppError`:
  ```rust
  #[derive(Error, Debug)]
  pub enum AppError {
      // ... existing variants
      #[error("Supabase cluster error: {message}")]
      SupabaseCluster {
          message: String,
          status_code: Option<u16>,
          endpoint: String,
          stack_trace: String,
      },
  }
  ```
- Update `to_payload(&self)`:
  - For all variants where diagnostic context is crucial, populate `backend_stack_trace` with `std::backtrace::Backtrace::capture().to_string()` when available.
  - Ensure the JSON serialization adheres to PascalCase `Status`, `Errors`, `Attributes`, and camelCase top-level properties.
- Audit all calls in `src-tauri/src/modules/supabase_client/` and `src-tauri/src/modules/supabase_sync/`:
  - Replace any un-annotated `let _ = ...` or `.ok()` with explicit `?` propagation or `crate::error::record_ignored(result, "context")` with a mandatory `// Justification:` comment.

### 2.5 Frontend Error Store & Debugger Integration

In `src/stores/error-store.ts`:
- Verify normalization of incoming backend errors:
  - Ensure `normalizeRawError()` extracts `backend_stack_trace` from serialized payloads.
  - Parse backend stack frames into `StackFrame` objects so the user can inspect exact file and line locations in the Error Modal.
- Provide clear differentiation between:
  - `captureError()`: User-facing actions (e.g., clicking "Test Connection" or "Save Supabase Config" fails).
  - `trackWarning()`: Silent background checks (e.g., 15-second background heartbeat encounters network drop).

---

## 3. Step-by-Step Implementation Roadmap

### Phase 1: Configuration & Auto-Seeding (Rust)
- [ ] Update `candidate_repo_secrets_paths()` and fallback seeding in `src-tauri/src/modules/supabase_sync/config_a.rs` to register the Lovable and Personal clusters.
- [ ] Ensure `normalize_supabase_url()` is invoked whenever endpoints are loaded, parsed from envelopes, or received from IPC.
- [ ] Verify endpoints are sorted by priority (`priority: 1` Lovable, `priority: 2` Personal).

### Phase 2: Client Routing & Resilient Probing (Rust)
- [ ] Implement `ClusterClient` or failover methods in `src-tauri/src/modules/supabase_client/client.rs`.
- [ ] Update `test_connection()` to evaluate the 3-stage probe ladder (Root `/rest/v1/` -> Table `/nodes` -> Health `/auth/v1/health`).
- [ ] Add unit tests in `src-tauri/src/modules/supabase_client/tests.rs` for URL normalization and failover routing.

### Phase 3: Error Handling & Stack Trace Capture (Rust)
- [ ] Add `SupabaseCluster` variant and stack trace capture in `src-tauri/src/error.rs`.
- [ ] Verify `to_payload()` correctly outputs `backend_stack_trace` and `Errors.Backend`.
- [ ] Audit `supabase_client` and `supabase_sync` for compliance with the "No Silent Errors" mandate.

### Phase 4: Frontend Error Store & Diagnostic Telemetry (TypeScript)
- [ ] Update `src/stores/error-store.ts` to seamlessly unpack `backend_stack_trace` from Tauri command invocations.
- [ ] Verify that UI modals show detailed backend and frontend stack traces alongside user click trails.
- [ ] Ensure background polling uses `trackWarning` to prevent jarring modals during network blips.

### Phase 5: Verification & Quality Gates
- [ ] Run `cargo fmt -- --check` in `src-tauri`.
- [ ] Run `cargo clippy --all-targets --all-features` in `src-tauri`.
- [ ] Run `npm run build` for frontend validation.
- [ ] Validate end-to-end connectivity against both live Supabase endpoints.

---

## 4. Verification & Testing Matrix

| Test ID | Target Component | Description | Expected Outcome |
| :--- | :--- | :--- | :--- |
| **TEST-01** | `types::normalize_supabase_url` | Test trailing slashes, `/rest/v1`, ports, and idempotence. | Correct normalized base URL without slashes or `/rest/v1`. |
| **TEST-02** | `config_a::auto_seed` | Auto-seed when `supabase_config.json` is missing or empty. | Config populated with Lovable (Priority 1) and Personal (Priority 2). |
| **TEST-03** | `client::test_connection` | Test connectivity against Lovable cluster (`[REDACTED_LOVABLE_URL]`). | Returns `Ok(EndpointTestResult { is_success: true, .. })` without modal error. |
| **TEST-04** | `client::test_connection` | Test connectivity against Personal cluster (`[REDACTED_PERSONAL_URL]`). | Returns `Ok(EndpointTestResult { is_success: true, .. })` without modal error. |
| **TEST-05** | `error::to_payload` | Trigger invalid key network error and inspect JSON payload. | Payload contains `backend_stack_trace` and structured `Errors.Backend`. |
| **TEST-06** | `useErrorStore` | Receive backend error payload in frontend store. | Error Modal displays parsed frames, source component, and click trail. |
