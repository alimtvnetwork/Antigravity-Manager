# Subtask Specification: Filter Search Bar & Counter Ergonomics Redesign

- **Subtask ID**: `232-subtask-02`
- **Parent Plan**: `232-account-switching-color-search-and-ui-ux-enhancements`
- **Target File**: `src/pages/instances/SwitchAccountDialog.tsx`
- **Spec Reference**: `02-spec/21-app/232-account-switching-color-search-and-ui-ux-enhancements/01-architecture-spec.md`
- **Status**: PENDING
- **Created**: 2026-10-28

---

## 1. Objective & Scope

Redesign the account filter search input and counter in `src/pages/instances/SwitchAccountDialog.tsx` into a unified, contiguous segmented pill capsule. This replaces the disjointed search input and floating text counter with an integrated dark-glass component featuring focus management, two-stage Escape key handling, and a dynamic filtered account ratio.

---

## 2. Technical Requirements

### 2.1 Ref Attachment & Focus Restoration
1. Import `useRef` from `react`.
2. Define `const searchInputRef = useRef<HTMLInputElement>(null);` inside `SwitchAccountDialog`.
3. Attach `ref={searchInputRef}` to the `<input>` element.
4. When the clear button is clicked:
   - Clear the search query (`setAccountSearchQuery('')`).
   - Immediately restore focus to the input via `searchInputRef.current?.focus()`.

### 2.2 Two-Stage Escape Key Handling
1. **Local Input Interception (`onKeyDown`)**:
   - When the user presses `Escape` while typing in the search input:
     - If `accountSearchQuery.trim().length > 0`: call `e.preventDefault()`, `e.stopPropagation()`, clear the query, and retain focus in the input.
     - If `accountSearchQuery` is already empty: do not intercept; allow the event to bubble to the modal's global listener to close the dialog.
2. **Global Window Listener (`keydown`)**:
   - In the `useEffect` listening for `Escape`:
     - If `accountSearchQuery.trim().length > 0`: clear `accountSearchQuery` and restore focus to `searchInputRef.current?.focus()` instead of closing the modal.
     - If `accountSearchQuery` is empty: invoke `close()` to dismiss the dialog.

### 2.3 Contiguous Segmented Pill Capsule Container
1. Merge the filter search bar and counter into a single cohesive capsule container:
   ```tsx
   <div className="pt-3.5 pb-1">
       <div className="relative flex items-center w-full px-3 py-1.5 rounded-full border border-slate-200/90 dark:border-[#40285e] bg-slate-50/80 dark:bg-[#090611]/80 shadow-2xs backdrop-blur-xs transition-all duration-150 focus-within:border-purple-400 dark:focus-within:border-[var(--ui-focus-ring)] focus-within:ring-2 focus-within:ring-purple-500/20 dark:focus-within:ring-[var(--ui-focus-ring)]/30 gap-2">
           ...
       </div>
   </div>
   ```
2. The outer capsule holds the single border and focus-within ring. No inner border or separate input ring should be present, eliminating visual nesting artifacts.

### 2.4 Borderless Inner Input & Placeholder Token Alignment
1. Search icon: `Search` icon with `w-3.5 h-3.5 text-slate-400 dark:text-slate-500 shrink-0`.
2. Inner input styling:
   - `flex-1 bg-transparent border-0 focus:outline-none focus:ring-0 p-0 text-xs text-slate-900 dark:text-slate-100 placeholder:text-slate-400 dark:placeholder:text-[var(--ui-text-muted)] leading-normal disabled:opacity-50`
   - Explicitly removes all default borders, padding bloat, and browser focus outlines.
3. Accessible clear button:
   - Rendered conditionally when `accountSearchQuery` is non-empty.
   - `p-0.5 rounded-full text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors cursor-pointer shrink-0`
   - `title="Clear filter (Esc)"`, `aria-label="Clear filter"`.

### 2.5 Segmented Counter Pill & Ratio Display
1. Add a subtle vertical divider line before the counter:
   - `<span className="w-px h-3.5 bg-slate-200/90 dark:bg-[#40285e] shrink-0" />`
2. Segmented counter badge:
   - When filtered (`accountSearchQuery.trim().length > 0`):
     - Displays `filtered.length / accounts.length accounts` (e.g., `2 / 46 accounts`).
   - When unfiltered:
     - Displays `accounts.length accounts` (or `1 account`).
   - Capsule badge styling:
     - `text-[11px] font-mono font-medium text-slate-500 dark:text-purple-300 select-none shrink-0 px-2 py-0.5 rounded-full bg-slate-200/60 dark:bg-[#1f1533]/90 border border-slate-300/50 dark:border-[#40285e]/60`

---

## 3. Implementation Blueprint

