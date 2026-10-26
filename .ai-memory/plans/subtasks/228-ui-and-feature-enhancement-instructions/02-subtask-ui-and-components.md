# Subtask Plan 02: UI Components, Menu Geometry & Universal 24-Hour Clock

- **Document ID**: `.ai-memory/plans/subtasks/228-ui-and-feature-enhancement-instructions/02-subtask-ui-and-components.md`
- **Parent Task**: `ui-and-feature-enhancement-instructions` (Sequence `228`)
- **Parent Plan**: [.ai-memory/plans/pending/228-ui-and-feature-enhancement-instructions.md](../../../plans/pending/228-ui-and-feature-enhancement-instructions.md)
- **Component Spec**: [02-spec/21-app/228-ui-and-feature-enhancement-instructions/02-component-spec.md](../../../../02-spec/21-app/228-ui-and-feature-enhancement-instructions/02-component-spec.md)
- **Status**: READY FOR EXECUTION
- **Author**: Spec Author 02
- **Created**: 2026-10-27
- **Target Release**: Minor Version Bump
- **Assigned Worker Waves**: Wave 2 (Workers 01, 02, 03) & Wave 3 (Workers 01, 03)

---

## 1. Context & Objectives

This subtask defines the exact, grounded implementation tasks for all UI components, visual geometry fixes, creation flow invariants, and universal 24-hour time formatting across Antigravity Manager.

### Core Objectives:
1. **Active Card Left Rail & Segmented Pill Capsule**:
   - Eliminate nested pill borders and invalid `py-0.2` in `src/pages/instances/InstanceCard.tsx`.
   - Export `ACTIVE_CAPSULE_SEGMENT_CLASSES` in `src/components/common/selectedState.ts`.
   - Add responsive CSS3 hover elevation (`hover:-translate-y-0.5 hover:shadow-md`).
2. **DetailTabs Baseline & Translucent Accent**:
   - Establish unified `h-7 flex items-center justify-center leading-none` baseline across all mode and view buttons in `src/components/instances/PromptTreeView/DetailTabs.tsx`.
   - Replace opaque `bg-blue-600 text-white` block with translucent dark-glass blue accent (`bg-blue-500/20 text-blue-600 dark:bg-blue-500/25 dark:text-blue-300`).
3. **Top Bar Capsule Curvature & Middle Button Hover**:
   - Fix window control hover rectangular leaking in `src/components/navbar/NavSettings.tsx`.
   - Add `overflow-hidden` and concentric curvature to segmented capsules in `src/components/navbar/InstanceSelector.tsx` and `src/components/navbar/NavMenu.tsx`.
4. **Default Clone Invariant on New Instance**:
   - Preselect the Default instance ID when opening the creation modal in `src/pages/instances/useInstancePage.ts` and `src/pages/instances/CreateEditDialogs.tsx`.
   - Add tactile hover elevation to "+ New Instance" buttons in `src/pages/instances/InstancesHeader.tsx` and `src/pages/instances/InstancesStates.tsx`.
5. **Canonical 24-Hour Clock Engine & 9 Call Sites**:
   - Author `formatTime24` and `formatDateTime24` in `src/utils/date.ts`.
   - Migrate all 9 uncontrolled `toLocaleTimeString()` and `toLocaleString()` call sites.
6. **Settings Modernization & Theme Picker Integration**:
   - Standardize control heights to `h-9` in `src/pages/Settings/tabs/GeneralTab.tsx`.
   - Integrate `ThemePicker` catalog directly into the General settings tab.
   - Upgrade low-contrast dark mode classes to WCAG AAA slate tokens.

---

## 2. File Ownership & Modification Matrix

