---
plan: 127-accounts-ui-supabase-sync-instance-compact-and-release
subtask: "01"
title: Accounts Table Grouping, Row Borders, and Quota Progress Bar VS Code Palette
domain: frontend-react
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/127-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md#31-quotaprogressbar--waterdrainprogressbar-color-overhaul
  screenshots:
    - assets/screenshots/127-accounts-styling-01.png
    - assets/screenshots/127-accounts-styling-02.png
    - assets/screenshots/127-accounts-styling-03.png
    - assets/screenshots/127-accounts-styling-04.png
target_files:
  - src/components/accounts/QuotaProgressBar.tsx
  - src/components/common/WaterDrainProgressBar.tsx
  - src/components/accounts/AccountTable.tsx
  - src/components/accounts/AccountRow.tsx
status: pending
---

# 01 — Accounts Table Grouping, Row Borders, and Quota Progress Bar VS Code Palette

## 1. Context & Motivation

The accounts view in Antigravity Manager is the primary dashboard for monitoring quota consumption and profile health. Based on user feedback and visual evidence in `assets/screenshots/127-accounts-styling-01.png` through `04.png`, several visual defects need immediate remediation:
1. **Jarring Neon Green Progress Bars**: Quota progress bars in `QuotaProgressBar.tsx` and `WaterDrainProgressBar.tsx` employ harsh neon greens (`#1af18d`, `#10b981`, `#059669`) with an over-saturated neon glow (`shadow-[0_0_8px_rgba(26,241,141,0.6)]`), clashing with the dark theme and causing visual fatigue.
2. **Missing Subtle Borders & Row Grouping**: The Accounts table rows lack clear horizontal separation, and the middle section (Gemini and Claude 4H and Weekly quotas) lacks visual grouping, making adjacent account rows and metrics difficult to distinguish.
3. **Harsh Status Badges**: Audit, disabled, and validation status badges use inconsistent, loud color schemes rather than muted, professional VS Code slate/teal/cyan aesthetics.

---

## 2. Target Files & Key Symbols

| Target File | Symbols / Locations | Scope of Modification |
|---|---|---|
| `src/components/accounts/QuotaProgressBar.tsx` | `getTrackGradient` (~L41-46)<br>`getNodeStyle` (~L65-81)<br>`getPercentColorClass` (~L48-52) | Replace neon greens with VS Code teal/cyan/sky palette. Soften milestone node glow. |
| `src/components/common/WaterDrainProgressBar.tsx` | `getTrackGradient` (~L19-24)<br>`getNodeStyle` (~L28-44) | Replace neon greens with teal/cyan/sky gradient. Soften milestone node glow. |
| `src/components/accounts/AccountTable.tsx` | Table rows `<tr>` (~L263-275, ~L1230-1280)<br>Middle section cells (~L718-775)<br>Status badges (~L558-610) | Enforce row borders `border-b border-slate-200/80 dark:border-slate-800/80`. Add middle section grouping background. Soften badges to VS Code palette. |
| `src/components/accounts/AccountRow.tsx` | Row `<tr>` (~L63-76)<br>Middle section cells (~L150-220)<br>Status badges | Align styling with `AccountTable.tsx` for standalone row usage. |

---

## 3. Concrete Implementation Steps

### 3.1 QuotaProgressBar Color Overhaul (`src/components/accounts/QuotaProgressBar.tsx`)
1. **Track Gradient**:
   Modify `getTrackGradient(pct: number)`:
   - For `pct >= 75`: return `'bg-gradient-to-r from-teal-500 via-cyan-500 to-sky-500'` (eliminating `#1af18d`, `#10b981`, `#059669`).
   - For `pct >= 50`: return `'bg-gradient-to-r from-cyan-600 via-teal-500 to-amber-400'`.
   - For `pct >= 25`: return `'bg-gradient-to-r from-amber-500 via-amber-400 to-orange-500'`.
   - For `pct < 25`: return `'bg-gradient-to-r from-amber-500 via-orange-500 to-rose-600'`.
2. **Milestone Checkpoint Nodes**:
   Modify `getNodeStyle(idx: number, isFilled: boolean)`:
   - For `idx === 0` (100% Milestone):
     Return `"bg-cyan-500 border-[1.5px] border-cyan-400 shadow-[0_0_6px_rgba(6,182,212,0.4)]"` (replacing harsh neon green `#1af18d` and `shadow-[0_0_8px_rgba(26,241,141,0.6)]`).
   - For `idx === 1` (75% Milestone):
     Return `"bg-teal-600 border-[1.5px] border-teal-500 shadow-none"` (replacing `#059669`).
   - For `idx === 2` (50% Milestone):
     Return `"bg-amber-500 border-[1.5px] border-amber-600 shadow-none"`.
   - For `idx === 3` (25% Milestone):
     Return `"bg-orange-500 border-[1.5px] border-orange-600 shadow-none"`.
   - For unfulfilled nodes (`!isFilled`):
     Retain `"bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300 dark:border-[#15334d]/60 shadow-none"`.
3. **Percentage Text Colors**:
   - `pct >= 50`: `text-teal-700 dark:text-cyan-400`
   - `pct >= 25`: `text-amber-700 dark:text-amber-400`
   - `pct < 25`: `text-rose-600 dark:text-rose-400`

