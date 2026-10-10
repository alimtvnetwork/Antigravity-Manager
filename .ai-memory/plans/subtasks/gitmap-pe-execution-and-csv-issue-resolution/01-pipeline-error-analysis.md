---
plan: gitmap-pe-execution-and-csv-issue-resolution
subtask: "001"
title: Pipeline Error Analysis & Exhaustive Broken Symbol Inventory
domain: backend-rust
depends_on: none
citations:
  architecture_spec: 02-spec/21-app/gitmap-pe-execution-and-csv-issue-resolution/01-architecture-spec.md
  coding_guidelines: 02-spec/02-coding-guidelines/02-canonical-size-tier.md
  error_management: 02-spec/03-error-manage/01-index.md
target_files:
  - src-tauri/src/lib.rs
  - src-tauri/src/proxy/rate_limit.rs
  - src-tauri/src/proxy/rate_limit/retryparsermode.rs
  - src-tauri/src/proxy/rate_limit/ratelimittracker_impl.rs
  - src-tauri/src/proxy/signature_cache.rs
  - src-tauri/src/proxy/signature_cache/cacheentry.rs
  - src-tauri/src/proxy/signature_cache/tests.rs
  - src-tauri/src/proxy/http_session_store.rs
  - src-tauri/src/proxy/http_session_store/httpsessionentry.rs
  - src-tauri/src/proxy/http_session_store/tests.rs
  - src-tauri/src/proxy/session_manager.rs
  - src-tauri/src/proxy/session_manager/sanitize_user_text_for_fingerprint.rs
  - src-tauri/src/proxy/upstream/client/client_calls.rs
  - src-tauri/src/proxy/upstream/client/mod.rs
status: pending
---

# 001 — Pipeline Error Analysis & Exhaustive Broken Symbol Inventory

## 1. Executive Context & Pipeline Failure Telemetry

In GitHub Actions CI Run `#38055868344` on commit `13a820d` (`main` branch), automated verification failed across macOS, Ubuntu, and Windows runners:
- **Library Compilation (`cargo check` / `clippy`):** 503 errors and 1,103 warnings.
- **Test Compilation (`agm-alim (lib test)`):** 606 errors and 1,108 warnings.
- **Top-level Halt:** `error[E0583]: file not found for module appruntimeflags` in `src-tauri/src/lib.rs`.

The failure was introduced during the mechanical file split in commits `22a87aae` and `50a0e58f` to satisfy the <= 500 lines per file guideline. The splitter partitioned single-file modules into separate files without upgrading cross-file struct fields and methods from `private` to `pub(crate)`, and injected illegal test re-exports (`pub(crate) use tests::tests;`).

This document provides the exhaustive symbol inventory, precise line mappings, visibility fixes, and verification contracts required to restore exit-code 0 across the entire CI/CD pipeline.

---

## 2. Exhaustive Broken Symbol Inventory & Remediation Matrix

### 2.1 Crate Root Module Resolution (`src-tauri/src/lib.rs`)

| File Relative Path | Line(s) | Current Code | Failure Diagnostic | Root Cause | Required Remediation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `src-tauri/src/lib.rs` | 23 | `mod appruntimeflags;` | `error[E0583]: file not found for module appruntimeflags` | Submodule files were placed in `src-tauri/src/lib/` instead of `src-tauri/src/`. In crate root (`lib.rs`), `rustc` searches `src/`, not `src/lib/`. | Add explicit path attribute: `#[path = "lib/appruntimeflags.rs"] mod appruntimeflags;` |
| `src-tauri/src/lib.rs` | 24 | `mod run;` | `error[E0583]: file not found for module run` | Same as above. `src/lib/run.rs` unmapped. | Add explicit path attribute: `#[path = "lib/run.rs"] mod run;` |
| `src-tauri/src/lib.rs` | 25 | `mod setup_app;` | `error[E0583]: file not found for module setup_app` | Same as above. `src/lib/setup_app.rs` unmapped. | Add explicit path attribute: `#[path = "lib/setup_app.rs"] mod setup_app;` |

