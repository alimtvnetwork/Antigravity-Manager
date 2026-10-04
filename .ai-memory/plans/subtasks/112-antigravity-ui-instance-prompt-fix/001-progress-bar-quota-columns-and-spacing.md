# Subtask 001 — Progress Bar Glow & Quota Table Two-Column Architecture

## Status: Pending
## Priority: High
## Assigned Worker: Worker 01 (Frontend Specialist — UI, Progress Bar, Themes & Cards)

---

## 1. Objectives & User Feedback Analysis

In user feedback for Plan 112:
1. **WaterDrainProgressBar Visual Noise**: Currently, every checkpoint bubble on the progress bar has a glowing drop-shadow (`shadow-[0_0_8px_rgba(26,241,141,0.6)]`), causing excessive glow and visual clutter. The glow must be strictly isolated to the **first bubble only** (`idx === 0`). The remaining bubbles retain their rounded border shape but have no drop-shadow.
2. **Color Ladder Transition**: The color progression must follow a calibrated health gradient:
   - 1st checkpoint / bubble: Green (`#1af18d`) with active glow.
   - 2nd checkpoint / bubble: Deep green (`#059669`).
   - 3rd checkpoint / bubble: Orangey yellow (`#eab308` / `#f59e0b`).
   - 4th checkpoint / bubble: Orange (`#f97316`).
   - Drain state (< 25% after last circle): Vibrant red (`#ef4444`).
3. **Account Table Two-Column Quota Separation**: Currently, 4H and Weekly quotas are crammed into a single table column (`4h / Weekly`) inside a `grid grid-cols-2`. This squashes the progress bars and creates cramped, illegible progress elements.
   - Split 4H quota and Weekly quota into **two separate, dedicated table columns** in `AccountTable.tsx` and `AccountRow.tsx`.
   - Remove redundant `"4h"` label text from the quota cell (show only model icon + progress bar + right-aligned stats).
   - Move percentage and countdown ETA into a unified right-aligned container on the right side of each cell.
   - Add the Gemini icon (`Gemini.Color`) directly in the 4H column header.
   - Reclaim unused whitespace by shrinking the "Last Used" date column from `w-[150px]` to `w-[95px]`, allowing quota progress bars to freely expand.

---

## 2. File Targets

| File Path | Description of Changes |
| :--- | :--- |
| `src/components/common/WaterDrainProgressBar.tsx` | Restrict glow effect to `idx === 0`, remove shadows on secondary bubbles, calibrate color transitions across nodes and track fill. |
| `src/components/accounts/QuotaItem.tsx` | Support optional label (suppress redundant text when empty), move percentage and ETA into a right-aligned container, expand progress bar flex width. |
| `src/components/accounts/AccountTable.tsx` | Split 4H and Weekly into two separate `<th>` and `<td>` columns, add Gemini icon in header, adjust date column width to `w-[95px]`. |
| `src/components/accounts/AccountRow.tsx` | Mirror two-column quota structure for non-sortable row fallback to maintain column parity. |

---

## 3. Exact Implementation Specifications

### 3.1 `WaterDrainProgressBar.tsx` Refactoring

#### Problem
In `src/components/common/WaterDrainProgressBar.tsx`, lines 51-55:
```tsx
isFilled
    ? "bg-[#1af18d] border-[1.5px] border-[#12b27d] shadow-[0_0_8px_rgba(26,241,141,0.6)]"
    : "bg-[rgba(18,178,125,0.2)] border border-[rgba(18,178,125,0.35)]"
```
Every filled checkpoint receives `shadow-[0_0_8px_rgba(26,241,141,0.6)]`.

#### Required Solution
1. Define checkpoint colors per index:
   - Index 0 (100% / 1st): Green `#1af18d` with glow `shadow-[0_0_8px_rgba(26,241,141,0.6)]`, border `#12b27d`.
   - Index 1 (75% / 2nd): Deep green `#059669`, border `#047857`, `shadow-none`.
   - Index 2 (50% / 3rd): Orangey yellow `#eab308`, border `#ca8a04`, `shadow-none`.
   - Index 3 (25% / 4th): Orange `#f97316`, border `#ea580c`, `shadow-none`.
2. Map track gradient according to clamped percentage:
   - `clamped >= 75`: `bg-gradient-to-r from-[#1af18d] to-[#059669]`
   - `clamped >= 50 && clamped < 75`: `bg-gradient-to-r from-[#059669] to-[#eab308]`
   - `clamped >= 25 && clamped < 50`: `bg-gradient-to-r from-[#eab308] to-[#f97316]`
   - `clamped < 25`: `bg-gradient-to-r from-[#f97316] to-[#ef4444]`
