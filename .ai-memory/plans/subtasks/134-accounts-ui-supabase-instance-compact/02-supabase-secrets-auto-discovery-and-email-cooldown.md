# Subtask: 02-supabase-secrets-auto-discovery-and-email-cooldown

- **Task ID**: `134-accounts-ui-supabase-instance-compact`
- **Subtask Number**: `02`
- **Status**: `[QUEUED]`
- **Spec Reference**: [02-spec/21-app/134-accounts-ui-supabase-instance-compact/01-architecture-spec.md](../../../../02-spec/21-app/134-accounts-ui-supabase-instance-compact/01-architecture-spec.md)
- **Parent Plan**: [.ai-memory/plans/134-accounts-ui-supabase-instance-compact.md](../../134-accounts-ui-supabase-instance-compact.md)

---

## 1. Objective

Implement backend and configuration mechanics to:
1. Automatically discover and seed Supabase root credentials from `d:\work\repo-secrets` (and `REPO_SECRETS_DIR`), eliminating manual database setup.
2. Prevent multiple instances or different machines from accessing the same Google account simultaneously using Supabase distributed leases (`workspace_leases` table).
3. Introduce a pre-switch lease lock gate in `account.rs` (`switch_account`) and `instance.rs` (`switch_instance_account`) that checks whether an account or email is held by another active machine before changing IDE state.
4. Enforce a configurable 30–60 minute account and email reuse cooldown window (`account_cooldown_minutes`), ensuring graceful fallback to the oldest cooling account when all accounts are in cooldown.
5. Expose the cooldown setting in `AutoSwitcherSettings.tsx` with quick selection pills (30m, 45m, 60m) styled with `rounded-[5px]`.

---

## 2. Target Files & Subsystems

- `src-tauri/src/modules/supabase_sync.rs` (candidate paths, auto-discovery, URL normalization)
- `src-tauri/src/modules/workspace_lease_manager.rs` (atomic lease acquisition, cross-node lock checks, email mapping)
- `src-tauri/src/modules/auto_switcher.rs` (cooldown evaluation, candidate pool partitioning, two-tier fallback)
- `src-tauri/src/modules/account.rs` (pre-switch lease conflict check)
- `src-tauri/src/modules/instance.rs` (instance-level pre-switch lease check)
- `src-tauri/src/models/config.rs` (`account_cooldown_minutes: u32` in `AutoProfileSwitcherConfig`)
- `src/components/settings/AutoSwitcherSettings.tsx` (settings selector, quick pills, explanatory tooltip)
- `src/types/config.ts` (TypeScript configuration contract)

---

## 3. Surgical Implementation Specifications

### 3.1 Supabase Secrets Auto-Discovery (`supabase_sync.rs`)
- **Probing Candidate Order (`candidate_repo_secrets_paths`)**:
  Include all host repo-secrets directories:
  1. `REPO_SECRETS_DIR` environment variable (direct file or subdirectory).
  2. `D:/work/repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json`
  3. `D:/work/repo-secrets/02-antigravity-manager/vault/supabase_config.json`
  4. `D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json`
  5. `D:/work/repo-secrets/03-supabase/02-lovable/supabase-credentials.json`
  6. Matching `C:/work/repo-secrets/...` drive paths.
  7. Fallback relative paths: `../repo-secrets/...`, `../../repo-secrets/...`, `repo-secrets/...`
  8. User home directory: `~/.antigravity_tools/repo-secrets/...`
- **Auto-Seed Action (`auto_seed_from_repo_secrets`)**:
  When `supabase_config.json` is missing or has no enabled endpoints:
  - Read candidate JSON file, stripping UTF-8 BOM if present.
  - Extract endpoint URL, public key, secret key, role (`root`), and human-readable name.
  - Normalize URL using `normalize_supabase_url` (strip trailing slashes, enforce `https://`).
  - Deduplicate endpoints to prevent redundant sync loops.
  - Auto-enable synchronization: `cfg.is_sync_enabled = true`.
  - Persist configuration atomically to disk.

### 3.2 Distributed Account Leases & Multi-Instance Lockout (`workspace_lease_manager.rs`)
- **Cross-Node Concurrency Contract**:
  Distributed leases are tracked in the Supabase `workspace_leases` table:
  - Fields: `account_id`, `account_email`, `node_id`, `node_alias`, `ip_address`, `profile_name`, `leased_at`, `expires_at`.
