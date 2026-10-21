# 02-ui-header-consolidation-spec: ModalHeader Iconification, DetailHeader Action Consolidation, and UI Layout Fixes Specification

- **Spec ID:** `02-spec/21-app/prompt-sending-and-enqueue-testing/02-ui-header-consolidation-spec.md`
- **Application:** Antigravity-Manager (`agm`, GUI, Prompt Tree Subsystem)
- **Status:** `APPROVED`
- **Lead Architect:** Antigravity Pairing Agent (Spec Authoring Subagent 2)
- **Associated Specs:**
  - `02-spec/21-app/prompt-sending-and-enqueue-testing/01-prompt-sending-and-enqueue-testing-spec.md`
  - `02-spec/21-app/145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/01-architecture-spec.md`
  - `02-spec/21-app/145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/02-component-and-ui-spec.md`
  - `AGENTS.md` (Segmented Pill Capsule Invariant & Dark-Glass Styling)

---

## 1. Executive Summary

This specification establishes the architectural, interaction, and styling standards for compacting and consolidating the header controls within the Antigravity-Manager **Project & Prompt Tree View** modal. 

Historically, user feedback and UI inspection revealed three critical design bottlenecks:
1. **ModalHeader Toolbar Bloat**: The top modal header contained text-heavy action buttons (`Reset`, `Refresh`, and `🖼️ Images`) inside the right-hand segmented capsule. This consumed over 280 horizontal pixels, forcing unwanted line wraps or header clipping on standard displays and squishing the instance path breadcrumb.
2. **DetailHeader Dispatch Action Fragmentation**: The prompt detail header rendered three separate split buttons side-by-side (`[ Copy | v ]`, `[ Send N | v ]`, `[ Enqueue | v ]`, and `[ Rerun xN | v ]`). Because `Send`, `Enqueue`, and `Rerun` all represent mutually exclusive dispatch operations toward the active conversation in the running IDE instance, rendering four disjoint split buttons created visual clutter, layout overflow, and cognitive friction.
3. **Z-Index Stacking & Relative Time Badge Clumping**: In `TreePanel.tsx`, the search and filter header lacked explicit stacking and backdrop blur, allowing scrolling conversation cards to bleed through. Unconstrained `flex-wrap` caused the `Hide Auto` pill to break onto a new line and overlap cards. Furthermore, tight card spacing (`space-y-0.5`) in `ProjectNode.tsx` and tight chip flex gaps in `PromptInstructionView.tsx` caused relative recency badges (`1d`, `2d`, `5d`) and turn chips (`T1`, `T2`) to clump together into an unreadable column.

This specification provides concrete component contracts, layout blueprints, and verification rules to solve each of these issues.

---

## 2. ModalHeader Iconification Specification

### 2.1 Architectural Invariant & Design Goals
Per `AGENTS.md`, all adjacent actions in the modal header must reside inside a single contiguous segmented dark-glass pill capsule (`rounded-full`, shared border, subtle divider lines, glass backdrop, tight typography). To maximize horizontal space for instance identifiers and sequence codes, non-dropdown actions must be iconified with high-contrast Lucide icons and informative hover tooltips.

### 2.2 Component Target: `src/components/instances/PromptTreeView/ModalHeader.tsx`

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [Layers] Prompt Tree View  v4.8.1  #1  Default  ...\Antigravity.exe                    │
│                                              [ 💾 | ↻ | ⟳ | 🖼️ | Sync: 15s ▾ | ⛶ | ✕ ] │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

#### 2.2.1 Reset Button Iconification
- **Previous State**: Text button with `RotateCcw` icon and `<span>{isResetting ? 'Resetting...' : 'Reset'}</span>` (width ~85px).
- **Consolidated State**: Pure icon button rendering Lucide `RotateCcw`.
- **Dimensions & Layout**: `w-7 h-7 flex items-center justify-center shrink-0 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer`.
- **Icon Styling**: `w-3.5 h-3.5 text-amber-500`, animated with `animate-spin` when `isResetting === true`.
- **Disabled State**: `disabled={isLoading || isResetting}` with `disabled:opacity-50 cursor-not-allowed`.
- **Tooltip**: Mandatory explicit attribute:
  ```html
  title="Reset & Regenerate Tree (Wipe SQLite DB and rebuild prompt tree and sequences fresh from disk transcripts)"
  ```
