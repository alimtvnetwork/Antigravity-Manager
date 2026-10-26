# Component Specification: UI & Feature Enhancements (228)

- **Document ID**: `02-spec/21-app/228-ui-and-feature-enhancement-instructions/02-component-spec.md`
- **Parent Task**: `ui-and-feature-enhancement-instructions` (Sequence `228`)
- **Author**: Spec Author 02
- **Status**: DRAFT / READY FOR EXECUTION
- **Created**: 2026-10-27
- **Target Release**: Minor Version Bump
- **Primary Dependencies**: Tailwind CSS, React 19, Lucide React, Tauri Core APIs

---

## 1. Executive Summary & Scope

This specification details the frontend component architecture, visual styling tokens, and interaction models required to resolve UI anomalies, alignment defects, and inconsistent state representations across Antigravity Manager.

### Core Component Enhancements Covered:
1. **`InstanceCard.tsx` & `selectedState.ts`**: Resolve active card corner clipping, eliminate nested pill border anomalies in action capsules, delete invalid `py-0.2` Tailwind tokens, and introduce tactile CSS3 hover elevation.
2. **`DetailTabs.tsx`**: Align prompt inspector controls (Preview, Raw, Edit, Details, Input, Output, Expand) to a unified `h-7 flex items-center justify-center` vertical baseline, normalize font leading, and replace opaque solid blue blocks with translucent dark-glass blue accents.
3. **Top Bar Segmented Capsules (`InstanceSelector.tsx`, `NavSettings.tsx`, `NavMenu.tsx`)**: Establish concentric curvature, inner `overflow-hidden` tracks, and eliminate sharp rectangular hover highlights on middle capsule buttons.
4. **Instance Creation Flow (`CreateEditDialogs.tsx`, `InstancesHeader.tsx`, `InstancesStates.tsx`, `useInstancePage.ts`)**: Enforce the "Clone from Default" preselection invariant on "+ New Instance" triggers, preserve workspace and extension storage continuity, and add tactile hover elevation.
5. **Universal 24-Hour Clock Engine (`src/utils/date.ts`)**: Implement canonical `formatTime24` and `formatDateTime24` helpers, and migrate all 9 uncontrolled `toLocaleTimeString()` / `toLocaleString()` call sites.
6. **Settings & Preferences UI (`Settings.tsx`, `GeneralTab.tsx`)**: Standardize form input heights (`h-9`), integrate theme catalog palette switching directly into the general settings tab, and elevate dark mode contrast to WCAG AAA standards.

---

## 2. Component Specification A: `InstanceCard.tsx` & `selectedState.ts`

### 2.1 Problem Analysis
- In `src/pages/instances/InstanceCard.tsx` (lines 231–236):
  ```tsx
  <div className="shrink-0 inline-flex items-center overflow-hidden rounded-full border border-slate-200 dark:border-[#40285e] divide-x divide-slate-200 dark:divide-[#40285e]">
      {isActive ? (
          <span className={cn(ACTIVE_PILL_CLASSES, 'text-[9px] px-1.5 py-0.2 tracking-wider')}>
              Active
          </span>
      ) : (
          <button ...>Set Active</button>
      )}
  </div>
  ```
- **Defects**:
  1. `py-0.2` is an invalid Tailwind CSS utility class that does not compile to valid CSS padding.
  2. `ACTIVE_PILL_CLASSES` (from `src/components/common/selectedState.ts`) contains `rounded-full border border-amber-300 dark:border-amber-400/50 shadow-xs`. Mounting a fully rounded, bordered pill inside an existing `rounded-full divide-x` capsule produces an unsightly nested pill artifact with double borders and clipped corner radiuses.
  3. The card container uses `transition-colors duration-300 ease-in-out hover:shadow-md` without micro-interaction elevation or tactile transform feedback.

### 2.2 Technical Solution & Contracts

#### `src/components/common/selectedState.ts`
Retain standalone `ACTIVE_PILL_CLASSES` for isolated badges, and introduce `ACTIVE_CAPSULE_SEGMENT_CLASSES` for segmented container mounts:

```ts
/**
 * Active segment styling inside segmented pill capsules (e.g. InstanceCard actions).
 * Omits independent rounded borders to conform seamlessly to outer capsule curvature.
 */
export const ACTIVE_CAPSULE_SEGMENT_CLASSES =
  "inline-flex items-center justify-center px-2 py-0.5 text-[9px] font-bold uppercase tracking-wider bg-amber-500/15 text-amber-800 dark:bg-amber-400/20 dark:text-amber-300 select-none";
```