3. In `checkpoints.map((cp, idx) => ...)`:
   - Check condition: `const isFirstBubble = idx === 0;`
   - Apply glow only when `isFirstBubble && isFilled`.
   - Non-first bubbles get clean circular shape with 1.5px border and no drop-shadow.

```tsx
// Specific checkpoint styling logic
const getNodeStyle = (idx: number, isFilled: boolean) => {
    if (!isFilled) {
        return "bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300/40 dark:border-[#15334d]/60 shadow-none";
    }
    switch (idx) {
        case 0: // 1st bubble: vibrant green with active glow
            return "bg-[#1af18d] border-[1.5px] border-[#12b27d] shadow-[0_0_8px_rgba(26,241,141,0.6)]";
        case 1: // 2nd bubble: deep green, no glow
            return "bg-[#059669] border-[1.5px] border-[#047857] shadow-none";
        case 2: // 3rd bubble: orangey yellow, no glow
            return "bg-[#eab308] border-[1.5px] border-[#ca8a04] shadow-none";
        case 3: // 4th bubble: orange, no glow
            return "bg-[#f97316] border-[1.5px] border-[#ea580c] shadow-none";
        default:
            return "bg-[#ef4444] border-[1.5px] border-[#dc2626] shadow-none";
    }
};
```

---

### 3.2 `QuotaItem.tsx` Layout Optimization

#### Problem
In `src/components/accounts/QuotaItem.tsx`, `label` is hardcoded to take up `max-w-[70px]`, and `resetTime` and `percentage` are scattered in separate sub-containers, leaving minimal width for `WaterDrainProgressBar`.

#### Required Solution
1. Make `label` optional or omit when empty string `""` is supplied. When `label` is empty or omitted, do not render an empty text box.
2. Group **Percentage** and **Reset Countdown** on the right side into a unified right-aligned container:
```tsx
{/* Right-aligned Stats Container: Countdown ETA + Percentage */}
<div className="flex items-center gap-1.5 shrink-0 justify-end ml-auto">
    {resetTime ? (
        <span className={cn("flex items-center gap-0.5 font-medium transition-colors text-[9px] font-mono", getTimeColorClass(resetTime))}>
            <Clock className="w-2.5 h-2.5 shrink-0" />
            {formatTimeRemaining(resetTime)}
        </span>
    ) : null}
    <span className={cn("text-right font-bold transition-colors text-[10px] min-w-[28px]", getTextColorClass(percentage))}>
        {percentage}%
    </span>
</div>
```
3. Expand progress bar container to take up all remaining flex width:
```tsx
<div className="flex-1 min-w-[48px] flex items-center px-1">
    <WaterDrainProgressBar percentage={percentage} />
</div>
```

---

### 3.3 `AccountTable.tsx` Two-Column Quota Separation

#### Header Structure Changes
Split the existing single column:
```tsx
{/* Old: Combined 4h / Weekly Column */}
<th className="... min-w-[320px] ...">
    ... 4h / Weekly ...
</th>
```

Into two explicit columns:
```tsx
{/* Column 1: 4H Quota with Gemini Icon and Model Toggle */}
<th className="px-2 py-1 text-left rtl:text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider min-w-[190px] whitespace-nowrap">
    <div className="flex items-center justify-between gap-1 w-full">
        <button
            type="button"
            onClick={() => handleSortToggle('reset_time')}
            className={cn(
                "inline-flex items-center gap-1 hover:text-blue-600 dark:hover:text-blue-400 transition-colors uppercase font-medium",
                sortConfig.key === 'reset_time' && "text-blue-600 dark:text-blue-400 font-semibold"
            )}
            title={t('accounts.table.sort_by_reset_time', 'Sort by reset time')}
        >
            <Gemini.Color className="w-3.5 h-3.5 shrink-0" />
            <span>4H {t('accounts.table.quota', 'Quota')}</span>
            {sortConfig.key === 'reset_time' ? (
                sortConfig.direction === 'asc' ? <ArrowUp className="w-3 h-3 text-blue-600" /> : <ArrowDown className="w-3 h-3 text-blue-600" />
            ) : (
                <ArrowUpDown className="w-3 h-3 text-gray-400 opacity-60" />
            )}
        </button>

        {/* Gemini / Claude Pill Switch */}
        <div className="inline-flex items-center p-0.5 rounded-md bg-slate-200/80 dark:bg-slate-900 border border-slate-300/80 dark:border-slate-800 text-[9px] font-semibold">
            <button
                type="button"
                onClick={() => setModelFilter('gemini')}
                className={cn("px-1.5 py-0.5 rounded-[4px] transition-all cursor-pointer font-medium", modelFilter === 'gemini' ? "bg-white dark:bg-slate-800 text-cyan-600 dark:text-cyan-400 font-bold shadow-2xs" : "text-gray-500 dark:text-slate-400")}
            >
                Gemini
            </button>
            <button
                type="button"
                onClick={() => setModelFilter('claude')}
                className={cn("px-1.5 py-0.5 rounded-[4px] transition-all cursor-pointer font-medium", modelFilter === 'claude' ? "bg-white dark:bg-slate-800 text-cyan-600 dark:text-cyan-400 font-bold shadow-2xs" : "text-gray-500 dark:text-slate-400")}
            >
                Claude
            </button>
        </div>
    </div>
</th>

{/* Column 2: Weekly Quota Column */}
<th className="px-2 py-1 text-left rtl:text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider min-w-[170px] whitespace-nowrap">
    <div className="flex items-center gap-1.5">
        <Clock className="w-3.5 h-3.5 text-gray-400" />
        <span>{t('accounts.table.weekly_quota', 'Weekly Quota')}</span>
    </div>
</th>

{/* Column 3: Reclaimed Whitespace for Date Column (Shrink to 95px) */}
<th className="px-2 py-1 text-left rtl:text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider w-[95px] whitespace-nowrap">
    <button
        type="button"
        onClick={() => handleSortToggle('last_used')}
        className={cn(
            "inline-flex items-center gap-1 hover:text-blue-600 dark:hover:text-blue-400 transition-colors uppercase font-medium",
            sortConfig.key === 'last_used' && "text-blue-600 dark:text-blue-400 font-semibold"
        )}
        title={t('accounts.table.sort_by_last_used', 'Sort by last used')}
    >
        <span>{t('accounts.table.last_used_short', 'Used')}</span>
        {sortConfig.key === 'last_used' ? (
            sortConfig.direction === 'asc' ? <ArrowUp className="w-3 h-3 text-blue-600" /> : <ArrowDown className="w-3 h-3 text-blue-600" />
        ) : (
            <ArrowUpDown className="w-3 h-3 text-gray-400 opacity-60" />
        )}
    </button>
</th>
```