- **Pre-Switch Lease Conflict Gate**:
  Implement a helper `check_account_lease_conflict`:
  ```rust
  pub async fn check_account_lease_conflict(
      account_id: &str,
      account_email: &str,
  ) -> Result<(), AppError> {
      let config = supabase_sync::load_config()?;
      let client = match get_root_client(&config) {
          Some(c) => c,
          None => return Ok(()), // Standalone mode: allow local switch
      };

      let node_id = supabase_sync::get_local_node_id();
      let now = Utc::now().timestamp();

      let query = if !account_email.is_empty() {
          format!(
              "or=(account_id.eq.{},account_email.eq.{})&select=*",
              account_id, account_email
          )
      } else {
          format!("account_id=eq.{}&select=*", account_id)
      };

      if let Ok(records) = client.select("workspace_leases", &query).await {
          if let Some(arr) = records.as_array() {
              for first in arr {
                  let current_owner = first.get("node_id").and_then(|v| v.as_str()).unwrap_or("");
                  let current_alias = first.get("node_alias").and_then(|v| v.as_str()).unwrap_or("another machine");
                  let current_profile = first.get("profile_name").and_then(|v| v.as_str()).unwrap_or("default");
                  let current_expires = first.get("expires_at").and_then(|v| v.as_i64()).unwrap_or(0);

                  if current_expires > now && current_owner != node_id {
                      return Err(AppError::Conflict(format!(
                          "Account is currently locked by remote machine '{}' (Profile: '{}', expires in {}s)",
                          current_alias,
                          current_profile,
                          current_expires - now
                      )));
                  }
              }
          }
      }

      Ok(())
  }
  ```
- **Pre-Switch Enforcement in `account.rs` and `instance.rs`**:
  Before shutting down running processes or writing IDE state files, execute `check_account_lease_conflict`. If an active foreign lease exists, abort the switch immediately and inform the caller.

### 3.3 Configurable 30–60m Cooldown & Candidate Partitioning (`auto_switcher.rs`)
- **Cooldown Window Evaluation**:
  ```rust
  let cooldown_minutes = app_config
      .as_ref()
      .map(|c| c.auto_profile_switcher.account_cooldown_minutes)
      .unwrap_or(60);
  let cooldown_secs: i64 = (cooldown_minutes as i64) * 60;

  let is_in_cooldown = |acc: &account::Account| -> bool {
      let is_recently_used = acc.last_used > 0 && (now_sec - acc.last_used) < cooldown_secs;
      let has_remote_lease = if let Some(lease) =
          crate::modules::workspace_lease_manager::find_cached_lease(&acc.id, &acc.email)
      {
          (lease.leased_at > 0 && (now_sec - lease.leased_at) < cooldown_secs)
              || lease.expires_at > now_sec
      } else {
          false
      };
      is_recently_used || has_remote_lease
  };
  ```
- **Two-Tier Candidate Pool Partitioning**:
  1. Inspect all candidate accounts not currently in active local use.
  2. If `is_in_cooldown(acc)` is `false`: push to `available_pool`.
  3. If `is_in_cooldown(acc)` is `true`: push to `cooldown_pool` along with `acc.last_used`.
  4. Selection Strategy:
     - If `available_pool` is non-empty: Sort by tier score descending, pick top candidate.
     - **Deadlock Prevention Fallback:** If `available_pool` is empty (all accounts cooling down): Sort `cooldown_pool` by `last_used` ascending (oldest cooldown first), pick top candidate as graceful fallback.

### 3.4 Frontend Settings Parity (`AutoSwitcherSettings.tsx`)
- Surfaced in the Auto Switcher settings tab:
  - Dropdown selector with options: 15m, 30m, 45m, 60m (Default), 120m.
  - Quick-select pill buttons: `[15m] [30m] [45m] [60m] [120m]` using `rounded-[5px]`.
  - Explanatory helper text: "Enforces cross-machine lease lock and skips recently used accounts until cooldown expires, with automatic fallback if all accounts are cooling down."

---

## 4. Verification & Quality Acceptance Criteria

1. **Auto-Discovery Verification**:
   - Starting the app or running `agm supabase load-json` automatically discovers endpoints from `D:/work/repo-secrets` (including `02-antigravity-and-event-manager` and `03-supabase`).
2. **Cross-Machine Lease Check**:
   - An active lease held by remote node `worker-node-2` blocks local switch attempts with an explanatory error notification.
3. **Email Cooldown Window**:
   - Setting cooldown to 30 minutes in UI or config filters out accounts used 15 minutes ago when accounts used >30 minutes ago exist.
4. **Deadlock Prevention Fallback**:
   - When ALL accounts have been used within the cooldown window, rotation selects the account with the oldest `last_used` timestamp without throwing an error or stalling.
5. **Settings UI**:
   - Quick pills and dropdown in `AutoSwitcherSettings.tsx` update `account_cooldown_minutes` reactively and persist to `app_config.json`.