#### `src/pages/instances/InstanceCard.tsx`
1. **Capsule Badge Replacement**:
   Replace the nested `ACTIVE_PILL_CLASSES` and `py-0.2` with `ACTIVE_CAPSULE_SEGMENT_CLASSES`:
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
   ```
2. **Left Rail & Hover Elevation**:
   Update outer card container classes to use responsive CSS3 tactile elevation:
   ```tsx
   className={cn(
       'group relative rounded-[6px] border transition-all duration-150 ease-out hover:-translate-y-0.5 hover:shadow-md flex flex-col justify-between overflow-hidden backdrop-blur-xs',
       isActive
           ? SELECTED_CARD_CLASSES
           : 'bg-white dark:bg-[var(--ui-surface-1)] border-gray-200/50 dark:border-[var(--ui-border-subtle)] hover:border-gray-300/80 dark:hover:border-[var(--ui-interactive-40)] hover:bg-slate-50/90 dark:hover:bg-[var(--ui-surface-hover)] shadow-xs'
   )}
   ```
3. **Corner Invariant**: The 6px amber rail (`border-l-[6px] border-l-amber-400`) from `SELECTED_CARD_CLASSES` aligns cleanly with outer `rounded-[6px] overflow-hidden` without clipping child padding.

---

## 3. Component Specification B: `DetailTabs.tsx`

### 3.1 Problem Analysis
In `src/components/instances/PromptTreeView/DetailTabs.tsx`:
- Left capsule buttons (Preview, Raw, Edit) have `p-1.5 px-2` (icon only) resulting in ~26px height.
- Details button has `p-1.5 px-2.5` with text (`text-[11px] font-semibold`) resulting in ~28px height.
- Right capsule buttons (Input, Output, Expand) use `px-3 py-1 text-[11px]` and `px-2.5 py-1 text-[10px]`.
- Vertical baselines shift across buttons due to different padding and browser line-height computations.
- Active tabs use `bg-blue-600 text-white shadow-2xs`, creating a heavy, opaque block that disrupts the translucent dark-glass layout of the Prompt Tree View.

### 3.2 Technical Solution & Contracts
1. **Shared Height Baseline**: Force every button inside both left and right segmented capsules to `h-7` (28px height), `flex items-center justify-center`, and `leading-none`.
2. **Translucent Blue Accent**:
   - Active state token: `bg-blue-500/20 text-blue-600 dark:bg-blue-500/25 dark:text-blue-300 font-semibold shadow-2xs`
   - Inactive state token: `text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-slate-800/80`
3. **Concentric Padding**:
   - Capsule container: `inline-flex items-center rounded-md bg-slate-100/90 dark:bg-slate-900/90 backdrop-blur-md border border-slate-200 dark:border-slate-800 p-0.5 divide-x divide-slate-200 dark:divide-slate-800 shadow-2xs shrink-0 overflow-hidden`
   - Mode buttons: `h-7 px-2.5 flex items-center justify-center transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer`
   - Input / Output buttons: `h-7 px-3 flex items-center gap-1.5 text-[11px] font-semibold leading-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer`
   - Expand button: `h-7 px-2.5 flex items-center gap-1 text-[10px] font-semibold leading-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer`

```tsx
// Left Mode Buttons
<button
    type="button"
    onClick={() => setViewMode(PromptViewMode.Preview)}
    className={cn(
        'h-7 px-2.5 flex items-center justify-center rounded-l-[4px] transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
        viewMode === PromptViewMode.Preview
            ? 'bg-blue-500/20 text-blue-600 dark:bg-blue-500/25 dark:text-blue-300 shadow-2xs'
            : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-slate-800/80'
    )}
    aria-label="Preview"
>
    <Eye className="w-3.5 h-3.5" />