#### Row Content (`AccountRowContent`) Changes
Replace the single merged `<td>` with two discrete cells:
```tsx
{/* 4H Quota Cell */}
<td className="px-2 py-0.5 align-middle">
    {isDisabled || account.quota?.is_forbidden || account.validation_blocked ? (
        <ErrorOrBlockedBanner account={account} />
    ) : (
        <QuotaItem
            label=""
            percentage={fourHourModel?.percentage ?? 0}
            resetTime={fourHourModel?.resetTime}
            isProtected={fourHourModel?.isProtected}
            liveLimit={fourHourModel?.liveLimit}
            Icon={fourHourModel?.Icon || (modelFilter === 'claude' ? Claude.Color : Gemini.Color)}
        />
    )}
</td>

{/* Weekly Quota Cell */}
<td className="px-2 py-0.5 align-middle">
    {isDisabled || account.quota?.is_forbidden || account.validation_blocked ? (
        <span className="text-[10px] text-gray-400 italic">--</span>
    ) : (
        <QuotaItem
            label=""
            percentage={weeklyCell.percentage}
            resetTime={weeklyCell.resetTime}
            Icon={modelFilter === 'claude' ? Claude.Color : Gemini.Color}
        />
    )}
</td>

{/* Last Used Date Cell (Shrunk to 95px, compact representation) */}
<td className="px-2 py-0.5 align-middle whitespace-nowrap w-[95px]">
    <span className="text-[10px] font-medium text-gray-600 dark:text-gray-400 font-mono" title={formatDateTime(account.last_used)}>
        {formatDateTimeShort(account.last_used)}
    </span>
</td>
```

---

## 4. Acceptance Criteria & Verification

1. **Glow Restriction**:
   - Inspect `WaterDrainProgressBar`.
   - Verify that checkpoint bubble 0 (`idx === 0`) has a distinct green glow drop-shadow.
   - Verify that checkpoint bubbles 1, 2, 3 have NO drop-shadow (`shadow-none`) and crisp 1.5px borders.
2. **Color Scale Progression**:
   - Set mock percentages: 100%, 70%, 45%, 15%.
   - Verify bubble and bar color transitions: `#1af18d` (1st green) -> `#059669` (deep green) -> `#eab308` (orangey yellow) -> `#f97316` (orange) -> `#ef4444` (red below 25%).
3. **Table Column Separation**:
   - Verify the Accounts table headers display distinct `4H Quota` and `Weekly Quota` columns.
   - Verify the Gemini brand icon appears in the 4H Quota column header.
   - Verify the redundant text string `"4h"` is absent from the table cells, replaced by model icon and progress bar.
   - Verify percentage and reset countdown appear grouped on the right edge of each quota cell.
4. **Whitespace Reclaim**:
   - Verify the "Last Used" date column is condensed to `~95px`.
   - Verify the quota columns expand smoothly across desktop and laptop viewport widths.