- **Accessibility**: `aria-label="Reset & Regenerate Tree"`.

#### 2.2.2 Refresh Button Iconification
- **Previous State**: Text button with `RefreshCw` icon and `<span>Refresh</span>` (width ~80px).
- **Consolidated State**: Pure icon button rendering Lucide `RefreshCw`.
- **Dimensions & Layout**: `w-7 h-7 flex items-center justify-center shrink-0 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer`.
- **Icon Styling**: `w-3.5 h-3.5 text-blue-500`, animated with `animate-spin` when `isLoading || isAutoSyncing || isRefreshing`.
- **Disabled State**: `disabled={isLoading}` with `disabled:opacity-50 cursor-not-allowed`.
- **Tooltip**: Mandatory explicit attribute:
  ```html
  title="Refresh All Projects & Prompts (Scan disk transcripts and update active prompts)"
  ```
- **Accessibility**: `aria-label="Refresh All Projects & Prompts"`.

#### 2.2.3 Images Filter Toggle Button Modernization
- **Previous State**: Raw emoji `<span>🖼️</span>` with text label `<span>Images</span>` and harsh opaque `bg-sky-600` when active.
- **Consolidated State**: Modernized Lucide `ImageIcon` (`Image as ImageIcon`) button with dark-glass styling.
- **Dimensions & Layout**: `w-7 h-7 flex items-center justify-center shrink-0 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer`.
- **Active State Dark-Glass Styling**:
  - Active: `bg-sky-500/25 text-sky-400 border-x border-sky-500/40 shadow-2xs`.
  - Inactive: `text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90`.
- **Icon Styling**: `<ImageIcon className="w-3.5 h-3.5" />`.
- **Tooltip**:
  ```tsx
  title={isImageFilterActive ? "Showing only conversations with images (Click to show all)" : "Filter conversations with images"}
  ```
- **Accessibility**: `aria-label="Toggle images filter"`.

### 2.3 Segment Sequence Contract in ModalHeader Capsule
The right-hand capsule in `ModalHeader.tsx` must maintain the exact order:
1. `Segment 1`: Import / Export Dropdown (`ImportExportMenu`, icon: `ArrowUpDown` or `Download`)
2. `Segment 2`: Reset Icon Button (`RotateCcw`, amber icon, wipe & rebuild DB)
3. `Segment 3`: Refresh Icon Button (`RefreshCw`, blue icon, transcript re-scan)
4. `Segment 4`: Images Filter Toggle (`ImageIcon`, dark-glass sky glow when active)
5. `Segment 5`: Sync Interval Selector Dropdown (`Clock` + interval badge + `ChevronDown`)
6. `Segment 6`: Fullscreen Toggle (`Maximize2` / `Minimize2`)
7. `Segment 7`: Close Button (`X`, rose hover state)

---

## 3. DetailHeader Action Consolidation Specification

### 3.1 Architectural Invariant & Design Goals
The Detail Header toolbar serves as the primary operational surface for inspecting, copying, and dispatching prompts.
Previously, four separate split dropdowns occupied the right side of `DetailHeader.tsx`:
- `[ Copy | v ]`
- `[ Send N | v ]`
- `[ Enqueue | v ]`
- `[ Rerun xN | v ]`

This violated the single-purpose interaction model. In practice:
- **Copying** is an outbound clipboard / file-export action.
- **Send**, **Enqueue**, and **Rerun** are execution dispatch actions targeting the active conversation in the running IDE instance.

