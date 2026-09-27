# Issue 19: Smart Switch Stale Quota Selection, Disabled Account Re-selection & Multi-Node Lease Isolation RCA

## Executive Summary
During manual and automated profile switching (`agm ff`, `agm smart-switch`, and UI "Smart Switch"), the system selected accounts with depleted/low quotas (e.g. 11%–20% remaining) instead of strictly requiring 100% 4-hour window quotas. Furthermore, when an operator disabled an exhausted account (e.g. `James`) in the UI, subsequent Smart Switch executions selected the disabled account again. 

Additionally, the system failed to proactively query distributed cluster leases (Supabase `workspace_leases`, recent Email Inbound switch events, and Telegram inbound commands), creating risks of cross-node concurrency collisions.

---

## 1. Symptoms & Incident Walkthrough
1. **User Triggered Smart Switch:** An operator clicked "Smart Switch" expecting a fully refilled profile (100% 4-hour rolling quota).
2. **Switch to Low-Credit Account:** The system switched to an account (`James`) with only 20% quota remaining.
3. **Disabled Account Toggle:** The operator explicitly toggled the account to disabled in the UI to prevent its use.
4. **Immediate Re-Selection:** The operator triggered Smart Switch again. The system re-selected `James` (now at 11%–20%), completely ignoring its disabled status and low quota.
5. **No Cross-Node Lease Awareness:** The switcher selected candidates without querying active Supabase leases or cross-node email broadcast events.

---

## 2. Root Cause Analysis (4-Part RCA)

### 2.1 Why Did the System Select a 20% Quota Account?
- **Relaxed Threshold Fallback Passes:**
  In `src-tauri/src/modules/auto_switcher.rs`:
  - Lines 842–878 contained a fallback pass that dropped the minimum quota threshold to `15.0%`.
  - In `select_and_verify_next_best_profile`, lines 1014–1047 maintained a `verified_fallback` buffer accepting any candidate `>= 25.0%`. If no 100% account was found, it picked the highest fallback candidate (e.g. 20%), directly violating the strict user requirement: *"< 100% is completely zero; do not touch it."*
  - In `src/stores/useInstanceStore.ts`: Lines 518–531 maintained `bestFallbackCandidate || eligibleAccounts[0]`.

### 2.2 Why Did the Switcher Re-Select the Disabled Account (`James`)?
- **Missing `proxy_disabled` Check:**
  - When an account is toggled in the UI, `toggle_proxy_status` sets `account.proxy_disabled = true`, leaving `account.disabled = false`.
  - In `auto_switcher.rs`, lines 769, 808, and 848 only checked:
    ```rust
    if acc.disabled || acc.validation_blocked {
        continue;
    }
    ```
    `acc.proxy_disabled` was completely unverified!
  - In `useInstanceStore.ts`, line 435 only checked:
    ```ts
    const isDisabled = Boolean(acc.disabled);
    ```
    It never evaluated `acc.proxy_disabled`!
  - In `commands/mod.rs`, `toggle_proxy_status` updated `{account_id}.json` directly without updating the summary index `index.json`.

### 2.3 Why Did CLI Commands Skip Live Verification?
- In `src-tauri/src/bin/agm.rs`, lines 3861, 4346, and 5078 called `auto_switcher::select_next_best_profile()`, which was purely speculative and performed zero live Google API quota refreshes.

### 2.4 Why Were Multi-Node Leases Ignored?
- `workspace_lease_manager::is_account_leased_by_other` checked an in-memory cache `ACTIVE_REMOTE_LEASES`, but `workspace_lease_manager::list_active_leases().await` was never invoked prior to candidate evaluation to hydrate this cache from Supabase Root DB.

---

## 3. Corrective Measures & Architectural Invariants

### 3.1 Strict 100% 4-Hour Quota Enforcement
- Any candidate with fresh 4-hour window quota `< 100.0%` is treated as zero (`0.0%`).
- Delete all relaxed fallback passes (`15.0%` pass and `verified_fallback`).
- If no candidate verifies at `>= 100.0%` after pre-switch live API refresh, the switcher must abort and report `None` / `"No healthy 100% quota account available"`.

### 3.2 Comprehensive Disabled Account Filtering
- All candidate selection routines across Rust and TypeScript MUST evaluate:
  ```rust
  if acc.disabled || acc.proxy_disabled || acc.validation_blocked || !acc.is_active {
      continue;
  }
  ```
- Before candidate selection, re-read accounts fresh from disk/store.

### 3.3 Multi-Channel In-Use & Lease Isolation
- Synchronously and asynchronously exclude accounts from:
  1. Local running instances and currently bound accounts.
  2. Supabase `workspace_leases` active leases held by other node IDs.
  3. Email Inbound recent cross-VM switch events within the lookback window (3600s).
  4. Telegram inbound active switches.

---

## 4. Verification Plan
- Unit test ensuring `proxy_disabled = true` accounts are strictly excluded.
- Unit test verifying candidate selection returns `None` when accounts have `< 100.0%` quota.
- End-to-end simulation of pre-switch live verification rejecting <100% accounts.