### 3.1 Hook & State Updates in `SwitchAccountDialog.tsx`
```tsx
import { useState, useMemo, useEffect, useRef } from 'react';

// Inside SwitchAccountDialog:
const searchInputRef = useRef<HTMLInputElement>(null);

// Escape key listener update:
useEffect(() => {
    if (!switchTargetInstance) return;
    const handleKeyDown = (e: KeyboardEvent) => {
        if (isAnySwitching) {
            return;
        }
        if (e.key === 'Escape') {
            if (accountSearchQuery.trim().length > 0) {
                setAccountSearchQuery('');
                searchInputRef.current?.focus();
                return;
            }
            close();
        }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
}, [close, isAnySwitching, switchTargetInstance, accountSearchQuery]);
```

### 3.2 JSX Structure
```tsx
{/* Filter Search Input & Counter Capsule */}
<div className="pt-3.5 pb-1">
    <div className="relative flex items-center w-full px-3 py-1.5 rounded-full border border-slate-200/90 dark:border-[#40285e] bg-slate-50/80 dark:bg-[#090611]/80 shadow-2xs backdrop-blur-xs transition-all duration-150 focus-within:border-purple-400 dark:focus-within:border-[var(--ui-focus-ring)] focus-within:ring-2 focus-within:ring-purple-500/20 dark:focus-within:ring-[var(--ui-focus-ring)]/30 gap-2">
        <Search className="w-3.5 h-3.5 text-slate-400 dark:text-slate-500 shrink-0" />
        <input
            ref={searchInputRef}
            type="text"
            value={accountSearchQuery}
            onChange={(e) => setAccountSearchQuery(e.target.value)}
            onKeyDown={(e) => {
                if (e.key === 'Escape' && accountSearchQuery.trim().length > 0) {
                    e.preventDefault();
                    e.stopPropagation();
                    setAccountSearchQuery('');
                    searchInputRef.current?.focus();
                }
            }}
            disabled={isAnySwitching}
            placeholder="Filter accounts by email or ID..."
            className="flex-1 bg-transparent border-0 focus:outline-none focus:ring-0 p-0 text-xs text-slate-900 dark:text-slate-100 placeholder:text-slate-400 dark:placeholder:text-[var(--ui-text-muted)] leading-normal disabled:opacity-50"
        />
        {accountSearchQuery && (
            <button
                type="button"
                onClick={() => {
                    setAccountSearchQuery('');
                    searchInputRef.current?.focus();
                }}
                disabled={isAnySwitching}
                className="p-0.5 rounded-full text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors cursor-pointer shrink-0 disabled:opacity-50"
                title="Clear filter (Esc)"
                aria-label="Clear filter"
            >
                <X className="w-3.5 h-3.5" />
            </button>
        )}
        <span className="w-px h-3.5 bg-slate-200/90 dark:bg-[#40285e] shrink-0" />
        <span className="text-[11px] font-mono font-medium text-slate-500 dark:text-purple-300 select-none shrink-0 px-2 py-0.5 rounded-full bg-slate-200/60 dark:bg-[#1f1533]/90 border border-slate-300/50 dark:border-[#40285e]/60">
            {accountSearchQuery.trim().length > 0
                ? `${filtered.length} / ${accounts.length} accounts`
                : `${accounts.length} ${accounts.length === 1 ? 'account' : 'accounts'}`}
        </span>
    </div>
</div>
```

---

## 4. Acceptance Criteria & Verification Checklist

- [ ] **Contiguous Capsule Architecture**: Search input, clear button, divider, and counter pill are enclosed within a single seamless `rounded-full` outer capsule without nested borders or disjointed margins.
- [ ] **Dark-Glass Styling Alignment**: Outer capsule adheres to `dark:bg-[#090611]/80`, `dark:border-[#40285e]`, and `dark:focus-within:border-[var(--ui-focus-ring)]`.
- [ ] **Two-Stage Escape Key Behavior**:
  - [ ] Pressing `Escape` when the search query has text clears the search input without closing the modal.
  - [ ] Pressing `Escape` when the search query is empty closes the modal dialog.
- [ ] **Input Focus Retention**:
  - [ ] Clicking the `X` clear button clears the query and restores focus to `searchInputRef`.
  - [ ] Pressing `Escape` inside the input clears the query and preserves focus in the input.
- [ ] **Segmented Filter Ratio Counter**:
  - [ ] Unfiltered state renders total account count (`Y accounts`).
  - [ ] Filtered state renders ratio badge (`X / Y accounts`) with monospace font.
- [ ] **Repository Hygiene**:
  - [ ] All file paths in documentation and code remain repository-relative.
  - [ ] Zero TypeScript diagnostic errors introduced in `src/pages/instances/SwitchAccountDialog.tsx`.
