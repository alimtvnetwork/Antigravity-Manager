# Specification: Fast Forward Algorithm Issues and Confirmation

## User Request (Verbatim)
> # Fast Forward Algorithm Issues and Confirmation: high priority instruction, non-negotiable task
> 
> A few of the issues in your fast forward algorithm. And you need to confirm that it is working as I am saying. If the weekly quota is less than 15%, and that should be controllable from the settings where we control for the auto switch. Usually by default it would be less than 15% weekly quota. It would be considered as zero. If any account has zero credits, we don't move there. We just skip those accounts. Now, if any accounts has any users in the four hours credit, let's say less than 1%, we never switch to that account. Which accounts we never switch in fast forward? One, if that has, in four hours credit, less than 100%, if any other instance is using it, if any other machine instance is using it, that we know using email or SSH, Gitmap, or Telegram, any one of the sources. If we know that, if anyone on the account in last four hours is using that, then we skip that. Then we move to the next one, next possible best one, which has the closest time to expire and has full 100% four-hour credit. That is our best choice. In case, if we don't have any one of them, then we try to find whichever has the highest weekly and four-hour credits. It would be the same algorithm will judge it, and pro multiplier or ultra multiplier would be already there. This is the conditional logic that you should check based on switching. If you understood this, then your job is to update the logic, update the algorithm, and also have a setting section in the UI, so that we can change the slider for this. Finally, do a minor bump and make a release, and make sure that CICD is green using Gitmap EE space hyphen T.
> 
> ## task: Fast Forward Algorithm Issues and Confirmation
> ## slug: fast-forward-algorithm-issues-and-confirmation

---

## 1. Architectural Intent & Systems Reasoning
The Fast Forward (FF) algorithm orchestrates intelligent, high-velocity account rotation for developer IDE instances and daemon workers when an active account encounters quota exhaustion or requires immediate rotation. To maximize resource efficiency and eliminate collision thrashing, the algorithm must enforce strict eligibility gates:
1. **Weekly Quota Floor**: Any candidate account whose remaining weekly quota percentage falls below the configurable threshold (`fast_forward_min_weekly_quota_percent`, default `15.0%`) is treated as having zero credits and is unconditionally excluded.
2. **Four-Hour Clean Headroom Invariant**: Any candidate account that has been touched in the 4-hour window (i.e. remaining 4-hour headroom < 100.0%) or has active/recent usage detected from external fleet sources (Email alerts, SSH delegations, GitMap telemetry, Telegram notifications, or local active instance bindings) within the last 4 hours is strictly disqualified.
3. **Earliest Expiry Priority**: Among candidate accounts that satisfy the 100% 4-hour credit and clean-instance invariants, the algorithm selects the candidate whose quota reset/expiry epoch is closest, ensuring quota windows expiring soonest are consumed before reset.
4. **Graceful Degraded Fallback**: When no candidate possesses 100% 4-hour headroom, the algorithm falls back to candidate dynamic scoring based on highest combined weekly and 4-hour headroom scaled by subscription tier multipliers (Ultra ×4.0, Pro ×2.0, Free ×1.0).
5. **Configurable UI Slider**: A dedicated control mounted in `AutoSwitcher` settings exposes the weekly quota threshold slider (0% to 50%, default 15%).

---

## 2. Technical Contracts & Data Models

### 2.1 Rust Backend (`src-tauri/src/modules/auto_switcher/fast_forward_target_resolver.rs`)
```rust
pub struct FfCandidateAccount {
    pub id: String,
    pub email: String,
    pub weekly_quota_ratio: f64,        // 0.0 to 1.0 (e.g. 0.85 = 85%)
    pub rolling_4h_ratio: f64,          // 0.0 to 1.0 (1.0 = 100% full headroom)
    pub reset_epoch_seconds: u64,       // Epoch timestamp when current window resets
    pub last_used_epoch_seconds: u64,   // Epoch timestamp when last used by any source
    pub is_active_on_other_instance: bool, // Detected via SSH, Email, Telegram, GitMap, or local instances
    pub tier_multiplier: f64,           // Ultra 4.0, Pro 2.0, Free 1.0
    pub is_disabled: bool,
}

pub struct FfResolutionConfig {
    pub min_weekly_quota_ratio: f64,    // Default 0.15 (15%)
    pub require_full_4h_headroom: bool, // Default true
    pub fleet_usage_window_seconds: u64, // Default 14400 (4 hours)
}
```

---

## 3. Scope Boundaries
- **In-Scope**:
  - Update `fast_forward_target_resolver.rs` to implement the 4-tier decision ladder.
  - Implement unit test suite verifying all selection rules and edge cases.
  - Expose `fast_forward_min_weekly_quota_percent` in `AutoProfileSwitcherConfig` and UI settings.
  - Add slider in `src/components/settings/AutoSwitcher/ModelSettings.tsx` or `ScoringAlgorithm.tsx`.
  - Bump minor version via `npm run bump minor`.
  - Validate with `gitmap ee -t` / CI/CD check.
- **Excluded**:
  - Unrelated proxy protocol adapters.
  - SQLite table schema rewrites for unrelated modules.
