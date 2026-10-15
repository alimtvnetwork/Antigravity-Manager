# Subtask Plan 01: UI Components & Browser-Based Builder Tool

**Document ID:** `.ai-memory/plans/subtasks/ui-and-ux-enhancements-for-antigravity-manager/01-ui-components-and-builder.md`  
**Parent Task:** `ui-and-ux-enhancements-for-antigravity-manager`  
**Parent Specs:**
- Architecture Specification: [01-architecture-spec.md](../../../../02-spec/21-app/ui-and-ux-enhancements-for-antigravity-manager/01-architecture-spec.md)
- Progress Bar Specification: [03-progress-bar-spec.md](../../../../02-spec/21-app/ui-and-ux-enhancements-for-antigravity-manager/03-progress-bar-spec.md)
**Status:** READY FOR EXECUTION  
**Version:** 1.0.0  
**Updated:** 2026-10-15  
**Target Release:** v4.200.0  

---

## 1. Context & Objectives

This subtask executes the foundational UI/UX refinements identified in the architecture and progress bar specifications. It directly targets window caption clipping, navbar space bloat, silent dropdown selections, modal backdrop header overlap, and progress bar milestone legibility, while establishing a zero-dependency browser-based prototyping toolchain.

### Core Objectives:
1. **TitleBar Close Button Inset & Segmented Capsule:**
   - Eliminate the window caption close button cut-off by introducing a 10px safe-area right inset (`pr-2.5`).
   - Wrap Windows/Linux caption controls into a unified dark-glass segmented pill capsule (`rounded-full`, `divide-x`, shared border) adhering strictly to `AGENTS.md`.
2. **NavSettings Icon-Only Compacting & Micro-Triangle Trigger:**
   - Reclaim over 120px of horizontal navbar space by stripping verbose text strings from top-level navigation buttons.
   - Refactor `NavSettings.tsx` into a 3-slot segmented capsule: Quick Clean (`RotateCcw`), Theme Catalogue (`Palette` with live gradient swatch), and Language (`EN` with micro-triangle caret `▼`).
3. **Dropdown Update Tactile Feedback Engine:**
   - Provide instant in-item visual confirmation (illuminated checkmark icon and accent background pulse) upon selecting a theme or language.
   - Enforce a 180ms glance delay before auto-dismissal, coupled with active error reporting via `useErrorStore` if persistence fails.
4. **Modal Z-Index Priority Hierarchy & Sticky Header Overlap Fix:**
   - Enforce universal React DOM portaling (`createPortal(..., document.body)`) across all dialogs and drawers.
   - Establish the canonical 8-tier z-index ladder (`z-sticky`: 100, `z-modal`: 10,000, `z-sys-modal`: 50,000, `z-critical`: 99,999).
   - Remove `isolation: 'isolate'` from `Navbar.tsx` and ensure modal backdrops cover the entire viewport, completely eliminating sticky header overlap.
5. **QuotaProgressBar Overhaul:**
   - Slim track height to exactly 10px (`h-2.5`).
   - Implement 20px diameter protruding milestone nodes (`w-5 h-5`) extending 5px above and 5px below the track.
   - Render illuminated checkmark (`v`) icons inside filled nodes.
   - Strictly eliminate circular dots at 0% and 100% boundary ends.
   - Enforce exactly 5 internal milestone checkpoints (`20%`, `40%`, `60%`, `80%`, `90%`).
   - Implement the 6-stop spatial gradient from dark red at 0% to neon green at 100%.
   - Configure 80–85% resting opacity with 100% brightness and ambient neon glow on table row hover.
6. **Browser-Based UI Builder Tool (`tools/ui-builder/index.html`):**
   - Author a standalone, zero-dependency HTML5/CSS3/ES6 interactive prototyping workbench for inspecting, tuning, and verifying UI components in isolation without launching the native desktop Tauri backend.

---

## 2. File Ownership Map

This subtask owns the modifications across the following 8 target files:

| File Path | Component / Layer | Role & Scope of Changes |
| :--- | :--- | :--- |
| `src/components/layout/TitleBar.tsx` | Desktop Window Caption | Add safe-area right inset (`pr-2.5`), wrap caption controls into a dark-glass segmented pill capsule, maintain macOS traffic lights parity. |
| `src/components/navbar/Navbar.tsx` | Main Application Header | Standardize sticky header z-index (`z-[100]`), remove `isolation: 'isolate'`, refine horizontal flex spacing between zones. |
| `src/components/navbar/NavSettings.tsx` | Top Navigation Utilities | Refactor into an icon-only 3-slot segmented capsule, implement micro-triangle caret trigger for languages, add dropdown selection checkmark feedback. |
| `src/components/common/ModalDialog.tsx` | Universal Modal Primitive | Enforce `z-[10000]` / `z-[50000]`, verify full-viewport backdrop isolation, intercept all click events to prevent header bleed-through. |
| `src/components/accounts/QuotaProgressBar.tsx` | Quota Visualization | Implement 10px track (`h-2.5`), 20px protruding nodes with checkmarks, eliminate boundary dots, 5 internal splits, 80-85% resting opacity with hover glow. |
| `src/components/common/WaterDrainProgressBar.tsx` | Water Drain Progress | Synchronize dimensional geometry, 10px track height, 20px protruding nodes, and spatial gradient with `QuotaProgressBar`. |
| `src/styles/ui-tokens.css` | Design Tokens | Declare `--ui-quota-gradient`, resting and hover glow tokens, and modal z-index custom properties. |
| `tools/ui-builder/index.html` | Developer Prototyping Tool | New standalone browser-based interactive workbench containing progress bar scrubber, segmented capsule previewer, and modal stacking simulator. |

---

## 3. Step-by-Step Implementation Tasks

### Phase 1: TitleBar Window Controls & Safe-Area Inset Fix

- [ ] **1.1 Analyze and Apply Safe-Area Inset in `TitleBar.tsx`:**
  - Locate the Windows/Linux caption controls wrapper:
    `className="flex items-center h-full"`
  - Refactor to include an explicit right safe-area inset:
    `className="flex items-center h-full pr-2.5 sm:pr-3"`
  - Verify that the close button is positioned at least 10px away from the extreme viewport boundary, completely preventing clipping from Windows 11 DWM rounded corners.
- [ ] **1.2 Wrap Caption Controls into a Contiguous Segmented Pill Capsule:**
  - Wrap `#titlebar-btn-minimize`, `#titlebar-btn-maximize`, and `#titlebar-btn-close` into a single dark-glass capsule:
    ```tsx
    <div className="flex items-center rounded-full bg-gray-100/90 dark:bg-[#171025]/90 border border-gray-200/60 dark:border-[#40285e] p-0.5 divide-x divide-gray-200/50 dark:divide-slate-700/60 shadow-xs mr-1">
        <button id="titlebar-btn-minimize" className="w-8 h-6 rounded-l-full ...">...</button>
        <button id="titlebar-btn-maximize" className="w-8 h-6 ...">...</button>
        <button id="titlebar-btn-close" className="w-8 h-6 rounded-r-full hover:bg-[#E81123] hover:text-white ...">...</button>
    </div>
    ```
- [ ] **1.3 Preserve macOS Traffic Lights Parity:**
  - Ensure `isMac` branch retains its left-aligned layout (`pl-3 gap-2`) with native traffic light colors (`#FF5F56`, `#FFBD2E`, `#27C93F`).

---

### Phase 2: NavSettings Compacting & Micro-Triangle Caret Trigger

- [ ] **2.1 Convert NavSettings Trigger into an Icon-Only 3-Slot Capsule:**
  - Update container classes:
    `className="flex items-center rounded-full bg-gray-100 dark:bg-[#171025]/90 border border-gray-200/60 dark:border-[#40285e] p-0.5 shadow-xs"`
