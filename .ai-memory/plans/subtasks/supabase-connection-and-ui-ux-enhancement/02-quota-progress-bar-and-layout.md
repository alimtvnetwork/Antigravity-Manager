---
plan: supabase-connection-and-ui-ux-enhancement
subtask: "02"
title: Quota Progress Bar Modernization, 5-Slot Checkpoint Grid & Layout Whitespace Compaction
domain: frontend/ui/quota
target_files:
  - src/components/accounts/QuotaProgressBar.tsx
  - src/components/common/WaterDrainProgressBar.tsx
  - src/components/accounts/AccountTable/AccountRowContent.tsx
  - src/pages/instances/InstanceCard.tsx
  - src/styles/ui-tokens.css
status: pending
---

# 02 — Quota Progress Bar Modernization, 5-Slot Checkpoint Grid & Layout Whitespace Compaction

## 1. Objective

Modernize the quota telemetry visualization across account tables and instance cards by implementing the canonical `#1af18d` green color token, an equidistant 5-slot checkpoint grid, terminal edge glowing blur effects, and eliminating trailing right-hand whitespace gaps:

1. **Green Color Token Standard**: Enforce `#1af18d` as the strict 100% capacity color token across `ui-tokens.css` and component scale maps, replacing legacy `#31c9a4`.
2. **Equidistant 5-Slot Checkpoints**: Standardize the milestone dots to exactly 5 slots at 20% intervals: `[100, 80, 60, 40, 20]`. Clamp input arrays to at most 5 nodes.
3. **Checkmark SVG Bolt Elimination**: Remove all checkmark SVG glyphs (`M2.5 6.5L4.8...`) from checkpoint nodes, replacing them with high-precision luminous circular nodes or critical percentage typography (`Math.round(clamped)%`) when `<25%`.
4. **Terminal 100% Glowing Blur Halo**: Render a vibrant glowing blur filter (`shadow-[0_0_12px_rgba(26,241,141,0.65)]` and pseudo-element radial aura) at the right threshold when quota capacity is $\ge 95\%$.
5. **Right-Hand Layout Densification**: Eliminate awkward 18-32px trailing whitespace gaps in `AccountRowContent.tsx` table cells and `InstanceCard.tsx` model containers by restructuring flex layouts and right-aligning countdown clocks and percentage figures.

---

## 2. Technical Scope & Target Files

```
+-------------------------------------------------+----------------------------------------------------+
| File Path                                       | Key Scope & Functional Changes                     |
+-------------------------------------------------+----------------------------------------------------+
| src/styles/ui-tokens.css                        | Update --ui-quota-gradient to terminate on #1af18d;|
|                                                 | update --ui-success and glow tokens.               |
| src/components/accounts/QuotaProgressBar.tsx     | Enforce [100, 80, 60, 40, 20] checkpoints; remove  |
|                                                 | SVG checkmark bolts; add >=95% right glow;         |
|                                                 | compact flex layout to eliminate right gap.        |
| src/components/common/WaterDrainProgressBar.tsx | Match 5-slot [100, 80, 60, 40, 20] checkpoints,    |
|                                                 | token #1af18d, and remove checkmark icons.         |
| src/components/accounts/AccountTable/           | Right-align 4H and Weekly quota bars in cells;     |
| AccountRowContent.tsx                           | compact cell padding to min-w-[195px].             |
| src/pages/instances/InstanceCard.tsx           | Expand progress bars to fill interior card width;  |
|                                                 | right-align quota metrics; compact margins.        |
+-------------------------------------------------+----------------------------------------------------+
```

---

## 3. Implementation Details

### 3.1 Token System Modernization (`src/styles/ui-tokens.css`)

