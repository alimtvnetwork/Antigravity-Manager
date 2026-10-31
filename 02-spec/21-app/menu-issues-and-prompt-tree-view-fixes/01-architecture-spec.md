# Architecture Specification: Singleton Context Menu & Dismissal Guard

## 1. Problem Classification & Root Cause Analysis

### 1.1 Double / Ghost Context Menu Overlap
- **Symptom**: User uploaded screenshot `media_1792347120731_45f06689.png` showing two identical context menus rendered simultaneously in the DOM with a ~15px horizontal displacement, producing duplicated shortcut labels (`Inspect nspect`, `> >`, `Queue Queue`, `Rerun Rerun`, `Copy Copy`, `UUID UUID`, `Pin Pin`, `F2 F2`, `Audit Audit`, `Handoff andoff`, `.json .json`, `.db ... .db`, `Delete Delete`).
- **Root Cause**:
  In `src/components/instances/PromptTreeView/ConversationNode.tsx`, two separate local React states exist:
  ```tsx
  const [contextMenuPos, setContextMenuPos] = useState<{ x: number; y: number } | null>(null);
  const [turnMenuPos, setTurnMenuPos] = useState<{ x: number; y: number; turnConv: AgmConversationNode } | null>(null);
  ```
  And in `src/components/instances/PromptTreeView/ProjectNode.tsx`:
  ```tsx
  const [projectMenuPos, setProjectMenuPos] = useState<{ x: number; y: number } | null>(null);
  const [subagentMenuPos, setSubagentMenuPos] = useState<{ x: number; y: number; conv: AgmConversationNode } | null>(null);
  ```
  When right-clicking a turn inside a conversation that already had an active context menu, `turnMenuPos` was set without resetting `contextMenuPos`. Both states rendered `<PromptTreeContextMenu>` via `createPortal(..., document.body)`. Because every tree node maintained its own isolated state, independent menus mounted concurrently.

### 1.2 Menu Dismissal Failures ("Menu does not go away")
- **Symptom**: Clicking outside the menu, scrolling the window, or pressing Escape failed to reliably dismiss active context menus.
- **Root Cause**:
  `PromptTreeContextMenu.tsx` registered `document.addEventListener('mousedown', handleClickOutside)` without `{ capture: true }`. Clicking on UI buttons, modal scrollbars, or tree elements invoked `e.stopPropagation()`, intercepting the bubbling phase and preventing `handleClickOutside` from executing. Additionally, `Date.now() - mountTime < 100` dropped early clicks, and window blur / global right-clicks lacked capture listeners.

---

## 2. Architectural Solution: Centralized Singleton Context Menu

### 2.1 State Hoisting to Modal / Panel Level
Instead of scattering context menu states across hundreds of tree node components, elevate context menu state to the top-level tree container (`src/components/instances/PromptTreeView/TreePanel.tsx` or `PromptTreeViewModal.tsx`):

```typescript
export interface ActiveContextMenuState {
    type: 'project' | 'conversation' | 'turn';
    x: number;
    y: number;
    projectId?: string;
    projectName?: string;
    project?: AgmProjectTreeNode;
    conversationId?: string;
    currentTitle?: string;
    conversationNode?: AgmConversationNode;
    repoPath: string;
    instanceId?: string;
    isProjectPinned?: boolean;
    isProjectArchived?: boolean;
    isPinned?: boolean;
}
```

### 2.2 Transparent Full-Screen Interceptor Layer
When `activeContextMenu` is non-null, render a transparent full-screen backdrop:
```tsx
{activeContextMenu && (
    <div
        className="fixed inset-0 z-[99998] bg-transparent cursor-default"
        onClick={handleCloseContextMenu}
        onContextMenu={(e) => {
            e.preventDefault();
            handleCloseContextMenu();
        }}
    />
)}
```
Any outside click, right-click, or tap hits this transparent barrier, immediately unmounting the menu.

### 2.3 Window Capture-Phase Listeners
In the singleton context menu component:
- `window.addEventListener('pointerdown', handleDismiss, { capture: true })`
- `window.addEventListener('contextmenu', handleDismiss, { capture: true })`
- `window.addEventListener('keydown', (e) => e.key === 'Escape' && onClose(), { capture: true })`
- `window.addEventListener('scroll', handleDismiss, { capture: true })`
- `window.addEventListener('blur', onClose)`

---

## 3. Disjoint Component Responsibilities
- `TreePanel.tsx`: Holds `activeContextMenu` state. Renders the singleton `<PromptTreeContextMenu>` and transparent backdrop.
- `ProjectNode.tsx`: Dispatches `onOpenContextMenu(...)` for project row and subagent rows. No local menu portals.
- `ConversationNode.tsx`: Dispatches `onOpenContextMenu(...)` for conversation row and turn rows. No local menu portals.
- `PromptTreeContextMenu.tsx`: Strict singleton render at `z-[99999]`.