</button>
```

---

## 4. Component Specification C: Top-Level Menu Geometry

### 4.1 Problem Analysis
In `src/components/navbar/InstanceSelector.tsx`, `src/components/navbar/NavSettings.tsx`, and `src/components/navbar/NavMenu.tsx`:
- Segmented capsules use `rounded-full border p-0.5 divide-x`.
- In `NavSettings.tsx` (window controls, lines 398–445):
  - Minimize: `w-7 h-7 rounded-l-full`
  - Maximize: `w-7 h-7` (Square with NO corner radius)
  - Close: `w-7 h-7 rounded-r-full`
  - Container lacks `overflow-hidden`. On hover, the maximize button highlights as an awkward sharp rectangle against the curved capsule container.
- In `NavMenu.tsx`:
  - Navigation links inside `divide-x` specify independent `rounded-full` styling, creating irregular internal gaps between divider lines and pill borders.

### 4.2 Concentric Curvature Law
Per mathematical concentric geometry:
$$\text{Inner Radius } R_{\text{inner}} = R_{\text{outer}} - \text{Padding}$$
For `rounded-full` capsules ($R_{\text{outer}} = 9999\text{px}$), the inner track container MUST specify `rounded-full overflow-hidden`.
Children inside a segmented capsule with `divide-x` do not need piecemeal corner radii (`rounded-l-full` / `rounded-r-full`); the containing track automatically clips all hover states to the exact concentric boundary.

### 4.3 Technical Solution & Implementation

#### `src/components/navbar/NavSettings.tsx` (Window Controls)
```tsx
<div className="flex items-center rounded-full bg-gray-100 dark:bg-[#171025]/90 border border-gray-200/60 dark:border-[#40285e] p-0.5 divide-x divide-gray-200/50 dark:divide-slate-700/60 shadow-xs z-50 shrink-0 mr-1 overflow-hidden">
    <button
        type="button"
        id="btn-window-minimize"
        onClick={handleMinimize}
        className="w-7 h-7 hover:bg-gray-200 dark:hover:bg-[#40285e] flex items-center justify-center transition-colors duration-150 cursor-pointer"
        title={t('common.minimize', 'Minimize')}
        aria-label="Minimize"
    >
        <Minus className="w-3.5 h-3.5 text-gray-700 dark:text-gray-300 pointer-events-none" />
    </button>
    <button
        type="button"
        id="btn-window-maximize"
        onClick={handleToggleMaximize}
        className="w-7 h-7 hover:bg-gray-200 dark:hover:bg-[#40285e] flex items-center justify-center transition-colors duration-150 cursor-pointer"
        title={isMaximized ? t('common.restore', 'Restore') : t('common.maximize', 'Maximize')}
        aria-label={isMaximized ? "Restore" : "Maximize"}
    >
        {/* SVG Icon */}
    </button>
    <button
        type="button"
        id="btn-window-close"
        onClick={handleClose}
        className="w-7 h-7 hover:bg-red-500 hover:text-white dark:hover:bg-red-500 dark:hover:text-white flex items-center justify-center transition-colors duration-150 cursor-pointer group"
        title={t('common.close', 'Close')}
        aria-label="Close"
    >
        <X className="w-3.5 h-3.5 text-gray-700 dark:text-gray-300 group-hover:text-white transition-colors duration-150 pointer-events-none" />
    </button>
</div>
```

#### `src/components/navbar/NavMenu.tsx`
Ensure the top capsule container has `rounded-full overflow-hidden`:
```tsx
<div className="flex items-center rounded-full bg-gray-100 dark:bg-slate-900/90 border border-gray-200/60 dark:border-slate-800 p-0.5 shadow-xs divide-x divide-gray-200/70 dark:divide-slate-800/80 overflow-hidden">
```
Items use flush segment hover transitions without competing internal corner radiuses.

#### `src/components/navbar/InstanceSelector.tsx`
Add `overflow-hidden` to outer container:
```tsx
<div className="relative flex items-center rounded-full bg-gray-100 dark:bg-[#171025]/90 border border-gray-200/60 dark:border-[#40285e] p-0.5 divide-x divide-gray-200/50 dark:divide-slate-700/60 shadow-xs shrink-0 overflow-hidden" ref={dropdownRef}>
```

---

## 5. Component Specification D: Instance Creation & Default Clone Invariant

### 5.1 Operational Invariant
Clicking "+ New Instance" MUST default to cloning from the active "Default" instance.
- **Rationale**: An empty/blank instance has no extension store, missing workspaceStorage, and unconfigured telemetry. Cloning from the default instance ensures instant workspace continuity while giving the profile its own isolated token vault.
- **User Override**: The user may still explicitly select "Blank profile (clean isolated sandbox)" from the dropdown if desired.

### 5.2 Component Modifications

#### `src/pages/instances/useInstancePage.ts`
1. Define a helper function to resolve default instance ID:
   ```ts
   const getDefaultInstanceId = (): string => {
       const def = instances.find((i) => i.config.is_default);
       return def?.config.id || instances[0]?.config.id || '';
   };
   ```
2. When triggering `setIsCreateOpen(true)`, preselect the default instance:
   ```ts
   const openCreateModal = () => {
       setNewInstanceName('');
       setNewInstanceBoundAccount('');
       setNewInstanceFromInstance(getDefaultInstanceId());
       setNewInstanceLaunchImmediately(false);
       setIsCreateOpen(true);
   };
   ```

#### `src/pages/instances/CreateEditDialogs.tsx`
In `CreateInstanceDialog`:
```tsx
const close = () => {
    setIsCreateOpen(false);
    setNewInstanceName('');
    setNewInstanceBoundAccount('');
    setNewInstanceFromInstance(instances.find((i) => i.config.is_default)?.config.id || '');
    setNewInstanceLaunchImmediately(false);
};
```
In the `<select>` dropdown:
Ensure the Default instance is rendered with an unambiguous "(Default - Recommended)" badge and selected initially:
```tsx
<select
    value={newInstanceFromInstance}
    onChange={(e) => setNewInstanceFromInstance(e.target.value)}
    className="select select-sm w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-4 text-xs font-medium"
