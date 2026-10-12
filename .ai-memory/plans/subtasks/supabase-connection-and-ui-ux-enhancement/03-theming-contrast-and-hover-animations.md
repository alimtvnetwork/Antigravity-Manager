---
plan: supabase-connection-and-ui-ux-enhancement
subtask: "03"
title: Theming Contrast Correction, Email Privacy Masking & CSS3 Hover Micro-Interactions
domain: frontend/theming/interactions
target_files:
  - src/styles/ui-tokens.css
  - src/pages/instances/InstanceCard.tsx
  - src/components/accounts/AccountTable/AccountRowContent.tsx
  - src/components/accounts/AccountTable/SortableAccountRow.tsx
  - src/components/accounts/AccountCard.tsx
  - src/utils/maskEmail.ts
status: pending
---

# 03 — Theming Contrast Correction, Email Privacy Masking & CSS3 Hover Micro-Interactions

## 1. Objective

Resolve the high-priority visual defect involving low-contrast purple-on-purple email badges, enforce account privacy with interactive email masking, and establish hardware-accelerated CSS3 hover animations across cards and table rows:

1. **Eliminate Purple-on-Purple Defect**: In `InstanceCard.tsx` and account components, replace the defective `dark:bg-[var(--ui-interactive-soft)] dark:text-[var(--ui-interactive)]` combination (violet `#7c3aed` text on violet tint, contrast ratio 2.1:1) with dark obsidian-violet glass (`bg-slate-900/90 dark:bg-[#120b20] border-slate-700 dark:border-[#40285e]`) paired with radiant amber/white text (`text-amber-200 dark:text-amber-300 font-mono` or `text-slate-100`, contrast ratio $\ge 9.8:1$).
2. **Interactive Email Masking with Reveal Toggle**: Unify privacy protection across `InstanceCard.tsx` and `AccountRowContent.tsx` using `maskEmail()`. Masked by default (`local[0..3]***`), the full email reveals on hover or click, preserving developer confidentiality during video calls and public demos.
3. **CSS3 Interactive Hover Animations & Elevations**: Implement subtle `-translate-y-0.5` vertical lifts, border glow transitions, and background highlights across table rows and card containers with cubic-bezier timing.
4. **WCAG AA Compliance**: Guarantee that all text labels, email badges, and secondary metadata achieve at least 4.5:1 contrast against dark mode surfaces.

---

## 2. Technical Scope & Target Files

```
+-------------------------------------------------+----------------------------------------------------+
| File Path                                       | Key Scope & Functional Changes                     |
+-------------------------------------------------+----------------------------------------------------+
| src/styles/ui-tokens.css                        | Calibrate high-contrast chip background and text   |
|                                                 | tokens; declare CSS3 elevation utilities.          |
| src/pages/instances/InstanceCard.tsx           | Replace purple-on-purple email badge; add email    |
|                                                 | dot masking + reveal toggle; add hover lift.       |
| src/components/accounts/AccountTable/           | Enhance row hover transitions; ensure mask toggle  |
| AccountRowContent.tsx                           | accessibility and high-contrast badge text.        |
| src/components/accounts/AccountTable/           | Apply cubic-bezier transition to table row TR;     |
| SortableAccountRow.tsx                          | smooth drag/hover elevation feedback.              |
| src/components/accounts/AccountCard.tsx         | Add CSS3 elevation and high-contrast email chips.  |
+-------------------------------------------------+----------------------------------------------------+
```

---

## 3. Implementation Details

### 3.1 Eliminating the Purple-on-Purple Defect (`src/pages/instances/InstanceCard.tsx`)

#### Defect Analysis:
In `src/pages/instances/InstanceCard.tsx`:
```tsx
// DEFECTIVE (Line 214):
'bg-blue-500/10 text-blue-700 border-blue-500/30 dark:bg-[var(--ui-interactive-soft)] dark:text-[var(--ui-interactive)] dark:border-[var(--ui-interactive-border)]'
```
In Royal Violet dark mode, `--ui-interactive` evaluates to `#7c3aed`. Text rendered with this color on `--ui-interactive-soft` (`rgb(124 58 237 / 0.12)`) blends into the background, causing severe illegibility.