The consolidated architecture reduces the toolbar to **two adjacent segmented capsules**:
1. **Capsule A (Clipboard & Export Capsule)**: `[ Copy | v ]` (preserves all multi-format copy options: rich HTML, plain text, single PNG, composite image PNG, markdown, JSON).
2. **Capsule B (Unified Dispatch Capsule)**: `[ Primary Action | v ]` (single segmented split button unifying Send, Enqueue, and Rerun).

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ CONV-001  [User Prompt]  ● RUNNING (PID: 1234 · 01:42)       [ Copy | ▾ ] [ Send Now ▾ ]│
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Component Target: `src/components/instances/PromptTreeView/DetailHeader.tsx`

#### 3.2.1 Primary Action Trigger Contract
The left half of the unified dispatch split button executes the currently selected dispatch mode with a single click:

| Active Mode | Button Label | Icon | Hotkey Badge | Click Behavior | Hover Style |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `send` (*Default*) | `Send` or `Send Now` | `RotateCw` (emerald-500, `animate-spin` on `isResending`) | `<kbd>N</kbd>` | Triggers `onResend()` directly | `text-emerald-700 dark:text-emerald-300 hover:bg-emerald-100/80 dark:hover:bg-[#40285e]` |
| `enqueue` | `Enqueue` | `ListPlus` (amber-500, `animate-pulse` on `isEnqueueing`) | None | Triggers `onEnqueue()` directly | `text-amber-800 dark:text-amber-300 hover:bg-amber-100/80 dark:hover:bg-[#40285e]` |
| `rerun` | `Rerun ×{rerunCount}` | `Repeat` (sky-500, `animate-spin` on `isRerunning`) | None | Triggers `handleRerun()` directly | `text-sky-700 dark:text-sky-300 hover:bg-sky-100/80 dark:hover:bg-[#40285e]` |

```tsx
// Primary Action Button Implementation Pattern
<button
    type="button"
    onClick={handlePrimaryDispatch}
    disabled={isPrimaryActionDisabled}
    data-ui-name="prompt-primary-dispatch-btn"
    data-ui-id={`prompt-primary-dispatch-${selectedConversation.conversation_id}`}
    className={cn(
        "flex items-center gap-1.5 px-3 py-1 text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50",
        dispatchMode === 'send' && "text-emerald-700 dark:text-emerald-300 hover:bg-emerald-100/80 dark:hover:bg-[#40285e]",
        dispatchMode === 'enqueue' && "text-amber-800 dark:text-amber-300 hover:bg-amber-100/80 dark:hover:bg-[#40285e]",
        dispatchMode === 'rerun' && "text-sky-700 dark:text-sky-300 hover:bg-sky-100/80 dark:hover:bg-[#40285e]"
    )}
    aria-label={primaryActionAriaLabel}
>
    {renderPrimaryActionIcon()}
    <span>{primaryActionLabel}</span>
    {dispatchMode === 'send' && (
        <kbd className="ml-0.5 px-1 py-0.2 bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 rounded text-[8.5px] font-mono font-bold">N</kbd>
    )}
</button>
```

#### 3.2.2 Chevron Toggle & Popover Flyout Architecture
The right half of the split button renders a chevron toggle (`ChevronDown`):
- Clicking toggles `isDispatchMenuOpen`.
- An accessible popover opens directly beneath the split button (`w-80 rounded-xl bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-xl p-2.5 backdrop-blur-md`).

```
┌────────────────────────────────────────────────────────┐
│ DISPATCH & EXECUTION CONFIGURATION            AGM:P001 │
├────────────────────────────────────────────────────────┤
│ Mode: [ Send Now ]  [ Enqueue FIFO ]  [ Rerun Turn ]   │
├────────────────────────────────────────────────────────┤
│ (Contextual Controls: Destination, Suffix, Times, Msg) │
├────────────────────────────────────────────────────────┤
│ [ Focus IDE Window                                   ] │
│ [ Full Inspector                                     ] │
├────────────────────────────────────────────────────────┤
│ [ Primary Action Execution Button                    ] │
│ FIFO Queue & Direct Dispatch Invariant                 │
└────────────────────────────────────────────────────────┘
```

