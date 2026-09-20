# Subtask: UI Error Manager Drawer Dark Mode Contrast

**Target Files:**
- `src/components/errors/error-history-drawer.tsx`
- `src/components/errors/error-details-dialog.tsx`

**Action:**
1. In `src/components/errors/error-history-drawer.tsx`:
   - Replace uncompiled `dark:bg-base-200/50` on error cards with `bg-white dark:bg-slate-800/90 border border-gray-200 dark:border-slate-700/80 shadow-xs`.
   - Update header, drawer body, search input, and filter chips with explicit dark text: `text-gray-900 dark:text-slate-100`.
   - Fix card body error message contrast: `text-gray-700 dark:text-slate-300 font-mono text-xs`.
   - Ensure badges and time chips have sharp, readable colors in dark mode.
2. In `src/components/errors/error-details-dialog.tsx`:
   - Ensure stack trace and error payloads render on dark slate backgrounds with high-contrast text.

**Constraints:**
- No white card backgrounds in dark mode.
- Relative git paths only.
- Strict implicit booleans.