| Target File | Primary Component | Line References | Scope of Changes |
| :--- | :--- | :---: | :--- |
| `src/components/common/selectedState.ts` | Selected State Tokens | L41–48 | Export `ACTIVE_CAPSULE_SEGMENT_CLASSES` token for segmented capsule mounts. |
| `src/pages/instances/InstanceCard.tsx` | Instance Card | L164–170, L231–236 | Add `hover:-translate-y-0.5 hover:shadow-md`, replace nested pill with active capsule segment, delete `py-0.2`. |
| `src/components/instances/PromptTreeView/DetailTabs.tsx` | Prompt Inspector Tabs | L40–95, L106–176 | Standardize `h-7 leading-none` baseline, apply translucent blue accent. |
| `src/components/navbar/NavSettings.tsx` | Window Controls | L397–446 | Add `overflow-hidden` to capsule container, eliminate sharp middle button hover leakage. |
| `src/components/navbar/InstanceSelector.tsx` | Profile Dropdown | L32–34 | Add `overflow-hidden` for concentric curvature. |
| `src/components/navbar/NavMenu.tsx` | Top Navigation Bar | L98–100 | Add `overflow-hidden` for concentric curvature. |
| `src/pages/instances/useInstancePage.ts` | Instance Page Hook | L57–61 | Pre-populate `newInstanceFromInstance` with Default instance ID on create modal trigger. |
| `src/pages/instances/CreateEditDialogs.tsx` | Creation Modal | L24–30, L86–97 | Preselect Default instance in dropdown, add `(Default - Recommended)` label. |
| `src/pages/instances/InstancesHeader.tsx` | Header Toolbar | L663–673 | Add tactile hover elevation to "+ New Instance" button. |
| `src/pages/instances/InstancesStates.tsx` | Filter Bar & Empty State | L32–42, L95–105 | Add tactile hover elevation to "+ New Instance" buttons. |
| `src/utils/date.ts` | Date/Time Utilities | L45–95 | Export canonical `formatTime24` and `formatDateTime24` helpers. |
| `src/components/audit/AuditPanel.tsx` | Audit Log Table | L120 | Replace `toLocaleTimeString()` with `formatTime24(ms, { withSeconds: true })`. |
| `src/components/common/NetworkMonitor.tsx` | Network Monitor | L96, L138 | Replace `toLocaleTimeString()` and `toLocaleString()` with `formatTime24` / `formatDateTime24`. |
| `src/components/debug/DebugConsole.tsx` | Debug Log Console | L20–22 | Replace custom `toLocaleTimeString` formatting with `formatTime24(date, { withMs: true })`. |
| `src/components/errors/error-history-drawer.tsx` | Error Drawer | L364–370 | Replace `toLocaleTimeString` with `formatTime24(err.createdAt, { withSeconds: true })`. |
| `src/components/errors/error-modal/chrome.tsx` | Error Diagnostics Modal | L109 | Replace `toLocaleTimeString()` with `formatTime24(error.createdAt, { withSeconds: true })`. |
| `src/components/proxy/monitor/LogTable.tsx` | Proxy Log Table | L242 | Replace `toLocaleTimeString()` with `formatTime24(log.timestamp, { withSeconds: true })`. |
| `src/components/proxy/monitor/LogDetailPanel.tsx` | Proxy Log Inspector | L109 | Replace `toLocaleString()` with `formatDateTime24(log.timestamp, { withSeconds: true })`. |
| `src/lib/error-report-generator.ts` | Error Report Engine | L186, L287 | Replace `toLocaleString()` and `toLocaleTimeString()` with canonical helpers. |
| `src/pages/Settings.tsx` | Settings Navigation | L25–95 | Standardize tab button heights and active pill styling. |
| `src/pages/Settings/tabs/GeneralTab.tsx` | General Preferences | L18–75, L80–200 | Standardize control heights to `h-9`, mount `ThemePicker`, upgrade dark mode contrast. |

---

## 3. Step-by-Step Implementation Tasks

### Step 1: Active Section Tokens & Card Polish (`selectedState.ts` & `InstanceCard.tsx`)
- **Target Files**:
  - `src/components/common/selectedState.ts`
  - `src/pages/instances/InstanceCard.tsx`
- **Actions**:
  1. In `src/components/common/selectedState.ts`, export `ACTIVE_CAPSULE_SEGMENT_CLASSES`:
     ```ts
     export const ACTIVE_CAPSULE_SEGMENT_CLASSES =
       "inline-flex items-center justify-center px-2 py-0.5 text-[9px] font-bold uppercase tracking-wider bg-amber-500/15 text-amber-800 dark:bg-amber-400/20 dark:text-amber-300 select-none";
     ```
  2. In `src/pages/instances/InstanceCard.tsx` (lines 164–170), replace `transition-colors duration-300 ease-in-out hover:shadow-md` with:
     ```tsx
     className={cn(
         'group relative rounded-[6px] border transition-all duration-150 ease-out hover:-translate-y-0.5 hover:shadow-md flex flex-col justify-between overflow-hidden backdrop-blur-xs',
         isActive
             ? SELECTED_CARD_CLASSES
             : 'bg-white dark:bg-[var(--ui-surface-1)] border-gray-200/50 dark:border-[var(--ui-border-subtle)] hover:border-gray-300/80 dark:hover:border-[var(--ui-interactive-40)] hover:bg-slate-50/90 dark:hover:bg-[var(--ui-surface-hover)] shadow-xs'
     )}
     ```
  3. In `src/pages/instances/InstanceCard.tsx` (lines 231–236), replace `ACTIVE_PILL_CLASSES` and `py-0.2`:
     ```tsx
     <div className="shrink-0 inline-flex items-center overflow-hidden rounded-full border border-slate-200 dark:border-[#40285e] divide-x divide-slate-200 dark:divide-[#40285e] bg-slate-50/50 dark:bg-slate-900/40">
         {isActive ? (
             <span className={ACTIVE_CAPSULE_SEGMENT_CLASSES}>
                 Active
             </span>
         ) : (
             <button
                 disabled={isBusy}
                 onClick={() => setActiveInstance(inst.config.id)}
                 className="text-[10px] text-gray-500 hover:text-blue-600 dark:text-gray-400 dark:hover:text-blue-400 transition-colors font-medium px-2 py-0.5 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                 title="Set as active instance for account switches"
             >
                 Set Active
             </button>
         )}
     </div>
     ```