#### 3.2.3 Flyout Mode Switcher Segmented Pills
The top section of the flyout contains a 3-way segmented pill bar allowing instant switching of the active primary dispatch mode:
1. **Send Now Pill**: Sets `dispatchMode = 'send'`. Highlights with `bg-emerald-500/20 text-emerald-600 dark:text-emerald-300 font-semibold`.
2. **Enqueue FIFO Pill**: Sets `dispatchMode = 'enqueue'`. Highlights with `bg-amber-500/20 text-amber-700 dark:text-amber-300 font-semibold`.
3. **Rerun Turn Pill**: Sets `dispatchMode = 'rerun'`. Highlights with `bg-sky-500/20 text-sky-600 dark:text-sky-300 font-semibold`.

#### 3.2.4 Contextual Parameters in Flyout
The body of the popover dynamically renders controls appropriate for the selected mode:

- **When `dispatchMode === 'send'` or `'enqueue'`**:
  - **Destination Selector**: Dropdown choosing between `Current Conv` and `New Conv`. If `New Conv` is selected, an inline text input appears for `New Conversation Title`.
  - **Suffix / Verification Template**: Dropdown selector (`None`, `Verification Presets`, `Custom Suffix Templates`).
  - **Quick IDE Actions**: `Focus IDE Window` (brings target IDE instance to foreground) and `Full Inspector` (expands full transcript modal).
  - **Bottom CTA**: Green `Send Immediately` button or Amber `Enqueue to FIFO Queue` button.

- **When `dispatchMode === 'rerun'`**:
  - **Times (N) Selector**: Segmented number pills `[1] [2] [3] [4] [5]` setting `rerunCount`.
  - **Message Option Selector**: Radio buttons for `Is it done?`, `Continue`, or `Randomized` (with preview of the random message pool).
  - **Image Attachment**: Staged image file picker or thumbnail preview with remove button.
  - **Bottom CTA**: Sky-tinted `Rerun ×{rerunCount} Now` button with spin animation.

#### 3.2.5 Preservation of the Adjacent Copy Split Button
The `[ Copy | v ]` split button remains directly adjacent on the left of the dispatch capsule. It retains all rich clipboard features:
- Primary button: Copy full prompt text + all embedded images (rich HTML + binary PNG composite).
- Flyout menu: Copy Clean Text, Copy Prompt with Local File Links (`file:///`), Copy with Base64 URIs, Copy Single Image PNG, Copy All Images Composite, Export Markdown (`.md`), Export JSON (`.json`).

---

## 4. UI Overlap & Layout Fixes

### 4.1 `TreePanel.tsx`: Header Overlap & Stacking Fix

#### 4.1.1 Defect Description
In `src/components/instances/PromptTreeView/TreePanel.tsx`, the search and filter header was placed above the scrollable project tree list without sticky positioning or an explicit stacking context:
```tsx
// DEFECTIVE PREVIOUS IMPLEMENTATION:
<div className="w-full flex-1 min-h-0 flex flex-col overflow-hidden">
    <div className="p-3 border-b border-slate-200 dark:border-[#40285e] shrink-0 space-y-2">
```
When user scrolled down the tree list, conversation cards scrolled beneath the semi-transparent filter controls. Furthermore, row 2 used unconstrained `flex-wrap`:
```tsx
// DEFECTIVE PREVIOUS IMPLEMENTATION:
<div className="flex items-center justify-between gap-1.5 pt-0.5 flex-wrap">
```
At narrow panel widths (<320px), the `[ Hide Auto | Scope ]` capsule wrapped onto a second line, visually colliding with and overlapping the top-most conversation card.

#### 4.1.2 Remediation Contract
1. **Explicit Stacking & Dark-Glass Backdrop**:
   Add sticky positioning, explicit z-index (`z-10`), and an opaque dark-glass backdrop to the filter container:
   ```tsx
   <div className="sticky top-0 z-10 p-3 border-b border-slate-200 dark:border-[#40285e] shrink-0 space-y-2 bg-slate-50/95 dark:bg-[#0c0817]/95 backdrop-blur-md">
   ```
