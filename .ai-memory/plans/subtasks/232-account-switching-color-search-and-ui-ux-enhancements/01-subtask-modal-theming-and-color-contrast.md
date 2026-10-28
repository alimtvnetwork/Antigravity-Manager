---
plan: 232-account-switching-color-search-and-ui-ux-enhancements
subtask: "01"
subtask_id: 232-subtask-01
title: Modal Theming and Color Contrast Remediation
domain: frontend-react
depends_on: none
citations:
  architecture_spec: 02-spec/21-app/232-account-switching-color-search-and-ui-ux-enhancements/01-architecture-spec.md
  coding_guidelines: .ai-memory/coding-guidelines.md
target_files:
  - src/pages/instances/SwitchAccountDialog.tsx
status: pending
---

# Subtask 01: Modal Theming and Color Contrast Remediation

- **Subtask ID**: `232-subtask-01`
- **Plan**: `232-account-switching-color-search-and-ui-ux-enhancements`
- **Target File**: `src/pages/instances/SwitchAccountDialog.tsx`
- **Spec Reference**: `02-spec/21-app/232-account-switching-color-search-and-ui-ux-enhancements/01-architecture-spec.md`

---

## 1. Context & Motivation

In the Switch Account dialog (`SwitchAccountDialog.tsx`), dark mode displays catastrophic white-on-white text and contrast breakdowns. 

### Root Cause Analysis (IACVT)
Under W3C CSS Color Module 4, Tailwind parses slash-opacity syntax (`<class>/<opacity>`) into `rgb(var(...) / <alpha>)`. Because `--ui-surface-1` and `--ui-canvas` in `ui-tokens.css` store 6-digit hex values (`#171025`, `#090611`), Chromium evaluates `rgb(#171025 / 0.95)` as **Invalid At Computed-Value Time (IACVT)**. As a result, the browser discards the dark mode rule and falls back to light-theme styling (`bg-white/95`), while typography remains configured for dark backgrounds (`dark:text-white`, `dark:text-slate-100`), resulting in illegible white-on-white text.

Additionally, arbitrary variable references with opacity modifiers or border variable fallbacks (`dark:border-[var(--ui-border-subtle)]`) cause inconsistent visual rendering.

To definitively solve this, all invalid variable opacity combinations and inconsistent border variables in `SwitchAccountDialog.tsx` must be converted to explicit, high-contrast dark-glass design tokens (`#171025`, `#1f1533`, `#090611`, `#40285e`).

---

## 2. Target Replacements Breakdown

The following replacements must be surgically applied to `src/pages/instances/SwitchAccountDialog.tsx`:

| Location | Element | Current Invalid / Broken Token | Remediated Solid Dark-Glass Token |
| :--- | :--- | :--- | :--- |
| **Line 187** | Modal container | `dark:bg-[var(--ui-surface-1)]/95 dark:border-[var(--ui-border-subtle)]` | `dark:bg-[#171025]/95 dark:border-[#40285e]` |
| **Line 191** | Header bottom border | `dark:border-[var(--ui-border-subtle)]` | `dark:border-[#40285e]` |
| **Line 212** | Bound quota capsule | `dark:bg-[var(--ui-canvas)]/80 dark:border-[var(--ui-border-subtle)]` | `dark:bg-[#090611]/80 dark:border-[#40285e]` |
| **Line 217, 222** | Header quota vertical dividers | `dark:bg-[var(--ui-border-subtle)]` | `dark:bg-[#40285e]` |
| **Line 238** | Header action capsule | `dark:bg-[var(--ui-canvas)] dark:border-[var(--ui-border-subtle)]` | `dark:bg-[#090611]/80 dark:border-[#40285e]` |
| **Line 248** | Header action capsule divider | `dark:bg-[var(--ui-border-subtle)]` | `dark:bg-[#40285e]` |
| **Line 333, 334** | Account card backgrounds & borders | `dark:bg-[var(--ui-surface-1)]/80 dark:border-[var(--ui-border-subtle)]` | `dark:bg-[#1f1533] dark:border-[#40285e]` |
| **Line 383** | Card quota metrics capsule | `dark:bg-[var(--ui-canvas)]/80 dark:border-[var(--ui-border-subtle)]` | `dark:bg-[#090611]/80 dark:border-[#40285e]` |
| **Line 388, 393** | Card quota metrics dividers | `dark:bg-[var(--ui-border-subtle)]` | `dark:bg-[#40285e]` |
| **Line 416** | Weekly quota divider | `dark:border-[var(--ui-border-subtle)]/50` | `dark:border-[#40285e]/50` |
| **Line 442** | Action button capsule | `dark:bg-[var(--ui-canvas)] dark:border-[var(--ui-border-subtle)]` | `dark:bg-[#090611]/80 dark:border-[#40285e]` |