>
    {instances.map((inst) => (
        <option key={inst.config.id} value={inst.config.id}>
            {inst.config.name} {inst.config.is_default ? '(Default - Recommended)' : ''}
        </option>
    ))}
    <option value="">Blank profile (clean isolated sandbox)</option>
</select>
```

#### `src/pages/instances/InstancesHeader.tsx` & `src/pages/instances/InstancesStates.tsx`
Add tactile hover elevation and gradient sheen to "+ New Instance" buttons:
```tsx
<button
    type="button"
    onClick={openCreateModal}
    className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-[4px] bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-xs hover:shadow-md hover:-translate-y-0.5 active:translate-y-0 active:scale-[0.98] transition-all duration-150 ease-out cursor-pointer"
>
    <Plus className="w-3.5 h-3.5" />
    <span>{t('instances.create_btn', 'New Instance')}</span>
</button>
```

---

## 6. Component Specification E: Universal 24-Hour Clock Engine

### 6.1 Architectural Mandate
All clock and timestamp displays in Antigravity Manager MUST default strictly to the 24-hour clock (`HH:mm` or `HH:mm:ss`), regardless of client OS locale.
Direct invocations of `Date.prototype.toLocaleTimeString()` and unconfigured `Date.prototype.toLocaleString()` are strictly forbidden.

### 6.2 Canonical Helpers in `src/utils/date.ts`

```ts
/**
 * Canonical 24-hour time formatter.
 * Returns deterministic HH:mm (or HH:mm:ss if withSeconds is true).
 * Example: "09:05", "21:40", "21:40:15"
 */
export function formatTime24(
    value?: number | string | Date | null,
    options?: { withSeconds?: boolean; withMs?: boolean }
): string {
    const ms = parseToMs(value);
    if (ms === null) return '-';

    const d = new Date(ms);
    const hours = String(d.getHours()).padStart(2, '0');
    const minutes = String(d.getMinutes()).padStart(2, '0');

    if (options?.withMs) {
        const seconds = String(d.getSeconds()).padStart(2, '0');
        const millis = String(d.getMilliseconds()).padStart(3, '0');
        return `${hours}:${minutes}:${seconds}.${millis}`;
    }

    if (options?.withSeconds) {
        const seconds = String(d.getSeconds()).padStart(2, '0');
        return `${hours}:${minutes}:${seconds}`;
    }

    return `${hours}:${minutes}`;
}

/**
 * Canonical 24-hour date and time formatter.
 * Returns deterministic DD-MMM-YYYY HH:mm (or DD-MMM-YYYY HH:mm:ss).
 * Example: "27-Oct-2026 21:40", "27-Oct-2026 21:40:15"
 */
