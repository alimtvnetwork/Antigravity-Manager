---
plan: 232-account-switching-color-search-and-ui-ux-enhancements
subtask: "03"
subtask_id: 232-subtask-03
title: Account Card Selection, Subscription Badges, and Tactile Action Buttons
domain: frontend-ui
depends_on:
  - 232-subtask-01
  - 232-subtask-02
citations:
  architecture_spec: 02-spec/21-app/232-account-switching-color-search-and-ui-ux-enhancements/01-architecture-spec.md
  master_plan: .ai-memory/plans/pending/232-account-switching-color-search-and-ui-ux-enhancements.md
  coding_guidelines: AGENTS.md
target_files:
  - src/pages/instances/SwitchAccountDialog.tsx
status: pending
---

# Subtask 03: Account Card Selection, Subscription Badges, and Tactile Action Buttons

- **Subtask ID**: `232-subtask-03`
- **Component**: `SwitchAccountDialog`
- **Target File**: `src/pages/instances/SwitchAccountDialog.tsx`
- **Related Spec**: `02-spec/21-app/232-account-switching-color-search-and-ui-ux-enhancements/01-architecture-spec.md`
- **Status**: PENDING

---

## 1. Context & Motivation

During dark mode inspection of the Switch Account modal (`SwitchAccountDialog.tsx`), three key interactive design flaws were identified in the account selection rows:

1. **IACVT Background on Account Cards**:
   - The account cards currently declare `dark:bg-[var(--ui-surface-1)]/80` and `dark:border-[var(--ui-border-subtle)]`. Because `--ui-surface-1` contains a 6-digit hex code (`#171025`), the Tailwind slash modifier evaluates to `rgb(#171025 / 0.8)` which Chromium treats as **Invalid At Computed-Value Time (IACVT)**. The browser discards the dark background declaration and falls back to the light mode `bg-white/80`, destroying dark mode contrast.
   - The row cards must use a dedicated solid dark-glass surface (`#1f1533`) with smooth hover elevation and border transition.
2. **Radio Circle Indicator Feedback**:
   - The radio indicator on each card lacks a distinct visual state transition between the currently bound account and selectable candidate accounts.
   - The indicator needs an emerald theme for the current account and an interactive purple glow on hover for candidate accounts.
3. **Subscription Badges & Visual Noise**:
   - Subscription badges (`PRO`, `ULTRA`, `FREE`) use generic pastel fills with minimal border contrast and unrefined pill geometry.
   - They must be elevated with structured borders, `rounded-md` geometry, refined padding (`px-2 py-0.5`), and WCAG AAA contrast compliant text tokens across both light and dark themes.
4. **Action Button Tactile Micro-Interactions**:
   - The "Select" action button uses a flat background without tactile depth or press feedback.
   - It requires a rich purple glass gradient (`bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500`), subtle glass border, soft colored shadow, and tactile micro-interactions (`active:scale-[0.98]`, `hover:-translate-y-0.5 active:translate-y-0`).

---

## 2. Target Files & Symbols

| Target File | Symbols / Nodes | Scope of Change |
| :--- | :--- | :--- |
| `src/pages/instances/SwitchAccountDialog.tsx` | Account row card container (`filtered.map`) | Replace `dark:bg-[var(--ui-surface-1)]/80` with `dark:bg-[#1f1533]`, refine borders, hover elevation (`hover:bg-slate-50 dark:hover:bg-[#281b42]`, `hover:border-purple-400 dark:hover:border-[#7c3aed]`), and handle disabled/active states. |
| `src/pages/instances/SwitchAccountDialog.tsx` | Radio circle indicator | Update indicator to emerald for current bound account (`border-emerald-500 bg-emerald-500/20 text-emerald-400`), and purple hover state for candidate accounts (`border-slate-300 dark:border-[#40285e] group-hover:border-purple-500 dark:group-hover:border-[#a855f7] group-hover:bg-purple-500/10 dark:group-hover:bg-[#7c3aed]/20`). |
| `src/pages/instances/SwitchAccountDialog.tsx` | Subscription tier badges | Style `PRO`, `ULTRA`, and `FREE` badges with `rounded-md`, `px-2 py-0.5`, explicit borders, and WCAG AAA contrast compliant tokens. |
| `src/pages/instances/SwitchAccountDialog.tsx` | "Select" action button | Implement purple glass gradient, glass border, drop shadow, and tactile micro-interactions (`active:scale-[0.98]`, `hover:-translate-y-0.5 active:translate-y-0`, `transition-all duration-150`). Maintain spinner state for `isThisSwitching`. |
| `src/pages/instances/SwitchAccountDialog.tsx` | Quota metric pill capsule | Replace `dark:bg-[var(--ui-canvas)]/80` with solid `dark:bg-[#090611]` and `dark:border-[#40285e]/80` to eliminate nested IACVT failure. |

---

## 3. Detailed Technical Requirements

