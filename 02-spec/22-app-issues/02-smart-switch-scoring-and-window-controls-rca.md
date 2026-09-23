# App Issue RCA 02: Smart Switch Scoring Defect, Missing Window Controls ACL, and Stack Trace Telemetry

> **Specification Reference:** `02-spec/22-app-issues/02-smart-switch-scoring-and-window-controls-rca.md`
> **Target Subsystem:** Fast Forward, Window Management (`NavSettings.tsx`, `TitleBar.tsx`), Error Store (`error-store.ts`, `error-modal.tsx`)
> **Severity:** High
> **Status:** Fixed
> **Date:** 2026-09-23

---

## 1. Reproduction

1. **Defect A: Fast-Forward Selected Exhausted Quota Account:**
   - In a multi-account environment with 35 accounts:
     - Account `rokixshohag1` has 21% weekly Gemini quota and 100% Claude quota.
     - Account `abidul.rasia` has 100% weekly Gemini quota and 100% Claude quota.
   - User clicks the Fast-Forward (smart switch) button in the top navbar.
   - **Observed Behavior:** The algorithm transfers/switches to `rokixshohag1` instead of `abidul.rasia`.
2. **Defect B: Window Minimize / Maximize Throws ACL Error:**
   - User clicks the Minimize (`-`) or Maximize (`+`) button on the top navbar.
   - **Observed Behavior:** The action fails and the global Error Modal opens with:
     `Command plugin:window|minimize not allowed by ACL`
     `Command plugin:window|maximize not allowed by ACL`
3. **Defect C: Error Modal Missing Stack Trace & Opaque Click Path:**
   - In the Error Modal for `plugin:window|minimize not allowed by ACL`:
     - The stack trace was empty / absent because Tauri IPC threw a string/plain object, and `error-store.ts` only read `err.stack` if `err instanceof Error`.
     - The user interaction flow displayed generic DOM tags (`span "5H" -> button "Weekly" -> path -> button "..." -> svg`) with no element IDs, accessible names, or XPath locators.

---

## 2. Cause

1. **Root Cause of Defect A (Scoring & Weekly Quota Skew):**
   - The legacy algorithm assigned up to +100,000 points solely for inactivity (`last_used == 0` or idle >= 4 hours).
   - In addition, `extractWeeklyQuotaPercent` inspected `acc.quota.models` (which stores the 4-hour / 5-hour quota percentages, where all accounts were at 100%) rather than extracting the true weekly quota buckets from `acc.quota.quota_groups`.
   - As a result, both `rokixshohag1` and `abidul.rasia` were evaluated as having 100% quota, and `rokixshohag1` won solely due to idle time or list order.
   - No pre-activation live refresh probe was executed to verify that the selected account still possessed the expected quota before transferring.
2. **Root Cause of Defect B (Tauri v2 ACL Window Restriction):**
   - In Tauri v2, window manipulation methods on `@tauri-apps/api/window` (such as `.minimize()`, `.toggleMaximize()`) invoke IPC plugin commands (`plugin:window|minimize`, `plugin:window|maximize`).
   - The repository lacked a `src-tauri/capabilities/default.json` defining `core:window:allow-minimize`, `core:window:allow-maximize`, etc.
   - Additionally, no custom Tauri backend commands existed to provide direct native window control as an unconstrained bridge.
3. **Root Cause of Defect C (Stack Trace & Telemetry Gaps):**
   - `buildCapturedError` in `error-store.ts` only extracted `rawError.stack` when `rawError instanceof Error`. Tauri errors thrown as strings had no stack trace attached.
   - `handleClickCapture` in `error-listener.ts` did not climb the DOM tree to the nearest interactive element, capturing raw inner SVG `<path>` and `<svg>` tags instead of the enclosing button with its ID, title, and XPath.

---

## 3. Fix

1. **Multi-Factor Multiplicative Scoring Algorithm:**
   - Formula: $\text{Score} = S_{\text{active}} \times M_{\text{tier}} \times Q_{\text{weekly}}$.
   - $S_{\text{active}} = 0$ if in use/active; $1$ if unused/available (zero 100k bonuses).
   - $M_{\text{tier}} = 5$ (Ultra), $3$ (Pro), $1$ (Free).
   - $Q_{\text{weekly}}$: Specifically extracted from `quota_groups` weekly buckets matching the UI's Weekly Quota view.
   - Randomized 50/50 directional tie-breaking ($A \leftrightarrow Z$) to prevent hot-spotting.
2. **Pre-Activation Live Quota Refresh Verification Loop:**
   - Before completing the switch, execute `fetch_account_quota` on the top candidate.
   - Re-score with refreshed data. If in use or if quota dropped below eligibility, demote candidate to 0 points / bottom of pool and evaluate the next best candidate.
3. **Tauri Capabilities & Custom Window Commands:**
   - Added `src-tauri/capabilities/default.json` with permissions `core:default`, `core:window:default`, `core:window:allow-minimize`, `core:window:allow-maximize`, `core:window:allow-toggle-maximize`, `core:window:allow-close`, `core:window:allow-is-maximized`, `core:window:allow-unminimize`.
   - Exported native commands: `minimize_window`, `maximize_window`, `toggle_maximize_window`, `close_window`, `is_window_maximized`.
   - Added explicit IDs (`btn-window-minimize`, `btn-window-maximize`, `btn-window-close`) and `data-xpath` attributes to window control buttons.
4. **Synthetic Stack Trace & XPath Interaction Tracking:**
   - If `err.stack` is absent, `buildCapturedError` generates a clean synthetic stack trace via `new Error().stack`.
   - `handleClickCapture` traverses to `target.closest('button, a, input, select, textarea, [role="button"], [id]')` and records `id`, `name`, and computes the full XPath locator (`//*[@id="..."]`).
   - The Error Modal Overview tab renders a dedicated Stack Trace card with 1-click copy.

---

## 4. Prevention

1. **Mathematical Isolation:** Unit tests in `src-tauri/src/modules/account.rs` and `src/services/instanceService.test.ts` assert the multiplicative scoring formula and verify `abidul.rasia` wins over `rokixshohag1`.
2. **Anonymized Testing Standard:** All test cases use synthetic identifiers (`test_user_a`, `test_user_b`), strictly avoiding real email addresses.
3. **CI/CD Quality Guard:** Capabilities manifest and window commands prevent future ACL regression.