2. **Elimination of Unconstrained `flex-wrap`**:
   Remove `flex-wrap` from row 2 and enforce rigid horizontal layout with `shrink-0` controls or compact styling:
   ```tsx
   <div className="flex items-center justify-between gap-1.5 pt-0.5 min-w-0">
   ```
3. **No Bleed-Through Guarantee**:
   The tree list container below the header (`<div className="flex-1 overflow-y-auto ...">`) must scroll cleanly behind the opaque backdrop without visible text bleed-through or card collision.

---

### 4.2 `PromptInstructionView.tsx` & `ProjectNode.tsx`: Relative Time Badge Clumping Fix

#### 4.2.1 Defect Description
Relative recency badges (`1d`, `2d`, `5d`, `3h`, `just now`) were rendered on the right side of conversation cards in `ProjectNode.tsx` / `ConversationNode.tsx` and in turn chip rows in `PromptInstructionView.tsx`.
- In `ProjectNode.tsx` and `ConversationNode.tsx`, conversation cards were rendered with ultra-compressed vertical spacing:
  ```tsx
  // DEFECTIVE PREVIOUS IMPLEMENTATION:
  <div key={`root-${root.primaryNode.conversation_id}`} className="space-y-0.5">
  <div className="pl-5 space-y-0.5 border-l-2 ...">
  ```
  When multiple finished conversations were listed consecutively, their right-aligned recency badges formed a visually clumped vertical stack with less than 2px separation, looking like an unreadable block of numbers.
- In `PromptInstructionView.tsx`, the chronological turn chips used tight gap spacing:
  ```tsx
  // DEFECTIVE PREVIOUS IMPLEMENTATION:
  <div className="flex items-center gap-1.5 flex-wrap">
  ```
  When conversations contained 10+ turns with status dots and image badges, the chips crowded together with insufficient vertical and horizontal line spacing.

#### 4.2.2 Remediation Contract
1. **Increase Card Vertical Spacing in `ProjectNode.tsx`**:
   - Change root conversation container spacing from `space-y-0.5` to `space-y-1.5`.
   - Change nested sub-point container spacing from `space-y-0.5 my-0.5` to `space-y-1.5 my-1`.
   - Change active project list spacing from `space-y-1` to `space-y-2`.
2. **Increase Turn Item Padding in `ConversationNode.tsx`**:
   - For multi-turn expanded views inside conversation cards, increase turn row padding from `py-0.5 px-2` to `py-1 px-2.5` and set `gap-2` between the title and the recency badge.
3. **Expand Chip Flex Gaps in `PromptInstructionView.tsx`**:
   - Change turn chip container from `gap-1.5 flex-wrap` to `gap-2 flex-wrap`.
   - Update turn chip dimensions to provide adequate breathing room: `px-2.5 py-1 text-xs font-mono font-semibold shadow-2xs`.
   - In the Session Overview metrics grid, ensure grid cells have `gap-2.5` to prevent text truncation.

---

## 5. Visual Component Contract

### 5.1 Component Props Specifications

#### 5.1.1 `ModalHeaderProps` Interface (Updated)
```typescript
export interface ModalHeaderProps {
    instanceName: string;
    appVersion?: string;
    sequenceNumber?: number;
    executablePath?: string;
    isFullscreen: boolean;
    setIsFullscreen: (v: boolean | ((prev: boolean) => boolean)) => void;
    isLoading: boolean;
    syncInterval: SyncInterval;
    setSyncInterval: (v: SyncInterval) => void;
    restoreFileInputRef: React.RefObject<HTMLInputElement | null>;
    onClose: () => void;
    onBackup: () => void;
    onRestore: () => void;
    onExportSqlite: () => void;
    onImportSqlite: () => void;
    onCopyAllTreePrompts: () => Promise<void> | void;
    onFileRestore: (e: React.ChangeEvent<HTMLInputElement>) => void;
    onRefreshAll: () => void;
    onResetTree: () => void;
    isTreeCollapsed?: boolean;
    onToggleTree?: () => void;
    isAutoSyncing?: boolean;
    isRefreshing?: boolean;
    isResetting?: boolean;
}
```

