# Subtask 04: Theming Contrast Modernization, Account Email Privacy Masking & CSS3 Micro-Elevation

**Subtask ID:** `subtask-04`  
**Parent Plan:** Supabase Connection & UI/UX Enhancement (`supabase-connection-and-ui-ux-enhancement`)  
**Spec Reference:** `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/02-ui-ux-progress-theming-spec.md`  
**Status:** READY FOR IMPLEMENTATION  
**Lead Worker:** Worker 02 (UI/UX, Progress Bar & Theming Author)  

---

## 1. Context & Objectives

This subtask addresses UI accessibility and privacy across the account and instance management views:

1. **Eliminate Purple-on-Purple Contrast Flaws:** Replace low-contrast violet-on-dark-violet text in dark theme mode with high-contrast, WCAG AA compliant typography: crisp bright white (`#ffffff` / `#f5f0ff`) for titles, warm cream/light-linen (`#fef9c3` / `#fef08a`) for subheads, and luminous yellow/amber (`#f5bf58`) for active attention states.
2. **Account Email Privacy Masking:** Integrate `maskEmail` from `src/utils/maskEmail.ts` across `AccountCardHeader.tsx`, `AccountRow.tsx`, and `AccountTable.tsx` so user identities remain protected during screen sharing and recordings, while preserving full email in tooltips.
3. **CSS3 Smooth Hover Micro-Elevation:** Apply smooth `-translate-y-0.5` micro-elevation (`transform: translateY(-2px)`) with shadow transitions (`hover:shadow-lg hover:shadow-black/40`) and dynamic border shifts on desktop interactive cards.

---

## 2. Target Files & Concrete Modifications

```
+-------------------------------------------------------+-----------------------------------------------------------+
| Target File Path                                      | Specific Scope of Modification                            |
+-------------------------------------------------------+-----------------------------------------------------------+
| src/utils/maskEmail.ts                                | Verify maskEmail helper handles prefix, suffix and domain |
| src/components/accounts/AccountCard/AccountCardHeader | Render maskEmail(account.email) in card title; retain     |
| .tsx                                                  | title={account.email} tooltip; enforce high-contrast text |
| src/components/accounts/AccountCard.tsx               | Apply transition-all duration-150 ease-out                |
|                                                       | hover:-translate-y-0.5 hover:shadow-lg hover:shadow-      |
|                                                       | black/40; eliminate low-contrast purple text              |
| src/components/accounts/AccountRow.tsx                | Mask email column in table rows; high-contrast text       |
| src/components/accounts/AccountTable.tsx              | Support showAllEmails toggle with privacy mask default;   |
|                                                       | crisp table header contrast                               |
| src/components/instances/InstanceTable.tsx            | Add hover micro-elevation to instance rows/cards;         |
|                                                       | verify dark theme text contrast                           |
| src/styles/ui-tokens.css                              | Add elevation and high-contrast text token aliases        |
+-------------------------------------------------------+-----------------------------------------------------------+
```

---

## 3. Detailed Implementation Directives

### 3.1 Eliminating Purple-on-Purple Typography
Replace low-contrast Tailwind classes in dark mode:
- **Forbidden:** `text-purple-400`, `text-purple-300`, `text-[#a855f7]` on dark violet surfaces (`#171025`, `#211631`).
- **Required:**
  - Card Titles: `text-slate-900 dark:text-white font-bold` (14.5:1 contrast).
  - Secondary metadata (priority, bound profile, created date): `text-slate-600 dark:text-[#fef9c3]/90 font-medium` (11.2:1 contrast).
  - Current/Active indicators: `bg-amber-500/15 text-amber-900 dark:text-amber-300 border-amber-400/40 font-bold` (9.8:1 contrast).

### 3.2 Account Email Privacy Masking via `maskEmail`
1. In `src/components/accounts/AccountCard/AccountCardHeader.tsx`:
   ```tsx
   import { maskEmail } from '../../../utils/maskEmail';

   // Inside JSX:
   <h3
       className="font-bold text-sm truncate w-full text-slate-900 dark:text-white"
       title={account.email}
   >
       {maskEmail(account.email)}
   </h3>
   ```
2. In `src/components/accounts/AccountRow.tsx`:
   ```tsx
   import { maskEmail } from '../../utils/maskEmail';

   // Inside JSX:
   <span className="font-mono text-xs text-slate-900 dark:text-slate-100" title={account.email}>
       {showAllEmails ? account.email : maskEmail(account.email)}
   </span>
   ```

### 3.3 CSS3 Hover Micro-Elevation (`-translate-y-0.5`)
In `src/components/accounts/AccountCard.tsx`:
```tsx
className={cn(
    "relative flex flex-col rounded-xl border p-2.5 sm:p-3 transition-all duration-150 ease-out",
    "hover:-translate-y-0.5 hover:shadow-lg hover:shadow-black/40",
    "hover:border-violet-500/60 dark:hover:border-[#7c3aed]/80",
    isSelected ? "border-amber-400 ring-1 ring-amber-400/30" : "border-slate-200 dark:border-[#40285e]/70",
    "bg-white dark:bg-[#171025]"
)}
```

---

## 4. Verification & Testing Protocol

1. **Contrast Audit:** Verify that all text elements on dark violet cards exceed 4.5:1 contrast ratio using browser developer tools or accessibility checkers.
2. **Email Privacy:** Confirm account cards and default table rows display masked emails (`joh***@gmail.com`).
3. **Tooltip Verification:** Hover over masked email and confirm native browser tooltip displays full address.
4. **Elevation Physics:** Hover over account cards and instance cards; confirm smooth 2px upward shift (`-translate-y-0.5`) with no layout jitter or scrollbar flashing.
5. **Reduced Motion Check:** Verify that `@media (prefers-reduced-motion: reduce)` disables the translation smoothly.