### 3.1 Account Card Styling & Hover Elevation
- **Card Background & Border**:
  - Light mode: `bg-white border-slate-200/80`
  - Dark mode: `dark:bg-[#1f1533] dark:border-[#40285e]/70`
- **Hover State (Candidate Accounts)**:
  - Light mode: `hover:bg-slate-50 hover:border-purple-400`
  - Dark mode: `dark:hover:bg-[#281b42] dark:hover:border-[#7c3aed]`
  - Smooth elevation transition: `transition-all duration-200`
- **Current Account Row**:
  - Light mode: `bg-emerald-500/10 border-emerald-400/50`
  - Dark mode: `dark:bg-emerald-950/30 dark:border-emerald-500/40`
  - Cursor: `cursor-default`
- **Disabled / Switching State**:
  - `opacity-60 cursor-not-allowed`

### 3.2 Radio Circle Indicator
- **Current Account**:
  - Outer circle: `w-4 h-4 rounded-full border border-emerald-500 bg-emerald-500/20 flex items-center justify-center shrink-0 mt-0.5`
  - Inner dot: `w-1.5 h-1.5 rounded-full bg-emerald-400 dark:bg-[#1af18d] shadow-xs`
- **Candidate Account**:
  - Outer circle: `w-4 h-4 rounded-full border border-slate-300 dark:border-[#40285e] flex items-center justify-center shrink-0 mt-0.5 transition-all duration-200`
  - Group hover: `group-hover:border-purple-500 dark:group-hover:border-[#a855f7] group-hover:bg-purple-500/10 dark:group-hover:bg-[#7c3aed]/20`

### 3.3 Subscription Tier Badges (WCAG AAA Contrast)
All badges must use `rounded-md`, refined padding `px-2 py-0.5`, explicit 1px borders, and high contrast typography:
- **PRO Badge**:
  - Light mode: `bg-sky-500/15 text-sky-700 border-sky-500/30`
  - Dark mode: `dark:bg-sky-500/20 dark:text-sky-300 dark:border-sky-400/40`
  - Typography: `font-bold text-[9px] tracking-wide uppercase`
- **ULTRA Badge**:
  - Light mode: `bg-purple-500/15 text-purple-700 border-purple-500/30`
  - Dark mode: `dark:bg-purple-500/20 dark:text-purple-300 dark:border-purple-400/40`
  - Typography: `font-bold text-[9px] tracking-wide uppercase`
- **FREE Badge**:
  - Light mode: `bg-slate-500/15 text-slate-700 border-slate-500/30`
  - Dark mode: `dark:bg-slate-500/20 dark:text-slate-300 dark:border-slate-400/40`
  - Typography: `font-bold text-[9px] tracking-wide uppercase`

### 3.4 Tactile "Select" Action Button
The button inside candidate rows must provide responsive tactile feedback:
- **Normal State**:
  - Background: `bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500`
  - Text & Geometry: `text-white font-semibold text-xs px-3.5 py-1.5 rounded-full`
  - Border: `border border-white/20 dark:border-purple-400/30`
  - Shadow: `shadow-md shadow-purple-500/20 hover:shadow-lg hover:shadow-purple-500/30`
- **Tactile Micro-Interactions**:
  - Hover: `hover:-translate-y-0.5`
  - Active: `active:translate-y-0 active:scale-[0.98]`
  - Timing: `transition-all duration-150 ease-out`
- **Disabled / Switching State**:
  - Disabled: `opacity-60 cursor-not-allowed active:scale-100 hover:translate-y-0`
  - Spinning icon: `<RotateCw className="w-3.5 h-3.5 animate-spin" />` with label `Switching...`

---

## 4. Implementation Blueprint

### 4.1 Account Row Card Structure in `src/pages/instances/SwitchAccountDialog.tsx`

```tsx
<div
    key={acc.id}
    role="button"
    tabIndex={isCurrent || isAnySwitching ? -1 : 0}
    onClick={() => {
        if (!isCurrent && !isAnySwitching) {
            handleSelect(acc);
        }
    }}
    onKeyDown={(e) => {
        if ((e.key === 'Enter' || e.key === ' ') && !isCurrent && !isAnySwitching) {
            e.preventDefault();
            handleSelect(acc);
        }
    }}
    className={cn(
        'ui-interactive-row p-3.5 rounded-xl border flex items-center justify-between gap-3 transition-all duration-200 group',
        isCurrent
            ? 'bg-emerald-500/10 dark:bg-emerald-950/30 border-emerald-400/50 dark:border-emerald-500/40 shadow-2xs cursor-default'
            : isAnySwitching
              ? 'opacity-60 cursor-not-allowed bg-white dark:bg-[#1f1533] border-slate-200/80 dark:border-[#40285e]/70'
              : 'bg-white dark:bg-[#1f1533] border-slate-200/80 dark:border-[#40285e]/70 hover:bg-slate-50 dark:hover:bg-[#281b42] hover:border-purple-400 dark:hover:border-[#7c3aed] hover:shadow-md cursor-pointer focus:outline-none focus:ring-1 focus:ring-purple-500'
    )}
>
```

