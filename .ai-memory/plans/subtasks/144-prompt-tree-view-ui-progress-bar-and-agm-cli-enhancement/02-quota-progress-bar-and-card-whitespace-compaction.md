# Subtask 02: Quota Progress Bar Overhaul & Card Whitespace Compaction

**Slug:** `144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement`  
**File:** `.ai-memory/plans/subtasks/144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement/02-quota-progress-bar-and-card-whitespace-compaction.md`  
**Target Components:**
- `src/components/accounts/QuotaProgressBar.tsx`
- `src/components/common/WaterDrainProgressBar.tsx`
- `src/components/accounts/QuotaItem.tsx`
- `src/components/accounts/AccountCard.tsx`
- `src/components/instances/InstanceTable.tsx`  
**Related Specs:** `02-spec/21-app/144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement/01-architecture-spec.md`  
**Status:** READY FOR EXECUTION  
**Lead Author:** Antigravity Architect (Author 01)  

---

## 1. Objective

Modernize the quota telemetry visualization and tighten layout density across account cards and instance tables. Specifically:
1. Cap the default milestone checkpoint nodes at $\le 5$ balls (`[100, 75, 50, 25, 0]`) to eliminate visual crowding.
2. Update the color track gradient to anchor on a rich dark-red left edge (`from-[#520808] via-rose-600 via-amber-400 via-emerald-400 to-[#1af18d]`) with white glass accents.
3. Replace misleading SVG checkmarks inside checkpoint balls with high-contrast numerical percentage typography (`{percentage}%`) when quota drops below 25%.
4. Compact padding across `AccountCard.tsx` (`p-3` -> `p-2 sm:p-2.5`) and table rows in `InstanceTable.tsx`, and enforce custom `scrollbar-thin` tokens across all scrollable containers.

---

## 2. Target Files & Key Modifications

```
+-------------------------------------------------+----------------------------------------------------+
| File Path                                       | Key Changes                                        |
+-------------------------------------------------+----------------------------------------------------+
| src/components/accounts/QuotaProgressBar.tsx     | Checkpoints capped at 5; deep dark-red gradient;   |
|                                                 | <25% numerical percentage typography in balls.     |
| src/components/common/WaterDrainProgressBar.tsx | Standardize checkpoints to [100, 75, 50, 25, 0];   |
|                                                 | update gradient and <25% numerical typography.     |
| src/components/accounts/QuotaItem.tsx            | Tighten padding, align contrast tokens.            |
| src/components/accounts/AccountCard.tsx          | Compact card padding to p-2.5; scrollbar-thin.     |
| src/components/instances/InstanceTable.tsx      | Tighten row vertical padding to py-1; scrollbars.  |
+-------------------------------------------------+----------------------------------------------------+
```

---

## 3. Detailed Component Specifications

### 3.1 `src/components/accounts/QuotaProgressBar.tsx`

#### 1. Checkpoint Balls Cap ($\le 5$ Balls):
Replace the legacy 11-ball array with the 5-milestone array:
```tsx
// Before (Line 30):
checkpoints = [100, 90, 80, 70, 60, 50, 40, 30, 20, 10, 0],

// After (Capped at <= 5 milestone checkpoints):
checkpoints = [100, 75, 50, 25, 0],
```
Enforce an array clamp guard inside the component body:
```tsx
const activeCheckpoints = (checkpoints && checkpoints.length > 0 ? checkpoints : [100, 75, 50, 25, 0]).slice(0, 5);
const sortedCheckpoints = [...activeCheckpoints].sort((a, b) => b - a);
```

#### 2. Deep Dark-Red Track Gradient:
Replace the light red gradient with the deep dark-red anchor:
```tsx
// Before:
const getTrackGradient = (_pct?: number) => {
    return 'bg-gradient-to-r from-rose-500 via-amber-400 via-emerald-400 to-[#1af18d]';
};

// After:
const getTrackGradient = (_pct?: number) => {
    return 'bg-gradient-to-r from-[#520808] via-rose-600 via-amber-400 via-emerald-400 to-[#1af18d]';
};
```

#### 3. Critical Threshold (< 25%) Typography in Milestone Balls:
Inside the checkpoint node rendering map (lines 128–156), evaluate whether the quota is in the critical zone (`< 25%`). If `clamped < 25`, render numerical text instead of the SVG checkmark:

