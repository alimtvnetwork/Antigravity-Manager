# Subtask 03: Supabase 5-Minute Cooldown Circuit Breaker, Polling Cadence & CLM Cache Stabilization

Spec Reference: [02-spec/21-app/gitmap-node-and-prompt-system-issues/03-migration-and-verification-spec.md](../../../02-spec/21-app/gitmap-node-and-prompt-system-issues/03-migration-and-verification-spec.md)

---

## 1. Scope & Objective

Implement circuit breaker resilience, polling cadence relaxation, account list caching, and conversation liveness module (CLM) cache stabilization to eliminate log spam, network retry storms, and JSON file thrashing.

### Target Files & Primary Symbols
- `src-tauri/src/modules/supabase_sync/types.rs`: `LAST_SUPABASE_ERROR_TIMESTAMP`, `SUPABASE_COOLDOWN_DURATION_SECS`, `is_supabase_in_cooldown`, `record_supabase_error`, `reset_supabase_cooldown`
- `src-tauri/src/modules/supabase_sync/fleet.rs`: `fetch_fleet_machines`
- `src-tauri/src/modules/supabase_sync/query.rs`: `query_fleet_machines`
- `src-tauri/src/modules/workspace_lease_manager/release.rs`: `list_active_leases`
- `src-tauri/src/commands/supabase.rs`: `test_supabase_endpoint`, `save_supabase_config`
- `src/components/navbar/instanceSelector/useInstanceSelector.ts`: `useInstanceSelector`
- `src/pages/instances/useInstancePage.ts`: `useInstancePage`
- `src/components/instances/fleet/useFleetMachines.ts`: `POLL_INTERVAL_SECONDS`
- `src-tauri/src/modules/account/crud.rs`: `ACCOUNT_LIST_CACHE`, `list_accounts`, `invalidate_account_list_cache`
- `src/stores/useAccountStore.ts`: `fetchAccounts`
- `src-tauri/src/modules/repo_db/detection.rs`: `detect_active_projects` (remove circular invalidation)

---

## 2. Implementation Phases & Step-by-Step Execution

### Phase 1: Supabase 5-Minute Cooldown Circuit Breaker

#### Step 1.1: Atomic State & Duration in `src-tauri/src/modules/supabase_sync/types.rs`
Define atomic error timestamp and public helper functions:
```rust
pub(crate) static LAST_SUPABASE_ERROR_TIMESTAMP: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
pub const SUPABASE_COOLDOWN_DURATION_SECS: u64 = 300;

pub fn is_supabase_in_cooldown() -> bool {
    let last = LAST_SUPABASE_ERROR_TIMESTAMP.load(Ordering::Relaxed);
    if last == 0 {
        return false;
    }
    let now = chrono::Utc::now().timestamp().max(0) as u64;
    now.saturating_sub(last) < SUPABASE_COOLDOWN_DURATION_SECS
}

pub fn record_supabase_error() {
    let now = chrono::Utc::now().timestamp().max(0) as u64;
    LAST_SUPABASE_ERROR_TIMESTAMP.store(now, Ordering::Relaxed);
}

pub fn reset_supabase_cooldown() {
    LAST_SUPABASE_ERROR_TIMESTAMP.store(0, Ordering::Relaxed);
}
```

#### Step 1.2: Gate `fetch_fleet_machines` in `src-tauri/src/modules/supabase_sync/fleet.rs`
- At the start of `fetch_fleet_machines()`, check:
  ```rust
  if is_supabase_in_cooldown() {
      tracing::debug!(target: "supabase_sync", "Supabase in 5m cooldown; returning local machine fallback");
      return Ok(vec![construct_local_machine_info(&config, &[])]);
  }
  ```
- On transport error or client query failure, invoke `record_supabase_error();` and log concise warning without backtrace spam:
  ```rust
  record_supabase_error();
  tracing::warn!(target: "supabase_sync", "Supabase transport error: {e}. Cooldown active for 300s");
  ```

#### Step 1.3: Gate `query_fleet_machines` in `src-tauri/src/modules/supabase_sync/query.rs`
- In `query_fleet_machines()`:
  ```rust
  if is_supabase_in_cooldown() {
      return Ok(build_local_and_ssh_fleet());
  }
  ```
- On transport error connecting to Supabase:
  ```rust
  record_supabase_error();
  tracing::warn!("[Fleet] Supabase transport error: {e}. Tripping 5m circuit breaker");
  return Ok(build_local_and_ssh_fleet());
  ```

#### Step 1.4: Gate `list_active_leases` in `src-tauri/src/modules/workspace_lease_manager/release.rs`
- In `list_active_leases()`:
  ```rust
  if crate::modules::supabase_sync::is_supabase_in_cooldown() {
      if let Ok(cache) = ACTIVE_REMOTE_LEASES.read() {
          return Ok(cache.values().cloned().collect());
      }
      return Ok(Vec::new());
  }
  ```
- On HTTP query error from client:
  ```rust
  crate::modules::supabase_sync::record_supabase_error();
  ```

#### Step 1.5: Cooldown Reset in `src-tauri/src/commands/supabase.rs`
- In `test_supabase_endpoint`: Call `supabase_sync::reset_supabase_cooldown();` before initiating connection probe.
- In `save_supabase_config`: Call `supabase_sync::reset_supabase_cooldown();` so new configuration takes effect immediately.

