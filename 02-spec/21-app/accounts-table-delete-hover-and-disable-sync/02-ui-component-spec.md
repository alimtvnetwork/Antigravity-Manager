# 02-ui-component-spec: Table Row Hover Inversion, Button Styling Overhaul & Accessibility Specification

- **Spec ID:** `02-spec/21-app/accounts-table-delete-hover-and-disable-sync/02-ui-component-spec.md`
- **Parent Plan:** `.ai-memory/plans/accounts-table-delete-hover-and-disable-sync.md`
- **Application:** Antigravity-Manager (`agm`, Web Frontend, Account Management)
- **Status:** `APPROVED`
- **Author:** Spec Author 02 (`self`)
- **Target Components:**
  - `src/components/accounts/AccountTable/SortableAccountRow.tsx`
  - `src/components/accounts/AccountTable/AccountRowContent.tsx`
  - `src/components/accounts/AccountTable/cells/ActionsCell.tsx`
  - `src/components/accounts/AccountTable.tsx`
  - `src/components/accounts/AccountCard/AccountCardActions.tsx`

---

## 1. Executive Summary & Design Rationale

This specification defines the UI interaction, visual styling, position-locking invariants, and accessibility architecture for the Accounts Table and Card action components.

### Core Objectives:
1. **Table Row Hover Inversion:** Invert the entire row styling on cursor hover to pure white background (`bg-white`) and pure black text (`text-black`), ensuring maximum contrast across light and dark themes.
2. **Position-Locking Invariant (Zero Layout Shift):** Strictly eliminate all dynamic font-weight inflation (never `hover:font-bold` or `hover:font-semibold`) and dynamic border/padding adjustments that previously induced column jitter and layout shifts.
3. **Action Button Color Scheme Overhaul:**
   - **Switch Account Button (`⇄`):** Refactored to a unique yellow highlighter styling (`bg-yellow-400 text-slate-900 border border-yellow-500 hover:bg-yellow-300 font-medium shadow-sm active:scale-95 rounded-[5px]`).
   - **Remove / Delete Button (`🗑️`):** Refactored to a solid red background with pure white icon/text (`bg-red-600 text-white border border-red-700 hover:bg-red-700 hover:text-white shadow-sm active:scale-95 rounded-[5px]`), matching visual evidence in `assets/screenshots/accounts-table-delete-hover-and-disable-sync-01.png`.
4. **Button Roles & Accessibility Compliance:** Enforce explicit `type="button"`, `role="button"`, and localized `aria-label` tags across all interactive action triggers in `ActionsCell.tsx` and `AccountCardActions.tsx`.
5. **Inner Button Highlighting:** Maintain clean, crisp visual pop for secondary buttons inside the white row on hover without visual wash-out.

---

## 2. Table Row Hover Inversion Architecture

### 2.1 Color Inversion Matrix

On row hover, the row transitions smoothly from its baseline state to a high-contrast inverted white row:

| Element | Default State (Dark / Light) | Hovered State (Inverted) | Tailwind Utility Classes |
|---|---|---|---|
| **Row Container (`<tr>`)** | `text-gray-800 dark:text-gray-200 bg-transparent` | Pure white background, pure black text | `hover:bg-white dark:hover:bg-white hover:text-black dark:hover:text-black` |
| **Email Cell Text** | `text-gray-900 dark:text-gray-100` | Pure black text | `group-hover:text-black dark:group-hover:text-black` |
| **Last Used Timestamp** | `text-gray-600 dark:text-gray-400 font-mono` | Pure black text | `group-hover:text-black dark:group-hover:text-black` |
| **Grip Handle Cell (`<td>`)** | `bg-transparent` | Pure white background | `group-hover:bg-white dark:group-hover:bg-white` |
| **Checkbox Cell (`<td>`)** | `bg-transparent` | Pure white background | `group-hover:bg-white dark:group-hover:bg-white` |
| **4h Quota Cell (`<td>`)** | `bg-slate-50/50 dark:bg-slate-900/40` | Pure white opaque background | `group-hover:bg-white dark:group-hover:bg-white` |
| **Weekly Quota Cell (`<td>`)** | `bg-slate-50/50 dark:bg-slate-900/40` | Pure white opaque background | `group-hover:bg-white dark:group-hover:bg-white` |
| **Actions Cell (`<td>`, sticky)** | `bg-white dark:bg-[#081826]` | Pure white 100% opaque background | `group-hover:bg-white dark:group-hover:bg-white` |

### 2.2 Position-Locking Invariant (Zero Layout Shift)

> [!IMPORTANT]
> **Strict Position-Locking Invariant:** Under no circumstances may hover states alter font weight (`font-bold`, `font-semibold`), border widths, margins, or padding. Layout shifts degrade user experience and cause horizontal table jitter.

