---
plan: 228-ui-and-feature-enhancement-instructions
subtask: "01"
title: Architectural Contracts, CSS3 Micro-Interactions, Active Indicator Invariants, Default Instance Cloning, 24h Clock, and Rerun FIFO Specifications
domain: fullstack-ts-rust
depends_on: none
citations:
  architecture_spec: 02-spec/21-app/228-ui-and-feature-enhancement-instructions/01-architecture-spec.md
  coding_guidelines: .ai-memory/coding-guidelines.md
  strictly_avoid: .ai-memory/strictly-avoid.md
target_files:
  - 02-spec/21-app/228-ui-and-feature-enhancement-instructions/01-architecture-spec.md
  - .ai-memory/plans/subtasks/228-ui-and-feature-enhancement-instructions/01-subtask-architecture-and-contracts.md
  - src/App.css
  - src/components/common/selectedState.ts
  - src/pages/instances/InstanceCard.tsx
  - src/components/instances/PromptTreeView/DetailTabs.tsx
  - src/components/navbar/NavMenu.tsx
  - src/components/navbar/InstanceSelector.tsx
  - src/pages/instances/CreateEditDialogs.tsx
  - src/pages/instances/InstancesHeader.tsx
  - src/utils/date.ts
  - src/components/instances/PromptTreeView/DetailHeader.tsx
  - src/components/instances/PromptTreeView/usePromptTreePromptActions.ts
status: ready_for_execution
---

# Subtask 01: Architectural Contracts, CSS3 Micro-Interactions, Active Indicator Invariants, Default Instance Cloning, 24h Clock, and Rerun FIFO Specifications

## 1. Context & Motivation
Antigravity Manager requires comprehensive polish to address UI friction, transition slowness, visual anomalies on active states, top-menu capsule styling, and dispatch/rerun contracts:
1. **Interactive Transition Slowness**: `src/App.css` enforces a universal `250ms` transition across all elements (`*, *::before, *::after`), causing buttons and inputs to feel sluggish and unresponsive during hover and active clicks.
2. **Active Instance Card Rail & Nested Pill Distortion**: Combining `border-l-[6px]` with `rounded-[5px]` on `InstanceCard.tsx` distorts top-left and bottom-left corner radii. Additionally, rendering `ACTIVE_PILL_CLASSES` (which contains `rounded-full border`) inside a segmented action capsule creates an unsightly "pill-in-pill" visual defect.
3. **Prompt Inspector Tab Misalignment**: View mode tabs (Preview, Raw, Edit) and Details in `DetailTabs.tsx` have varying heights, padding, and vertical baseline offsets, alongside a stark, opaque blue block background that disrupts dark-glass styling.
4. **Top-Level Menu Capsule Bleed**: Segmented capsules in `NavMenu.tsx` contain nested items with rectangular hover states that bleed outside rounded-full boundaries.
5. **New Instance Empty Sandbox Flaw**: Clicking "+ New Instance" defaults to a blank profile rather than duplicating the designated "Default" instance, depriving users of cloned settings, extensions, and workspace state.
6. **Universal 24-Hour Clock Invariant**: Scattered usages of `toLocaleTimeString()` introduce AM/PM layout width jitter and locale divergence across views.
7. **Prompt Rerun FIFO Invariant**: Multi-round rerun turns must strictly enqueue via SQLite `active_prompts` with monotonic ordering (`ORDER BY created_at ASC, id ASC`) and bracketed attachments formatted with canonical semicolon syntax (`@[file;<path>]`).

---

## 2. Architecture & Invariant Contracts

### Contract A: CSS3 Micro-Interactions & Transition Lag Exemption
- **File**: `src/App.css`
- **Rule**: All interactive controls (`button`, `input`, `select`, `textarea`, `[role="button"]`, `.btn-tactile`) must be exempted from universal `250ms` delays.
- **Micro-Interaction Token**:
  - Duration: `100ms !important`
  - Timing function: `cubic-bezier(0.16, 1, 0.3, 1) !important`
  - Active compression: `active:scale-[0.98]`
  - Properties: `color, background-color, border-color, box-shadow, transform, opacity !important`

### Contract B: Active Instance Indicator & Capsule Nesting Invariant
- **Files**: `src/components/common/selectedState.ts`, `src/pages/instances/InstanceCard.tsx`
- **Rule 1 (Corner Radius Invariant)**: The outer card border must remain symmetrical `1px` on all four edges (`border border-amber-400/35`). The `6px` amber active indicator must be implemented as an absolute inner left rail (`absolute left-0 top-0 bottom-0 w-1.5 bg-amber-400 rounded-l-[4px] pointer-events-none`) or cleanly bounded inset element to prevent corner clipping.
- **Rule 2 (Capsule Nesting Invariant)**: Segmented capsules must never contain an internal pill with its own outer border. Inside segmented capsules, the active indicator must seamlessly occupy its segment with borderless, dark-glass tinting (`px-2 py-0.5 text-[9px] font-bold tracking-wider bg-amber-400/15 text-amber-900 dark:text-amber-300`).

