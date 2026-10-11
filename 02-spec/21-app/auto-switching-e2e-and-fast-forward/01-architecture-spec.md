# Architecture Specification: Resilient Auto-Switching Algorithm, Multi-Tier Quota Caching, Proxy Sync & Crash Hardening

> **Target:** `02-spec/21-app/auto-switching-e2e-and-fast-forward/01-architecture-spec.md`  
> **Status:** APPROVED  
> **Author:** @aukgit  
> **Version:** 1.0.0  
> **Slug:** `auto-switching-e2e-and-fast-forward`  
> **Scope:** Auto-Switcher Subsystem, Candidate Scoring Inversion Fix, Strict Model Isolation, Multi-Tier Quota Caching (L1/L2), Proxy Rate-Limit Bridge, Telemetry Broadcasting, Bounded Transcript Parsing, Process Hardening  

---

## 1. Executive Summary & Subsystem Architecture

Antigravity Manager (AGM) orchestrates multi-account authentication, local and remote reverse-proxy routing, and autonomous profile rotation across isolated IDE instances. The core of this coordination is the **Auto-Switcher Subsystem** (`src-tauri/src/modules/auto_switcher/`), which monitors active account quota, scores candidate accounts, and performs automated account rotation when quotas deplete or HTTP 429 (`RESOURCE_EXHAUSTED`) errors occur.

### 1.1 High-Level Component Interaction

