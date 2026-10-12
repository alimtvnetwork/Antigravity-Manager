# UI/UX Progress Bar, Theming & Layout Compacting Specification

**Document ID:** `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/02-ui-ux-progress-theming-spec.md`  
**Parent Epic:** Supabase Connection & UI/UX Enhancement  
**Status:** DRAFT / APPROVED FOR IMPLEMENTATION  
**Target Version:** v4.191.0+  
**Target Components:**
- `src/components/accounts/QuotaProgressBar.tsx`
- `src/components/common/WaterDrainProgressBar.tsx`
- `src/components/accounts/QuotaItem.tsx`
- `src/components/accounts/AccountCard.tsx`
- `src/components/accounts/AccountCard/AccountCardHeader.tsx`
- `src/components/accounts/AccountCard/AccountCardQuota.tsx`
- `src/components/accounts/AccountRow.tsx`
- `src/components/accounts/AccountTable.tsx`
- `src/components/instances/InstanceTable.tsx`
- `src/components/instances/table/InstanceRowCells.tsx`
- `src/styles/ui-tokens.css`
- `src/utils/maskEmail.ts`

---

## 1. Executive Summary & Design Rationale

As Antigravity-Manager has scaled to support multi-account quota tracking, weekly quota constraints, and multi-instance IDE sandboxes, several UI/UX presentation friction points have emerged:

1. **Quota Visualization Inconsistencies:** The progress indicators employ mixed color models and misleading SVG checkmark/check-bolt glyphs inside quota milestone nodes. For a quota gauge, a checkmark incorrectly communicates task completion rather than consumed/remaining capacity.
2. **5-Slot 20% Interval Standard:** Existing components mix 11-node, 5-node, and variable-node arrays. We standardize exactly 5 milestone slots/nodes mapped to 20% quintile intervals (`0-20%`, `20-40%`, `40-60%,` `60-80%`, `80-100%`) with luminous glowing endpoints.
3. **Progress Green (`#1af18d`):** The peak healthy quota state must anchor to the canonical terminal neon green (`#1af18d`), blending backwards through emerald, amber, and deep crimson.
4. **Layout Sprawl & Whitespace Compaction:** Quota bars and instance tables currently consume variable horizontal widths that cause misalignment across screen sizes. Quota telemetry columns must be firmly compacted to the right-hand side (`w-36`/`w-44` `ml-auto`).
5. **Theming & Color Contrast Deficiencies:** Under the Royal Violet theme, secondary labels and metadata frequently render as dark purple or muted violet on dark purple card backgrounds (`#171025`, `#211631`), failing WCAG AA (4.5:1) accessibility contrast. The interface must mandate deep dark surfaces with crisp white, cream, and warm yellow typography.
6. **Privacy Masking:** Raw account emails displayed across cards and tables pose data leakage hazards during live screen shares, screencasts, and public demonstrations. The `maskEmail` utility must be consistently integrated with instant hover disclosure.
7. **Interactive Micro-Elevation:** Desktop cards and interactive capsules must exhibit tactile CSS3 hover micro-elevation (`-translate-y-0.5`) with dynamic depth contrast shifts.

---

## 2. Progress Bar Architecture & 5-Slot Quintile System

### 2.1 The Terminal Progress Green Palette (`#1af18d`)

The quota color scale represents health and availability. The canonical neon terminal green `#1af18d` serves as the pinnacle healthy indicator (100% full quota):

```css
/* Color Ramp Definition */
--ui-quota-stop-100: #1af18d; /* Neon Terminal Green (100% / Healthy) */
--ui-quota-stop-80:  #31c9a4; /* Vibrant Mint Emerald (80%) */
--ui-quota-stop-60:  #e7d36a; /* Luminous Warm Yellow (60%) */
--ui-quota-stop-40:  #f5bf58; /* Warm Amber (40%) */
--ui-quota-stop-20:  #f49a45; /* Warning Orange (20%) */
--ui-quota-stop-0:   #ef6a8a; /* Critical Rose Red (0% / Depleted) */
--ui-quota-floor:    #520808; /* Deep Crimson Shadow */
```