### Contract C: Prompt Inspector Tab Alignment & Dark-Glass Background
- **File**: `src/components/instances/PromptTreeView/DetailTabs.tsx`
- **Rule 1 (Baseline & Height Contract)**: Capsule container height fixed at `h-7 min-h-[28px]`. All buttons enforce `h-full inline-flex items-center justify-center text-xs font-medium`.
- **Rule 2 (Translucent Active Styling)**: Discard opaque `bg-blue-600 text-white`. Replace with theme-adaptive translucent glass:
  - Light mode: `bg-sky-500/15 text-sky-700 shadow-2xs font-semibold`
  - Dark mode: `dark:bg-sky-500/20 dark:text-sky-300 dark:border-sky-500/30 shadow-2xs font-semibold`

### Contract D: Top-Level Menu Capsule Geometry
- **Files**: `src/components/navbar/NavMenu.tsx`, `src/components/navbar/InstanceSelector.tsx`
- **Rule**: Outer capsules must specify `rounded-full border p-0.5 overflow-hidden`.
- **Segment Radius**:
  - First child: `rounded-l-full`
  - Middle children: `rounded-none`
  - Last child: `rounded-r-full`
  - Hover highlights must not produce rectangular artifacts outside the rounded capsule perimeter.

### Contract E: Default Instance Cloning Invariant
- **Files**: `src/pages/instances/CreateEditDialogs.tsx`, `src/pages/instances/InstancesHeader.tsx`
- **Rule**: When opening the create instance dialog, `newInstanceFromInstance` must automatically default to the instance ID where `inst.config.is_default === true` (or fall back to `activeInstanceId`).
- The option in the dropdown must be clearly labeled: `Default Profile (Clones extensions, settings & policies)`.

### Contract F: Universal 24-Hour Clock Contract
- **File**: `src/utils/date.ts`
- **Rule**: Implement `formatTime24(value?: number | string | Date | null, options?: { withSeconds?: boolean, fallback?: string }): string`.
- Guarantees strict 2-digit zero-padded `HH:mm` or `HH:mm:ss` formatting. Prohibit direct calls to `toLocaleTimeString()` across frontend components.

### Contract G: Prompt Rerun FIFO & Modal Simplification
- **Files**: `src/components/instances/PromptTreeView/DetailHeader.tsx`, `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- **Rule 1 (Mode Button Labels)**: Simplify labels to `"Send"`, `"Enqueue"`, and `"Rerun"`. Detailed mode behavior moves to tooltips (`title` / ARIA).
- **Rule 2 (Dropdown Width)**: Expand dropdown containers to `min-w-[280px]` or `w-full` to eliminate text truncation.
- **Rule 3 (FIFO Monotonicity)**: In `dispatchRerunMessages`, all $N$ turns must invoke `enqueuePrompt` with monotonically increasing timestamps (`created_at = Date.now() + i`).
- **Rule 4 (Attachment Format)**: Canonical bracketed semicolon syntax: `@[file;<path>]`.
- **Rule 5 (Pre-Deployment Prompt)**: Include pre-deployment verification template: `"Have you tested these below instructions with end-to-end testing and released it?"`.

---

## 3. Target Files, Symbols & Responsibilities

| Target File | Symbols / Classes / Functions | Primary Responsibility |
| :--- | :--- | :--- |
| `src/App.css` | `button, input, select, textarea`, `.btn-tactile` | Transition lag elimination; apply 100ms cubic-bezier tactile overrides. |
| `src/components/common/selectedState.ts` | `SELECTED_CARD_CLASSES`, `ACTIVE_PILL_CLASSES` | Update card selection tokens; decouple 6px border-left from card perimeter. |
| `src/pages/instances/InstanceCard.tsx` | `InstanceCard`, Active segmented button | Implement absolute inset rail; eliminate nested pill-in-pill defect in header capsule. |
| `src/components/instances/PromptTreeView/DetailTabs.tsx` | `DetailTabs`, `PromptViewMode` buttons | Unify vertical height to 28px/32px; replace stark blue with translucent dark-glass active state. |
| `src/components/navbar/NavMenu.tsx` | `NavMenu`, top navigation capsule | Enforce capsule boundary and segment radius mapping; eliminate rectangular hover bleeds. |
| `src/components/navbar/InstanceSelector.tsx` | `InstanceSelector` | Align trigger and fast-forward buttons within uniform segmented capsule geometry. |
| `src/pages/instances/CreateEditDialogs.tsx` | `CreateInstanceDialog` | Pre-select Default instance for cloning; update clone dropdown labeling. |
| `src/pages/instances/InstancesHeader.tsx` | `InstancesHeader` | Initialize `newInstanceFromInstance` with default instance ID on button click. |
| `src/utils/date.ts` | `formatTime24`, `FormatTime24Options` | Implement canonical 24-hour clock formatter; export helper. |
| `src/components/instances/PromptTreeView/DetailHeader.tsx` | `DetailHeader`, mode buttons | Simplify labels to "Send", "Enqueue", "Rerun"; expand message dropdown width. |
| `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | `dispatchRerunMessages` | Enforce FIFO enqueue sequencing across all rounds with monotonic timestamps. |