1. **Font Weight Immutability:**
   - **FORBIDDEN:** `hover:font-bold`, `hover:font-semibold`, `group-hover:font-bold`, `group-hover:font-medium`.
   - **REQUIRED:** Font weight remains identical between resting and hovered states (`font-medium` or `font-normal`). Color inversion (`text-black`) provides contrast without geometric expansion.
2. **Border Width Immutability:**
   - The left indicator border must remain strictly static at `border-l-2`.
   - Border color may transition (`border-l-transparent hover:border-l-blue-500/80`), but the width must never change to `border-l-[3px]` or `border-l-4`.
3. **Padding & Spacing Immutability:**
   - Cell paddings (`py-0.5`, `px-2`) and flex gap tokens (`gap-1`, `gap-x-2`) remain constant.

### 2.3 Quota Cell & Sticky Cell Opacity Alignment

- **The Problem:** The Quota cells (`min-w-[185px]`) previously declared hardcoded background tints (`bg-slate-50/50 dark:bg-slate-900/40`). When the parent `<tr>` hovered to white, these cells appeared as darker rectangular islands inside the white row.
- **The Solution:** Add `group-hover:bg-white dark:group-hover:bg-white` to both the 4h quota `<td>` and weekly quota `<td>` in `src/components/accounts/AccountTable/AccountRowContent.tsx`.
- **Sticky Actions Cell:** In `src/components/accounts/AccountTable/cells/ActionsCell.tsx`, the right-pinned column uses `sticky right-0 z-10`. To prevent horizontally scrolled content from bleeding through while ensuring consistent row inversion, the cell must declare:
  ```tsx
  className={cn(
      "px-2 py-0.5 sticky right-0 z-10 w-[220px] xl:w-[280px] shadow-[-12px_0_12px_-12px_rgba(0,0,0,0.1)] dark:shadow-[-12px_0_12px_-12px_rgba(255,255,255,0.05)] text-center align-middle transition-colors border-b border-slate-200/90 dark:border-slate-800/90",
      isFocused
          ? "bg-teal-50 dark:bg-[#171025]"
          : selected
          ? "bg-blue-50 dark:bg-[#171025]"
          : isCurrent
          ? "bg-blue-50 dark:bg-[#091b2c]"
          : "bg-white dark:bg-[#081826]",
      "group-hover:bg-white dark:group-hover:bg-white"
  )}
  ```

---

## 3. Action Buttons Color Scheme Overhaul

### 3.1 Switch Account Button (`⇄`) — Yellow Highlighter Styling

The Switch button triggers switching the active account for Antigravity instances. It must stand out prominently from secondary utility actions.

- **Design Metaphor:** Yellow highlighter marker (`#FACC15` / `yellow-400`) with dark slate typography and subtle gold border.
- **Tailwind Class Specification:**
  ```tsx
  className={cn(
      "p-1 rounded-[5px] transition-all font-medium shadow-sm active:scale-95 border",
      (isSwitching || isDisabled)
          ? "bg-yellow-200/60 text-slate-500 border-yellow-300/60 cursor-not-allowed opacity-60"
          : "bg-yellow-400 text-slate-900 border-yellow-500 hover:bg-yellow-300 hover:text-black hover:border-yellow-600"
  )}
  ```
- **States:**
  - **Resting:** Vibrant yellow background (`bg-yellow-400`), high-contrast dark text (`text-slate-900`), border `border-yellow-500`.
  - **Hover:** Bright lighter yellow (`hover:bg-yellow-300`), pure black icon (`hover:text-black`), sharpened border (`hover:border-yellow-600`).
  - **Active / Pressed:** Physical press feedback (`active:scale-95`).
  - **Disabled / Switching:** Subdued pastel amber (`bg-yellow-200/60 text-slate-500 border-yellow-300/60 cursor-not-allowed opacity-60`).

### 3.2 Remove / Delete Account Button (`🗑️`) — Solid Red Warning Styling

The Remove button permanently removes the account configuration. It must clearly signal a destructive action matching visual design guidelines.

- **Design Reference:** `assets/screenshots/accounts-table-delete-hover-and-disable-sync-01.png`
- **Tailwind Class Specification:**
  ```tsx
  className={cn(
      "p-1 rounded-[5px] transition-all font-medium shadow-sm active:scale-95 border",
      isDisabled
          ? "bg-red-300/60 text-white/70 border-red-400/60 cursor-not-allowed opacity-60"
          : "bg-red-600 text-white border-red-700 hover:bg-red-700 hover:text-white hover:border-red-800"
  )}
  ```