---

### 2.2 Rate Limit Tracker (`src-tauri/src/proxy/rate_limit/`)

| File Relative Path | Line(s) | Broken Symbol | Current Visibility | Failure Diagnostic | Required Remediation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `src-tauri/src/proxy/rate_limit/retryparsermode.rs` | 95 | `RateLimitTracker.limits` | `private` field | `error[E0616]: field limits of struct RateLimitTracker is private` (in `ratelimittracker_impl.rs` & `ratelimittracker_impl_2.rs`) | Change to `pub(crate) limits: DashMap<String, RateLimitInfo>,` |
| `src-tauri/src/proxy/rate_limit/retryparsermode.rs` | 97 | `RateLimitTracker.quota_limits` | `private` field | Potential `E0616` on cross-file quota access | Change to `pub(crate) quota_limits: DashMap<(String, String), QuotaBucketLimit>,` |
| `src-tauri/src/proxy/rate_limit/retryparsermode.rs` | 99 | `RateLimitTracker.failure_counts` | `private` field | `error[E0616]: field failure_counts of struct RateLimitTracker is private` (in `ratelimittracker_impl.rs:170`, `_2.rs:97`) | Change to `pub(crate) failure_counts: DashMap<String, (u32, SystemTime)>,` |
| `src-tauri/src/proxy/rate_limit/retryparsermode.rs` | 114 | `RateLimitTracker::get_limit_key` | `private fn` | `error[E0624]: method get_limit_key is private` (in `ratelimittracker_impl.rs:283`, `_2.rs:9,14,29,30,33`) | Change to `pub(crate) fn get_limit_key(&self, account_id: &str, model: Option<&str>) -> String` |
| `src-tauri/src/proxy/rate_limit/ratelimittracker_impl.rs` | 302 | `RateLimitTracker::parse_rate_limit_reason` | `private fn` | `error[E0624]: method parse_rate_limit_reason is private` (in `ratelimittracker_impl_2.rs:311, 326`) | Change to `pub(crate) fn parse_rate_limit_reason(&self, body: &str) -> RateLimitReason` |
| `src-tauri/src/proxy/rate_limit/ratelimittracker_impl.rs` | 366 | `RateLimitTracker::parse_retry_time_from_body` | `private fn` | `error[E0624]: method parse_retry_time_from_body is private` (in `ratelimittracker_impl_2.rs:141, 159, 167`) | Change to `pub(crate) fn parse_retry_time_from_body(&self, body: &str) -> Option<u64>` |
| `src-tauri/src/proxy/rate_limit.rs` | 9 | `tests` re-export | `pub(crate) use ratelimittracker_impl_2::tests;` | Namespace clutter & test symbol leak into crate root | Remove line 9 completely. Tests execute via module discovery. |

---

### 2.3 Signature Cache (`src-tauri/src/proxy/signature_cache/`)

| File Relative Path | Line(s) | Broken Symbol | Current Visibility | Failure Diagnostic | Required Remediation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `src-tauri/src/proxy/signature_cache/cacheentry.rs` | 70 | `SignatureCache::new` | `private fn` | `error[E0624]: associated function new is private` (in `tests.rs:9, 19, 26, 35, 81, 112, 124`) | Change to `pub(crate) fn new() -> Self` |
| `src-tauri/src/proxy/signature_cache.rs` | 11 | `tests::tests` re-export | `pub(crate) use tests::tests;` | `error[E0364] / error[E0365]`: Re-exporting non-public or colliding test module | Remove line 11 completely. Module tests are declared with `#[cfg(test)] mod tests;`. |
| `src-tauri/src/proxy/signature_cache/tests.rs` | 4 | `mod tests` | `pub(crate) mod tests` inside `tests.rs` | Namespace nesting defect | Change to `mod tests {` (private to `tests.rs`). |

---

