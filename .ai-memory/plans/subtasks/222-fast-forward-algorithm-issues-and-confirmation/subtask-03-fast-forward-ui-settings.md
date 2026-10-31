# Subtask 03: Fast-Forward Settings UI Slider & Config Synchronization

## 1. Objective
Add UI controls and synchronize config models for the minimum weekly quota percentage (default 15%):
1. **Frontend Types**:
   - Add `min_weekly_quota_percent?: number;` to `src/types/config.ts` and `src/services/instanceService/autoSwitcher.ts`.
   - Set default `min_weekly_quota_percent: 15.0` in `src/components/settings/AutoSwitcher/switcherTypes.ts`.
2. **Settings UI Component**:
   - In `src/components/settings/AutoSwitcher/ScoringAlgorithm.tsx` (or `ModelSettings.tsx`):
     - Mount a slider for Minimum Weekly Quota Floor (range 0% to 50%, step 1%, default 15%).
     - Include quick preset buttons: `15% (Default)`, `10%`, `0% (Disabled)`.
     - Connect to `onChange({ ...currentConfig, min_weekly_quota_percent: val })`.
3. **Backend Config Synchronization**:
   - In `src-tauri/src/models/config.rs`:
     - Add `#[serde(default = "default_min_weekly_quota_percent")] pub min_weekly_quota_percent: f64` to `AutoProfileSwitcherConfig`.
     - In `impl Default for AutoProfileSwitcherConfig`, set `min_weekly_quota_percent: 15.0`.
4. **Scoring parity**:
   - In `src-tauri/src/modules/auto_switcher/scoring.rs`, wire `min_weekly_quota_percent` into weekly quota floor evaluation.

## 2. Disjoint File Ownership
- `src/types/config.ts`
- `src/services/instanceService/autoSwitcher.ts`
- `src/components/settings/AutoSwitcher/switcherTypes.ts`
- `src/components/settings/AutoSwitcher/ScoringAlgorithm.tsx`
- `src-tauri/src/models/config.rs`
- `src-tauri/src/modules/auto_switcher/scoring.rs`
