# Master Specification: Switch Account Modal Color, Search & UI/UX Enhancements

- **Spec ID**: 232-account-switching-color-search-and-ui-ux-enhancements
- **Category**: Frontend UI/UX, Theme Engine & Design System
- **Status**: ACTIVE / IN PROGRESS
- **Author**: Antigravity Lead Orchestrator
- **Target File**: `src/pages/instances/SwitchAccountDialog.tsx`

---

## 1. Executive Summary & Root Cause Analysis

### 1.1 Problem Statement
When inspecting the Switch Account modal (`SwitchAccountDialog.tsx`) under dark mode, users experienced a catastrophic UI contrast breakdown:
1. The modal container rendered with a bright white background (`bg-white/95`) instead of the Royal Violet dark-glass surface (`#171025`).
2. Inside the white container, typography configured for dark mode (`dark:text-white`, `dark:text-slate-100`, `dark:text-slate-300`) rendered as white/light gray on white, making account names, titles, and quota metrics completely illegible.
3. The search bar input rendered with dark background (`dark:bg-[var(--ui-canvas)]`), clashing harshly against the accidental white background.
4. The account count label (`46 accounts`) floated without proper visual anchoring.
5. The action "Select" button lacked cohesive tactile micro-interactions and consistent pill encapsulation.

### 1.2 Root Cause Analysis (IACVT in Chromium CSS Engine)
- In `SwitchAccountDialog.tsx`, commit updates introduced Tailwind arbitrary opacity syntax on CSS variables containing hex codes:
  - Line 187: `dark:bg-[var(--ui-surface-1)]/95`
  - Line 212: `dark:bg-[var(--ui-canvas)]/80`
  - Line 333: `dark:bg-[var(--ui-surface-1)]/80`
  - Line 334: `dark:bg-[var(--ui-surface-1)]/80`
  - Line 383: `dark:bg-[var(--ui-canvas)]/80`
- Under W3C CSS Color Module 4, Tailwind translates `<class>/<opacity>` into `rgb(var(...) / <alpha>)`.
- Because `--ui-surface-1` and `--ui-canvas` in `ui-tokens.css` store 6-digit hex color strings (`#171025`, `#090611`), Chromium evaluates `rgb(#171025 / 0.95)` as **Invalid At Computed-Value Time (IACVT)**.
- Per W3C rules, the browser discards the invalid declaration and falls back to the light-theme default (`bg-white/95`, `bg-white/80`, `bg-slate-100/80`), producing a stark white modal while dark text styles remain active.

---

## 2. Technical Remediation Specifications

### 2.1 CSS Variable Opacity Elimination & Solid Dark Tokens
All invalid slash-opacity modifiers on CSS variables MUST be replaced with solid theme variables or explicit dark-glass tokens:
1. **Modal Base Container**:
   - Before: `dark:bg-[var(--ui-surface-1)]/95 dark:border-[var(--ui-border-subtle)]`
   - After: `bg-white dark:bg-[#171025] border border-slate-200/90 dark:border-[#40285e] shadow-2xl rounded-2xl`
2. **Account Row Cards**:
   - Before: `bg-white/80 dark:bg-[var(--ui-surface-1)]/80 border-slate-200/90 dark:border-[var(--ui-border-subtle)]`
   - After: `bg-white dark:bg-[#1f1533] border border-slate-200/80 dark:border-[#40285e]/70 hover:bg-slate-50 dark:hover:bg-[#281b42] hover:border-purple-400 dark:hover:border-[#7c3aed]`
3. **Current Account Highlight Row**:
   - After: `bg-emerald-500/10 dark:bg-emerald-950/30 border-emerald-400/50 dark:border-emerald-500/40`
4. **Segmented Metric Capsules (Header & Card)**:
   - Before: `dark:bg-[var(--ui-canvas)]/80 dark:border-[var(--ui-border-subtle)]`
   - After: `bg-slate-100/90 dark:bg-[#090611] border border-slate-200 dark:border-[#40285e]/80`

### 2.2 Search & Filter Input Ergonomics
1. **Search Input Container**:
   - Use `w-full pl-9 pr-8 py-2 text-xs bg-slate-50 dark:bg-[#090611] border border-slate-200/90 dark:border-[#40285e] rounded-full focus:outline-none focus:ring-1 focus:ring-[#7c3aed] text-slate-900 dark:text-slate-100 placeholder:text-slate-400 dark:placeholder:text-slate-500 transition-colors shadow-inner`
2. **Account Counter Pill**:
   - Enclose the count in a neat segmented capsule: `px-2.5 py-1 rounded-full bg-slate-100 dark:bg-[#090611] border border-slate-200/80 dark:border-[#40285e]/80 text-[11px] font-medium text-slate-600 dark:text-purple-300 font-mono`

### 2.3 Account Row & Button Modernization
1. **Subscription Tier Badges**:
   - PRO: `bg-sky-500/15 text-sky-600 dark:text-sky-400 border border-sky-500/30 font-bold text-[9px] px-1.5 py-0.5 rounded shadow-2xs`
   - ULTRA: `bg-purple-500/15 text-purple-600 dark:text-purple-400 border border-purple-500/30 font-bold text-[9px] px-1.5 py-0.5 rounded shadow-2xs`
   - FREE: `bg-slate-500/15 text-slate-600 dark:text-slate-400 border border-slate-500/30 font-bold text-[9px] px-1.5 py-0.5 rounded shadow-2xs`
2. **Tactile Select Action Button**:
   - Enforce segmented capsule wrapping with `.btn-tactile` micro-interactions:
     `px-4 py-1.5 rounded-full text-xs font-semibold bg-[#7c3aed] hover:bg-[#6d28d9] text-white shadow-xs hover:shadow-md transition-all active:scale-[0.98]`
