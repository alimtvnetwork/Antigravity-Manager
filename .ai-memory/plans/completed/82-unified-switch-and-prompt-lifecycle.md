# Consolidated Plan [82]: Unified Account Switch & Prompt Lifecycle Implementation

**Canonical Spec Reference:** [02-spec/21-app/39-unified-switch-and-prompt-lifecycle.md](../../../02-spec/21-app/39-unified-switch-and-prompt-lifecycle.md)  
**Root Cause Analysis:** [02-spec/22-app-issues/25-duplicate-telemetry-and-switch-prompt-lifecycle.md](../../../02-spec/22-app-issues/25-duplicate-telemetry-and-switch-prompt-lifecycle.md)  
**Status:** `Completed & Verified`  
**Execution Loops:** 1 Lead Orchestration Loop  

---

## 1. Context & Task Origin

The user identified two critical issues:
1. In switch telemetry, `previous_email`, `selected_email`, and `predicted_email` displayed the identical email address (`rm7419799@gmail.com`).
2. The user required a clear architectural specification, Mermaid diagram, and verification of the switch button delegation flow enforcing prompt backup before IDE termination, credential switching, IDE restart, and prompt re-injection.
3. Scoring clarification: scores must be normalized by dividing by 1000.0 (compact 0.0000..0.5000), not adding 1000, and accounts with <100% quota must receive a score of 0.
4. Remote leases on Supabase must automatically expire if inactive for >6h–10h without pings or credit loss.
5. Minor version release and GitMap Pipeline verification.

---

## 2. Completed Subtasks Summary

- ✅ **Subtask 01: Distinct Multi-Role Telemetry Implementation**
  - Updated `src-tauri/src/modules/notification_hub.rs` to compute distinct `predicted_next_email` via candidate pool lookup and fallback to `"(none / pool exhausted)"` rather than echoing `selected_clean`.
  - Updated `src-tauri/src/bin/agm.rs` (`cmd_switch_if_low_credit` and `cmd_fast_forward`) to compute the actual next predicted account, and return `null` for `previous_account` and `selected_account` when no switch occurred.
  - Updated `src-tauri/src/modules/auto_switcher.rs`, `account.rs`, and `instance.rs` to pass true next-candidate predictions.

- ✅ **Subtask 02: Candidate Scoring & Active Account Exclusion**
  - Updated `src/services/instanceService.ts` to filter `currentAccountId` and `activeInUseAccountIds` from candidate selection, preventing accounts from rotating into themselves.
  - Enforced normalized scoring (divided by 1000.0) and 0-score assignment for <100% quota accounts without elapsed reset times.

- ✅ **Subtask 03: 5-Step Non-Destructive Prompt Lifecycle Verification**
  - Verified that `DesktopIntegration::on_account_switch` and `instance::switch_account_to_instance` strictly execute:
    1. Backup running prompts via AGM/SQLite before IDE termination.
    2. Gracefully close the IDE ("Kill First").
    3. Inject credentials into state storage ("Write Second").
    4. Relaunch IDE ("Start Third").
    5. Re-inject backed-up prompts from SQLite back into workspace sessions.

- ✅ **Subtask 04: Specification & Architectural Diagram Documentation**
  - Authored canonical specification `02-spec/21-app/39-unified-switch-and-prompt-lifecycle.md` with complete Mermaid diagram.
  - Authored 4-part RCA `02-spec/22-app-issues/25-duplicate-telemetry-and-switch-prompt-lifecycle.md`.
  - Updated registries in `02-spec/21-app/readme.md` and `02-spec/22-app-issues/readme.md`.

- ✅ **Subtask 05: Pre-Flight Verification & Quality Gates**
  - `cargo fmt -- --check`: Passed with 0 diffs.
  - `cargo test --lib auto_switcher`: 8 passed, 0 failed.
  - `cargo test --lib notification_hub`: 1 passed, 0 failed.
  - `cargo check --bin agm`: Passed with 0 errors.
  - `cargo check --bin agm-alim`: Passed with 0 errors.
  - `npm run build`: Passed with 0 errors.
