# Subtask 01: Steps 1 & 2 - Boolean Logic Modernization & Guard Clause Inversion

- **Parent Plan**: `.ai-memory/plans/pending/179-test-case-implementation-and-task-management.md`
- **Spec Reference**: `02-spec/21-app/179-test-case-implementation-and-task-management/spec.md#step-1-boolean-logic-modernization-in-proxy-pool-candidate-routing`
- **Target Files**:
  - `src-tauri/src/proxy/proxy_pool/get_global_proxy_pool.rs`
  - `src-tauri/src/modules/email_inbound/imap.rs`
- **Status**: [COMPLETED]

## Description
1. Step 1: Modernized candidate qualification in `src-tauri/src/proxy/proxy_pool/get_global_proxy_pool.rs`, extracting affirmative helpers:
   - `is_proxy_eligible_for_shared_pool(proxy: &ProxyEntry, auto_failover: bool, bound_ids: &HashSet<String>) -> bool`
   - `is_bound_proxy_eligible(proxy: &ProxyEntry, auto_failover: bool) -> bool`
   Eliminated compound negative chains in shared pool filter and bound proxy lookup.
2. Step 2: Extracted `is_telemetry_from_current_machine` and `is_telemetry_event_active` in `src-tauri/src/modules/email_inbound/imap.rs`, replacing nested condition blocks with clean early-continue guard clauses.