```tsx
{showCheckpoints && sortedCheckpoints.map((cp) => {
    const isFilled = clamped >= cp;
    const leftPos = cp >= 100 ? '100%' : cp <= 0 ? '0%' : `${cp}%`;
    const transform = cp >= 100 ? 'translate(-100%, -50%)' : cp <= 0 ? 'translate(0, -50%)' : 'translate(-50%, -50%)';
    const isCritical = clamped < 25;

    return (
        <div
            key={cp}
            className={cn(
                "absolute top-1/2 w-3.5 h-3.5 rounded-full flex items-center justify-center transition-all duration-300 z-10 pointer-events-none",
                getNodeStyle(cp, isFilled)
            )}
            style={{ left: leftPos, transform }}
            title={`Checkpoint ${cp}%`}
        >
            {isFilled ? (
                isCritical && cp <= 25 ? (
                    <span className="text-[7.5px] font-black font-mono text-white leading-none tracking-tighter select-none">
                        {Math.round(clamped)}%
                    </span>
                ) : (
                    <svg
                        className="w-2 h-2 fill-none stroke-current text-white stroke-[2.5]"
                        viewBox="0 0 12 12"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    >
                        <path d="M2.5 6.5L4.8 8.8L9.5 3.5" />
                    </svg>
                )
            ) : (
                <span className="w-1 h-1 rounded-full bg-slate-400/40 dark:bg-white/20" />
            )}
        </div>
    );
})}
```

---

### 3.2 `src/components/common/WaterDrainProgressBar.tsx`

Align `WaterDrainProgressBar` with identical checkpoint and gradient constraints:
1. Default `checkpoints` set to `[100, 75, 50, 25, 0]`.
2. Track gradient updated to:
   ```tsx
   const getTrackGradient = (pct: number) => {
       if (pct >= 75) return 'bg-gradient-to-r from-emerald-400 to-[#1af18d]';
       if (pct >= 50) return 'bg-gradient-to-r from-emerald-500 to-[#1af18d]';
       if (pct >= 25) return 'bg-gradient-to-r from-amber-400 via-amber-500 to-orange-500';
       return 'bg-gradient-to-r from-[#520808] via-rose-600 to-rose-700';
   };
   ```
3. Numerical percentage typography for balls when `clamped < 25%`.

---

### 3.3 `src/components/accounts/QuotaItem.tsx`

1. Container height and padding compaction:
   - Reduce height from `h-[22px]` to `h-[20px]`.
   - Reduce horizontal padding from `px-1.5` to `px-1.2`.
2. Ensure `WaterDrainProgressBar` child component dynamically inherits the 5-ball capped layout without horizontal overflow.

---

### 3.4 `src/components/accounts/AccountCard.tsx`

1. **Card Container Padding Compaction:**
   In line 180:
   ```tsx
   // Before:
   "flex flex-col p-3 rounded-xl border border-slate-200/80 dark:border-slate-800/80 border-l-2 transition-all duration-200 bg-white dark:bg-[#0c2438]",

   // After:
   "flex flex-col p-2 sm:p-2.5 rounded-xl border border-slate-200/80 dark:border-slate-800/80 border-l-2 transition-all duration-200 bg-white dark:bg-[#0c2438]",
   ```
2. **Header Margins:**
   - Reduce header margin from `mb-2` to `mb-1.5`.
3. **Scrollbar Standardization:**
   In line 285 (`{/* Quota display */}`):
   ```tsx
   // Before:
   <div className="flex-1 px-2 mb-2 overflow-y-auto scrollbar-none">

   // After:
   <div className="flex-1 px-1.5 mb-1.5 overflow-y-auto scrollbar-thin scrollbar-thumb-slate-200 dark:scrollbar-thumb-[#15334d]/60 scrollbar-track-transparent">
   ```

---

### 3.5 `src/components/instances/InstanceTable.tsx`

1. **Table Cell Padding Compaction:**
   In table row cells (e.g., line 290):
   ```tsx
   // Before:
   <td className="px-2 py-1.5 whitespace-nowrap min-w-[160px] max-w-[200px]">

   // After:
   <td className="px-2 py-1 whitespace-nowrap min-w-[150px] max-w-[190px]">
   ```
2. **Quota Progress Bar Container:**
   - Optimize spacing between Primary Model (4H) and Weekly Quota rows from `space-y-1.5` to `space-y-1`.
3. **Scrollbar Styling:**
   - Ensure the outer table container applies `scrollbar-thin scrollbar-thumb-slate-300 dark:scrollbar-thumb-[#15334d]`.

---

## 4. Acceptance Criteria & Quality Gates

1. **Milestone Ball Cap:** Verified that no quota progress bar renders more than 5 milestone checkpoint balls.
2. **Deep Dark-Red Anchor:** Progress bar gradient distinctly displays `#520808` dark red at 0–15% quota.
3. **Critical Zone Typography:** When quota is under 25%, checkpoint balls display the numerical percentage (e.g., `14%`) with no checkmark SVG.
4. **Card Padding Density:** Account cards render with `p-2.5` padding, creating a cleaner, more compact dashboard layout.
5. **Scrollbar Thinning:** No thick default OS scrollbars appear inside cards or table containers.
