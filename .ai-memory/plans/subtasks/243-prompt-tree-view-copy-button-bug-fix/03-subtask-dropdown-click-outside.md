# Subtask 03: Dropdown Popover Click-Outside Dismissal

**Task Slug**: `prompt-tree-view-copy-button-bug-fix`  
**Subtask**: `03-subtask-dropdown-click-outside`  
**Sequence**: `243-03`  
**Spec Reference**: `02-spec/21-app/243-prompt-tree-view-copy-button-bug-fix/02-component-and-attachment-spec.md`  

## Objectives
1. Import `useClickOutside` from `src/hooks/useClickOutside.ts` into `DetailHeader.tsx`.
2. Wrap Capsule A + Flyout 1 in `copyMenuContainerRef` and auto-dismiss on outside click.
3. Wrap Capsule B + Flyout 2 in `dispatchMenuContainerRef` and auto-dismiss on outside click.
4. Wrap Capsule C + Flyout 3 in `breakdownMenuContainerRef` and auto-dismiss on outside click.
5. Retain `onMouseLeave` or keyboard escape as secondary dismiss options.