---

## 4. Concrete Implementation Steps

### Step 1: CSS3 Tactile Transition & Micro-Interaction Tokens (`src/App.css`)
1. In `src/App.css`, immediately following the universal transition declaration (`*, *::before, *::after`), define interactive element exemptions:
   ```css
   button,
   input,
   select,
   textarea,
   [role="button"],
   .btn-tactile {
     transition-duration: 100ms !important;
     transition-timing-function: cubic-bezier(0.16, 1, 0.3, 1) !important;
     transition-property: color, background-color, border-color, box-shadow, transform, opacity !important;
   }

   .btn-tactile:active {
     transform: scale(0.98);
   }
   ```
2. Verify that high-frequency interactions feel instantaneous and snappy.

### Step 2: Active Card Inset Rail & Capsule Cleanliness (`src/components/common/selectedState.ts`, `InstanceCard.tsx`)
1. In `src/components/common/selectedState.ts`, ensure `SELECTED_CARD_CLASSES` maintains uniform `border border-amber-400/35` without distorting the left edge:
   ```typescript
   export const SELECTED_CARD_CLASSES =
     "bg-slate-100 dark:bg-slate-900 text-slate-900 dark:text-white border border-amber-400/35 font-semibold shadow-xs";
   ```
2. In `src/pages/instances/InstanceCard.tsx`:
   - Render the amber rail as an absolute inset element inside the card:
     ```tsx
     {isActive && (
         <span
             className="absolute left-0 top-0 bottom-0 w-1.5 bg-amber-400 rounded-l-[4px] pointer-events-none z-10"
             aria-hidden="true"
         />
     )}
     ```
   - In the top-right action capsule, replace the nested `ACTIVE_PILL_CLASSES` with a flush segmented indicator:
     ```tsx
     {isActive ? (
         <span className="text-[9px] font-bold tracking-wider px-2 py-0.5 bg-amber-400/15 text-amber-900 dark:text-amber-300 select-none">
             Active
         </span>
     ) : (
         <button
             disabled={isBusy}
             onClick={() => setActiveInstance(inst.config.id)}
             className="text-[10px] text-gray-500 hover:text-blue-600 dark:text-slate-400 dark:hover:text-slate-200 transition-colors font-medium px-2 py-0.5 cursor-pointer disabled:opacity-50"
         >
             Set Active
         </button>
     )}
     ```

### Step 3: Prompt Inspector Tab Baseline & Translucent Dark-Glass Styling (`DetailTabs.tsx`)
1. In `src/components/instances/PromptTreeView/DetailTabs.tsx`:
   - Enforce uniform height on view mode container: `h-7`.
   - Update Preview, Raw, Edit buttons:
     - Replace `bg-blue-600 text-white shadow-2xs` with:
       `bg-sky-500/15 text-sky-700 dark:bg-sky-500/20 dark:text-sky-300 dark:border dark:border-sky-500/30 shadow-2xs font-semibold`
     - Keep unselected buttons in `text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-800/60`.
   - Update Details button to share identical `h-full`, centering `<Activity className="w-3.5 h-3.5" />`, label `"Details"`, and green pulse dot on the exact same vertical baseline.

### Step 4: Top-Level Menu Capsule Geometry & Hover Cleanliness (`NavMenu.tsx`)
1. In `src/components/navbar/NavMenu.tsx`:
   - Ensure the outer capsule has `rounded-full border border-gray-200/60 dark:border-slate-800 p-0.5 shadow-xs divide-x overflow-hidden`.
   - Apply contextual border-radius classes to children:
     - Accounts button: `rounded-l-full`
     - Instances & Telegram buttons: `rounded-none`
     - Settings & More buttons: `rounded-r-full` (or contextual based on position)
   - Ensure hover background fills the section completely without protruding beyond the pill edge.