In `src/styles/ui-tokens.css`, `--ui-quota-gradient` is updated to incorporate `#1af18d`:

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
```

### 2.2 Exactly 5 Slots/Nodes at 20% Intervals

The checkpoint milestone array is strictly standardized across `QuotaProgressBar.tsx` and `WaterDrainProgressBar.tsx` to exactly 5 nodes:

```typescript
export const QUOTA_SLOT_INTERVALS = [100, 80, 60, 40, 20] as const;
```

Each slot represents the threshold of a 20% operational quintile:
- **Slot 5 (100%):** Represents 80%–100% (Full capacity, maximum operational headroom)
- **Slot 4 (80%):** Represents 60%–80% (Substantial capacity, healthy operational range)
- **Slot 3 (60%):** Represents 40%–60% (Moderate capacity, steady state)
- **Slot 2 (40%):** Represents 20%–40% (Approaching threshold, caution zone)
- **Slot 1 (20%):** Represents 0%–20% (Critical depletion, auto-switch armed)

### 2.3 Glyph Removal & Illuminated Glowing Slots

#### Anti-Pattern: Misleading Checkmarks
Previous implementations rendered a checkmark SVG (`<path d="M2.5 6.5L4.8 8.8L9.5 3.5" />`) inside every filled node. In quota telemetry, checkmarks falsely imply that a quota has been "completed" or "checked off".

#### Solution: Pure Glowing Illuminated Node Slots
1. Remove all SVG checkmark and check-bolt elements from node rendering.
2. Render node slots as illuminated circular beads/apertures:
   - **Filled Nodes:** Solid color fill corresponding to the scale stop (`#1af18d`, `#31c9a4`, `#e7d36a`, `#f49a45`, `#ef6a8a`), framed by a crisp 1.5px border.
   - **Active Head Node (Highest Filled):** Radiates a restrained multi-layer glow:
     - `100% node`: `box-shadow: 0 0 10px rgba(26, 241, 141, 0.85), 0 0 3px rgba(26, 241, 141, 1)`
     - `80% node`: `box-shadow: 0 0 8px rgba(49, 201, 164, 0.75)`
     - `60% node`: `box-shadow: 0 0 6px rgba(231, 211, 106, 0.65)`
     - `40% node`: `box-shadow: 0 0 6px rgba(245, 191, 88, 0.65)`
     - `20% node`: `box-shadow: 0 0 8px rgba(239, 106, 138, 0.8)`
   - **Unfilled Nodes:** Hollow muted aperture with dark violet surface (`bg-[#171025]`, `border-[#40285e]`), containing a subtle 1px center pip (`bg-white/20`).
   - **Critical State (<20%):** When quota drops below the 20% boundary, the lowest node illuminates in critical red with an ultra-compact monospace percentage numeral (`text-[7px] font-black font-mono text-white`).

```tsx
{/* Canonical Node Rendering Replacement */}
{isFilled ? (
    isCritical && cp <= 20 ? (
        <span className="font-black font-mono text-white text-[7px] leading-none tracking-tighter select-none">
            {Math.round(clamped)}%
        </span>
    ) : (
        <span className="w-1.5 h-1.5 rounded-full bg-white/90 shadow-xs" />
    )
) : (
    <span className="w-1 h-1 rounded-full bg-slate-400/40 dark:bg-white/15" />
)}
```

---

## 3. Right-Aligned Compacting (`w-36`/`w-44` `ml-auto`)

### 3.1 Layout Density Constraints

Unconstrained progress bars in grid cards and data tables expand arbitrarily, causing jagged column alignments and visual turbulence. To achieve clean, predictable vertical flow across both Account and Instance views:

