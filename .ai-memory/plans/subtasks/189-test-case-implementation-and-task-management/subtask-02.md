# Subtask 02: UI Accessibility Modernization for Toast and ShortcutLegendCard

## Scope & Target Files
- `src/components/common/Toast.tsx` (Step 3 - Track C Step 23)
- `src/components/common/ShortcutLegendCard.tsx` (Step 4 - Track C Step 24)

## Status
`[COMPLETED]`

## Completed Tasks
- [x] Add dynamic `role={type === 'error' ? 'alert' : 'status'}` and `aria-live={type === 'error' ? 'assertive' : 'polite'}` in `Toast.tsx`.
- [x] Add `aria-atomic="true"` and accessible `aria-label="Dismiss notification"` on Toast dismiss button.
- [x] Add `role="region"` and `aria-label="Keyboard Shortcuts"` to `ShortcutLegendCard.tsx`.
- [x] Enhance `<kbd>` elements with screen-reader friendly descriptions (`<span className="sr-only">Shortcut keys: ...</span>`).
- [x] Add accessible label `aria-label="Close keyboard shortcuts legend"` to dismiss button.