### 4.2 Radio Circle Indicator Blueprint

```tsx
{/* Circular Radio Indicator */}
<div
    className={cn(
        'w-4 h-4 rounded-full border flex items-center justify-center shrink-0 mt-0.5 transition-all duration-200',
        isCurrent
            ? 'border-emerald-500 bg-emerald-500/20 text-emerald-400 dark:text-[#1af18d]'
            : 'border-slate-300 dark:border-[#40285e] group-hover:border-purple-500 dark:group-hover:border-[#a855f7] group-hover:bg-purple-500/10 dark:group-hover:bg-[#7c3aed]/20'
    )}
>
    {isCurrent && <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 dark:bg-[#1af18d] shadow-xs" />}
</div>
```

### 4.3 Subscription Badge Blueprint

```tsx
<span
    className={cn(
        'text-[9px] font-bold px-2 py-0.5 rounded-md border shadow-2xs tracking-wide uppercase',
        tier.includes('ULTRA')
            ? 'bg-purple-500/15 text-purple-700 dark:text-purple-300 border-purple-500/30 dark:border-purple-400/40'
            : tier.includes('PRO')
              ? 'bg-sky-500/15 text-sky-700 dark:text-sky-300 border-sky-500/30 dark:border-sky-400/40'
              : 'bg-slate-500/15 text-slate-700 dark:text-slate-300 border-slate-500/30 dark:border-slate-400/40'
    )}
>
    {tier}
</span>
```

### 4.4 Tactile "Select" Action Button Blueprint

```tsx
{/* Action button: rendered only for non-current accounts */}
{!isCurrent && (
    <div className="shrink-0">
        <button
            type="button"
            disabled={isAnySwitching}
            onClick={(e) => {
                e.stopPropagation();
                handleSelect(acc);
            }}
            className={cn(
                'px-4 py-1.5 rounded-full text-xs font-semibold text-white transition-all duration-150 cursor-pointer',
                'border border-white/20 dark:border-purple-400/30 shadow-md shadow-purple-500/20',
                isAnySwitching
                    ? 'opacity-60 cursor-not-allowed bg-purple-600'
                    : 'bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 hover:shadow-lg hover:shadow-purple-500/30 hover:-translate-y-0.5 active:translate-y-0 active:scale-[0.98]'
            )}
        >
            {isThisSwitching ? (
                <span className="flex items-center gap-1.5">
                    <RotateCw className="w-3.5 h-3.5 animate-spin" />
                    <span>Switching...</span>
                </span>
            ) : (
                'Select'
            )}
        </button>
    </div>
)}
```

---

## 5. Acceptance Criteria

- [ ] **AC-1**: Dark mode card background uses solid `#1f1533` (no slash-opacity modifiers on CSS variables), eliminating IACVT white-fallback in dark mode.
- [ ] **AC-2**: Hovering candidate account cards elevates background to `dark:bg-[#281b42]` with border accent `dark:border-[#7c3aed]`.
- [ ] **AC-3**: The currently bound account row clearly indicates selection with `bg-emerald-500/10 dark:bg-emerald-950/30` background and `border-emerald-400/50 dark:border-emerald-500/40` border.
- [ ] **AC-4**: The radio circle renders emerald with a center dot for the current account, and triggers purple border/tint hover transitions for candidate accounts.
- [ ] **AC-5**: Subscription badges (`PRO`, `ULTRA`, `FREE`) use `rounded-md`, `px-2 py-0.5`, explicit 1px borders, and compliant high-contrast foreground text across dark and light modes.
- [ ] **AC-6**: The "Select" action button features a purple glass gradient (`bg-gradient-to-r from-purple-600 to-indigo-600`), glass border, and tactile micro-interactions (`active:scale-[0.98]`, `hover:-translate-y-0.5 active:translate-y-0`).
- [ ] **AC-7**: When switching is active, the target button displays a spinning `RotateCw` icon with "Switching..." text, and buttons disable hover translations.
- [ ] **AC-8**: All paths in documentation and code comments adhere to strictly repository-relative Git paths.

---

## 6. Verification Checklist

- [ ] **V-1**: Open Switch Account dialog in Dark Mode: verify cards render with deep violet `#1f1533` background, not white.
- [ ] **V-2**: Hover over multiple candidate accounts: confirm smooth border transition to `#7c3aed` and subtle elevation.
- [ ] **V-3**: Verify the radio circle indicator: emerald for current account, purple hover for candidate accounts.
- [ ] **V-4**: Verify tier badges for PRO, ULTRA, and FREE accounts for proper padding, rounded corners, and clear legibility.
- [ ] **V-5**: Click "Select": verify tactile micro-interaction press feedback (`active:scale-[0.98]`), spinner display, and successful account switch.
- [ ] **V-6**: Run pre-flight lint checks: `npm run build` or targeted linter passes with zero errors.
