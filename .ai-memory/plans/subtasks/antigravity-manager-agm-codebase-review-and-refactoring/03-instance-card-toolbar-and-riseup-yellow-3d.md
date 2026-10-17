# Subtask 03: Instance Card & Account Card Toolbar Redesign & Riseup Yellow 3D Button

**Parent Plan:** `.ai-memory/plans/antigravity-manager-agm-codebase-review-and-refactoring.md`  
**Spec Reference:** [02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/02-component-and-ui-spec.md](../../../../02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/02-component-and-ui-spec.md)  
**Owner:** Worker 02  
**Status:** Queued  

---

## 1. Owned Files
- `src/components/accounts/AccountCard/AccountCardActions.tsx`
- `src/pages/instances/InstanceCardActions.tsx`
- `src/components/accounts/AccountTable/cells/ActionsCell.tsx`

---

## 2. Implementation Steps

### 2.1 Icon Reordering & Tag Relocation
1. In `src/components/accounts/AccountCard/AccountCardActions.tsx`:
   - Replace the 4th icon (`Tag` custom label) with the Fast-Forward button (`FastForward`, `>>`).
   - Wire Fast-Forward click handler to `useInstanceStore.getState().fastForwardInstance(...)` with loading spin state.
   - Relocate the `Tag` button into the overflow More menu popover/dropdown:
     - When clicked from the dropdown, trigger `setIsEditingLabel(true)`.
2. In `src/pages/instances/InstanceCardActions.tsx`:
   - Ensure the primary action capsule slots Fast-Forward alongside Launch/Stop and Switch Account.
   - Any tag/label action remains cleanly housed inside `cardMoreId` popover menu.

### 2.2 Idle Button Flattening
1. In `AccountCardActions.tsx` and `InstanceCardActions.tsx`:
   - **Switch Button (Yellow):**
     - When idle: `bg-transparent text-slate-500 dark:text-slate-400 border-transparent opacity-40 group-hover:opacity-100`.
     - On group hover: elevate to full visibility.
   - **Delete Button (Red):**
     - When idle: `bg-transparent text-slate-400 border-transparent opacity-40 group-hover:opacity-100`.
     - On group hover: `group-hover:bg-red-500/10 group-hover:text-red-500 group-hover:border-red-500/30`.
     - On direct hover: `hover:!bg-red-600 hover:!text-white hover:!border-red-700 shadow-xs`.

### 2.3 Authoritative Riseup Yellow 3D Gradient
Apply the exact 3D yellow gradient to the Switch button on hover/active states across both card and table action components:
- **Gradient Fill:** `bg-gradient-to-b from-[#FDE047] via-[#FACC15] to-[#EAB308]`
- **Border:** `border border-[#CA8A04]`
- **3D Depth Shadow:** `shadow-[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.2)]`
- **Active Press State:** `active:translate-y-0.5 active:shadow-[0_0_0_0_#A16207]`
- **Typography:** `text-slate-950 font-bold font-mono`

```tsx
className={cn(
    "p-1.5 rounded-[5px] transition-all font-bold inline-flex items-center justify-center",
    "opacity-40 group-hover:opacity-100",
    "group-hover:bg-gradient-to-b group-hover:from-[#FDE047] group-hover:via-[#FACC15] group-hover:to-[#EAB308]",
    "group-hover:border-[#CA8A04] group-hover:text-slate-950",
    "group-hover:shadow-[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.2)]",
    "active:translate-y-0.5 active:shadow-[0_0_0_0_#A16207]"
)}
```

---

## 3. Verification & Quality Gates
- `npm run build` passes with zero type errors.
- Visual inspection confirms Fast-Forward occupies 4th icon slot.
- Visual inspection confirms Switch and Delete buttons are flattened when idle and illuminate on group hover with 3D Riseup Yellow depth.