1. **Card View (`AccountCard.tsx`, `QuotaItem.tsx`):**
   - The quota bar container must lock to a disciplined compact envelope: `w-36` (144px) on compact viewports, or `w-44` (176px) on expanded viewports.
   - Pinned to the right margin via `ml-auto`.
   - Leaves ample room on the left for model badges, lock status, and token type labels.

2. **Table View (`InstanceTable.tsx`, `InstanceRowCells.tsx`):**
   - In `QuotaCell`, constrain the model and weekly quota bars to `w-36 sm:w-44 ml-auto`.
   - Quota table header `<th className="text-right w-44">` matches the column constraint precisely.
   - Stack 4H rolling quota and Weekly quota neatly with `space-y-1` and zero horizontal jitter.

```tsx
{/* Standardized QuotaCell Layout */}
export function QuotaCell({ data }: { data: RowCellData }) {
    return (
        <td className="px-2 py-1 whitespace-nowrap text-right">
            <div className="w-36 sm:w-44 ml-auto space-y-1 text-right">
                {data.percentage !== null && (
                    <div className="flex flex-col items-end">
                        <div className="flex items-center justify-between w-full text-[10px] font-mono text-slate-400">
                            <span className="truncate max-w-[85px] font-medium">{data.modelName || 'Primary'}</span>
                            <span className="font-bold text-white">{Math.round(data.percentage)}%</span>
                        </div>
                        <QuotaProgressBar percentage={data.percentage} resetTime={data.modelResetTime} compact />
                    </div>
                )}
                {data.weeklyQuota !== null && (
                    <div className="flex flex-col items-end pt-0.5 border-t border-slate-800/50">
                        <div className="flex items-center justify-between w-full text-[9.5px] font-mono text-slate-500">
                            <span>Weekly</span>
                            <span className="font-bold text-amber-300">{Math.round(data.weeklyQuota.percentage)}%</span>
                        </div>
                        <QuotaProgressBar isWeekly percentage={data.weeklyQuota.percentage} resetTime={data.weeklyQuota.resetTime} compact />
                    </div>
                )}
            </div>
        </td>
    );
}
```

---

## 4. Theming Contrast Modernization: Eliminating Purple-on-Purple

### 4.1 Contrast Failure Analysis
In dark theme mode, several components currently render secondary labels with:
- Text: `text-purple-400`, `text-purple-300`, `text-[#a855f7]`, or `--ui-text-muted: #8f7fa8`
- Surface: `bg-[#171025]`, `bg-[#211631]`, or `bg-[#40285e]`

This creates low contrast ratios (~2.4:1 to 3.1:1), falling well below the WCAG 2.1 AA requirement of 4.5:1. In dimly lit environments or on non-calibrated displays, these elements blur into illegibility.

### 4.2 Enforced Color Rules

| Role | Forbidden Palette | Mandatory Replacement | Hex Value | Min Contrast |
| :--- | :--- | :--- | :--- | :--- |
| **Primary Headers / Titles** | Violet tinted gray | Pure Bright White | `#ffffff` / `#f5f0ff` | 14.5:1 |
| **Secondary Metadata / Subheads** | `text-purple-400`, `#8f7fa8` | Cream / Light Linen | `#fef9c3` / `#fef08a` | 11.2:1 |
| **Active States / Attention Pills** | `text-purple-300`, `#a855f7` | Luminous Warm Amber / Yellow | `#f5bf58` / `#fde047` | 9.8:1 |
| **Action & Icon Accents** | Dark lavender | High-Vibrancy Mint / Terminal Green | `#1af18d` / `#38bdf8` | 10.5:1 |
| **Surface Backgrounds** | Faded purples | Deep Black-Violet Canvas | `#090611` / `#171025` | Base |
| **Dividers & Borders** | Soft purple fill | Crisp Low-Opacity Outline | `#40285e` / `border-slate-800` | 3.5:1 |