### 3.2 WaterDrainProgressBar Color Overhaul (`src/components/common/WaterDrainProgressBar.tsx`)
1. **Track Gradient**:
   Modify `getTrackGradient(pct: number)`:
   - `pct >= 75`: return `'bg-gradient-to-r from-teal-500 to-cyan-500'` (replacing `#1af18d` to `#059669`).
   - `pct >= 50`: return `'bg-gradient-to-r from-cyan-600 to-amber-400'`.
   - `pct >= 25`: return `'bg-gradient-to-r from-amber-500 to-orange-500'`.
   - `pct < 25`: return `'bg-gradient-to-r from-orange-500 to-rose-600'`.
2. **Milestone Checkpoint Nodes**:
   Modify `getNodeStyle(idx: number, isFilled: boolean)`:
   - `case 0`: return `"bg-cyan-500 border-[1.5px] border-cyan-400 shadow-[0_0_6px_rgba(6,182,212,0.4)]"`.
   - `case 1`: return `"bg-teal-600 border-[1.5px] border-teal-500 shadow-none"`.
   - Retain cases 2 and 3 with clean amber/orange borders.

### 3.3 AccountTable & AccountRow Borders and Grouping (`AccountTable.tsx`, `AccountRow.tsx`)
1. **Row Borders**:
   - Verify every table row `<tr>` includes:
     `border-b border-slate-200/80 dark:border-slate-800/80`
   - Ensure clean hover transitions:
     `hover:bg-slate-50/80 dark:hover:bg-[#0f273d]/60 hover:border-l-blue-500/70`
2. **Middle Section Grouping (4H & Weekly Quota Columns)**:
   - In both `AccountTable.tsx` and `AccountRow.tsx`, give the 4H Quota and Weekly Quota cells a subtle background grouping tone:
     `px-2.5 py-1 align-middle min-w-[210px] bg-slate-50/40 dark:bg-slate-900/25`
   - Add a subtle divider border between the email column and the quota section:
     `border-l border-slate-200/50 dark:border-slate-800/50`
   - In table header (`<thead>`), ensure the Gemini/Claude filter capsule is clearly aligned with the quota section.
3. **Softened Status and Audit Badges**:
   - `DISABLED`: `px-1.5 py-0.5 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-slate-600 dark:text-slate-400 border border-slate-300 dark:border-slate-700 text-[9px] font-bold`
   - `PROXY_DISABLED`: `px-1.5 py-0.5 rounded-[5px] bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-400 border border-amber-300/40 text-[9px] font-bold`
   - `FORBIDDEN`: `px-1.5 py-0.5 rounded-[5px] bg-rose-50 dark:bg-rose-950/40 text-rose-700 dark:text-rose-400 border border-rose-300/40 text-[9px] font-bold`
   - `VALIDATION_BLOCKED`: `px-1.5 py-0.5 rounded-[5px] bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-400 border border-amber-300/40 text-[9px] font-bold`
   - `CURRENT`: `px-1.5 py-0.5 rounded-[5px] bg-blue-100 dark:bg-amber-400/15 text-blue-800 dark:text-amber-300 border border-blue-200 dark:border-amber-400/30 text-[9px] font-bold`

---

## 4. Constraints

- **Strict Frontend Boundary**: Do NOT edit any backend Rust files (`supabase_sync.rs`, `auto_switcher.rs`, `config.rs`) or instance components (`InstanceTable.tsx`, `Instances.tsx`).
- **No Git Commands**: Under no circumstances should `git add`, `git commit`, `git push`, or `git checkout` be executed.
- **Preserve Functionality**: Preserve all DND drag-and-drop sortable bindings, click handlers, priority edits, custom label modals, and i18n translation keys.

---

## 5. Out of Scope

- Modifying Instance Table columns, path truncation, or action capsules (covered in Subtask 02).
- Modifying Instances card view grid or "Rotate to Next Best" header button (covered in Subtask 02).
- Supabase repo-secrets auto-discovery and account lease locking (covered by Subagent 2).

---

## 6. Verification Steps

1. Run pre-flight build check:
   ```bash
   npm run build
   ```
2. Inspect `QuotaProgressBar.tsx` and `WaterDrainProgressBar.tsx`:
   - Search for `#1af18d`, `#10b981`, `#059669` -> Must return 0 occurrences.
   - Verify presence of `from-teal-500 via-cyan-500 to-sky-500`.
   - Verify checkpoint 0 shadow is `shadow-[0_0_6px_rgba(6,182,212,0.4)]`.
3. Inspect `AccountTable.tsx` and `AccountRow.tsx`:
   - Verify `border-b border-slate-200/80 dark:border-slate-800/80` on rows.
   - Verify middle section quota cells have grouping styling.
   - Verify all status badges use softened 5–6px radius (`rounded-[5px]`).

---

## 7. Done When

- [ ] All occurrences of `#1af18d`, `#10b981`, and `#059669` removed from `QuotaProgressBar.tsx`.
- [ ] All occurrences of `#1af18d` and `#059669` removed from `WaterDrainProgressBar.tsx`.
- [ ] Milestone checkpoint 0 uses soft cyan node styling (`bg-cyan-500 border-cyan-400 shadow-[0_0_6px_rgba(6,182,212,0.4)]`).
- [ ] `AccountTable.tsx` and `AccountRow.tsx` rows have crisp `border-b border-slate-200/80 dark:border-slate-800/80`.
- [ ] Quota columns in the middle section have visual grouping and subtle boundaries.
- [ ] Status badges (`DISABLED`, `FORBIDDEN`, `PROXY_DISABLED`, `VALIDATION_BLOCKED`) are softened to VS Code palette.
- [ ] `npm run build` succeeds with zero errors.
