# Subtask 40.1: Fix ModalDialog Sizing and Action Button Visibility

## Scope
Modify `src/components/common/ModalDialog.tsx` to:
1. Accept `maxWidth?: string` prop (default `'max-w-sm'`).
2. Add `max-h-[90vh] flex flex-col` to `.modal-box`.
3. Wrap children in a flex-1 scrollable container (`flex-1 min-h-0 overflow-y-auto`) so the modal never pushes buttons outside the viewport.
4. Ensure footer buttons are pinned to bottom with `shrink-0 pt-4 mt-3 border-t border-gray-100 dark:border-base-200`.

## Target Files
- `src/components/common/ModalDialog.tsx`
