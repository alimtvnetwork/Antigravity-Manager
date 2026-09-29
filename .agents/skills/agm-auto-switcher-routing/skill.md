---
name: agm-auto-switcher-routing
description: Specialized skill for managing auto-switcher model routing, candidate scoring algorithms, cooldown guards, and task recovery snapshots in Antigravity-Manager.
---

# AGM Auto-Switcher Model Routing & Watchdog Engine

This skill provides comprehensive architectural guidance for the Auto-Switcher subsystem, candidate scoring, model routing rules, and task recovery workflows in Antigravity-Manager.

---

## 1. UI Architecture & Symmetrical Card Layout

In `src/components/settings/AutoSwitcherSettings.tsx`:
- **Left Column**:
  - Master Auto-Switcher Enable Toggle.
  - Check Interval Slider (15s to 600s, default 60s).
  - Low-Quota Trigger Percentage Slider (1% to 50%, default 10%).
  - Rotation Cooldown Guard Slider (60s to 600s, default 180s) to balance heights with the right column.
- **Right Column (Primary Evaluated Model Card)**:
  - High-contrast card with `Cpu` badge, `Trigger Metric` status, and dynamic contextual trait banners:
    - **Gemini 3.8 Flash**: High-throughput tracking against rolling 5-hour quota reset windows.
    - **Claude Sonnet 4.6**: Failover triggered by deep reasoning token budget depletion.
    - **Gemini Pro**: Baseline standard chat quota headroom across profiles.
  - Task Continuity & Watchdog Panel:
    - Snapshot & auto-resume pending tasks (`has_auto_resume`).
    - 2-minute background crash watchdog (`has_crash_watchdog`).
    - Auto-focus restored window (`auto_focus_window`).
    - Recency cutoff window (5 to 60 min, default 60 min).

---

## 2. Multiplicative Candidate Scoring Formula

$$\text{Score} = S_{\text{active}} \times M_{\text{tier}} \times Q_{\text{weekly}}$$

1. **$S_{\text{active}}$ (Active State Penalty)**:
   - $1.0$ if account is idle/unbound.
   - $0.0$ if actively running in an instance.
2. **$M_{\text{tier}}$ (Subscription Tier Multiplier)**:
   - Ultra: $5.0$
   - Pro: $3.0$
   - Free / Standard: $1.0$
3. **$Q_{\text{weekly}}$ (Quota Headroom Percentage)**:
   - Evaluates weekly quota groups from `retrieveUserQuotaSummary`.
   - If period has finished (`is_period_finished == true`), sets $Q_{\text{weekly}} = 100.0$.
4. **Tie-Breaking**:
   - 50/50 randomized directional tie-breaker prevents concurrent instances from swarming the same account.

---

## 3. Pre-Activation Validation & Fallback Ladder

1. Before activating a candidate account, `AutoSwitcher` executes a live pre-activation probe via `fetch_account_quota`.
2. If the candidate returns 429, 403, or invalid grant, it is marked as degraded and the switcher demotes the candidate and evaluates the next highest score.
3. Once an account passes validation, tokens are written to `state.vscdb`, keyring is updated, and proxy accounts are reloaded.