### 4.3 Component-Level Contrast Rectifications
1. `AccountCardHeader.tsx`: Change account title from conditional `text-blue-900 dark:text-blue-200` to `text-slate-900 dark:text-white font-bold`.
2. `AccountCard.tsx`: Change secondary stats labels (`Priority`, `Bound`, `Last Switch`) from muted violet to `text-amber-100/90 dark:text-amber-200/90` or `text-slate-300`.
3. `QuotaItem.tsx`: Change remaining time font from `text-gray-400` to `text-amber-300 font-mono font-semibold` when warning, and `text-emerald-300 font-mono font-semibold` when healthy.

---

## 5. Account Email Privacy Masking

### 5.1 Privacy Requirements & Threat Model
Antigravity-Manager is frequently used during pair programming, video recording, and team presentations. Exposing full corporate or personal Google account emails (`john.doe.engineering@company.com`) creates privacy and security risks.

### 5.2 Implementation via `maskEmail`

The helper in `src/utils/maskEmail.ts` is leveraged:

```typescript
export function maskEmail(email: string): string {
    const local = (email.split('@')[0] || email).trim();
    if (!local) return '***';
    if (local.length <= 2) return `${local[0]}***`;
    const shown = Math.min(4, local.length - 1);
    const domain = email.includes('@') ? `@${email.split('@')[1]}` : '';
    return `${local.slice(0, shown)}***${domain ? domain.slice(0, 3) + '..' : ''}`;
}
```

### 5.3 UI Integration Rules
1. **Cards (`AccountCardHeader.tsx`):**
   - Display `maskEmail(account.email)` as the primary heading.
   - Maintain the full email in the native browser tooltip via `title={account.email}`.
   - Include an optional click-to-copy or eye icon toggle to reveal full email for 5 seconds.
2. **Rows & Tables (`AccountRow.tsx`, `AccountTable.tsx`):**
   - Under standard view, render `maskEmail(account.email)` with monospace rendering (`font-mono text-xs`).
   - If `showAllEmails` toggle in table toolbar is active, render full email.

---

## 6. CSS3 Smooth Hover Micro-Elevation

### 6.1 Interactive Micro-Elevation Specification
To improve desktop responsiveness and physical affordance:

1. **Card Elevation Token:**
   - Class: `transition-all duration-150 ease-out hover:-translate-y-0.5 hover:shadow-lg hover:shadow-black/40`
   - Movement: `-2px` on the Y-axis.
   - Hardware acceleration: handled by GPU transform matrix.

2. **Dynamic Border Contrast Shift:**
   - Resting state: `border border-slate-200 dark:border-[#40285e]/70`
   - Hover state: `hover:border-violet-500/60 dark:hover:border-[#7c3aed]/80`
   - Active/Selected state: `border-amber-400 dark:border-[#f5bf58] ring-1 ring-amber-400/30`

3. **Pill & Capsule Button Elevation:**
   - Buttons within segmented capsules adopt subtle scale and brightness transition:
   - `transition-all duration-150 ease-out active:scale-[0.98] hover:bg-white/10 dark:hover:bg-[#40285e]`

---

## 7. Verification Checklist & Success Criteria

- [ ] `QuotaProgressBar.tsx` contains exactly 5 slots `[100, 80, 60, 40, 20]`.
- [ ] No SVG checkmark or check-bolt paths exist inside progress bar markup.
- [ ] Terminal Green `#1af18d` is the 100% gradient endpoint and head glow color.
- [ ] Quota columns in accounts and instances are compacted to `w-36`/`w-44` and right-aligned (`ml-auto`).
- [ ] Dark theme contrast across `AccountCard` displays white/cream/yellow text with zero purple-on-purple illegibility.
- [ ] Account emails default to privacy masked format via `maskEmail`.
- [ ] Interactive cards micro-elevate on hover via `-translate-y-0.5`.