#### Correction:
Replace the styling with deep obsidian glass and radiant high-contrast text:
```tsx
const [isEmailUnmasked, setIsEmailUnmasked] = useState(false);

// Inside the account row of InstanceCard.tsx:
<div
    className={cn(
        'ui-interactive-row py-1 px-2.5 rounded-md bg-gray-50/80 dark:bg-[var(--ui-surface-2)] border border-gray-100 dark:border-[var(--ui-border-subtle)] mb-2.5 flex items-center justify-between gap-1.5 transition-colors',
        'group-hover:border-gray-300 dark:group-hover:bg-[var(--ui-surface-hover)] dark:group-hover:border-[var(--ui-interactive-40)]'
    )}
>
    <div className="flex items-center gap-1.5 min-w-0 flex-1">
        <Mail className="w-3 h-3 text-gray-400 dark:text-[var(--ui-text-secondary)] group-hover:text-amber-400 transition-colors shrink-0" />
        <span className="text-[10px] text-gray-500 dark:text-gray-400 font-medium shrink-0">Account:</span>
        {displayEmail ? (
            <button
                type="button"
                onClick={(e) => {
                    e.stopPropagation();
                    setIsEmailUnmasked((prev) => !prev);
                }}
                onMouseEnter={() => setIsEmailUnmasked(true)}
                onMouseLeave={() => setIsEmailUnmasked(false)}
                className={cn(
                    'px-2 py-0.5 rounded-md text-[11px] font-semibold font-mono border flex items-center gap-1.5 min-w-0 shadow-2xs transition-all duration-150 cursor-pointer text-left',
                    'bg-slate-100 text-slate-900 border-slate-300',
                    'dark:bg-[#120b20] dark:text-amber-200 dark:border-[#40285e] dark:hover:border-amber-400/50 dark:hover:text-amber-100'
                )}
                title={isEmailUnmasked ? `Account Email: ${displayEmail} (Click to mask)` : `Account Email: ${maskEmail(displayEmail)} (Click or hover to reveal)`}
            >
                <span className="w-1.5 h-1.5 rounded-full shrink-0 bg-[#1af18d] shadow-[0_0_4px_rgba(26,241,141,0.8)]" />
                <span className="truncate">{isEmailUnmasked ? displayEmail : maskEmail(displayEmail)}</span>
            </button>
        ) : (
            <span className="text-[11px] text-gray-400 italic">Unassigned</span>
        )}
    </div>
    {boundAccount && (
        <TierBadge tier={boundAccount.quota?.subscription_tier} size="xs" className="shrink-0" />
    )}
</div>
```

### 3.2 Standardizing Interactive Masking in `AccountRowContent.tsx`

Ensure the double-click and hover unmask interactions are prominent and use the high-contrast palette:
```tsx
<span
    className={cn(
        "font-medium text-xs break-all transition-colors cursor-pointer select-text",
        isFocused || selected
            ? "text-blue-950 dark:text-amber-200 font-bold"
            : isCurrent
            ? "text-blue-900 dark:text-amber-300 font-bold drop-shadow-xs"
            : "text-gray-900 dark:text-slate-100 group-hover:text-blue-600 dark:group-hover:text-amber-300"
    )}
    title={showAllEmails || showEmail || isHoverUnmasked ? account.email : maskEmail(account.email)}
    onClick={() => setIsHoverUnmasked((prev) => !prev)}
    onDoubleClick={(event) => {
        event.stopPropagation();
        setIsHoverUnmasked(true);
        openPriorityEditor();
    }}
>
    {showAllEmails || showEmail || isHoverUnmasked ? account.email : maskEmail(account.email)}
</span>
```

### 3.3 CSS3 Hover Elevations & Interactive Transitions

#### 1. In `src/styles/ui-tokens.css`:
Add reusable interactive classes:
```css
/* Smooth interactive elevation lift for cards */
.ui-hover-card {
  transition: transform 180ms cubic-bezier(0.16, 1, 0.3, 1),
              box-shadow 180ms cubic-bezier(0.16, 1, 0.3, 1),
              border-color 180ms cubic-bezier(0.16, 1, 0.3, 1),
              background-color 180ms ease-out;
}

.ui-hover-card:hover {
  transform: translateY(-1.5px);
  box-shadow: 0 8px 20px -4px rgba(0, 0, 0, 0.4),
              0 0 0 1px rgba(124, 58, 237, 0.25);
}

/* Subtle elevation for table rows */
.ui-hover-row {
  transition: background-color 120ms ease-out,
              border-color 120ms ease-out;
}

.ui-hover-row:hover td {
  background-color: var(--ui-surface-hover);
}
```

#### 2. In `InstanceCard.tsx`:
Add `.ui-hover-card` to the outer card container:
```tsx
<div
    key={inst.config.id}
    ref={isActive ? activeCardRef : undefined}
    className={cn(
        'group relative rounded-[5px] border flex flex-col justify-between overflow-hidden backdrop-blur-xs ui-hover-card',
        isActive
            ? SELECTED_CARD_CLASSES
            : 'bg-white dark:bg-[var(--ui-surface-1)] border-gray-200/50 dark:border-[var(--ui-border-subtle)] hover:border-gray-300/80 dark:hover:border-[var(--ui-interactive-40)] hover:bg-slate-50/90 dark:hover:bg-[var(--ui-surface-hover)] shadow-xs'
    )}
>
```

#### 3. In `SortableAccountRow.tsx`:
Add `.ui-hover-row` to the table row:
```tsx
<tr
    ref={setNodeRef}
    style={style}
    className={cn(
        "group relative select-none ui-hover-row",
        isDragging && "opacity-50 z-50",
        isSelected ? "bg-blue-50/80 dark:bg-[#2a1c3f]" : "bg-transparent"
    )}
    {...attributes}
    {...listeners}
>
```

---

## 4. Verification & Testing Checklist

- [ ] Inspect dark mode in `InstanceCard.tsx`: verify email badge uses dark background `#120b20` and amber text `#fde047` / `#fef08a`. Confirm contrast ratio $\ge 7:1$.
- [ ] Verify that violet `#7c3aed` text on violet background is completely eliminated.
- [ ] Check email masking on instance card: verify initial display is masked (`ab***@...` or `ab***`). Hover or click and confirm full email is revealed.
- [ ] Verify hover elevations: hover mouse over Instance Cards; verify smooth 1.5px lift without jitter.
- [ ] Verify table row hover: hover mouse over Accounts table rows; verify smooth dark-violet highlight on all cells.
- [ ] Run `npm run build` to confirm zero TypeScript compilation errors or CSS syntax issues.