- **States:**
  - **Resting:** Solid red fill (`bg-red-600`), pure white icon (`text-white`), border `border-red-700`.
  - **Hover:** Darker crimson red (`hover:bg-red-700 hover:text-white`), border `hover:border-red-800`.
  - **Active / Pressed:** Physical press feedback (`active:scale-95`).
  - **Disabled:** Subdued washed red (`bg-red-300/60 text-white/70 border-red-400/60 cursor-not-allowed opacity-60`).

---

## 4. Inner Button Highlighting & Semantic Roles

### 4.1 Inner Button Highlighting Inside Inverted Row

When the row background turns pure white on hover:
1. Action icons container in `ActionsCell.tsx` increases opacity from resting to full: `opacity-80 group-hover:opacity-100 transition-opacity`.
2. Secondary buttons (Refresh `↻`, More `···`, Edit Label `🏷️`, Warmup `✨`, Proxy `⊚`) feature elevated contrast:
   - Text/Icon: `text-slate-600 dark:text-slate-400 group-hover:text-slate-800`
   - Background hover: `hover:bg-slate-200/80 dark:hover:bg-slate-200/80`
   - Border radius: Consistent `rounded-[5px]`.

### 4.2 Accessibility & WAI-ARIA Matrix

All interactive button elements across `ActionsCell.tsx` and `AccountCardActions.tsx` must implement semantic attributes:

| Action Button | Icon | `type` | `role` | Localized `aria-label` |
|---|---|---|---|---|
| **Refresh** | `RefreshCw` | `"button"` | `"button"` | `t('common.refresh', 'Refresh')` |
| **Switch Account** | `ArrowRightLeft` | `"button"` | `"button"` | `t('accounts.switch_account', 'Switch Account')` |
| **Switch Classic (Card)** | `ArrowRightLeft` | `"button"` | `"button"` | `t('accounts.switch_to_classic', 'Switch to Antigravity (Classic)')` |
| **Switch IDE (Card)** | `Repeat2` | `"button"` | `"button"` | `t('accounts.switch_to_ide', 'Switch to Antigravity IDE')` |
| **Switch CLI (Card)** | `Terminal` | `"button"` | `"button"` | `t('accounts.switch_to_agy', 'Switch to Antigravity CLI (agy)')` |
| **More Actions** | `MoreHorizontal` | `"button"` | `"button"` | `t('common.more', 'More actions')` |
| **Account Details** | `Info` | `"button"` | `"button"` | `t('common.details', 'Details')` |
| **Device Fingerprint** | `Fingerprint` | `"button"` | `"button"` | `t('accounts.device_fingerprint', 'Device Fingerprint')` |
| **Export Account** | `Download` | `"button"` | `"button"` | `t('common.export', 'Export')` |
| **Edit Custom Label** | `Tag` | `"button"` | `"button"` | `t('accounts.edit_label', 'Edit Label')` |
| **Warmup Account** | `Sparkles` | `"button"` | `"button"` | `t('accounts.warmup_this', 'Warm up Account')` |
| **Toggle Proxy** | `ToggleLeft` / `ToggleRight` | `"button"` | `"button"` | `account.proxy_disabled ? t('accounts.enable_proxy', 'Enable Proxy') : t('accounts.disable_proxy', 'Disable Proxy')` |
| **Delete / Remove** | `Trash2` | `"button"` | `"button"` | `t('common.delete', 'Delete Account')` |

---

## 5. Visual Hierarchy & Card View Parity

### 5.1 Parity Rules for `AccountCardActions.tsx`
The Accounts view supports switching between Table view and Grid/Card view. While Card view is not a table row with inverted hover, its action buttons must adhere to the identical visual palette:
- **Switch Buttons:** Yellow highlighter styling with `bg-yellow-400 text-slate-900 border border-yellow-500 hover:bg-yellow-300 font-medium shadow-sm active:scale-95 rounded-[5px]`.
- **Remove Button:** Solid red background with `bg-red-600 text-white border border-red-700 hover:bg-red-700 hover:text-white shadow-sm active:scale-95 rounded-[5px]`.
- **Accessibility:** All buttons receive explicit `type="button"`, `role="button"`, and `aria-label`.

---

## 6. Verification Criteria & Automated Lint Gates

1. **Visual Contrast Verification:**
   - In light mode and dark mode, hovering over an account row turns the background to pure `#FFFFFF` across all cells (handle, checkbox, email, 4h quota, weekly quota, last used, sticky actions).
   - Text color transitions to pure `#000000`.
2. **Zero Layout Jitter Gate:**
   - Measuring bounding client rectangles (`getBoundingClientRect()`) before and during hover reveals exactly 0px shift in column widths, row heights, or cell horizontal offsets.
3. **Button Semantics Gate:**
   - Validating DOM output confirms that 100% of action `<button>` tags contain `type="button"`, `role="button"`, and non-empty `aria-label`.
4. **Pre-flight Quality Gates:**
   - TypeScript compilation passes: `npm run build`.
