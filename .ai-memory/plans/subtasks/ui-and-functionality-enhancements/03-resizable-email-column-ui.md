---
plan: ui-and-functionality-enhancements
subtask: "03"
title: AccountTable Resizable Email Column with Drag Handle and LocalStorage Persistence
domain: frontend-ui
depends_on: 02-quota-bar-glow-width-and-circle-consistency.md
citations:
  architecture_spec: ../../../../02-spec/21-app/ui-and-functionality-enhancements/01-architecture-and-ui-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src/components/accounts/AccountTable.tsx
  - src/components/accounts/AccountTable/SortableAccountRow.tsx
  - src/components/accounts/AccountTable/AccountRowContent.tsx
  - src/components/accounts/AccountTable/types.ts
status: pending
---

# Subtask 03: AccountTable Resizable Email Column with Drag Handle and LocalStorage Persistence

## 1. Context & Motivation
In multi-account environments, developers frequently use email addresses of varying lengths (from short aliases like `dev@gm.com` to long corporate emails with team prefixes like `alexander.montgomery+staging-us-east@enterprise-corp.com`).
Currently, the Email column has a fixed width (`w-[260px]`), causing longer email addresses to truncate prematurely or wrap awkwardly.
The user requested an interactive, resizable Email column:
- Minimum width: `220px` (guarantees core action controls and badges fit).
- Maximum width: `550px` (prevents overwhelming quota progress bars and action buttons).
- Default width: `260px` (or restored from `localStorage`).
- Column header drag handle with intuitive hover and active drag indicators.
- Automatic persistence to `localStorage` across page reloads and app restarts.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src/components/accounts/AccountTable/types.ts` | `SortableAccountRowProps`, `AccountRowContentProps` | Add optional `emailColWidth?: number` prop. |
| `src/components/accounts/AccountTable.tsx` | `AccountTable`, drag resize handlers, state | 1. Initialize `emailColWidth` state from `localStorage` (`agm_account_table_email_width`).<br/>2. Add `handleMouseDownResize` callback registering window-level mouse listeners.<br/>3. Mount interactive drag handle on Email `<th>`.<br/>4. Pass `emailColWidth` down to row components. |
| `src/components/accounts/AccountTable/SortableAccountRow.tsx` | `SortableAccountRow` | Pass `emailColWidth` to `AccountRowContent`. |
| `src/components/accounts/AccountTable/AccountRowContent.tsx` | `AccountRowContent` | Apply `style={{ width: emailColWidth, minWidth: emailColWidth, maxWidth: emailColWidth }}` to email `<td>`. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Update Types in `src/components/accounts/AccountTable/types.ts`
1. Add `emailColWidth?: number;` to `AccountRowContentProps`.
2. Add `emailColWidth?: number;` to `SortableAccountRowProps`.

### 3.2 Step 2: Implement Column Resizing Logic in `src/components/accounts/AccountTable.tsx`

1. Define constants at top of `AccountTable.tsx`:
   ```tsx
   const STORAGE_KEY_EMAIL_WIDTH = 'agm_account_table_email_width';
   const MIN_EMAIL_WIDTH = 220;
   const MAX_EMAIL_WIDTH = 550;
   const DEFAULT_EMAIL_WIDTH = 260;
   ```
2. Initialize `emailColWidth` and `isResizing` state:
   ```tsx
   const [emailColWidth, setEmailColWidth] = useState<number>(() => {
       try {
           const saved = localStorage.getItem(STORAGE_KEY_EMAIL_WIDTH);
           if (saved) {
               const parsed = parseInt(saved, 10);
               if (Number.isFinite(parsed)) {
                   return Math.min(MAX_EMAIL_WIDTH, Math.max(MIN_EMAIL_WIDTH, parsed));
               }
           }
       } catch (err) {
           useErrorStore.getState().trackWarning(err, {
               source: 'AccountTable.initEmailWidth',
               triggerAction: 'localStorage.getItem',
           });
       }
       return DEFAULT_EMAIL_WIDTH;
   });

   const [isResizing, setIsResizing] = useState(false);
   ```
3. Implement `handleMouseDownResize`:
   ```tsx
   const handleMouseDownResize = useCallback((e: React.MouseEvent) => {
       e.preventDefault();
       e.stopPropagation();
       setIsResizing(true);
       document.body.style.userSelect = 'none';
       document.body.style.cursor = 'col-resize';

       const startX = e.clientX;
       const startWidth = emailColWidth;

       const handleMouseMove = (moveEvent: MouseEvent) => {
           const deltaX = moveEvent.clientX - startX;
           const newWidth = Math.min(MAX_EMAIL_WIDTH, Math.max(MIN_EMAIL_WIDTH, startWidth + deltaX));
           setEmailColWidth(newWidth);
       };

       const handleMouseUp = (upEvent: MouseEvent) => {
           setIsResizing(false);
           document.body.style.userSelect = '';
           document.body.style.cursor = '';
           window.removeEventListener('mousemove', handleMouseMove);
           window.removeEventListener('mouseup', handleMouseUp);
           const finalDelta = upEvent.clientX - startX;
           const finalWidth = Math.min(MAX_EMAIL_WIDTH, Math.max(MIN_EMAIL_WIDTH, startWidth + finalDelta));
           try {
               localStorage.setItem(STORAGE_KEY_EMAIL_WIDTH, String(finalWidth));
           } catch (err) {
               useErrorStore.getState().trackWarning(err, {
                   source: 'AccountTable.handleMouseUpResize',
                   triggerAction: 'localStorage.setItem',
               });
           }
       };

       window.addEventListener('mousemove', handleMouseMove);
       window.addEventListener('mouseup', handleMouseUp);
   }, [emailColWidth]);
   ```
4. Update the Email `<th>` element with width style and drag handle:
   ```tsx
   <th
       className="px-2 py-0.5 text-left rtl:text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider whitespace-nowrap relative select-none"
       style={{
           width: `${emailColWidth}px`,
           minWidth: `${emailColWidth}px`,
           maxWidth: `${emailColWidth}px`,
       }}
   >
       <span>{t('accounts.table.email')}</span>

       {/* Resizing Drag Handle */}
       <div
           onMouseDown={handleMouseDownResize}
           className={cn(
               "absolute right-0 top-0 bottom-0 w-2 cursor-col-resize flex items-center justify-center transition-colors group z-20",
               isResizing ? "bg-blue-500/30" : "hover:bg-blue-500/20"
           )}
           title="Drag to resize Email column (220px - 550px)"
       >
           <div
               className={cn(
                   "w-0.5 h-4 rounded-full transition-colors",
                   isResizing
                       ? "bg-blue-600 dark:bg-cyan-400 h-full"
                       : "bg-slate-300 dark:bg-slate-700 group-hover:bg-blue-500"
               )}
           />
       </div>
   </th>
   ```
5. Pass `emailColWidth={emailColWidth}` to `<SortableAccountRow />`.

### 3.3 Step 3: Forward Prop in `src/components/accounts/AccountTable/SortableAccountRow.tsx`
1. Accept `emailColWidth` from props and forward it to `<AccountRowContent {...props} emailColWidth={emailColWidth} />`.

### 3.4 Step 4: Apply Width in `src/components/accounts/AccountTable/AccountRowContent.tsx`
1. In `AccountRowContent.tsx`, extract `emailColWidth` from props.
2. Apply the dynamic width styling to the first `<td>` element:
   ```tsx
   <td
       className="px-2 py-0.5 align-middle border-b border-slate-200/90 dark:border-slate-800/90"
       style={emailColWidth ? {
           width: `${emailColWidth}px`,
           minWidth: `${emailColWidth}px`,
           maxWidth: `${emailColWidth}px`,
       } : undefined}
       onMouseLeave={() => setIsHoverUnmasked(false)}
   >
   ```

---

## 4. Visual & DOM Invariants
- The drag handle MUST be positioned at the right border (`right: 0`) of the Email `<th>`.
- The column width MUST never violate the strict boundaries `[220px, 550px]`.
- Resizing MUST be live and reflected immediately across both header and table body cells.
- The cursor during drag MUST remain `col-resize` and text selection MUST be disabled to prevent awkward text highlighting during mouse movement.
- On mouse release, the final width MUST persist to `localStorage` under `agm_account_table_email_width`.

---

## 5. Verification Checklist
- [ ] Hover over right edge of Email column header: cursor changes to `col-resize`, drag divider turns blue.
- [ ] Click and drag rightward: column expands smoothly up to 550px.
- [ ] Click and drag leftward: column shrinks smoothly down to 220px.
- [ ] Drag beyond 550px: column stops expanding at exactly 550px.
- [ ] Drag below 220px: column stops shrinking at exactly 220px.
- [ ] Release mouse button: new width is preserved.
- [ ] Refresh page: persisted width from `localStorage` is loaded correctly.
- [ ] Table rows remain in perfect alignment with column headers at all widths.
- [ ] Pre-flight checks: `npm run build` succeeds without TypeScript compilation errors.
