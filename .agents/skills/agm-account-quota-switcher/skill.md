---
name: agm-account-quota-switcher
description: Specialized skill for managing account authentication, Google OAuth token refresh, 5h rolling vs 7-day weekly quota calculation, candidate scoring, and auto-profile switching in Antigravity-Manager.
---

# AGM Account, Quota Engine & Auto-Switcher Architecture

This skill provides comprehensive architectural guidance, calculation rules, scoring algorithms, and maintenance procedures for the Account Management, Quota Scraping, and Intelligent Auto-Profile Switcher subsystems in Antigravity-Manager.

---

## 1. Subsystem Architecture Overview

The account and quota lifecycle coordinates four key modules:

```
+-------------------------------------------------------------------------+
|                           state.vscdb / SQLite                          |
|         ItemTable: antigravityUnifiedStateSync.oauthToken (Protobuf)    |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                  src-tauri/src/modules/account.rs                       |
|  - Token extraction, decryption & persistence                           |
|  - Active account state & multi-IDE target paths (VS Code, Cursor, IDE) |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                   src-tauri/src/modules/quota.rs                        |
|  - Google Cloud / Antigravity Cloud Quota Scraping                      |
|  - 5-Hour Rolling Bucket vs 7-Day Weekly Reset Window                   |
|  - Subscription Tier Resolution (Ultra / Pro / Free)                    |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                src-tauri/src/modules/auto_switcher.rs                   |
|  - Multiplicative Candidate Scoring (Tier Multiplier * Quota)           |
|  - Quota Exhaustion Detection (429 / RESOURCE_EXHAUSTED)                |
|  - Instance-Protected Smart Rotation & Workspace Lease Verification     |
|  - Task Recovery Snapshotting & Recent Prompt Auto-Resume (<1h)         |
+-------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/modules/account.rs` | Account persistence, token refresh cycles, credential extraction from `state.vscdb`, active account pointer. |
| `src-tauri/src/modules/quota.rs` | Remote quota fetching (`fetch_quota_with_cache`), quota group parsing, 5h rolling window vs weekly quota evaluation, reset time formatting. |
| `src-tauri/src/modules/auto_switcher.rs` | Auto-switching daemon, candidate scoring algorithm, threshold comparison, process recycling, task recovery snapshots. |
| `src-tauri/src/modules/oauth.rs` & `oauth_server.rs` | Google OAuth 2.0 PKCE flow, local loopback redirect server, authorization code exchange. |
| `src-tauri/src/commands/account.rs` & `commands/mod.rs` | Registered Tauri IPC commands for account listing, deletion, quota refresh, and manual switching. |
| `src/stores/useAccountStore.ts` & `src/stores/useInstanceStore.ts` | Zustand stores for reactive account lists, quota progress bars, and switcher status. |
| `src/components/settings/AutoSwitcherSettings.tsx` | UI configuration panel for auto-switcher thresholds, model routing priorities, cooldown intervals, and fast-forward shortcut. |
| `src/hooks/useFastForwardShortcut.ts` | Global hotkey listener (`Ctrl+Alt+F` / `Cmd+Opt+F`) for instant manual account rotation. |

---

## 3. Quota Calculation Rules (5h Rolling Bucket vs Weekly Reset)

In `src-tauri/src/modules/quota.rs`, Antigravity returns quota groups with multiple bucket windows:
1. **5-Hour Rolling Bucket (`h`)**:
   - Represents the short-term interactive rate limit bucket.
   - Refills continuously as the 5-hour rolling window advances.
2. **7-Day Weekly Reset Window (`w`)**:
   - Represents the macro weekly allocation based on subscription tier.

### Rule for Active Quota Resolution:
```rust
// If weekly quota is exhausted (<= 0.001), model is strictly limited to 0%, use weekly reset time.
if w.remaining_fraction <= 0.001 {
    Some(w)
} else {
    // When weekly quota is healthy, always prioritize the 5h bucket to accurately reflect rolling quota.
    Some(h)
}
```

---

## 4. Multiplicative Candidate Scoring Algorithm

When selecting the next best candidate profile for auto-switching or manual rotation (`score_candidate_account` in `auto_switcher.rs`):

$$\text{Final Score} = \text{Active Factor} \times \text{Tier Multiplier} \times \text{Effective Quota Percent}$$

1. **Tier Multipliers**:
   - **Ultra**: `5.0`
   - **Pro**: `3.0`
   - **Free / Standard**: `1.0`
2. **Weekly / Rolling Quota Percent**:
   - Extracted from model quota buckets or minimum bottleneck across quota groups.
3. **Period Finished Override**:
   - If the account's reset boundary has passed (`is_period_finished == true`), the provider refreshes credits upon next query; quota is evaluated as `100.0%`.
4. **Disqualification Criteria**:
   - Account is `disabled`.
   - Account is `validation_blocked` (e.g. 403 Forbidden).
   - Account is actively leased by another IDE instance (`workspace_lease_manager::is_account_leased_by_other`).
   - Account is bound to the currently running instance target.

---

## 5. Instance-Protected Smart Rotation

When rotating accounts (via manual button, auto-switcher daemon, or fast-forward shortcut):
- **Never disrupt active instances**: Never steal an account bound to an actively running IDE process.
- **Snapshot preservation**: Prior to process termination, capture `TaskRecoverySnapshot` containing the active project path and prompt context.
- **Auto-resume**: Upon switching to the new profile, inspect recent prompts within the `<1 hour` window and automatically resume context in the new sandbox.