---

## 3. Concrete Implementation Steps

### 3.1 Modal Shell & Header Container (Lines 187, 191)
- In the main dialog wrapper:
  ```tsx
  // Target: Line 187
  // Replace:
  className="backdrop-blur-md bg-white/95 dark:bg-[var(--ui-surface-1)]/95 border border-slate-200/90 dark:border-[var(--ui-border-subtle)] shadow-2xl rounded-2xl p-6 w-full max-w-lg max-h-[85vh] flex flex-col"
  // With:
  className="backdrop-blur-md bg-white/95 dark:bg-[#171025]/95 border border-slate-200/90 dark:border-[#40285e] shadow-2xl rounded-2xl p-6 w-full max-w-lg max-h-[85vh] flex flex-col"
  ```
- In the modal header:
  ```tsx
  // Target: Line 191
  // Replace:
  className="flex items-center justify-between pb-3.5 border-b border-slate-200/90 dark:border-[var(--ui-border-subtle)]"
  // With:
  className="flex items-center justify-between pb-3.5 border-b border-slate-200/90 dark:border-[#40285e]"
  ```

### 3.2 Header Telemetry & Action Capsules (Lines 212, 217, 222, 238, 248)
- In the bound quota capsule (Line 212):
  ```tsx
  // Replace:
  className="inline-flex items-center rounded-full border border-slate-200/90 dark:border-[var(--ui-border-subtle)] bg-slate-100/80 dark:bg-[var(--ui-canvas)]/80 backdrop-blur-xs px-2 py-0.5 text-[9px] font-mono shadow-2xs text-slate-600 dark:text-[var(--ui-text-secondary)]"
  // With:
  className="inline-flex items-center rounded-full border border-slate-200/90 dark:border-[#40285e] bg-slate-100/80 dark:bg-[#090611]/80 backdrop-blur-xs px-2 py-0.5 text-[9px] font-mono shadow-2xs text-slate-600 dark:text-[var(--ui-text-secondary)]"
  ```
- In the vertical dividers within the bound quota capsule (Lines 217, 222):
  ```tsx
  // Replace:
  <span className="w-px h-2 bg-slate-200 dark:bg-[var(--ui-border-subtle)] mx-1 shrink-0" />
  // With:
  <span className="w-px h-2 bg-slate-200 dark:bg-[#40285e] mx-1 shrink-0" />
  ```
- In the header action segmented capsule (Line 238) and divider (Line 248):
  ```tsx
  // Replace Line 238:
  className="inline-flex items-center rounded-full border border-slate-200/90 dark:border-[var(--ui-border-subtle)] bg-slate-100/80 dark:bg-[var(--ui-canvas)] p-0.5 backdrop-blur-xs shadow-2xs"
  // With:
  className="inline-flex items-center rounded-full border border-slate-200/90 dark:border-[#40285e] bg-slate-100/80 dark:bg-[#090611]/80 p-0.5 backdrop-blur-xs shadow-2xs"

  // Replace Line 248:
  <span className="w-px h-3.5 bg-slate-200 dark:bg-[var(--ui-border-subtle)] shrink-0" />
  // With:
  <span className="w-px h-3.5 bg-slate-200 dark:bg-[#40285e] shrink-0" />
  ```