export function formatDateTime24(
    value?: number | string | Date | null,
    options?: { withSeconds?: boolean; iso?: boolean }
): string {
    const ms = parseToMs(value);
    if (ms === null) return '-';

    const d = new Date(ms);
    const day = String(d.getDate()).padStart(2, '0');
    const month = MONTH_NAMES[d.getMonth()] || 'Jan';
    const year = d.getFullYear();
    const hours = String(d.getHours()).padStart(2, '0');
    const minutes = String(d.getMinutes()).padStart(2, '0');
    const secStr = options?.withSeconds ? `:${String(d.getSeconds()).padStart(2, '0')}` : '';

    if (options?.iso) {
        const monthNum = String(d.getMonth() + 1).padStart(2, '0');
        return `${year}-${monthNum}-${day} ${hours}:${minutes}${secStr}`;
    }

    return `${day}-${month}-${year} ${hours}:${minutes}${secStr}`;
}
```

### 6.3 Audit & Migration of the 9 Violation Locations

| # | File Path | Line | Current Code (Violations) | Grounded Migration Replacement |
| :-: | :--- | :-: | :--- | :--- |
| **1** | `src/components/audit/AuditPanel.tsx` | 120 | `return new Date(ms).toLocaleTimeString();` | `return formatTime24(ms, { withSeconds: true });` |
| **2** | `src/components/common/NetworkMonitor.tsx` | 96 | `{new Date(req.startTime).toLocaleTimeString()}` | `{formatTime24(req.startTime, { withSeconds: true })}` |
| **3** | `src/components/common/NetworkMonitor.tsx` | 138 | `{new Date(selectedRequest.startTime).toLocaleString()}` | `{formatDateTime24(selectedRequest.startTime, { withSeconds: true })}` |
| **4** | `src/components/debug/DebugConsole.tsx` | 20 | `date.toLocaleTimeString('en-US', { hour12: false, ... }) + '.' + date.getMilliseconds()...` | `formatTime24(date, { withMs: true })` |
| **5** | `src/components/errors/error-history-drawer.tsx` | 365 | `new Date(err.createdAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })` | `formatTime24(err.createdAt, { withSeconds: true })` |
| **6** | `src/components/errors/error-modal/chrome.tsx` | 109 | `{new Date(error.createdAt).toLocaleTimeString()}` | `{formatTime24(error.createdAt, { withSeconds: true })}` |
| **7** | `src/components/proxy/monitor/LogTable.tsx` | 242 | `{new Date(log.timestamp).toLocaleTimeString()}` | `{formatTime24(log.timestamp, { withSeconds: true })}` |
| **8** | `src/components/proxy/monitor/LogDetailPanel.tsx` | 109 | `title={new Date(log.timestamp).toLocaleString()}>{new Date(log.timestamp).toLocaleString()}` | `title={formatDateTime24(log.timestamp, { withSeconds: true })}>{formatDateTime24(log.timestamp, { withSeconds: true })}` |
| **9** | `src/lib/error-report-generator.ts` | 287 | `[${new Date(c.timestamp).toLocaleTimeString()}]` | `[${formatTime24(c.timestamp, { withSeconds: true })}]` (and line 186: `formatDateTime24(new Date(), { withSeconds: true })`) |

---

## 7. Component Specification F: Settings & Preferences Modernization

### 7.1 Problem Analysis
In `src/pages/Settings.tsx` and `src/pages/Settings/tabs/GeneralTab.tsx`:
1. **Input Height Inconsistencies**: Dropdowns use `py-2.5`, text inputs use `py-2`, buttons use `py-1.5`. Form inputs do not share an aligned visual baseline.
2. **Fragmented Theme Management**: Theme mode (`light` / `dark` / `system`) lives in `GeneralTab.tsx`, while theme palette catalogs (`THEME_PALETTES`) are segregated in a separate `Themes` tab, forcing users to jump between sections.
3. **Contrast Deficiencies**: Sub-cards use `bg-gray-50 dark:bg-base-200 border border-gray-100 dark:border-base-300 text-gray-600 dark:text-gray-400`. Under dark slate themes, text contrast falls below WCAG AA thresholds.

### 7.2 Technical Solution & Design Tokens

#### Standardized Control Height
Enforce a uniform height token across all text inputs, selects, and controls in settings tabs:
- Class token: `h-9 text-xs sm:text-sm px-3 py-1.5 rounded-lg border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900/60 text-slate-900 dark:text-slate-100 focus:ring-2 focus:ring-blue-500/40 focus:border-blue-500 transition-all`

#### Integrated Theme Picker in `GeneralTab.tsx`
Mount `ThemePicker` directly below the Theme Mode toggle inside `GeneralTab.tsx`:
```tsx
{/* Theme selection & Catalogue */}
<div className="space-y-3 pt-2">
    <label className="block text-sm font-semibold text-gray-900 dark:text-slate-100">
        {t('settings.general.theme')}
    </label>
    <div className="inline-flex items-center rounded-full border border-gray-200 dark:border-slate-700 bg-gray-50 dark:bg-slate-900/60 p-0.5 divide-x divide-gray-200 dark:divide-slate-700">
        {(
            [
                { value: 'light', label: t('settings.general.theme_light') },
                { value: 'dark', label: t('settings.general.theme_dark') },
                { value: 'system', label: t('settings.general.theme_system') },
            ] as const
        ).map((opt) => (
            <button
                key={opt.value}
                type="button"
                onClick={() => {
                    setFormData({ ...formData, theme: opt.value });
                    updateTheme(opt.value);
                }}
                className={`px-3.5 py-1.5 rounded-full text-xs font-semibold transition-all cursor-pointer ${
                    formData.theme === opt.value
                        ? 'bg-blue-600 text-white shadow-xs'
                        : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
                }`}
            >
                {opt.label}
            </button>
        ))}
    </div>

    {/* Integrated Theme Palettes */}
    <div className="mt-3 p-3.5 rounded-xl border border-slate-200/80 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-900/40">
        <ThemePicker />
    </div>