### 2.4 HTTP Session Store (`src-tauri/src/proxy/http_session_store/`)

| File Relative Path | Line(s) | Broken Symbol | Current Visibility | Failure Diagnostic | Required Remediation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `src-tauri/src/proxy/http_session_store/httpsessionentry.rs` | 42 | `HttpSessionStore.sessions` | `private` field | `error[E0616]: field sessions of struct HttpSessionStore is private` (in `tests.rs:81, 82, 112, 113, 116, 120`) | Change to `pub(crate) sessions: HashMap<String, StoredSession>,` |
| `src-tauri/src/proxy/http_session_store/httpsessionentry.rs` | 46 | `HttpSessionStore::new` | `private fn` | `error[E0624]: associated function new is private` (in `tests.rs:60, 87`) | Change to `pub(crate) fn new() -> Self` |
| `src-tauri/src/proxy/http_session_store/httpsessionentry.rs` | 52 | `HttpSessionStore::get` | `private fn` | `error[E0624]: method get is private` (in `tests.rs:62, 78, 89, 90`) | Change to `pub(crate) fn get(&mut self, response_id: &str) -> Option<(HttpSessionEntry, SessionParent)>` |
| `src-tauri/src/proxy/http_session_store/httpsessionentry.rs` | 67 | `HttpSessionStore::insert` | `private fn` | `error[E0624]: method insert is private` (in `tests.rs:61, 88`) | Change to `pub(crate) fn insert(&mut self, response_id: String, entry: HttpSessionEntry)` |
| `src-tauri/src/proxy/http_session_store/httpsessionentry.rs` | 79 | `HttpSessionStore::insert_delta` | `private fn` | `error[E0624]: method insert_delta is private` (in `tests.rs:68, 93, 102`) | Change to `pub(crate) fn insert_delta(&mut self, ...)` |
| `src-tauri/src/proxy/http_session_store.rs` | 27 | `tests::tests` re-export | `pub(crate) use tests::tests;` | Illegal test re-export into crate namespace | Remove line 27 completely. |

---

### 2.5 Session Manager (`src-tauri/src/proxy/session_manager/`)

| File Relative Path | Line(s) | Broken Symbol | Current Visibility | Failure Diagnostic | Required Remediation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `src-tauri/src/proxy/session_manager.rs` | 9-11 | `SessionManager` struct re-export | Missing re-export | Callers importing `crate::proxy::session_manager::SessionManager` fail | Add `pub use sanitize_user_text_for_fingerprint::SessionManager;` |
| `src-tauri/src/proxy/session_manager.rs` | 10 | `tests::tests` re-export | `pub(crate) use tests::tests;` | Illegal test re-export into crate namespace | Remove line 10 completely. |

---

### 2.6 Ancillary Proxy Subsystems (Upstream Client, Proxy Pool, Mappers)