### 3.3 Account Card Rows (Lines 333, 334)
- In the account card item container:
  ```tsx
  // Replace Lines 333-334:
  isAnySwitching
      ? 'opacity-60 cursor-not-allowed bg-white/80 dark:bg-[var(--ui-surface-1)]/80 border-slate-200/90 dark:border-[var(--ui-border-subtle)]'
      : 'bg-white/80 dark:bg-[var(--ui-surface-1)]/80 border-slate-200/90 dark:border-[var(--ui-border-subtle)] hover:bg-slate-50/90 dark:hover:bg-[var(--ui-surface-hover)] hover:border-purple-400/50 dark:hover:border-[var(--ui-border-emphasis)] hover:shadow-2xs cursor-pointer focus:outline-none focus:ring-1 focus:ring-[var(--ui-focus-ring)]'
  // With:
  isAnySwitching
      ? 'opacity-60 cursor-not-allowed bg-white/80 dark:bg-[#1f1533] border-slate-200/90 dark:border-[#40285e]'
      : 'bg-white/80 dark:bg-[#1f1533] border-slate-200/90 dark:border-[#40285e] hover:bg-slate-50/90 dark:hover:bg-[var(--ui-surface-hover)] hover:border-purple-400/50 dark:hover:border-[var(--ui-border-emphasis)] hover:shadow-2xs cursor-pointer focus:outline-none focus:ring-1 focus:ring-[var(--ui-focus-ring)]'
  ```

### 3.4 Quota Metrics Capsules & Dividers (Lines 383, 388, 393, 416)
- In the card quota metrics capsule (Line 383) and dividers (Lines 388, 393):
  ```tsx
  // Replace Line 383:
  className="inline-flex items-center rounded-full border border-slate-200/90 dark:border-[var(--ui-border-subtle)] bg-slate-100/80 dark:bg-[var(--ui-canvas)]/80 backdrop-blur-xs px-2.5 py-0.5 text-[10px] font-mono shadow-2xs text-slate-600 dark:text-[var(--ui-text-secondary)]"
  // With:
  className="inline-flex items-center rounded-full border border-slate-200/90 dark:border-[#40285e] bg-slate-100/80 dark:bg-[#090611]/80 backdrop-blur-xs px-2.5 py-0.5 text-[10px] font-mono shadow-2xs text-slate-600 dark:text-[var(--ui-text-secondary)]"

  // Replace Lines 388, 393:
  <span className="w-px h-2.5 bg-slate-200 dark:bg-[var(--ui-border-subtle)] mx-1.5 shrink-0" />
  // With:
  <span className="w-px h-2.5 bg-slate-200 dark:bg-[#40285e] mx-1.5 shrink-0" />
  ```
- In the weekly quota divider (Line 416):
  ```tsx
  // Replace Line 416:
  <div className={cn('space-y-0.5 w-full', fourH !== null && 'pt-1 border-t border-slate-200/50 dark:border-[var(--ui-border-subtle)]/50')}>
  // With:
  <div className={cn('space-y-0.5 w-full', fourH !== null && 'pt-1 border-t border-slate-200/50 dark:border-[#40285e]/50')}>
  ```

### 3.5 Action Button Capsule (Line 442)
- In the non-current account action button segmented capsule wrapper:
  ```tsx
  // Replace Line 442:
  <div className="inline-flex items-center rounded-full border border-slate-200/90 dark:border-[var(--ui-border-subtle)] bg-slate-100/80 dark:bg-[var(--ui-canvas)] p-0.5 backdrop-blur-xs shadow-2xs shrink-0">
  // With:
  <div className="inline-flex items-center rounded-full border border-slate-200/90 dark:border-[#40285e] bg-slate-100/80 dark:bg-[#090611]/80 p-0.5 backdrop-blur-xs shadow-2xs shrink-0">
  ```

---

## 4. Acceptance Criteria & Verification Checklist

- [ ] **Zero IACVT Declarations**: No `dark:bg-[var(...)]/<alpha>` or `dark:border-[var(...)]/<alpha>` patterns remain in `src/pages/instances/SwitchAccountDialog.tsx`.
- [ ] **Dark-Glass Modal Background**: Modal container uses `dark:bg-[#171025]/95` and `dark:border-[#40285e]`, rendering a dark royal violet surface instead of accidental bright white.
- [ ] **Contrast Preservation**: All typography (`dark:text-white`, `dark:text-slate-100`, `dark:text-slate-300`, `dark:text-slate-400`) maintains high contrast (WCAG AAA/AA) against `#171025` and `#1f1533`.
- [ ] **Segmented Capsules Consistency**: Telemetry capsules, action button wrappers, and header controls strictly utilize `dark:bg-[#090611]/80` and `dark:border-[#40285e]`.
- [ ] **Dividers & Borders**: All internal vertical and horizontal dividers utilize `dark:bg-[#40285e]` and `dark:border-[#40285e]/50`.
- [ ] **Pre-Flight Validation**: `npm run build` compiles without syntax or style errors.