---

### Phase 2: Transaction Frequency & Polling Cadence Optimization

#### Step 2.1: Relax `useInstanceSelector.ts` Cadence
In `src/components/navbar/instanceSelector/useInstanceSelector.ts`:
- Change interval from 4000ms to 60000ms:
  ```typescript
  useEffect(() => {
      const canRun = isTauri();
      if (!canRun) return;
      fetchInstances();
      const interval = setInterval(fetchInstances, 60000);
      return () => clearInterval(interval);
  }, [fetchInstances]);
  ```
- Add event-driven fetch on dropdown open:
  ```typescript
  useEffect(() => {
      if (isOpen && isTauri()) {
          fetchInstances();
      }
  }, [isOpen, fetchInstances]);
  ```

#### Step 2.2: Relax `useInstancePage.ts` Cadence
In `src/pages/instances/useInstancePage.ts`:
- Replace the fixed 15,000ms timer with adaptive 60,000ms (visible tab) / 300,000ms (hidden tab):
  ```typescript
  const pollInterval = typeof document !== 'undefined' && document.hidden ? 300000 : 60000;
  const timer = setInterval(() => {
      performThrottledPoll();
  }, pollInterval);
  ```

#### Step 2.3: Relax `useFleetMachines.ts` Cadence
In `src/components/instances/fleet/useFleetMachines.ts`:
- Update constant:
  ```typescript
  const POLL_INTERVAL_SECONDS = 300;
  ```

#### Step 2.4: 30-Second TTL Cache in `src-tauri/src/modules/account/crud.rs`
- Add in-memory cache definition:
  ```rust
  use std::sync::RwLock;
  use std::time::Instant;

  struct CachedAccountList {
      accounts: Vec<Account>,
      cached_at: Instant,
  }

  static ACCOUNT_LIST_CACHE: Lazy<RwLock<Option<CachedAccountList>>> = Lazy::new(|| RwLock::new(None));
  const ACCOUNT_LIST_CACHE_TTL_SECS: u64 = 30;

  pub fn invalidate_account_list_cache() {
      if let Ok(mut lock) = ACCOUNT_LIST_CACHE.write() {
          *lock = None;
      }
  }
  ```
- In `list_accounts()`: Check `ACCOUNT_LIST_CACHE.read()`. If present and `cached_at.elapsed().as_secs() < ACCOUNT_LIST_CACHE_TTL_SECS`, return cloned `accounts`. On miss, read disk files, write to cache, and return.
- In `save_account_at_path`, `delete_account`, `add_account`, `update_account_priority`: call `invalidate_account_list_cache()`.

#### Step 2.5: Debounce in `src/stores/useAccountStore.ts`
- Implement in-flight deduplication:
  ```typescript
  let inFlightFetchPromise: Promise<void> | null = null;
  let lastFetchTimestamp = 0;

  fetchAccounts: async () => {
      const now = Date.now();
      if (inFlightFetchPromise) {
          return inFlightFetchPromise;
      }
      if (now - lastFetchTimestamp < 1500 && get().accounts.length > 0) {
          return;
      }
      lastFetchTimestamp = now;
      set({ loading: true, error: null });
      inFlightFetchPromise = (async () => {
          try {
              const accounts = await accountService.listAccounts();
              set({ accounts, loading: false });
          } catch (error) {
              set({ error: String(error), loading: false });
          } finally {
              inFlightFetchPromise = null;
          }
      })();
      return inFlightFetchPromise;
  },
  ```

---

### Phase 3: CLM Cache Sabotage Remediation

#### Step 3.1: Remove Circular Invalidation from `src-tauri/src/modules/repo_db/detection.rs`
- In `detect_active_projects`:
- Locate line 254:
  ```rust
  invalidate_prompt_tree_cache(Some(target_id));
  ```
- Remove this call completely so `get_project_conversation_tree_cached` hits its 120s TTL cache on periodic checks.

---

## 3. Verification & Quality Gates

### 3.1 Automated Verification Commands
```bash
# 1. Rust Formatting
cd src-tauri && cargo fmt -- --check

# 2. Rust Compiler & Clippy Gate
cd src-tauri && cargo clippy --all-targets --all-features

# 3. Frontend Typecheck and Build
npm run build
```

### 3.2 Acceptance Verification Checklist
- [ ] `LAST_SUPABASE_ERROR_TIMESTAMP` triggers 300s cooldown on network failures.
- [ ] Background fleet queries complete in < 1ms when in cooldown.
- [ ] User testing endpoint or saving configuration resets cooldown to 0 immediately.
- [ ] `useInstanceSelector` interval is 60s; opens immediately fetch instances.
- [ ] `useFleetMachines` countdown begins at 300s.
- [ ] `list_accounts()` serves subsequent calls from 30s TTL cache without re-reading 46 JSON files.
- [ ] Mutating account operations invalidate `ACCOUNT_LIST_CACHE`.
- [ ] `useAccountStore.fetchAccounts` coalesces concurrent invocations.
- [ ] `invalidate_prompt_tree_cache` removed from `detect_active_projects:254`.
- [ ] `PROMPT_LIVENESS_PROBE` and `InstancePromptAudit` log spam eliminated during idle operation.
- [ ] All pre-flight gates green (`cargo fmt`, `cargo clippy`, `npm run build`).