| File Relative Path | Line(s) | Broken Symbol | Current Visibility | Failure Diagnostic | Required Remediation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `src-tauri/src/proxy/upstream/client/client_calls.rs` | 5 | `UpstreamClient::build_url` | `private fn` | `error[E0624]: associated function build_url is private` (in `tests.rs:10, 16`) | Change to `pub(crate) fn build_url(base_url: &str, method: &str, query_string: Option<&str>) -> String` |
| `src-tauri/src/proxy/upstream/client/types.rs` | 14 | `UpstreamClient.client_cache` | `private` field | `error[E0616]: field client_cache of struct UpstreamClient is private` (in `client_core.rs:71, 72`) | Change to `pub(crate) client_cache: DashMap<String, Client>,` |
| `src-tauri/src/proxy/upstream/client/mod.rs` | 19 | `tests` re-export | `pub use tests::*;` | Warning / collision in module re-export | Remove line 19 completely. |
| `src-tauri/src/proxy/proxy_pool/get_global_proxy_pool.rs` | 15-20 | `ProxyPoolManager` fields | `private` fields (`account_bindings`, `config`) | `error[E0616]: field account_bindings / config is private` (in `proxypoolmanager_impl.rs:123, 130, 293, 302`) | Change both fields to `pub(crate)` |
| `src-tauri/src/proxy/pipeline/inbound/align.rs` | 166 | `strip_thinking_prefix` | `private fn` | `error[E0624]: associated function strip_thinking_prefix is private` (in `process.rs:246`) | Change to `pub(crate) fn strip_thinking_prefix(text: &str) -> String` |
| `src-tauri/src/proxy/mappers/claude/streaming/state.rs` | 10 | `StreamingState.used_tool` | `private` field | `error[E0616]: field used_tool is private` (in `tests.rs:119, 156, 213, etc.`) | Change to `pub(crate) used_tool: bool,` |
| `src-tauri/src/proxy/mappers/openai/streaming/tests_a.rs` & `_b.rs` | Top | Trait import | Missing `StreamExt` | `error[E0599]: no method named next found for struct Pin<Box<dyn Stream>>` | Add `use futures::StreamExt;` |
| `src-tauri/src/proxy/mappers/gemini/wrapper/v2_compression.rs` | 74, 90 | Dereference on `&mut bool` | Raw assignment to borrow | `error[E0308]: mismatched types, expected &mut bool, found bool` | Change `compression_applied = false;` to `*compression_applied = false;` (and `true`) |
| `src-tauri/src/proxy/mappers/gemini/wrapper/v2_compression.rs` | 80, 106 | Unary not on `&mut bool` | `!compression_applied` | `error[E0600]: cannot apply unary operator ! to type &mut bool` | Change to `!*compression_applied` |

---

## 3. Surgical Remediation Implementation Sequence

Execution must proceed in strict dependency order to achieve progressive compiler convergence:

### Step 3.1: Fix Root Crate Path Resolution (`src-tauri/src/lib.rs`)
Directly patch `src-tauri/src/lib.rs` to inform `rustc` of the location of `appruntimeflags`, `run`, and `setup_app`:
```rust
#[path = "lib/appruntimeflags.rs"]
mod appruntimeflags;
#[path = "lib/run.rs"]
mod run;
#[path = "lib/setup_app.rs"]
mod setup_app;
```
*Outcome:* Eliminates the fatal `error[E0583]` blocker that halted top-level compilation on macOS and Ubuntu runners.

### Step 3.2: Rate Limit Visibility & Namespace Sanitization
1. In `src-tauri/src/proxy/rate_limit/retryparsermode.rs`, add `pub(crate)` to `limits`, `quota_limits`, `failure_counts`, and `get_limit_key`.
2. In `src-tauri/src/proxy/rate_limit/ratelimittracker_impl.rs`, add `pub(crate)` to `parse_rate_limit_reason` and `parse_retry_time_from_body`.
3. In `src-tauri/src/proxy/rate_limit.rs`, remove `pub(crate) use ratelimittracker_impl_2::tests;`.
*Outcome:* Resolves all 28 `E0616` and `E0624` errors in the `rate_limit` subsystem.

### Step 3.3: Signature Cache Constructor Visibility & Re-export Removal
1. In `src-tauri/src/proxy/signature_cache/cacheentry.rs`, update `fn new()` to `pub(crate) fn new() -> Self`.
2. In `src-tauri/src/proxy/signature_cache.rs`, remove `pub(crate) use tests::tests;`.
*Outcome:* Resolves 8 private constructor errors (`E0624`) across `signature_cache/tests.rs`.

### Step 3.4: HTTP Session Store Method & Field Visibility
1. In `src-tauri/src/proxy/http_session_store/httpsessionentry.rs`:
   - Set `pub(crate) sessions: HashMap<String, StoredSession>,` on struct `HttpSessionStore`.
   - Set `pub(crate) fn new()`, `pub(crate) fn get(...)`, `pub(crate) fn insert(...)`, and `pub(crate) fn insert_delta(...)`.