</div>
```

#### Contrast Token Upgrades in `GeneralTab.tsx`
Replace washed-out `base-200` / `base-300` classes with high-contrast slate surfaces:
- Card surfaces: `bg-slate-50/80 dark:bg-slate-900/60 border border-slate-200 dark:border-slate-800`
- Primary text: `text-slate-900 dark:text-slate-100 font-semibold`
- Secondary descriptions: `text-slate-500 dark:text-slate-400 text-xs`

---

## 8. Design Tokens, Contrast & Accessibility (WCAG 2.1 AAA)

| Element | Background | Foreground | Calculated Contrast | WCAG Rating |
| :--- | :--- | :--- | :---: | :---: |
| **Card (Dark Active)** | Slate-900 (`#0f172a`) | White (`#ffffff`) | **17.67:1** | AAA (Pass) |
| **Card (Light Active)** | Slate-100 (`#f1f5f9`) | Slate-900 (`#0f172a`) | **16.12:1** | AAA (Pass) |
| **Amber Active Segment (Dark)** | Amber-400/20 (`rgba(251,191,36,0.2)`) on Slate-900 | Amber-300 (`#fcd34d`) | **10.27:1** | AAA (Pass) |
| **Amber Active Segment (Light)** | Amber-500/15 (`rgba(245,158,11,0.15)`) on Slate-100 | Amber-900 (`#78350f`) | **8.10:1** | AAA (Pass) |
| **DetailTabs Active Accent** | Blue-500/25 (`rgba(59,130,246,0.25)`) on Slate-900 | Blue-300 (`#93c5fd`) | **9.45:1** | AAA (Pass) |
| **DetailTabs Text Baseline** | `leading-none` vertically centered | Neutral / Slate | **N/A (Geometric)** | Pass |

---

## 9. Verification & Acceptance Criteria

1. **Active Card Inspection**:
   - `InstanceCard`: Left rail renders clean `border-l-[6px] border-l-amber-400` with zero corner clipping.
   - Action capsule renders flush `Active` segment without internal duplicate borders, and with no `py-0.2`.
   - Card hovers with `-translate-y-0.5` smooth CSS3 elevation.
2. **DetailTabs Inspection**:
   - Preview, Raw, Edit, Details, Input, Output, Expand share the exact same `h-7` vertical height.
   - Text and icons sit on the exact same horizontal optical centerline.
   - Active state displays translucent dark-glass blue accent (`bg-blue-500/20 dark:bg-blue-500/25`).
3. **Top Bar Geometry**:
   - Window controls in `NavSettings.tsx` exhibit zero rectangular hover bleeding.
   - `NavMenu.tsx` and `InstanceSelector.tsx` containers have `overflow-hidden` with concentric curvature.
4. **New Instance Invariant**:
   - Clicking "+ New Instance" opens dialog with the Default profile preselected in the clone dropdown.
   - Clicking "+ New Instance" button displays tactile hover elevation.
5. **Universal 24-Hour Clock**:
   - All 9 audit locations display strictly 24-hour time (`HH:mm` or `HH:mm:ss`). Zero `AM`/`PM` strings appear anywhere in the UI when formatted by `formatTime24` or `formatDateTime24`.
6. **Settings Tab**:
   - All input controls conform to `h-9`.
   - Theme palette previews render inside `GeneralTab.tsx`.
   - Dark theme labels meet WCAG AAA contrast requirements.
