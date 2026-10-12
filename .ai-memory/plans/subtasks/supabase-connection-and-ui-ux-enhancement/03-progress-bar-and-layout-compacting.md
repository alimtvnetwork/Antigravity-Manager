# Subtask 03: Quota Progress Bar Terminal Green, 5-Slot Quintiles & Layout Compacting

**Subtask ID:** `subtask-03`  
**Parent Plan:** Supabase Connection & UI/UX Enhancement (`supabase-connection-and-ui-ux-enhancement`)  
**Spec Reference:** `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/02-ui-ux-progress-theming-spec.md`  
**Status:** READY FOR IMPLEMENTATION  
**Lead Worker:** Worker 02 (UI/UX, Progress Bar & Theming Author)  

---

## 1. Context & Objectives

This subtask modernizes the quota progress visualization and locks layout widths to eliminate erratic spacing:

1. **Terminal Progress Green (`#1af18d`):** Integrate the canonical `#1af18d` neon terminal green as the pinnacle healthy state (100% endpoint) across progress gradients and glow filters.
2. **Exactly 5 Slots/Nodes at 20% Intervals:** Restructure checkpoints to exactly 5 milestone slots representing 20% quintiles (`[100, 80, 60, 40, 20]` representing 80-100%, 60-80%, 40-60%, 20-40%, 0-20%).
3. **Glyph Elimination:** Remove misleading SVG checkmark/check-bolt icons from milestone nodes; replace with pure illuminated glowing circular LED apertures.
4. **Layout Compacting (`w-36`/`w-44` `ml-auto`):** Dedicate a disciplined, right-aligned compact envelope (`w-36` on mobile/compact, `w-44` on desktop) with `ml-auto` in both account cards and instance tables.

---

## 2. Target Files & Concrete Modifications

```
+-------------------------------------------------------+-----------------------------------------------------------+
| Target File Path                                      | Specific Scope of Modification                            |
+-------------------------------------------------------+-----------------------------------------------------------+
| src/styles/ui-tokens.css                              | Update --ui-quota-gradient to terminate at #1af18d;        |
|                                                       | define quintile stop tokens and restrained endpoint glows |
| src/components/accounts/QuotaProgressBar.tsx          | Standardize checkpoints to [100, 80, 60, 40, 20];         |
|                                                       | remove SVG checkmarks; render illuminated LED slots;       |
|                                                       | critical <20% percentage numeral fallback                 |
| src/components/common/WaterDrainProgressBar.tsx       | Standardize checkpoints to [100, 80, 60, 40, 20];         |
|                                                       | replace checkmarks with glowing pips; gradient alignment  |
| src/components/accounts/QuotaItem.tsx                 | Constrain progress container to w-36 sm:w-44 ml-auto;    |
|                                                       | align countdown and percentage typography                 |
| src/components/instances/table/InstanceRowCells.tsx   | Constrain QuotaCell model and weekly bars to w-36/w-44    |
|                                                       | ml-auto; stack 4H and Weekly quotas with tight spacing    |
| src/components/instances/InstanceTable.tsx            | Update table header width to w-44 text-right              |
+-------------------------------------------------------+-----------------------------------------------------------+
```

---

## 3. Detailed Implementation Directives

### 3.1 Token Definition in `src/styles/ui-tokens.css`
Update `--ui-quota-gradient` with the standardized 6-stop ramp:

```css
--ui-quota-gradient: linear-gradient(
  90deg,
  #ef6a8a 0%,
  #f49a45 20%,
  #f5bf58 40%,
  #e7d36a 60%,
  #31c9a4 80%,
  #1af18d 100%
);
--ui-quota-glow-100: 0 0 10px rgba(26, 241, 141, 0.85);
--ui-quota-glow-80:  0 0 8px rgba(49, 201, 164, 0.70);
```

### 3.2 Checkpoint Scale Array in `src/components/accounts/QuotaProgressBar.tsx`
Replace the legacy `[100, 75, 50, 25, 0]` default with:

```typescript
export const QUOTA_SLOT_INTERVALS = [100, 80, 60, 40, 20] as const;

const CHECKPOINT_SCALE_COLORS: Record<number, string> = {
    100: '#1af18d',
    80:  '#31c9a4',
    60:  '#e7d36a',
    40:  '#f5bf58',
    20:  '#ef6a8a',
};
```

### 3.3 Node Rendering Without SVG Checkmarks
In `QuotaProgressBar.tsx` and `WaterDrainProgressBar.tsx`:
- Eliminate `<svg className="..." viewBox="0 0 12 12"><path d="M2.5 6.5L4.8 8.8L9.5 3.5" /></svg>`.
- In filled state, render glowing LED apertures:
  - If `clamped < 20` and `cp <= 20`: Render `<span className="font-black font-mono text-white text-[7px] leading-none">{Math.round(clamped)}%</span>`.
  - Otherwise: Render `<span className="w-1.5 h-1.5 rounded-full bg-white/90 shadow-xs" />`.
- Apply restrained glow to the highest filled node.

### 3.4 Right-Aligned Compacting (`w-36`/`w-44` `ml-auto`)
- In `src/components/accounts/QuotaItem.tsx`: Wrap the progress bar and stats inside a container with `w-36 sm:w-44 ml-auto`.
- In `src/components/instances/table/InstanceRowCells.tsx`: Update `QuotaCell`:
  ```tsx
  export function QuotaCell({ data }: { data: RowCellData }) {
      return (
          <td className="px-2 py-1 whitespace-nowrap text-right">
              <div className="w-36 sm:w-44 ml-auto space-y-1">
                  {/* Model 4H Quota */}
                  {/* Weekly Quota */}
              </div>
          </td>
      );
  }
  ```

---

## 4. Verification & Testing Protocol

1. **Visual Slot Count:** Confirm exactly 5 milestone dots appear across all quota bars at 20%, 40%, 60%, 80%, and 100%.
2. **No Checkmarks:** Inspect DOM tree to ensure zero SVG path elements exist inside checkpoint nodes.
3. **Color Ramp Accuracy:** Verify 100% node renders in neon `#1af18d` with green halo glow.
4. **Column Alignment:** In both Account Cards and Instance Table, verify that progress bars maintain consistent width (`w-36`/`w-44`) and flush right-alignment (`ml-auto`).
5. **Critical Threshold:** Simulate 15% quota and confirm critical red slot with `{15}%` text is displayed.
