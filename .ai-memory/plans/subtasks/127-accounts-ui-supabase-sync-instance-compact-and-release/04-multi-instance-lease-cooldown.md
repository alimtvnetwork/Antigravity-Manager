---
plan: 127-accounts-ui-supabase-sync-instance-compact-and-release
subtask: "04"
title: Multi-Instance Account Exclusivity, Email Usage Cooldown, and Settings UI
domain: backend-rust-and-frontend
depends_on:
  - "03"
citations:
  app_spec: ../../../../02-spec/21-app/127-accounts-ui-supabase-sync-instance-compact-and-release/02-component-spec.md#22-multi-instance-account-exclusivity-workspace_lease_managerrs
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/workspace_lease_manager.rs
  - src-tauri/src/modules/auto_switcher.rs
  - src-tauri/src/models/config.rs
  - src/types/config.ts
  - src/components/settings/AutoSwitcherSettings.tsx
status: pending
---

# 04 — Multi-Instance Account Exclusivity, Email Usage Cooldown, and Settings UI

## 1. Context & Motivation
Developers frequently operate multiple instances of Antigravity on a single machine or distribute workloads across separate physical and virtual nodes. When multiple instances share an account pool, collisions occur if two nodes switch to the same account simultaneously, resulting in rapid rate limiting and depleted 4-hour quotas.

To prevent collisions:
1. Distributed leases registered in Supabase Root DB (`workspace_leases` table) must strictly block any other instance or machine from acquiring the same account or email.
2. A configurable **Email Usage Cooldown Window** (`account_cooldown_minutes`, default: 60 mins, range: 15–120 mins) must filter candidate accounts so recently used accounts are skipped.
3. If **all** healthy accounts are currently cooling down, the auto-switcher must not stall or fail; it must gracefully fall back to the account with the oldest `last_used` timestamp.
4. During profile rotation, the bound email address must be passed directly in-memory to the lease acquisition function to avoid redundant synchronous disk reads.
5. The settings UI must expose quick pill presets and full minute selection for developer convenience.

---

## 2. Target Files & Symbols
- `src-tauri/src/modules/workspace_lease_manager.rs`
  - `is_account_or_email_leased_by_other(account_id: &str, email: &str) -> bool`
  - `acquire_lease_with_details(account_id: &str, account_email: &str, profile_name: &str, ttl_secs: i64) -> Result<LeaseResult, AppError>`
- `src-tauri/src/modules/auto_switcher.rs`
  - `select_candidate_profiles(...) -> Result<Vec<ProfileCandidate>, String>`
  - Auto-switch lease spawn at ~L1579
- `src-tauri/src/models/config.rs`
  - `AutoProfileSwitcherConfig::account_cooldown_minutes: u32`
- `src/types/config.ts`
  - `AutoProfileSwitcherConfig.account_cooldown_minutes?: number`
- `src/components/settings/AutoSwitcherSettings.tsx`
  - Quick pills (15m, 30m, 45m, 60m, 90m, 120m), dropdown, and tooltip

---

## 3. Implementation Steps

### 3.1 Distributed Exclusivity Verification (`workspace_lease_manager.rs`)
1. In `is_account_or_email_leased_by_other(account_id: &str, email: &str)`:
   - Check cached leases in `ACTIVE_REMOTE_LEASES`.
   - Match by cleaned account ID (`k == acc_id` or `lease.account_id == acc_id`) OR cleaned email (`lease.account_email == email` or `lease.profile_name == email`).
   - Determine whether lease is active:
     - Must belong to a remote node (`lease.node_id != local_node`).
     - Must not be stale (`now - lease.leased_at <= stale_timeout_secs`).
     - Must be within lockout window OR unexpired (`(now - lease.leased_at < lockout_window_secs) || lease.expires_at > now`).
   - Return `true` if any valid remote lease blocks the account.

### 3.2 Candidate Separation & Graceful Fallback (`auto_switcher.rs`)
1. In `select_candidate_profiles()`:
   - Read `account_cooldown_minutes` (default 60) and compute `cooldown_secs = (account_cooldown_minutes as i64) * 60`.
   - Implement `is_in_cooldown` predicate checking both `last_used` recency and active remote lease cache.
   - For all accounts meeting 100% quota or elapsed reset time:
     - If `is_in_cooldown(&acc)`: push to `cooldown_pool` as `(candidate, acc.last_used)`.
     - Else: push to `available_pool`.