#### 5.1.2 `DetailHeaderProps` Interface (Consolidated)
```typescript
export type DispatchActionMode = 'send' | 'enqueue' | 'rerun';

export interface DetailHeaderProps {
    previewTab?: 'instruction' | 'results';
    setPreviewTab?: (v: 'instruction' | 'results') => void;
    showAllWords?: boolean;
    setShowAllWords?: (v: boolean | ((prev: boolean) => boolean)) => void;
    isTruncated?: boolean;
    selectedConversation: AgmConversationNode;
    activePromptText: string;
    confirmationSuffix: string;
    setConfirmationSuffix: (v: string) => void;
    conversationTargetMode?: 'current' | 'new';
    setConversationTargetMode?: (v: 'current' | 'new') => void;
    newConversationTitle?: string;
    setNewConversationTitle?: (title: string) => void;
    isCopiedText: boolean;
    setIsCopiedText: (v: boolean) => void;
    isCopiedRaw: boolean;
    isCopied: boolean;
    setIsCopied: (v: boolean) => void;
    isCopiedAll?: boolean;
    setIsCopiedAll?: (v: boolean) => void;
    isEnqueueing: boolean;
    isResending: boolean;
    isFocusing: boolean;
    isCopyMenuOpen?: boolean;
    setIsCopyMenuOpen?: (v: boolean | ((prev: boolean) => boolean)) => void;
    isDispatchMenuOpen?: boolean;
    setIsDispatchMenuOpen?: (v: boolean | ((prev: boolean) => boolean)) => void;
    dispatchMode?: DispatchActionMode;
    setDispatchMode?: (mode: DispatchActionMode) => void;
    hasImages: boolean;
    instancePid?: number;
    instanceSeqNum: number;
    instanceExeName: string;
    instanceNameDisplay: string;
    formatDuration: (secs: number) => string;
    elapsedSeconds: number;
    onCopyAllWithRichImages?: () => void;
    onCopyCleanText?: () => void;
    onCopyImageAsText?: () => void;
    onCopyImageForOneNoteOrIde?: () => void;
    onCopySingleImagePng?: () => void;
    onCopyWithImages: () => void;
    onSaveImages: () => void;
    onExport: (format: 'md' | 'json') => void;
    onFocusIde: () => void;
    onResend: () => void;
    onEnqueue: () => void;
    rerunRepoPath?: string;
    onOpenInspector: (conv: AgmConversationNode) => void;
    onRenameConversation?: (conversationId: string, newTitle: string, instId?: string) => Promise<void>;
    selectedTurnNumber?: number | null;
    setIsDetailsModalOpen?: (v: boolean) => void;
}
```

---

## 6. Exact Verification Checklist

The following verification matrix defines the non-negotiable test gates for implementation:

| Test ID | Area | Action / Test Case | Expected Outcome | Pass/Fail Gate |
| :--- | :--- | :--- | :--- | :--- |
| **TC-01** | `ModalHeader` | Inspect Reset button | Button is icon-only (`RotateCcw`, amber icon, `w-7 h-7`). No `Reset` text rendered. | Pass |
| **TC-02** | `ModalHeader` | Hover over Reset button | Native browser tooltip displays: `"Reset & Regenerate Tree (Wipe SQLite DB and rebuild prompt tree and sequences fresh from disk transcripts)"`. | Pass |
| **TC-03** | `ModalHeader` | Click Reset button | Triggers `onResetTree()`. `RotateCcw` spins while `isResetting === true`. Button is disabled during reset. | Pass |
| **TC-04** | `ModalHeader` | Inspect Refresh button | Button is icon-only (`RefreshCw`, blue icon, `w-7 h-7`). No `Refresh` text rendered. | Pass |
| **TC-05** | `ModalHeader` | Hover over Refresh button | Native browser tooltip displays: `"Refresh All Projects & Prompts (Scan disk transcripts and update active prompts)"`. | Pass |
| **TC-06** | `ModalHeader` | Click Refresh button | Triggers `onRefreshAll()`. `RefreshCw` spins while refreshing. Button is disabled during refresh. | Pass |
| **TC-07** | `ModalHeader` | Inspect Images toggle | Uses Lucide `ImageIcon` (`Image as ImageIcon`). No raw emoji `🖼️` or text `Images`. | Pass |
| **TC-08** | `ModalHeader` | Toggle Images filter ON | Button adopts dark-glass sky styling: `bg-sky-500/25 text-sky-400 border-x border-sky-500/40`. Only image conversations shown in tree. Tooltip updates to `"Showing only conversations with images (Click to show all)"`. | Pass |
| **TC-09** | `DetailHeader` | Action toolbar structure | Toolbar contains exactly two adjacent segmented capsules: `[ Copy | ▾ ]` and `[ Primary Action | ▾ ]`. | Pass |
| **TC-10** | `DetailHeader` | Primary action default mode | Default mode is `Send Immediately` (Emerald `RotateCw` icon, `Send` label, `<kbd>N</kbd>` badge). Clicking sends prompt immediately. | Pass |
| **TC-11** | `DetailHeader` | Open Dispatch Flyout | Clicking chevron `▾` opens consolidated flyout. Flyout header displays `DISPATCH CONFIGURATION AGM:P001`. | Pass |
| **TC-12** | `DetailHeader` | Switch Mode to Enqueue | Clicking `Enqueue FIFO` pill in flyout updates primary split button to amber `ListPlus` icon + `Enqueue` label. Clicking primary executes `onEnqueue()`. | Pass |
| **TC-13** | `DetailHeader` | Switch Mode to Rerun | Clicking `Rerun Turn` pill in flyout updates primary split button to sky `Repeat` icon + `Rerun ×{N}` label. Contextual controls switch to rerun repeat count and message options. | Pass |
| **TC-14** | `DetailHeader` | Copy button preservation | Adjacent `[ Copy | ▾ ]` button remains operational with all formats: full HTML/image composite, plain text, PNG, Base64, MD, JSON export. | Pass |
| **TC-15** | `TreePanel` | Header background & stacking | Tree header has `sticky top-0 z-10 bg-slate-50/95 dark:bg-[#0c0817]/95 backdrop-blur-md`. As tree scrolls down, cards never bleed through header. | Pass |
| **TC-16** | `TreePanel` | No `flex-wrap` overlap | Filter rows do not wrap. `Hide Auto` / `Show Auto` pill remains inline without overlapping conversation cards below. | Pass |
| **TC-17** | `ProjectNode` | Time badge spacing | Vertical spacing between conversation cards is `space-y-1.5` / `space-y-2`. Relative time badges (`1d`, `2d`, `5d`) do not clump into tight stacks. | Pass |
| **TC-18** | `PromptInstructionView` | Turn chip flex gap | Chronological turn chips (`T1`, `T2`, `T3...`) use `gap-2` flex spacing. Multi-turn sequences render with clean breathing room. | Pass |

---

## 7. Migration & Rollback Strategy

1. **Additive & Backward-Compatible State**:
   - `dispatchMode` defaults to `'send'`, maintaining 100% functional parity with the existing default send behavior.
   - All underlying handler functions (`onResend`, `onEnqueue`, `handleRerun`, `handleCopyAll`, `onResetTree`, `onRefreshAll`) remain intact and are invoked via identical props.
2. **Rollback Plan**:
   - If visual defects occur on specific OS platforms (e.g. Linux WebKit rendering of dark-glass filters), the CSS classes can be reverted cleanly per-component without altering backend IPC calls or database schemas.