### Step 2: DetailTabs Vertical Alignment & Translucent Accent (`DetailTabs.tsx`)
- **Target File**: `src/components/instances/PromptTreeView/DetailTabs.tsx`
- **Actions**:
  1. Add `overflow-hidden` to both left and right capsule wrappers (lines 40 and 106).
  2. For Left Mode Capsule buttons (Preview, Raw, Edit, Details; lines 45–95):
     - Replace variable padding with `h-7 px-2.5 flex items-center justify-center leading-none`.
     - Replace active state `bg-blue-600 text-white shadow-2xs` with translucent accent: `bg-blue-500/20 text-blue-600 dark:bg-blue-500/25 dark:text-blue-300 shadow-2xs font-semibold`.
     - For the Details button, use `h-7 px-2.5 flex items-center gap-1.5 text-[11px] font-semibold leading-none`.
  3. For Right Stats & Capsule buttons (Input, Output, Expand Full/Collapse; lines 106–176):
     - Replace `py-1` with `h-7 px-3 flex items-center gap-1.5 text-[11px] font-semibold leading-none`.
     - Active tab style: `bg-blue-500/20 text-blue-600 dark:bg-blue-500/25 dark:text-blue-300 shadow-2xs font-semibold`.
     - Expand button style: `h-7 px-2.5 flex items-center gap-1 text-[10px] font-semibold leading-none`.

### Step 3: Top-Level Menu Geometry & Capsule Hover (`NavSettings.tsx`, `NavMenu.tsx`, `InstanceSelector.tsx`)
- **Target Files**:
  - `src/components/navbar/NavSettings.tsx`
  - `src/components/navbar/NavMenu.tsx`
  - `src/components/navbar/InstanceSelector.tsx`
- **Actions**:
  1. In `src/components/navbar/NavSettings.tsx` (lines 397–446):
     - Add `overflow-hidden` to the window controls container.
     - Simplify button corner radii to let the outer container handle clipping:
       - Minimize: `w-7 h-7 hover:bg-gray-200 dark:hover:bg-[#40285e] flex items-center justify-center transition-colors cursor-pointer`
       - Maximize: `w-7 h-7 hover:bg-gray-200 dark:hover:bg-[#40285e] flex items-center justify-center transition-colors cursor-pointer`
       - Close: `w-7 h-7 hover:bg-red-500 hover:text-white dark:hover:bg-red-500 dark:hover:text-white flex items-center justify-center transition-colors cursor-pointer`
  2. In `src/components/navbar/NavMenu.tsx` (line 98):
     - Add `overflow-hidden` to the capsule container:
       ```tsx
       <div className="flex items-center rounded-full bg-gray-100 dark:bg-slate-900/90 border border-gray-200/60 dark:border-slate-800 p-0.5 shadow-xs divide-x divide-gray-200/70 dark:divide-slate-800/80 overflow-hidden">
       ```
  3. In `src/components/navbar/InstanceSelector.tsx` (line 32):
     - Add `overflow-hidden` to outer container:
       ```tsx
       <div className="relative flex items-center rounded-full bg-gray-100 dark:bg-[#171025]/90 border border-gray-200/60 dark:border-[#40285e] p-0.5 divide-x divide-gray-200/50 dark:divide-slate-700/60 shadow-xs shrink-0 overflow-hidden" ref={dropdownRef}>
       ```

### Step 4: Default Clone Invariant & Hover Elevation (`useInstancePage.ts`, `CreateEditDialogs.tsx`, `InstancesHeader.tsx`, `InstancesStates.tsx`)
- **Target Files**:
  - `src/pages/instances/useInstancePage.ts`
  - `src/pages/instances/CreateEditDialogs.tsx`
  - `src/pages/instances/InstancesHeader.tsx`
  - `src/pages/instances/InstancesStates.tsx`
- **Actions**:
  1. In `src/pages/instances/useInstancePage.ts`:
     - Add helper:
       ```ts
       const defaultId = instances.find((i) => i.config.is_default)?.config.id || instances[0]?.config.id || '';
       ```
     - When initializing or resetting creation state in `openCreateModal` or action handlers, pre-populate `setNewInstanceFromInstance(defaultId)`.
  2. In `src/pages/instances/CreateEditDialogs.tsx`:
     - In `close()` (line 28), reset `setNewInstanceFromInstance` to the default instance ID rather than `''`.
     - In the `<select>` dropdown (lines 86–97), render Default instance first with `(Default - Recommended)` label.
  3. In `src/pages/instances/InstancesHeader.tsx` (lines 663–673) and `InstancesStates.tsx` (lines 32–42):
     - Upgrade "+ New Instance" button with tactile hover: `hover:bg-blue-500 hover:shadow-md hover:-translate-y-0.5 active:translate-y-0 active:scale-[0.98] transition-all duration-150 ease-out`.