- [ ] **2.2 Slot 1: Quick Clean Icon Trigger:**
  - Render `RotateCcw` icon button with `w-7 h-7 rounded-l-full` and border divider.
  - Retain tooltips: `title={t('nav.quick_clean', 'Antigravity Cache & Retention Clean')}`.
- [ ] **2.3 Slot 2: Theme Catalogue Trigger (Swatch + Palette Icon):**
  - Render active theme gradient swatch (10px circle) and `Palette` icon (14px).
  - Remove all verbose text labels ("Theme", palette names) from the top-level trigger.
- [ ] **2.4 Slot 3: Language Selector with Micro-Triangle Trigger:**
  - Render 2-letter uppercase language code (`EN`, `ZH`, `JA`, `ES`, `DE`, `RU`, `AR`).
  - Replace bulky chevron with the standardized micro-triangle caret:
    ```tsx
    <svg 
        className={`w-2 h-2 text-gray-400 group-hover:text-gray-200 transition-transform duration-150 ${isPrefsOpen ? 'rotate-180' : ''}`} 
        viewBox="0 0 8 6" 
        fill="currentColor"
    >
        <path d="M0 1.5L4 5.5L8 1.5H0Z" />
    </svg>
    ```

---

### Phase 3: Dropdown Update Tactile Feedback Engine

- [ ] **3.1 Add Visual Selection Feedback in Theme Catalogue:**
  - When a user clicks a theme option in `NavSettings.tsx`:
    1. Render an animated checkmark (`Check` icon with `w-3.5 h-3.5 text-emerald-500 animate-in zoom-in-75 duration-100`).
    2. Apply active highlight styling to the clicked button (`bg-emerald-500/15 border-emerald-500/40`).
    3. Introduce a 180ms glance delay before closing `isThemeOpen`.
- [ ] **3.2 Add Visual Selection Feedback in Language Selector:**
  - Highlight selected language row with `bg-emerald-500/15 text-emerald-600 dark:text-emerald-400` and display `Check` icon.
  - Close menu after 180ms delay.
- [ ] **3.3 Error Interception (No Silent Errors):**
  - Verify `saveConfig` in `Navbar.tsx` routes errors to `useErrorStore.getState().captureError(err, { source: 'NavSettings.tsx', triggerAction: 'handleLanguageChange' })`.

---

### Phase 4: Modal Z-Index Priority Hierarchy & Sticky Header Overlap Fix

- [ ] **4.1 Standardize Navbar Z-Index in `src/components/navbar/Navbar.tsx`:**
  - Refactor sticky header styling:
    `style={{ position: 'sticky', top: 0, zIndex: 100 }}`
  - Remove `isolation: 'isolate'` from `Navbar.tsx` to prevent accidental stacking context traps.
- [ ] **4.2 Audit Modal Dialogs for Root DOM Portaling:**
  - Ensure all modals use `createPortal(modalContent, document.body)`.
  - Verify `ModalDialog.tsx` renders at `z-[10000]` with full-screen backdrop:
    `className="modal modal-open z-[10000] fixed inset-0 items-center justify-center p-4 overflow-y-auto bg-black/60 backdrop-blur-sm"`
- [ ] **4.3 Event Isolation & Header Shielding:**
  - Ensure modal backdrop covers `fixed inset-0` with `pointer-events-auto` and stops event propagation, preventing clicks from penetrating to the sticky navbar beneath it.
- [ ] **4.4 Standardize System & Critical Modals:**
  - Configure `AgyCleanModal` and `UnifiedBackupModal` to `z-[50000]`.
  - Configure `ErrorModal` to `z-[99999]`.

---

### Phase 5: QuotaProgressBar & WaterDrainProgressBar Overhaul

- [ ] **5.1 Update Dimensional Geometry in `src/components/accounts/QuotaProgressBar.tsx`:**
  - Update track height from `h-[18px]` to `h-2.5` (10px).
  - Update compact density track height to `h-2` (8px).