2. In `src-tauri/src/proxy/http_session_store.rs`, remove `pub(crate) use tests::tests;`.
*Outcome:* Resolves 14 visibility errors (`E0616`, `E0624`) across `http_session_store/tests.rs`.

### Step 3.5: Session Manager Export & Test Namespace Cleaning
1. In `src-tauri/src/proxy/session_manager.rs`, add `pub use sanitize_user_text_for_fingerprint::SessionManager;`.
2. Remove `pub(crate) use tests::tests;`.
*Outcome:* Ensures clean type export and eliminates test namespace collisions.

### Step 3.6: Ancillary Subsystems Visibility & Syntax Alignment
1. In `src-tauri/src/proxy/upstream/client/client_calls.rs`, mark `pub(crate) fn build_url`.
2. In `src-tauri/src/proxy/proxy_pool/get_global_proxy_pool.rs`, mark `pub(crate)` on `account_bindings` and `config`.
3. In `src-tauri/src/proxy/pipeline/inbound/align.rs`, mark `pub(crate) fn strip_thinking_prefix`.
4. In `src-tauri/src/proxy/mappers/gemini/wrapper/v2_compression.rs`, dereference `*compression_applied`.
5. In `src-tauri/src/proxy/mappers/openai/streaming/tests_a.rs` & `tests_b.rs`, import `use futures::StreamExt;`.
*Outcome:* Eliminates all remaining secondary errors across upstream, proxy pool, and streaming adapters.

---

## 4. Coding Guidelines & Invariant Adherence

- **Affirmative Booleans:** All added or modified flags must use affirmative naming (`is_enabled`, `has_parent`, `is_quota_exhausted`).
- **Strict Relative Git Paths:** All documented file references must be relative to repository root (`src-tauri/...`). Never use absolute paths (`C:\...`) or `file:///` URLs.
- **No Silent Errors:** Any error propagation touched must preserve `AppError` routing and structured logging.

---

## 5. Verification Gate & Quality Acceptance Criteria

### 5.1 Local Pre-Flight Evidence Expectations
```bash
# 1. Format check
cd src-tauri && cargo fmt -- --check

# 2. Comprehensive Clippy gate (includes full compilation check)
cd src-tauri && cargo clippy --all-targets --all-features

# 3. Targeted test execution for modified proxy modules
cd src-tauri && cargo test proxy::rate_limit
cd src-tauri && cargo test proxy::signature_cache
cd src-tauri && cargo test proxy::http_session_store
cd src-tauri && cargo test proxy::session_manager
cd src-tauri && cargo test proxy::upstream::client
```
All commands must terminate with **Exit Code 0** with zero errors.

### 5.2 CI Pipeline Target
- Pipeline run `#38055868344` replacement run must complete with `conclusion: success` across all jobs (`Check Rust Code` and `Build Tauri App` on macOS, Ubuntu, and Windows).
- `gitmap pe` must output `0 failed section(s)`.

---

## 6. Acceptance Criteria Checklist

- [ ] `src-tauri/src/lib.rs` resolves `appruntimeflags`, `run`, and `setup_app` without `E0583`.
- [ ] `RateLimitTracker.limits`, `quota_limits`, `failure_counts`, `get_limit_key`, `parse_rate_limit_reason`, and `parse_retry_time_from_body` have `pub(crate)` visibility.
- [ ] `SignatureCache::new` has `pub(crate)` visibility.
- [ ] `HttpSessionStore.sessions` and methods `new`, `get`, `insert`, `insert_delta` have `pub(crate)` visibility.
- [ ] `pub(crate) use tests::tests;` is purged from `signature_cache.rs`, `http_session_store.rs`, `session_manager.rs`, and `rate_limit.rs`.
- [ ] `UpstreamClient::build_url` and `ProxyPoolManager` fields have `pub(crate)` visibility.
- [ ] `agm-alim` library and test targets compile with zero errors (`0 errors`, `exit 0`).
