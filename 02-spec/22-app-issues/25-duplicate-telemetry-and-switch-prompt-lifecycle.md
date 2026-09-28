# 4-Part Root Cause Analysis: Duplicate Telemetry Emails and Switch Prompt Lifecycle Invariants

**Document ID:** `02-spec/22-app-issues/25-duplicate-telemetry-and-switch-prompt-lifecycle.md`  
**Status:** `Resolved / Implemented`  
**Author:** Antigravity AI Orchestrator  
**Date:** 2026-09-28  

---

## 1. Reproduction & Symptoms

When an account rotation was triggered (either automatically via the low-credit monitor or manually via CLI/UI), the emitted machine telemetry and notification payloads displayed identical email addresses across distinct fields:

```json
{
  "node_alias": "W3",
  "local_ip": "192.168.1.12",
  "previous_email": "rm7419799@gmail.com",
  "predicted_email": "rm7419799@gmail.com",
  "selected_email": "rm7419799@gmail.com",
  "quota_percent": 100.0,
  "credit_before_switch": 100.0
}
```

The user observed that `previous_email`, `predicted_email`, and `selected_email` showed the exact same email address (`rm7419799@gmail.com`), creating confusion and defeating the predictive purpose of the switch telemetry.

Additionally, user feedback highlighted the critical importance of ensuring the switch button enforces the full 5-step non-destructive lifecycle:
1. Backup running prompts via AGM (`.antigravity_resume_task.json` / SQLite `pipeline.db`).
2. Close the Antigravity IDE process.
3. Switch credentials in state storage.
4. Relaunch Antigravity IDE.
5. Re-inject backed-up prompts into active workspaces.

---

## 2. Root Cause Analysis (4 Core Drivers)

1. **Default Fallback to `selected_clean` in Notification Hub:**  
   In `src-tauri/src/modules/notification_hub.rs` (lines 174–176 and 298), if `predicted_next_email` was `None`, the code explicitly defaulted to copying `selected_clean` (`details.predicted_next_email = Some(selected_clean.clone())`) and unwrapped to `details.selected_email`. Any switch with uninitialized prediction automatically cloned `selected_email` into `predicted_email`.

2. **Hardcoded Duplication in CLI Commands (`agm.rs`):**  
   In `src-tauri/src/bin/agm.rs` (line 5313 in `cmd_switch_if_low_credit` and line 6275 in `cmd_fast_forward`), `predicted_email` was hardcoded to `let predicted_email = selected_email.clone();` and `"predicted_next_account": status_after.active_account_email`. Furthermore, when `rotated == false` (no switch needed), `previous_account`, `current_account`, `predicted_next_account`, and `selected_account` all output `current_email`.

3. **Rotation Callers Not Computing the Next Candidate in Pool:**  
   In `src-tauri/src/modules/auto_switcher.rs` (lines 1470, 1545, and 1715), `predicted_email` was assigned to `Some(candidate.email.clone())` (the candidate being rotated into) rather than looking up the *subsequent* candidate that would succeed `candidate`.

4. **Current Account Unfiltered in Frontend Smart Ranking (`instanceService.ts`):**  
   In `src/services/instanceService.ts` (`rankSmartCandidates`), `currentAccountId` and `activeInUseAccountIds` were not filtered from the initial `eligible` pool. Because `(a.fourHourQuotaPercent >= 100)` was sorted first before score, an active account with 100% quota was sorted to index 0 and selected as the candidate, causing the system to attempt rotating the active account into itself (`rm7419799@gmail.com -> rm7419799@gmail.com`).

---

## 3. Concrete Code Fix & Remediation

1. **True Candidate Prediction (`auto_switcher.rs`, `account.rs`, `instance.rs`):**  
   All rotation call sites now query `select_candidate_profiles` with `pred_exclusions = [candidate.account_id, candidate.email, ...]` to find the true next candidate in line.
2. **Defensive Exclusions in Notification Hub (`notification_hub.rs`):**  
   Eliminated `Some(selected_clean.clone())` fallback. If `predicted_next_email` is missing or matches `selected_clean` or `final_prev`, `notification_hub` autonomously resolves the next candidate from the database excluding both `selected_clean` and `final_prev`. If no other candidate exists, it strictly outputs `"(none / pool exhausted)"`.
3. **CLI Telemetry Cleanup (`agm.rs`):**  
   - In `cmd_switch_if_low_credit`, dynamically queries `select_candidate_profiles` excluding `selected_email` and `prev_email`.
   - When `rotated == false`, `previous_account` and `selected_account` are set to `null` (since no rotation occurred), while `current_account` reflects the active profile and `predicted_next_account` reflects the next standby candidate.
   - In `cmd_fast_forward`, computes distinct `predicted_next_account` using candidate pool discovery.
4. **Current Account & In-Use Exclusion in Frontend (`instanceService.ts`):**  
   `rankSmartCandidates` strictly excludes `currentAccountId` and `activeInUseAccountIds` during initial filtering, enforces normalized score sorting (`b.score - a.score` where score is divided by 1000), and eliminates self-rotation.
5. **The 5-Step Switch Lifecycle Verification:**  
   Verified that both desktop integration (`DesktopIntegration::on_account_switch`) and instance switcher (`switch_account_to_instance`) strictly execute:
   - Step 1: Backup running prompts (`repo_db::backup_running_prompts`).
   - Step 2: Terminate IDE processes (`process::close_antigravity`).
   - Step 3: Inject swapped credentials into `state.vscdb`, `storage.json`, and OS keyring.
   - Step 4: Relaunch IDE (`process::start_antigravity_with_fallback_path`).
   - Step 5: Re-inject backed-up prompts (`repo_db::resend_all_running_commands` / `dispatch_running_prompts`).

---

## 4. Prevention & Quality Invariants

- **Multi-Role Distinctness Invariant:** In unit tests and integration tests, verify that `previous_email != selected_email` on successful rotation, and `predicted_email != selected_email` whenever candidate pool size $> 1$.
- **Pool Exhaustion Indicator:** If an instance has only 1 eligible account in the pool, `predicted_email` must display `(none / pool exhausted)`, never echoing the selected account.
- **Score Normalization:** Normalized scores must remain within `[0.0000, 0.5000]`. Adding `+ 1000` is strictly forbidden.
- **Zero-Drop Prompt Contract:** Never terminate an IDE process without first snapshotting active prompts in SQLite and `.antigravity_resume_task.json`.