### Step 5: Universal 24-Hour Clock Engine (`date.ts` & 9 Call Sites)
- **Target Files**:
  - `src/utils/date.ts`
  - 9 component call sites
- **Actions**:
  1. In `src/utils/date.ts`, implement and export `formatTime24` and `formatDateTime24`:
     - `formatTime24(value, { withSeconds, withMs })`: Returns `HH:mm`, `HH:mm:ss`, or `HH:mm:ss.SSS`.
     - `formatDateTime24(value, { withSeconds, iso })`: Returns `DD-MMM-YYYY HH:mm` or `DD-MMM-YYYY HH:mm:ss`.
  2. Migrate Call Site 1: `src/components/audit/AuditPanel.tsx` (line 120):
     - Replace `return new Date(ms).toLocaleTimeString();` with `return formatTime24(ms, { withSeconds: true });`.
  3. Migrate Call Sites 2 & 3: `src/components/common/NetworkMonitor.tsx`:
     - Line 96: `{formatTime24(req.startTime, { withSeconds: true })}`
     - Line 138: `{formatDateTime24(selectedRequest.startTime, { withSeconds: true })}`
  4. Migrate Call Site 4: `src/components/debug/DebugConsole.tsx` (lines 20–22):
     - Replace custom logic with `const timeStr = formatTime24(date, { withMs: true });`.
  5. Migrate Call Site 5: `src/components/errors/error-history-drawer.tsx` (lines 364–370):
     - Replace `toLocaleTimeString` with `const time = formatTime24(err.createdAt, { withSeconds: true });`.
  6. Migrate Call Site 6: `src/components/errors/error-modal/chrome.tsx` (line 109):
     - Replace `toLocaleTimeString()` with `{formatTime24(error.createdAt, { withSeconds: true })}`.
  7. Migrate Call Site 7: `src/components/proxy/monitor/LogTable.tsx` (line 242):
     - Replace `toLocaleTimeString()` with `{formatTime24(log.timestamp, { withSeconds: true })}`.
  8. Migrate Call Site 8: `src/components/proxy/monitor/LogDetailPanel.tsx` (line 109):
     - Replace `toLocaleString()` with `formatDateTime24(log.timestamp, { withSeconds: true })`.
  9. Migrate Call Site 9: `src/lib/error-report-generator.ts` (lines 186 and 287):
     - Line 186: `formatDateTime24(new Date(), { withSeconds: true })`
     - Line 287: `formatTime24(c.timestamp, { withSeconds: true })`

### Step 6: Settings & Preferences Modernization (`Settings.tsx` & `GeneralTab.tsx`)
- **Target Files**:
  - `src/pages/Settings.tsx`
  - `src/pages/Settings/tabs/GeneralTab.tsx`
- **Actions**:
  1. In `src/pages/Settings.tsx`:
     - Align tab button padding to `px-4 py-1.5 rounded-full text-xs sm:text-sm font-semibold`.
  2. In `src/pages/Settings/tabs/GeneralTab.tsx`:
     - Standardize all `<select>` and `<input>` heights to `h-9 text-xs sm:text-sm px-3 py-1.5 rounded-lg border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900/60`.
     - Import and render `ThemePicker` from `src/components/settings/ThemePicker.tsx` inside the Theme section.
     - Upgrade sub-cards from `bg-gray-50 dark:bg-base-200` to `bg-slate-50/80 dark:bg-slate-900/60 border border-slate-200 dark:border-slate-800` for WCAG AAA contrast.

---

## 4. Verification Evidence & Quality Gates

1. **Static Analysis & Compilation Gate**:
   - Verify zero TypeScript compiler errors (`npm run build` or targeted tsc).
   - Verify zero broken imports across all 9 date migration call sites.
2. **Visual Inspection**:
   - Inspect active instance card: confirm amber left rail is unbroken and action badge is flush without nested pill borders.
   - Inspect DetailTabs: confirm Preview, Raw, Edit, Details, Input, Output, Expand sit on a shared `h-7` horizontal axis with translucent blue highlights.
   - Inspect top navbar: hover over window controls; confirm zero rectangular corner artifacts.
   - Inspect New Instance modal: confirm Default instance is preselected in the clone dropdown.
   - Inspect AuditPanel, NetworkMonitor, DebugConsole, ErrorDrawer, LogTable: confirm all timestamps strictly use 24-hour time (`HH:mm:ss`).