### Step 5: "+ New Instance" Default Clone Selection (`CreateEditDialogs.tsx`, `InstancesHeader.tsx`)
1. In `src/pages/instances/InstancesHeader.tsx`:
   - When opening the create modal (`onClick`), locate the default instance:
     ```tsx
     const defaultInst = instances.find((i) => i.config.is_default) || instances[0];
     setNewInstanceFromInstance(defaultInst ? defaultInst.config.id : '');
     setNewInstanceName('');
     setIsCreateOpen(true);
     ```
2. In `src/pages/instances/CreateEditDialogs.tsx`:
   - Ensure the dropdown option for the default instance is prominently marked:
     `${inst.config.name} (Default - Recommended)`

### Step 6: Universal 24-Hour Time Formatting Helper (`src/utils/date.ts`)
1. In `src/utils/date.ts`, add and export `formatTime24`:
   ```typescript
   export interface FormatTime24Options {
       withSeconds?: boolean;
       fallback?: string;
   }

   export function formatTime24(
       value?: number | string | Date | null,
       options?: FormatTime24Options
   ): string {
       const ms = parseToMs(value);
       if (ms === null) return options?.fallback ?? '-';

       const d = new Date(ms);
       const hours = String(d.getHours()).padStart(2, '0');
       const minutes = String(d.getMinutes()).padStart(2, '0');

       if (options?.withSeconds) {
           const seconds = String(d.getSeconds()).padStart(2, '0');
           return `${hours}:${minutes}:${seconds}`;
       }

       return `${hours}:${minutes}`;
   }
   ```
2. Audit all instances of `toLocaleTimeString()` in `src/` and replace with `formatTime24`.

### Step 7: Prompt Rerun FIFO & Dispatch Modal Polish (`DetailHeader.tsx`, `usePromptTreePromptActions.ts`)
1. In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
   - Update dispatch mode button labels:
     - Change `"Send (Preempt)"` to `"Send"`.
     - Change `"Enqueue (FIFO)"` to `"Enqueue"`.
     - Change `"Rerun Turn"` to `"Rerun"`.
   - Maintain comprehensive explanations inside the `title` attribute for rich tooltip inspection.
   - Widen message option dropdown: add `min-w-[280px]` or `w-full` class to eliminate text clipping.
2. In `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`:
   - In `dispatchRerunMessages`, ensure every round invokes `enqueuePrompt` with sequential timestamps:
     `timestamp: Date.now() + i`
   - Guarantee bracketed attachments follow canonical semicolon format: `@[file;<path>]`.

---

## 5. Verification Criteria & Test Plan

1. **CSS3 Micro-Interaction Verification**:
   - Inspect computed CSS styles in browser DevTools on `<button>` elements.
   - Confirm `transition-duration: 100ms` and `active:scale-[0.98]` micro-interactions are active.
   - Confirm zero lag when hovering or clicking buttons.
2. **Active Card Geometry & Capsule Verification**:
   - Select an instance card in `Instances.tsx`.
   - Verify the amber rail is cleanly rendered on the left without distorting the `rounded-[5px]` corner radius.
   - Verify the top-right active indicator does not exhibit double borders or nested pill artifacts.
3. **Prompt Inspector Tabs Alignment**:
   - Navigate to Prompt Tree inspector.
   - Verify Preview, Raw, Edit, and Details buttons share a single continuous baseline and 28px/32px height.
   - Verify the active tab displays translucent dark-glass sky styling (`bg-sky-500/20 text-sky-300`) instead of solid blue.
4. **Top-Level Menu Capsule Verification**:
   - Hover over top-level navigation items (Accounts, Instances, Telegram, Settings).
   - Verify hover backgrounds are strictly contained within the capsule curvature with zero rectangular overflow.
5. **New Instance Default Clone Verification**:
   - Click "+ New Instance" button.
   - Verify "Clone Settings & Extensions From" dropdown automatically defaults to the Default profile.
6. **24-Hour Time Verification**:
   - Check all timestamps in instance cards and prompt trees.
   - Confirm all times display in strict 24-hour format (`HH:mm` or `HH:mm:ss`), with zero AM/PM markers.
7. **Rerun FIFO Invariant Verification**:
   - Trigger a 3-turn rerun.
   - Check SQLite database queries: confirm all 3 entries are inserted into `active_prompts` with status `queued`/`backed_up` and ordered via `ORDER BY created_at ASC, id ASC`.

---

## 6. Rollback & Risk Mitigation
- If CSS3 micro-interaction overrides affect third-party components, restrict `.btn-tactile` rules specifically to internal buttons using scoped class selectors.
- If an instance has no default profile configured, ensure `newInstanceFromInstance` gracefully falls back to empty string without throwing null reference errors.
- If `formatTime24` receives invalid timestamps, it returns fallback `'-'` cleanly without throwing runtime exceptions.