Update `--ui-success` and the spatial `--ui-quota-gradient` scale:
```css
/* In :root */
--ui-success: #1af18d; /* Canonical high-luminance neon mint green */
--ui-success-glow: rgb(26 241 141 / 0.45); /* Radiant terminal glow */

/* 5-stop spatial gradient */
--ui-quota-gradient: linear-gradient(
  90deg,
  #ef6a8a 0%,    /* Depleted / critical */
  #f49a45 25%,   /* Low warning */
  #f5bf58 50%,   /* Mid capacity */
  #70ddb0 75%,   /* Healthy */
  #1af18d 100%   /* Full / 100% capacity */
);
```

### 3.2 Checkpoint Nodes & Glow Implementation (`src/components/accounts/QuotaProgressBar.tsx`)

#### 1. Scale Color Mapping & Default Checkpoints:
```tsx
export const CHECKPOINT_SCALE_COLORS: Record<number, string> = {
    100: '#1af18d',
    80: '#70ddb0',
    60: '#e7d36a',
    40: '#f49a45',
    20: '#ef6a8a',
};

const DEFAULT_CHECKPOINTS = [100, 80, 60, 40, 20];
```

#### 2. Array Clamp & Sorted Normalization:
```tsx
const activeCheckpoints = (checkpoints && checkpoints.length > 0 ? checkpoints : DEFAULT_CHECKPOINTS).slice(0, 5);
const sortedCheckpoints = [...activeCheckpoints].sort((a, b) => b - a);
```

#### 3. Terminal Glow Filter:
```tsx
const isAtCapacity = clamped >= 95;
```
Wrap the progress bar track with a container rendering the glowing blur halo:
```tsx
<div className="relative flex-1 min-w-[65px] flex items-center">
    {/* Terminal Right 100% Glowing Blur Filter */}
    {isAtCapacity && (
        <div
            className="absolute right-0 top-1/2 -translate-y-1/2 w-4 h-4 rounded-full bg-[#1af18d] blur-[6px] opacity-70 pointer-events-none z-0"
            aria-hidden="true"
        />
    )}

    <div
        className={cn(
            "relative w-full rounded-full overflow-hidden bg-slate-200 dark:bg-[var(--ui-quota-track)] border border-slate-300 dark:border-[var(--ui-quota-track-border)]",
            heightClassName
        )}
        role="progressbar"
        aria-valuenow={clamped}
    >
        ...
    </div>
```

#### 4. Clean Dot Rendering (Removing SVG Checkmark Bolt):
```tsx
{showCheckpoints && sortedCheckpoints.map((cp) => {
    const isFilled = clamped >= cp;
    const leftPos = cp >= 100 ? '100%' : cp <= 0 ? '0%' : `${cp}%`;
    const transform = cp >= 100 ? 'translate(-100%, -50%)' : cp <= 0 ? 'translate(0, -50%)' : 'translate(-50%, -50%)';
    const isCritical = clamped < 25;
    const scaleColor = CHECKPOINT_SCALE_COLORS[cp] ?? '#1af18d';

    return (
        <div
            key={cp}
            className={cn(
                "absolute top-1/2 rounded-full flex items-center justify-center transition-all duration-300 z-10 pointer-events-none border-[1.5px]",
                compact ? "w-2.5 h-2.5" : "w-3 h-3",
                isFilled
                    ? (cp === 100 && isAtCapacity
                        ? 'shadow-[0_0_8px_rgba(26,241,141,0.8)] border-[#1af18d]'
                        : 'shadow-none')
                    : 'bg-slate-200/50 dark:bg-[var(--ui-surface-2)] border-slate-300 dark:border-[var(--ui-border-subtle)] shadow-none'
            )}
            style={{
                left: leftPos,
                transform,
                ...(isFilled ? { backgroundColor: scaleColor, borderColor: scaleColor } : {}),
            }}
            title={`Checkpoint ${cp}%`}
        >
            {isFilled ? (
                isCritical && cp <= 20 ? (
                    <span className={cn("font-black font-mono text-white leading-none tracking-tighter select-none", compact ? "text-[6px]" : "text-[7px]")}>
                        {Math.round(clamped)}%
                    </span>
                ) : (
                    // Clean dot center instead of SVG checkmark bolt
                    <span className="w-1 h-1 rounded-full bg-white/90" />
                )
            ) : (
                <span className="w-0.5 h-0.5 rounded-full bg-slate-400/40 dark:bg-white/20" />
            )}
        </div>
    );
})}
```