- [ ] **5.2 Overhaul Milestone Checkpoint Nodes:**
  - Update node diameter from `w-3.5 h-3.5` to `w-5 h-5` (20px).
  - Center nodes vertically with `top-1/2 -translate-y-1/2 -translate-x-1/2`.
  - Verify that each node protrudes 5px above and 5px below the 10px track.
- [ ] **5.3 Render SVG Checkmark (`v`) Glyphs Inside Nodes:**
  - Inside filled milestone nodes, render the inline checkmark SVG:
    ```tsx
    <svg className="w-3 h-3 text-white pointer-events-none" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M3.5 8.5L6.5 11.5L12.5 4.5" />
    </svg>
    ```
- [ ] **5.4 Filter Out Boundary End Nodes (0% and 100%):**
  - Enforce filtering in `QuotaProgressBar.tsx`:
    ```typescript
    const internalCheckpoints = (checkpoints || [20, 40, 60, 80, 90])
        .filter((cp) => cp > 0 && cp < 100)
        .slice(0, 5);
    ```
  - Verify zero beads render at the hard ends of the progress bar.
- [ ] **5.5 Implement Full-Width Spatial Gradient:**
  - Verify `linear-gradient(90deg, #7f1d1d 0%, #dc2626 15%, #ea580c 35%, #eab308 55%, #10b981 75%, #1af18d 100%)` spans the entire track.
  - Reveal filled proportion via width mask or inner clipped container.
- [ ] **5.6 Implement Resting Opacity & Table Row Hover Glow:**
  - Set default container opacity to `opacity-80` or `opacity-85`.
  - Bind to parent `group` hover (`group-hover:opacity-100 group-hover:filter group-hover:drop-shadow-[0_0_8px_rgba(26,241,141,0.7)]`).
- [ ] **5.7 Synchronize `src/components/common/WaterDrainProgressBar.tsx`:**
  - Apply identical 10px track, 20px protruding nodes, checkmark glyphs, and hover glow rules.

---

### Phase 6: Browser-Based UI Builder Prototyping Tool

- [ ] **6.1 Create `tools/ui-builder/index.html`:**
  - Author a standalone, zero-dependency HTML5 application with modern responsive layout.
- [ ] **6.2 Implement Interactive Progress Bar Workbench:**
  - Add real-time slider for quota percentage (0% to 100%).
  - Include live interactive progress bar matching all explicit IDs (`agm-progress-container`, `agm-progress-track`, `agm-progress-fill`, `agm-progress-node-1` through `agm-progress-node-5`, `agm-progress-glow`).
  - Add interactive toggle for table row hover glow simulation.
- [ ] **6.3 Implement Segmented Capsule & NavSettings Workbench:**
  - Render interactive TitleBar window controls preview with safe-area insets.
  - Render interactive NavSettings 3-slot capsule (Recycle, Swatch+Palette, Language with micro-triangle).
  - Include transparent borderless `<select>` dropdown testing area.
- [ ] **6.4 Implement Modal Stacking & Z-Index Simulator:**
  - Interactive layer stack demo: Sticky navbar at `z-100` versus modal dialog at `z-10000`.
  - Visual proof demonstrating zero header overlap when the modal is active.
- [ ] **6.5 Implement Code Snippet Exporter:**
  - Provide a one-click "Copy HTML/CSS" button allowing any developer or AI to extract production-ready code.

---

### Phase 7: Verification, Pre-Flight Gates & Invariants Audit

- [ ] **7.1 Verify Relative Paths Hygiene:**
  - Scan all authored specifications and code references to verify 100% relative Git paths.
  - Enforce zero absolute filesystem paths.
- [ ] **7.2 Verify No Git Commands Executed:**
  - Confirm that zero git commands (`git add`, `git commit`, `git status`, `git diff`) were invoked.
- [ ] **7.3 Local Pre-Flight Checks:**
  - Run frontend typecheck/build if required: `npm run build`.
  - Verify zero clippy or linter regressions.
- [ ] **7.4 Adherence to AGENTS.md:**
  - Confirm segmented pill capsules, transparent selects, and error tracking compliance.
