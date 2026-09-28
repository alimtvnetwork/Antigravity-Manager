# Subtask 01: Account Switch Algorithm 98% Threshold Simulation & Multi-VM Collision Shielding

Traceability ID: Task-02
Spec Reference: [02-spec/21-app/68-account-switch-98pct-e2e-parallel-prompt-backup-and-instance-verification.md](../../../02-spec/21-app/68-account-switch-98pct-e2e-parallel-prompt-backup-and-instance-verification.md)
Target Files: src-tauri/src/services/account_switcher.rs, src-tauri/src/services/supabase_sync.rs, src-tauri/src/services/email_service.rs
Action:
- Configure auto-switch quota trigger threshold to 98% during testing simulation mode.
- Enforce the candidate verification loop: once the highest-scoring candidate account is identified, execute a live pre-activation refresh. If the refreshed balance remains unchanged and valid, select it; otherwise, re-evaluate ranking.
- Query active cluster leases in Supabase (`agm_cluster_leases`) to confirm no other VM holds an active lease on the target account.
- Inspect inbound email status telemetry for any remote VM holding the target profile. Skip any account locked or marked in-use by another node.
- Delegate the final account rotation to the Fast-Forward button handler to execute profile switching without killing the running IDE window.

Acceptance Criteria:
- Quota drops below 98% trigger the candidate selection process.
- Pre-activation refresh verifies quota consistency prior to profile rotation.
- In-use accounts across other cluster nodes are skipped.
- Profile swap is delegated to the Fast-Forward button bridge.

Targeted Verification:
- Cargo clippy check on `account_switcher.rs`: PASSED (0 errors).
- Live 98% simulation threshold tested with `agm switch-if-low-credit -t 98 --force`: PASSED.
- Account ranking, live Google API quota refresh, Supabase lease check, and inbound email lease filtering: VERIFIED.
- Fast-Forward rotation executed: VERIFIED.

Status: Completed
Verification Date: 2026-09-28