### 3.3 Layout Compaction & Trailing Space Elimination

#### 1. In `QuotaProgressBar.tsx`:
Replace `w-[84%] max-w-[84%]` and `w-[16%] max-w-[16%]` with full fluid width and tightly budgeted metrics:
```tsx
// Outer container:
<div className={cn("w-full flex items-center justify-between", compact ? "gap-1.5" : "gap-2", className)}>
    ...
    {/* Metrics column on right: fixed compact width, right-aligned */}
    <div className={cn("flex flex-col items-end shrink-0 min-w-[46px]", compact ? "leading-tight" : "leading-snug")}>
        ...
    </div>
</div>
```

#### 2. In `AccountRowContent.tsx`:
Ensure table cells right-align the quota container and utilize available width efficiently:
```tsx
{/* 4-Hour Quota Cell */}
<td className="px-2 py-0.5 align-middle min-w-[195px] max-w-[230px] bg-slate-50/50 dark:bg-slate-900/40 border-l border-slate-300 dark:border-[#40285e] border-b border-slate-200/90 dark:border-slate-800/90">
    <div className="w-full flex items-center justify-end">
        <QuotaProgressBar
            compact
            percentage={fourHourModel?.percentage ?? 0}
            resetTime={fourHourModel?.resetTime}
            isProtected={fourHourModel?.isProtected}
            liveLimit={fourHourModel?.liveLimit}
            checkpoints={[100, 80, 60, 40, 20]}
        />
    </div>
</td>

{/* Weekly Quota Cell */}
<td className="px-2 py-0.5 align-middle min-w-[195px] max-w-[230px] bg-slate-50/50 dark:bg-slate-900/40 border-r border-slate-300 dark:border-[#40285e] border-b border-slate-200/90 dark:border-slate-800/90">
    <div className="w-full flex items-center justify-end">
        <QuotaProgressBar
            compact
            isWeekly
            percentage={weeklyCell.percentage}
            resetTime={weeklyCell.resetTime}
            checkpoints={[100, 80, 60, 40, 20]}
        />
    </div>
</td>
```

#### 3. In `InstanceCard.tsx`:
Ensure Gemini model quota and Weekly quota rows expand across 100% of the interior card width with zero dead right-hand whitespace:
```tsx
<div className="space-y-1">
    <div className="flex items-center justify-between text-xs">
        <div className="flex items-center gap-1.5 min-w-0">
            <Gemini.Color className="w-3.5 h-3.5 shrink-0" />
            <span className="font-semibold text-gray-800 dark:text-gray-200 text-xs truncate">
                {geminiModel.display_name || geminiModel.name || 'Gemini 3.1 Pro'} (4H)
            </span>
        </div>
    </div>
    <QuotaProgressBar
        percentage={geminiModel.percentage}
        resetTime={geminiModel.reset_time}
        checkpoints={[100, 80, 60, 40, 20]}
    />
</div>
```

---

## 4. Verification & Testing Checklist

- [ ] Inspect `src/styles/ui-tokens.css`: verify `--ui-quota-gradient` ends at `#1af18d` and `--ui-success` is `#1af18d`.
- [ ] Inspect `QuotaProgressBar.tsx`: verify `DEFAULT_CHECKPOINTS` equals `[100, 80, 60, 40, 20]`.
- [ ] Check checkpoint nodes: confirm no `<svg><path d="M2.5 6.5L4.8...` exists inside checkpoint nodes.
- [ ] Verify right glow: set percentage to 100% and verify the glowing blur filter renders at the right threshold.
- [ ] Verify table layout: inspect Accounts table in browser; verify progress bar expands smoothly and metrics align to the right border without a 20px+ dead gap.
- [ ] Verify card layout: inspect Instance Cards; verify quota bars fill container width evenly.