2. Selection & Graceful Fallback:
   ```rust
   let selected_candidates = if !available_pool.is_empty() {
       available_pool.sort_by(|a, b| match b.score.partial_cmp(&a.score) {
           Some(std::cmp::Ordering::Equal) | None => {
               a.email.to_lowercase().cmp(&b.email.to_lowercase())
           }
           Some(ord) => ord,
       });
       available_pool
   } else if !cooldown_pool.is_empty() {
       crate::modules::logger::log_info(&format!(
           "[AutoSwitcher] All {} healthy account(s) are currently in cooldown. Triggering graceful fallback to oldest last_used account.",
           cooldown_pool.len()
       ));
       cooldown_pool.sort_by(
           |(a_cand, a_last), (b_cand, b_last)| match a_last.cmp(b_last) {
               std::cmp::Ordering::Equal => match b_cand.score.partial_cmp(&a_cand.score) {
                   Some(std::cmp::Ordering::Equal) | None => a_cand
                       .email
                       .to_lowercase()
                       .cmp(&b_cand.email.to_lowercase()),
                   Some(ord) => ord,
               },
               ord => ord,
           },
       );
       cooldown_pool.into_iter().map(|(cand, _)| cand).collect()
   } else {
       Vec::new()
   };
   ```

### 3.3 Direct In-Memory Email Propagation at L1579 (`auto_switcher.rs`)
1. In `auto_switcher.rs` around line 1579:
   - Replace generic `acquire_lease(&target_acc_id, &target_inst_id, lease_ttl)` with:
     ```rust
     let target_acc_id = target.account_id.clone();
     let target_email = target.email.clone();
     let target_inst_id = target.instance_id.clone();
     let lease_ttl = crate::modules::workspace_lease_manager::get_default_lease_ttl_secs();
     tauri::async_runtime::spawn(async move {
         let _ = crate::modules::workspace_lease_manager::acquire_lease_with_details(
             &target_acc_id,
             &target_email,
             &target_inst_id,
             lease_ttl,
         )
         .await;
     });
     ```
   - This eliminates redundant filesystem reads to `accounts/<id>.json`.

### 3.4 Backend Serde Configuration (`src-tauri/src/models/config.rs`)
1. Ensure `AutoProfileSwitcherConfig` declares:
   ```rust
   fn default_account_cooldown_minutes() -> u32 {
       60
   }

   #[serde(default = "default_account_cooldown_minutes")]
   pub account_cooldown_minutes: u32,
   ```
2. Verify `Default` trait initializes `account_cooldown_minutes: 60`.

### 3.5 Frontend Settings UI & Types
1. In `src/types/config.ts`:
   - Add `account_cooldown_minutes?: number;` to `AutoProfileSwitcherConfig`.
2. In `src/components/settings/AutoSwitcherSettings.tsx`:
   - Render Account Reuse Cooldown Window section with:
     - Header label: `Account Reuse Cooldown (Minutes)` with Clock icon.
     - Value display: `<span className="font-mono font-bold">{currentConfig.account_cooldown_minutes ?? 60} min</span>`.
     - Select input with values `15`, `30`, `45`, `60`, `90`, `120`.
     - Quick pill buttons (`[15, 30, 45, 60, 90, 120]`) with active highlight:
       `bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-400 border-indigo-300 dark:border-indigo-800`.
     - Informative caption:
       *"Enforces cross-machine lease lock and skips recently used accounts until cooldown expires, with automatic fallback if all accounts are cooling down."*

---

## 4. Constraints & Invariants
- **Zero-Build & Zero-Test Execution**: Do not run `npm build`, `cargo build`, or test commands during this authoring phase.
- **Strictly Preserve Logic**: Ensure the 100% 4-hour quota gate and subscription tier scoring are preserved intact.
- **Graceful Fallback Guarantee**: Never leave an instance without a candidate if healthy accounts exist in cooldown.

---

## 5. Acceptance Criteria

| # | Condition | Expected Outcome |
| :--- | :--- | :--- |
| **AC-1** | Account leased by Node-A in Supabase `workspace_leases` | Node-B skips this account during `select_candidate_profiles()` when evaluating candidate profiles. |
| **AC-2** | Account used 10 minutes ago with `account_cooldown_minutes = 60` | Account placed in `cooldown_pool`; skipped in favor of accounts not in cooldown. |
| **AC-3** | All available accounts have been used within the last 30 minutes | System activates graceful fallback, selecting the account with the oldest `last_used` timestamp. |
| **AC-4** | Auto-switch triggers on depleted quota | `acquire_lease_with_details` receives `target.email` directly in-memory without calling `load_account()`. |
| **AC-5** | User clicks `45m` quick pill in Settings UI | `account_cooldown_minutes` updates to `45` in configuration and saves to disk. |