```
+---------------------------------------------------------------------------------------------------+
|                                      Reverse Proxy Gateway                                        |
|                          (src-tauri/src/proxy/rate_limit.rs & token_manager/)                     |
|                                                                                                   |
|   - Real-time HTTP 429 / 403 Detection                                                            |
|   - RateLimitTracker (In-memory locks & retry_after headers)                                      |
|   - LiveLimitStatus (Persisted on models in Account.live_limited_models)                          |
+------------------------------------------------+--------------------------------------------------+
                                                 | (Bidirectional Rate-Limit State Sync)
                                                 v
+---------------------------------------------------------------------------------------------------+
|                                    Auto-Switcher Daemon & Engine                                  |
|                            (src-tauri/src/modules/auto_switcher/)                                 |
|                                                                                                   |
|  +-------------------------+    +--------------------------+    +-------------------------------+ |
|  |     Daemon / Ticker     | -> |   Candidate Selection    | -> |   Scoring & Model Isolation   | |
|  |       (daemon.rs)       |    |      (selection.rs)      |    |          (scoring.rs)         | |
|  +-------------------------+    +--------------------------+    +-------------------------------+ |
|               |                               |                                 |                 |
|               v                               v                                 v                 |
|  +-------------------------+    +--------------------------+    +-------------------------------+ |
|  |   Quota State Check     |    |   Proxy Cooldown Guard   |    |    Pre-Activation Refresh     | |
|  |       (quota.rs)        |    |     (selection.rs)       |    |      (quota_fetch.rs)         | |
|  +-------------------------+    +--------------------------+    +-------------------------------+ |
+------------------------------------------------+--------------------------------------------------+
                                                 |
                                                 v
+---------------------------------------------------------------------------------------------------+
|                                 Multi-Tier Quota Cache & Storage                                  |
|                       (src-tauri/src/modules/quota/ & src-tauri/src/modules/account/)             |
|                                                                                                   |
|   - L1: In-Memory TTL Cache (60s window, proactive invalidation on reset boundary or 429/403)     |
|   - L2: Persistent Disk Cache (state.vscdb SQLite & accounts.json)                                |
+------------------------------------------------+--------------------------------------------------+
                                                 |
                                                 v
+---------------------------------------------------------------------------------------------------+
|                              Telemetry, Events & Desktop Hardening                                |
|                                                                                                   |
|   - Desktop Event Broadcast: `auto_switcher_switched` payload (email, tier, quota %, reason)     |
|   - CLI Telemetry Parity: Dual-tag logs `[AGM:P001 | GM:#1]`                                      |
|   - Bounded Transcript Parsing: repo_db/transcript.rs reverse-tail reader                         |
|   - Safe Process Inspection: Targeted sysinfo queries avoiding global thread contention          |
+---------------------------------------------------------------------------------------------------+
```

### 1.2 Module Responsibilities

| Subsystem Module | Source File | Architectural Responsibility |
|---|---|---|
| `scoring.rs` | `src-tauri/src/modules/auto_switcher/scoring.rs` | Computes candidate profile suitability scores using tier multipliers, model-isolated 4h/5h rolling quota, and cycle headroom. |
| `selection.rs` | `src-tauri/src/modules/auto_switcher/selection.rs` | Filters eligible accounts, enforces proxy rate-limit cooldown exclusions, sorts by score, and executes pre-activation Google API verification. |
| `quota.rs` | `src-tauri/src/modules/auto_switcher/quota.rs` | Evaluates model quotas, handles reset boundary timestamp calculations, and detects depletion conditions. |
| `quota_fetch.rs` | `src-tauri/src/modules/account/quota_fetch.rs` | Fetches live quotas from Google Cloud / Antigravity Cloud APIs, refreshes OAuth tokens, and coordinates L1 cache hydration. |
| `rate_limit.rs` | `src-tauri/src/proxy/rate_limit.rs` | Tracks active upstream throttling, lockout deadlines, and retry delays across models and accounts. |
| `transcript.rs` | `src-tauri/src/modules/repo_db/transcript.rs` | Inspects agent conversation transcripts to extract active prompts, tool outputs, and execution states. |
| `markdown.tsx` | `src/components/instances/PromptTreeView/markdown.tsx` | Renders inline markdown, truncated context callouts, and code blocks in the Prompt Tree View. |

---

## 2. Root Cause Analysis (RCA)

### 2.1 RCA 1: Scoring Inversion (Hours Elapsed Inversion & Floor Flattening)

#### Defect Symptom
When the auto-switcher evaluated candidate accounts, freshly reset accounts with 100% quota were consistently ranked at the bottom of the candidate pool, while accounts that had been running for several days and were closer to depletion were given higher scores and selected first.

#### Architectural Breakdown
In `src-tauri/src/modules/auto_switcher/scoring.rs`, the base weekly score was computed via `compute_weekly_hours_elapsed` and `calculate_weekly_base_score`:

```rust
// Legacy implementation in scoring.rs
pub(crate) fn compute_weekly_hours_elapsed(bucket: &QuotaBucket, now_sec: i64) -> f64 {
    const TOTAL_WEEK_HOURS: f64 = 168.0;
    let reset_ts = match parse_reset_time_to_unix(&bucket.reset_time) {
        Some(ts) => ts,
        None => return TOTAL_WEEK_HOURS,
    };
    let remaining_secs = reset_ts.saturating_sub(now_sec).max(0);
    let remaining_hours = remaining_secs as f64 / 3600.0;
    (TOTAL_WEEK_HOURS - remaining_hours).max(0.0).min(TOTAL_WEEK_HOURS)
}
```

1. **Hours Elapsed Inversion**: For a fresh account that just reset, `remaining_hours` is approximately `168.0`. Therefore, `TOTAL_WEEK_HOURS - remaining_hours` evaluates to `0.0`.
2. **Multiplicative Zeroing**: When computing `weekly_effective_score`:
   $$\text{weekly\_effective\_score} = \text{eff\_pct} \times \text{hours\_elapsed} = 100.0 \times 0.0 = 0.0$$
3. **Descending Sort Penalty**: In `src-tauri/src/modules/auto_switcher/selection.rs`, candidates were sorted in **descending** order:
   ```rust
   available_pool.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
   ```
   A freshly reset account with 100% quota received a score of `0.0`, positioning it at the very bottom of the candidate pool. Conversely, an account that had been active for 150 hours with only 20% quota remaining received a score of $20.0 \times 150.0 = 3000.0$, improperly winning the selection.
4. **Floor Flattening**: The returned score was truncated via `.floor()`, eliminating fine-grained decimal differences between candidates with identical whole numbers.

---

### 2.2 RCA 2: Cross-Model Quota Poisoning

#### Defect Symptom
An account possessed 100% healthy quota for the requested model (e.g., `gemini-2.5-pro`), yet `calculate_4h_window_quota` evaluated the account's quota as 0% or low percentage, preventing rotation to that account.

#### Architectural Breakdown
In `src-tauri/src/modules/auto_switcher/scoring.rs` and `quota.rs`:

```rust
// Legacy scoring.rs lines 240-246
// 3. Minimum across any actively consumed models (percentage < 100)
for m in &quota_data.models {
    if m.percentage < 100 {
        let pct = m.percentage as f64;
        min_pct = Some(min_pct.map_or(pct, |cur| cur.min(pct)));
    }
}
```

1. **Universal Model Leakage**: The quota query looped over all models returned in `quota_data.models`. If the account had consumed tokens on any arbitrary model (such as `gemini-1.5-flash` at 2%, an embedding model at 0%, or an internal chat tool model), step 3 took the global minimum across all consumed models.
2. **Model Context Isolation Failure**: Even when the user specifically requested routing for `gemini-2.5-pro` or standard reasoning tasks, the presence of any other depleted model poisoned the evaluation, reducing the candidate's reported quota to the lowest common denominator.
3. **Missing Wildcard Normalization**: Inability to isolate target model families (e.g., `gemini-*-pro`, `gemini-*-flash`) resulted in arbitrary non-target models dominating rotation decisions.

---

### 2.3 RCA 3: Missing In-Memory TTL Quota Caching (L1 Cache Void)

#### Defect Symptom
Frequent candidate scoring checks and background daemon ticks triggered repeated, unthrottled remote network requests to Google Cloud endpoints. Under rapid evaluation or multi-instance polling, this caused Google API rate limits (HTTP 429) on the management channel and high CPU usage.

#### Architectural Breakdown
1. **Direct Remote Pass-Through**: While `fetch_quota_with_cache` in `src-tauri/src/modules/quota/fetch_cached.rs` was intended to cache quota, it lacked an in-memory TTL mechanism. It queried remote endpoints on every single call unless project resolution alone was cached.
2. **Absence of Memory Tier**: There was no L1 cache layer in `quota_fetch.rs` with a short (e.g. 60-second) TTL. Every pre-activation probe and background status calculation performed full HTTP round-trips.
3. **Uncoordinated Invalidation**: The system lacked proactive invalidation triggers:
   - When wall-clock time passed the quota `reset_time`, the system continued reading stale disk data instead of immediately invalidating the cache.
   - When a 429 or 403 error was returned by an upstream call, the cache was not instantly purged to reflect the degraded state.

---

### 2.4 RCA 4: Lack of Proxy Rate-Limit Synchronization (Switcher & Proxy Gap)

#### Defect Symptom
The auto-switcher rotated an active IDE instance to a "healthy" candidate account, only for the reverse proxy to immediately reject requests from that account with HTTP 429 / `RESOURCE_EXHAUSTED`.

#### Architectural Breakdown
1. **Decoupled State Stores**:
   - The Reverse Proxy maintained active lockout states in `RateLimitTracker` (`src-tauri/src/proxy/rate_limit.rs`) and model lockouts in `Account.live_limited_models`.
   - The Auto-Switcher in `src-tauri/src/modules/auto_switcher/selection.rs` evaluated only static database flags:
     ```rust
     if acc.disabled || acc.proxy_disabled || acc.validation_blocked {
         continue;
     }
     ```
2. **Omission of Cooldown Inspection**: The candidate selection engine did not query `RateLimitTracker::get_remaining_wait` or check if `acc.live_limited_models` contained an active lock (`until > now_sec`) for the target model.
3. **Premature Candidate Selection**: Accounts in temporary 60s–300s proxy lockouts were treated as available, causing immediate failure loops upon rotation.

---

### 2.5 RCA 5: UI & Process Crashes: Unbounded Transcript Reading & Uncached Sysinfo

#### Defect Symptom
AGM UI experienced intermittent freezes or memory exhaustion (OOM), and Chrome/Electron IDE processes crashed during candidate inspections or prompt tree viewing.

#### Architectural Breakdown
1. **Unbounded Transcript Loading**: In `src-tauri/src/modules/repo_db/transcript.rs`, `inspect_conversation_transcript` used `std::fs::read_to_string` to load entire `transcript.jsonl` files into memory. In long-running autonomous coding sessions, these files exceed 50MB–100MB. Parsing every line with `serde_json::from_str` consumed hundreds of megabytes of RAM.
2. **Quadratic Full-Transcript Scans**: For every line marked as truncated, `get_line_from_full_transcript` opened `transcript_full.jsonl` and scanned lines from index 0, resulting in $O(N \times M)$ file rescans.
3. **Catastrophic Regex Backtracking in UI**: In `src/components/instances/PromptTreeView/markdown.tsx`, `tokenRegex` with unanchored capturing groups `(.*?)` was instantiated repeatedly inside render loops for every list item, inline code token, and blockquote, freezing the React main thread.
4. **Global Sysinfo Process Enumeration**: Process cleaners and instance checks repeatedly instantiated `sysinfo::System::new()` and called `refresh_processes(ProcessesToUpdate::All)`, enumerating every process on the host OS and causing thread contention and CPU spikes.

---

## 3. Modernized Scoring Algorithm & Mathematical Formalism

### 3.1 Scoring Formulation

To guarantee that freshly reset accounts and high-headroom accounts are prioritized, the scoring algorithm must evaluate quota percentage, remaining weekly cycle headroom, and subscription tier.

$$\text{FinalScore} = S_{\text{active}} \times M_{\text{tier}} \times \left( Q_{\text{target\_4h}} \times W_{\text{headroom}} + Q_{\text{weekly}} \times \beta \right)$$

Where:

1. **$S_{\text{active}}$ (Active State & Lease Guard)**:
   - $S_{\text{active}} = 1.0$ if the account is idle, unbound, and unleased.
   - $S_{\text{active}} = 0.0$ if the account is currently bound to a running instance, leased by another node in Supabase, or in proxy rate-limit cooldown.
2. **$M_{\text{tier}}$ (Subscription Tier Multiplier)**:
   - Configurable via `app_config.auto_profile_switcher`:
     * Ultra Tier: $M_{\text{tier}} = 5.0$
     * Pro Tier: $M_{\text{tier}} = 3.0$
     * Standard / Free: $M_{\text{tier}} = 1.0$
3. **$Q_{\text{target\_4h}}$ (Model-Isolated Rolling Quota)**:
   - Evaluates the rolling 4h/5h bucket strictly for `target_model` (0.0 to 100.0).
   - If `is_period_finished == true`, $Q_{\text{target\_4h}} = 100.0$.
4. **$W_{\text{headroom}}$ (Headroom & Freshness Weight)**:
   - Rewards accounts with long remaining headroom until reset:
     $$W_{\text{headroom}} = 1.0 + \frac{\text{remaining\_hours}}{168.0}$$
   - For an account that just reset ($\text{remaining\_hours} = 168.0$), $W_{\text{headroom}} = 2.0$.
   - For an account near reset ($\text{remaining\_hours} = 0.0$), $W_{\text{headroom}} = 1.0$.
5. **$Q_{\text{weekly}}$ & $\beta$ (Weekly Quota & Damping Factor)**:
   - $Q_{\text{weekly}}$: Macro weekly quota (0.0 to 100.0). If $< 8.0\%$, treated as $0.0$.
   - $\beta = 0.2$: Damping factor ensuring rolling 4h interactive capacity remains the primary decision driver.
6. **Tie-Breaking Rule**:
   - When candidate scores are within a 0.001 margin, tie-breaking uses case-insensitive alphabetical email ordering to maintain deterministic selection across evaluations.

```
       Candidate Scoring Curve Comparison:
  Score
    ▲
500 ┼─────────────────────────── Fresh Reset (100% Quota, 168h headroom) [Score ~ 500.0]
    │                          *
400 ┼                         *
    │                        *
300 ┼                       *  Mid-Cycle (100% Quota, 84h headroom) [Score ~ 375.0]
    │                      *
200 ┼                     *
    │                    *     Late-Cycle (100% Quota, 10h headroom) [Score ~ 265.0]
100 ┼                   *
    │                  *       Degraded / Partial Quota (50% Quota) [Score ~ 100.0]
  0 ┼─────────────────*──────── Depleted / In Cooldown [Score = 0.0]
    └────────────────────────────────────────────────────────► Account State
```

---

## 4. Model Isolation & Quota Evaluation Architecture

### 4.1 Strict Target Model Isolation

The quota evaluation engine must strictly decouple the requested model from unrelated consumed models.

```rust
pub fn calculate_model_isolated_quota(
    account: &Account,
    target_model: &str,
) -> Option<f64> {
    let quota_data = account.quota.as_ref()?;
    if quota_data.is_forbidden {
        return Some(0.0);
    }

    let target_clean = target_model.trim().to_lowercase();
    let is_flash_family = target_clean.contains("flash");
    let is_pro_family = target_clean.contains("pro");

    // 1. Direct model exact or prefix match
    for m in &quota_data.models {
        let name_lower = m.name.to_lowercase();
        let is_match = if is_flash_family {
            name_lower.contains("flash")
        } else if is_pro_family {
            name_lower.contains("pro") && !name_lower.contains("flash")
        } else {
            name_lower.contains(&target_clean)
        };

        if is_match {
            return Some(m.percentage as f64);
        }
    }

    // 2. Short-window quota groups matching target
    if let Some(ref groups) = quota_data.quota_groups {
        for g in groups {
            for b in &g.buckets {
                let win = b.window.to_lowercase();
                let bid = b.bucket_id.to_lowercase();
                let is_short_window = win.contains("5h")
                    || win.contains("4h")
                    || bid.contains("5h")
                    || bid.contains("4h");

                if is_short_window && (0.0..=1.0).contains(&b.remaining_fraction) {
                    // Match group name or bucket ID against target family
                    let matches_family = if is_flash_family {
                        bid.contains("flash") || g.group_name.to_lowercase().contains("flash")
                    } else if is_pro_family {
                        bid.contains("pro") || g.group_name.to_lowercase().contains("pro")
                    } else {
                        true
                    };

                    if matches_family {
                        return Some((b.remaining_fraction * 100.0).round());
                    }
                }
            }
        }
    }

    // 3. Fallback: Return None rather than leaking an unrelated depleted model
    None
}
```

### 4.2 Invalidation of Arbitrary Model Minimums

The legacy loop (`if m.percentage < 100 { min_pct = min_pct.min(pct); }`) is entirely eliminated. An account that consumed 99% of its text-embedding or Gemini Flash quota will not be penalized when evaluating candidate readiness for Gemini Pro reasoning workloads.

---

## 5. Multi-Tier Quota Caching Architecture (L1 / L2)

To eliminate redundant remote queries while ensuring data freshness, quota state is managed across two tiers:

```
  Quota Request: fetch_account_quota_smart(account_id, target_model)
                               │
                               ▼
            +--------------------------------------+
            |    L1: In-Memory TTL Cache Check     |
            |   (Key: account_id, TTL: 60 seconds) |
            +--------------------------------------+
                               │
                ┌──────────────┴──────────────┐
                ▼ [Hit & Valid]               ▼ [Miss or Expired]
        Return Cached Quota         +--------------------------------------+
                                    |    L2: Local Storage / Disk Check    |
                                    |        (state.vscdb / Account)       |
                                    +--------------------------------------+
                                                       │
                                        ┌──────────────┴──────────────┐
                                        ▼ [Valid & Fresh]             ▼ [Expired or Boundary Passed]
                                Update L1 & Return              +--------------------------------------+
                                                                |    Remote Google API Live Refresh    |
                                                                |       (fetch_quota_with_retry)       |
                                                                +--------------------------------------+
                                                                                   │
                                                                                   ▼
                                                                Update L2 Disk & Hydrate L1 Cache
```

### 5.1 L1 In-Memory Cache Specifications

- **Structure**: `Arc<DashMap<String, CachedQuotaRecord>>` or `Arc<RwLock<HashMap<String, CachedQuotaRecord>>>`.
- **Fields**:
  * `account_id: String`
  * `quota_data: QuotaData`
  * `cached_at_sec: i64`
  * `expires_at_sec: i64` (TTL: exactly 60 seconds)
  * `reset_time_unix: Option<i64>`
- **Proactive Invalidation Rules**:
  1. **Reset Boundary Passage**: If `now_sec >= reset_time_unix`, the entry is invalidated immediately, prompting a fresh query to harvest replenished credits.
  2. **Upstream 429 / 403 Response**: Whenever the proxy or switcher receives HTTP 429 or HTTP 403, L1 cache for that account is purged instantly.
  3. **Explicit User Invalidation**: Invoking `agm quota refresh` or clicking the GUI refresh action clears L1.

---

## 6. Switcher & Reverse Proxy Rate-Limit Bridge

### 6.1 Bidirectional State Bridge

The auto-switcher candidate selector (`src-tauri/src/modules/auto_switcher/selection.rs`) must directly query proxy rate-limit data before approving an account for rotation.

```rust
// Switcher-Proxy Rate-Limit Verification Guard
pub fn is_candidate_in_proxy_cooldown(
    acc: &Account,
    target_model: &str,
    now_sec: i64,
) -> bool {
    // 1. Check Account-level live limit status
    for (model_key, limit_status) in &acc.live_limited_models {
        if limit_status.until > now_sec {
            if model_key.eq_ignore_ascii_case(target_model)
                || model_key.is_empty()
                || target_model.is_empty()
            {
                return true;
            }
        }
    }

    // 2. Check validation block status
    if acc.validation_blocked {
        if let Some(until) = acc.validation_blocked_until {
            if until > now_sec {
                return true;
            }
        } else {
            return true;
        }
    }

    false
}
```

### 6.2 Pre-Activation Verification Pipeline

1. **Candidate Discovery**: Rank profiles by modernized score.
2. **Proxy Cooldown Filter**: Exclude accounts currently undergoing rate-limit lockout.
3. **Supabase Lease Verification**: Confirm no remote cluster node holds an active lease via `workspace_lease_manager`.
4. **Live Pre-Activation Probe**: Before changing credentials in `state.vscdb`, execute a live Google API quota query. If the response returns 429, 403, or low quota, downgrade the candidate and proceed to the next profile in the ranked list.

---

## 7. Telemetry, Desktop Event Broadcaster & CLI Parity

### 7.1 Structured Event Contract

Upon completing an account switch, the system emits an atomic payload to notify desktop UI components, logs, and remote daemons:

```json
{
  "event": "auto_switcher_switched",
  "payload": {
    "instance_id": "inst_primary_01",
    "previous_account_id": "acc_old_123",
    "previous_email": "engineer-1@example.com",
    "new_account_id": "acc_new_456",
    "new_email": "engineer-2@example.com",
    "subscription_tier": "Ultra",
    "verified_quota_percent": 100.0,
    "target_model": "gemini-2.5-pro",
    "switch_reason": "LowQuotaThreshold",
    "timestamp": 1728640000,
    "is_period_finished": false
  }
}
```

### 7.2 CLI & Daemon Logging Standards

- **CLI Output Format**: Follow the dual-sequence tag format:
  `[AGM:P001 | GM:#1] Switched instance 'inst_primary_01' to engineer-2@example.com (Tier: Ultra, Quota: 100.0%, Target: gemini-2.5-pro)`
- **Tauri Event Bridge**: Emit `app_handle.emit("auto_switcher_switched", &payload)`.

---

## 8. Chat & Chrome Crash Prevention Hardening

### 8.1 Bounded Transcript Inspection (`repo_db/transcript.rs`)

To prevent memory bloat and UI freezes during transcript inspection:

1. **Tail-Only Reverse Reading**: Instead of reading the entire file with `std::fs::read_to_string`, read lines backwards from the end of the file up to a maximum depth of 100 lines using a buffered reverse seek or `rev_lines`.
2. **Single-Pass Full Transcript Lookup**: When a truncated field is detected in `transcript.jsonl`, do not re-scan `transcript_full.jsonl` from line 0. Maintain an index or perform a single indexed lookup.
3. **Memory Ceiling**: Enforce an upper bound of 1 MB of memory consumption per inspection pass regardless of transcript size.

### 8.2 Safe Process & Sysinfo Hygiene

1. **Targeted PID Refresh**: Replace global `system.refresh_processes(ProcessesToUpdate::All)` with targeted `ProcessesToUpdate::Some(&[target_pid])`.
2. **Shared System Snapshot**: Eliminate ad-hoc `sysinfo::System::new()` instantiations; reuse a centralized, debounced process snapshot cache with a 2-second TTL.
3. **PID Protection Safety Gate**: Never kill launcher or helper processes whose PIDs match the current AGM application or non-target instances.

### 8.3 Markdown Regex Rendering Hardening (`markdown.tsx`)

1. **Static Regex Hoisting**: Move `tokenRegex` definition outside `parseInlineMarkdown` to eliminate regex re-compilation on every token.
2. **Elimination of Catastrophic Backtracking**: Rewrite capture groups with non-greedy bounded tokens to avoid backtracking spikes on large code blocks.
3. **Memoized Element Parsing**: Wrap inline markdown parsing in `useMemo` based on content hashes.

---

## 9. Verification & Quality Gates

| Verification Gate | Command | Acceptance Standard |
|---|---|---|
| Rust Formatting | `cd src-tauri && cargo fmt -- --check` | Zero formatting diffs. |
| Rust Clippy | `cd src-tauri && cargo clippy --all-targets --all-features` | Zero warnings and zero errors. |
| Auto-Switcher Tests | `cd src-tauri && cargo test auto_switcher` | 100% test pass rate for scoring, selection, and quota logic. |
| Frontend Build | `npm run build` | Clean production build with zero TypeScript or bundling errors. |
| E2E Switcher Simulation | `scripts/test-instance-e2e.ps1` | Verifies account ranking, 100% quota prioritization, and proxy rate-limit avoidance. |
