# Component Specification: SSH Fleet & Prompt Tree System

> **Spec Document:** `02-component-spec.md`  
> **Parent Slug:** `ssh-and-prompt-management-testing`  
> **Architecture Spec Reference:** [02-spec/21-app/ssh-and-prompt-management-testing/01-architecture-spec.md](02-spec/21-app/ssh-and-prompt-management-testing/01-architecture-spec.md)  
> **Target Scope:** UI component contracts, props, state flows, dark-glass styling standards, and multimodal clipboard integrations.  
> **Author:** `@aukgit`  

---

## 1. Executive Summary & Design System Standards

The SSH Fleet and Prompt Management system provides high-density observability and control over multi-machine nodes and local execution instances. This component specification codifies the UI components responsible for prompt tree navigation, multimodal clipboard workflows, fleet telemetry visualization, and Telegram inbound management.

### Design System Invariants
1. **Contiguous Segmented Dark-Glass Capsules**: Adjacent action buttons, window controls, and mode switchers must be encapsulated inside unified segmented containers (`rounded-lg` or `rounded-md`, border `border-slate-200/80 dark:border-[#40285e]/90`, background `bg-slate-100/90 dark:bg-[#171025]/90`, `backdrop-blur-md`, subtle dividers `divide-x divide-slate-200/70 dark:divide-[#40285e]/80`). Loose circular or disjointed buttons are strictly prohibited.
2. **Transparent Borderless Dropdowns**: Any `<select>` elements mounted inside capsules must implement transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white pill-in-pill visual defects.
3. **4-Tier Prompt Origin Classification**: All prompts and turns render within the standardized hierarchy:
   - `USER_PROMPT` (Sky badge, User icon)
   - `SUBAGENT_INSTRUCTION` (Purple badge, Bot icon)
   - `SYSTEM_MESSAGE` (Zinc badge, Terminal icon)
   - `TOOL_OUTPUT` (Amber badge, Wrench icon)
4. **Credential Redaction Invariant**: Never render administrative usernames (`Administrator`, `root`, `${adminuser}`) or password hints in card headers, tables, or summary tooltips.
5. **Default-Hidden Collapsible Tree & Automated Filter**: The Prompt Tree panel defaults to collapsed (`isTreeCollapsed = true`) and automated entries default to filtered (`isAutomatedHidden = true`) to maximize screen real estate for inspection.

---

## 2. Component Specifications

### 2.1 `PromptTreeViewModal`
- **File Location:** `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
- **Role:** Top-level modal container orchestrating prompt tree data loading, state synchronization, filtering, and detail inspection.

#### Props Interface (`PromptTreeViewModalProps`)
```typescript
interface PromptTreeViewModalProps {
    isOpen: boolean;
    onClose: () => void;
    instanceId: string;
    instanceName: string;
    sequenceNumber?: number;
    executablePath?: string;
    initialSelectedProjectId?: string;
}
```

#### State & Subsystem Hooks
- `usePromptTreeState(instanceId)`: Supplies reactive state including `treeData`, `isTreeCollapsed`, `isAutomatedHidden`, `activeFilter`, `categoryFilter`, `selectedProjectScope`, `selectedConversation`, `activePromptText`, and `syncInterval`.
- `usePromptTreeData({ state, instanceId, isOpen })`: Manages background polling, project tree fetching (`list_all_prompts`, `scan_conversations`), and conversation selection.
- `usePromptTreeCoreActions`: Handles backup, restore, SQLite import/export, and project pin/archive operations.
- `usePromptTreePromptActions`: Handles rerun, resend, and queuing dispatches.

#### Error Boundary (`PromptTreeErrorBoundary`)
Renders an inline error recovery UI if transcript parsing or malformed JSONL causes a crash:
- Captures stack traces via `useErrorStore.getState().trackWarning`.
- Displays error message in a dark-glass monospace container (`bg-slate-50 dark:bg-[#211631] border border-slate-200 dark:border-[#40285e]`).
- Provides a "Reset Details View" button to restore UI stability without closing the modal.

#### Layout Composition
```
┌────────────────────────────────────────────────────────────────────────┐
│ ModalHeader: [Layers] Title #Seq InstanceName ...\exe | [Capsule 1] [Capsule 2] │
├────────────────────────────┬───────────────────────────────────────────┤
│ TreePanel (Collapsible)    │ DetailHeader: Conversation Title & Capsule Actions│
│ - Project Scope Selector   ├───────────────────────────────────────────┤
│ - Search Input             │ DetailTabs: Instruction vs Results View   │
│ - 4-Tier Category Capsule  ├───────────────────────────────────────────┤
│ - Status Filter Capsule    │ Detail Content Area:                      │
│ - Tree Nodes / Stale Group │ - PromptInstructionView (Markdown/Images) │
│                            │ - ResultsView (Tool outputs / Logs)       │
└────────────────────────────┴───────────────────────────────────────────┘
```

---

### 2.2 `TreePanel`
- **File Location:** `src/components/instances/PromptTreeView/TreePanel.tsx`
- **Role:** Collapsible navigation sidebar rendering projects, conversations, prompt origin tags, and filters.

#### Props Interface (`TreePanelProps`)
```typescript
type ActiveFilter = 'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived';
type CategoryFilter = 'all' | 'user' | 'subagent' | 'system';

interface TreePanelProps {
    isLoading?: boolean;
    treeData: AgmProjectTreeNode[];
    activeProjects: AgmProjectTreeNode[];
    archivedProjects: AgmProjectTreeNode[];
    archivedCount: number;
    searchQuery: string;
    setSearchQuery: (v: string) => void;
    activeFilter: ActiveFilter;
    setActiveFilter: (v: ActiveFilter) => void;
    categoryFilter: CategoryFilter;
    setCategoryFilter: (v: CategoryFilter) => void;
    selectedProjectScope: string;
    setSelectedProjectScope: (v: string) => void;
    isArchivedProjectsExpanded: boolean;
    setIsArchivedProjectsExpanded: (v: boolean | ((prev: boolean) => boolean)) => void;
    instanceName: string;
    instanceId: string;
    expandedProjects: Record<string, boolean>;
    expandedStaleGroups: Record<string, boolean>;
    pinnedProjectIds: string[];
    archivedProjectIds: string[];
    refreshingProjectId: string | null;
    selectedProjectId: string | undefined;
    selectedConversationId: string | undefined;
    onSelectProject: (project: AgmProjectTreeNode) => void;
    onToggleProjectExpand: (projectId: string) => void;
    onToggleStaleGroup: (projectId: string) => void;
    onSelectConversation: (conv: AgmConversationNode, project: AgmProjectTreeNode) => void;
    onOpenInspector: (conv: AgmConversationNode, repoPath: string) => void;
    onRefreshProject: (projectId: string, e: React.MouseEvent) => void;
    onTogglePin: (projectId: string, e: React.MouseEvent) => void;
    onToggleArchive: (projectId: string, e: React.MouseEvent) => void;
    onResendQueued?: () => void;
    isResendingQueued?: boolean;
    isAutomatedHidden?: boolean;
    setIsAutomatedHidden?: (v: boolean | ((prev: boolean) => boolean)) => void;
    isCollapsed?: boolean;
    onToggleCollapse?: () => void;
    onRenameConversation?: (conversationId: string, newTitle: string) => Promise<void> | void;
    onResendPrompts?: (count: number | 'all') => void;
    onEnqueue?: () => void;
    onRerunConversation?: (convoId: string, nudgeType: string) => void;
    onProjectContextMenu?: (e: React.MouseEvent, repoName: string, project?: AgmProjectTreeNode) => void;
}
```

#### Key UI Elements
1. **Scope Selector & Collapse Toggle:**
   - Single-line flex bar with folder icon, transparent `<select>` scoping to specific projects or "All Projects", pin toggle button, and `PanelLeftClose` button to collapse the tree sidebar into a thin trigger icon.
2. **Category Filter Segmented Capsule (Row 1):**
   - 4-tier capsule: `All` (Blue), `User` (Sky with User icon), `Subagent` (Purple with Bot icon), `System` (Slate with Terminal icon).
3. **Status Filter Segmented Capsule & Automated Filter (Row 2):**
   - Status filters: `Running` (Emerald with pulsing dot), `Queued` (Amber dot), `Pinned` (Blue with Pin icon).
   - Automated toggle pill button: `isAutomatedHidden` toggle with `EyeOff` / `Eye` icon ("Hide Auto" / "Show Auto") to filter or reveal automated system turns.
   - Archive filter dropdown: "All (N)" vs "Archived (N)".
4. **Project Nodes & Stale Grouping:**
   - Active projects render at top; stale/inactive conversations are collapsed under a collapsible group toggle to minimize visual clutter.

---

### 2.3 `ModalHeader`
- **File Location:** `src/components/instances/PromptTreeView/ModalHeader.tsx`
- **Role:** Modal header bar displaying instance identity and two segmented dark-glass control capsules.

#### Props Interface (`ModalHeaderProps`)
```typescript
interface ModalHeaderProps {
    instanceName: string;
    sequenceNumber?: number;
    executablePath?: string;
    isFullscreen: boolean;
    setIsFullscreen: (v: boolean | ((prev: boolean) => boolean)) => void;
    isLoading: boolean;
    syncInterval: SyncInterval; // 'off' | '15s' | '30s' | '1m' | '2m'
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
}
```

#### Dual Capsule Structure
1. **Capsule 1: Data Operations Capsule** (`rounded-md bg-slate-100/90 dark:bg-[#171025]/90 border border-slate-200/80 dark:border-[#40285e]/90 divide-x shadow-2xs`):
   - `ImportExportMenu`: Dropdown menu for Export SQLite, Export JSON, Import SQLite, Import JSON, and Copy All Tree Prompts.
   - `Reset`: Clears memory cache and re-scans live transcripts without dropping running/queued prompts.
   - `Refresh`: Triggers instant poll with spinning `RefreshCw` icon.
   - `Sync`: Transparent `<select>` with options `Off`, `15s`, `30s`, `1m`, `2m`.
2. **Capsule 2: Window Controls Capsule** (`divide-x`):
   - `Show Tree / Hide Tree`: Toggle with `PanelLeftOpen` / `PanelLeftClose` icons.
   - `Fullscreen`: `Maximize2` / `Minimize2` toggle.
   - `Close`: `X` icon with rose hover effect (`hover:bg-rose-100/80 dark:hover:bg-rose-950/40 text-rose-500`).

---

### 2.4 `DetailHeader`
- **File Location:** `src/components/instances/PromptTreeView/DetailHeader.tsx`
- **Role:** Toolbar and header for the active conversation details view, providing title editing, turn metadata, clipboard actions, and dispatch triggers.

#### Props Interface (`DetailHeaderProps`)
```typescript
interface DetailHeaderProps {
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
    isSendMenuOpen?: boolean;
    setIsSendMenuOpen?: (v: boolean | ((prev: boolean) => boolean)) => void;
    isEnqueueMenuOpen?: boolean;
    setIsEnqueueMenuOpen?: (v: boolean | ((prev: boolean) => boolean)) => void;
    isExportMenuOpen?: boolean;
    setIsExportMenuOpen?: (v: boolean | ((prev: boolean) => boolean)) => void;
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
    onCopyWithImages: () => void;
    onSaveImages: () => void;
    onExport: (format: 'md' | 'json') => void;
    onFocusIde: () => void;
    onResend: () => void;
    onEnqueue: () => void;
    rerunRepoPath?: string;
    onOpenInspector: (conv: AgmConversationNode) => void;
    selectedTurnNumber?: number | null;
    setIsDetailsModalOpen?: (v: boolean) => void;
}
```

#### Key Capabilities
- **Double-Click Inline Title Rename**: Users can double-click the conversation title to rename it directly; changes persist to SQLite `repo_prompts.db`.
- **4-Tier Origin Badge & Turn Indicator**: Visual badge with appropriate tier icon and color, turn number (e.g., `#3 Turn`), and relative duration timestamp.
- **Unified Copy Dropdown Capsule**:
  - `Copy Clean Text`: Plain text without markdown image attachments.
  - `Copy as Markdown`: Complete markdown including relative links and data URIs.
  - `Copy with Rich Images`: HTML Format with embedded Base64 `<img>` tags for OneNote/Word.
  - `Copy Prompt & AI`: Structured export of user prompt paired with assistant response.
- **Dispatch Segmented Capsule**:
  - Primary button: `Send` or `Enqueue` with active loading spinner.
  - Target selector: `Current Conversation` vs `New Conversation (+)` with inline title input.
  - Suffix template modal launcher: Quick injection of verification prompts (e.g., "Verify build", "Run tests").

---

### 2.5 `PromptImageContextMenu`
- **File Location:** `src/components/instances/PromptTreeView/PromptImageContextMenu.tsx`
- **Role:** Floating contextual menu triggered on right-clicking any rendered prompt image attachment.

#### Props Interface (`PromptImageContextMenuProps`)
```typescript
interface PromptImageContextMenuProps {
    isOpen: boolean;
    position: { x: number; y: number };
    imageSrc: string;
    altText?: string;
    allImages?: string[];
    onClose: () => void;
    onOpenPreview: (src: string, alt?: string) => void;
}
```

#### Event Handlers & Viewport Clamping
- **Viewport Boundary Clamping**: Prevents menu clipping on screen edges (`x = Math.min(position.x, window.innerWidth - 242)`, `y = Math.min(position.y, window.innerHeight - 222)`).
- **Dismissal Listeners**: Closes on outside mouse clicks (`mousedown`), `Escape` keypress, or viewport scroll (`scroll` capture phase).
- **Menu Items**:
  1. `Copy Single Image (PNG)`: Calls `writeNativeImageToClipboard(imageSrc)` to copy binary PNG blob.
  2. `Copy All Attached Images`: Calls `copyAllImagesAsRichHtml(allImages)` to format multi-image HTML for OneNote.
  3. `Copy Image as Text (Markdown)`: Calls `copyImageAsMarkdownText(imageSrc)` to copy `![alt](src)` markdown.
  4. `View Fullscreen Lightbox`: Opens `PromptImagePreviewModal`.
  5. `Copy Image URL / Data`: Copies raw URI / Base64 string to clipboard.
  6. `Download Image File`: Triggers programmatic browser download with timestamped filename.

---

### 2.6 `PromptImagePreviewModal`
- **File Location:** `src/components/instances/PromptTreeView/PromptImagePreviewModal.tsx`
- **Role:** Fullscreen dark-glass lightbox providing pan, zoom, clipboard copy, and download controls for prompt images.

#### Props Interface (`PromptImagePreviewModalProps`)
```typescript
interface PromptImagePreviewModalProps {
    isOpen: boolean;
    imageSrc: string | null;
    altText?: string;
    onClose: () => void;
}
```

#### State & Interactions
- `zoomScale`: Zoom level bounded between `0.25x` (25%) and `5.0x` (500%).
- `panPosition`: `{ x: number, y: number }` tracking pan displacement during mouse dragging.
- `isPanning`: Boolean indicating active mouse drag.
- **Keyboard Shortcuts**:
  - `Escape`: Closes lightbox.
  - `+` or `=`: Zoom in by +25%.
  - `-`: Zoom out by -25%.
  - `0`: Reset zoom and pan to 100% / center.
- **Mouse Wheel**: Smooth zoom in/out with 15% delta steps.
- **Toolbar Capsule**:
  - Zoom controls: `ZoomOut`, percentage badge (font-mono), `ZoomIn`, `RotateCcw` (reset).
  - Copy PNG button: Calls `writeNativeImageToClipboard` with emerald check confirmation.
  - Copy as Text button: Calls `copyImageAsMarkdownText` for clean markdown insertion.
  - Download button & Close button (`X`).

---

### 2.7 `FleetMachineRow`
- **File Location:** `src/components/instances/fleet/FleetMachineRow.tsx`
- **Role:** Table row displaying remote node health, IP address, active profiles, masked bound emails, and running prompt status.

#### Props Interface (`FleetMachineRowProps`)
```typescript
interface FleetMachineRowProps {
    machine: FleetMachineInfo;
    isRowUnmasked: boolean;
    copiedIp: string | null;
    onCopyIp: (ip: string) => void;
    onToggleMask: (nodeId: string) => void;
}
```

#### Data Invariants & Credential Redaction
1. **Administrative Username Sanitization**:
   - Explicitly filters out `administrator@`, `root@`, `${adminuser}`, `${linuxuser}`, and `administrator` from `machine.bound_emails` and `machine.active_instances`.
   - Cleans OS info string: `machine.os_info?.replace(/administrator|root/gi, '').trim()`.
2. **Telemetry Columns**:
   - **Machine & Status**: Online status (emerald pinging dot vs slate offline dot), Node Alias, `ONLINE`/`OFFLINE` pill badge, `Mesh` badge for SSH mesh nodes, sanitized OS string, truncated `#NodeId`.
   - **IP Address**: Dark-glass capsule with font-mono IP address and 1-click `Copy` button.
   - **Running Instance(s)**: Count badge (`N insts`) and pill capsules for each active profile with emerald pulsing running indicator.
   - **Bound Account Email(s)**: Displayed as masked emails (e.g. `u***@example.com`) by default; includes toggle `Eye`/`EyeOff` button to unmask for authorized administrators.
   - **Running Prompts**: Emerald glowing badge (`N Running` with `Zap` icon) when in-flight prompts > 0; slate `Idle` pill otherwise.
   - **Last Heartbeat**: Relative timestamp (e.g. `12s ago`, `2m ago`) with hover tooltip showing full locale date/time.

---

### 2.8 `TelegramGuideModal`
- **File Location:** `src/components/telegram/TelegramGuideModal.tsx`
- **Role:** Dedicated setup wizard and terminal CLI reference modal launched from the navbar settings dropdown.

#### Props Interface (`TelegramGuideModalProps`)
```typescript
interface TelegramGuideModalProps {
    isOpen: boolean;
    onClose: () => void;
    botUsername?: string | null;
    isBotRunning?: boolean;
}
```

#### Content Sections
1. **Status Header**:
   - Indicates connection health: emerald "Bot Connected" with pulsing dot or slate "Setup Required".
2. **3-Step Setup Guide Cards**:
   - **Step 1: Create Bot via @BotFather**: Step counter badge, instructions to run `/newbot`, and external link button to `https://t.me/botfather`.
   - **Step 2: Obtain Bot Token & Send Initial Message**: Instructions for copying token and sending `/start`.
   - **Step 3: Auto-Detect Chat ID & Enable Listener**: Explains automatic chat ID polling and toggling the background listener daemon.
3. **Recommended Terminal CLI Commands**:
   - Renders 5 actionable CLI commands with 1-click copy:
     - `agm telegram status`: Inspect daemon connectivity and webhook health.
     - `agm telegram send "..."`: Send test alert to authorized chat.
     - `agm telegram listen`: Launch continuous background listener daemon.
     - `agm telegram setup`: Interactive step-by-step terminal wizard.
     - `agm prompt --instance default "..."`: Remote prompt dispatch via CLI.
4. **Footer**:
   - `ShieldCheck` badge: "All credentials stored encrypted in split SQLite vault".
   - "Got It" dismissal button.

---

## 3. Styling & Accessibility Matrix

| Component | Container Style | Capsule Style | Focus & A11y Standards |
|---|---|---|---|
| `PromptTreeViewModal` | `fixed inset-0 z-50 bg-black/60 backdrop-blur-xs` | Rounded-lg dark-glass headers | `role="dialog"`, `aria-modal="true"`, Escape key dismissal |
| `TreePanel` | `w-80 sm:w-96 border-r border-slate-200 dark:border-[#40285e]` | Segmented 4-tier filter capsules | `role="region"`, `aria-label="Prompt Tree Navigation"` |
| `ModalHeader` | `border-b px-4 py-2.5 bg-slate-50/80 dark:bg-[#090611]` | Segmented data & control capsules | Contrast ratio ≥ 4.5:1, transparent select borderless focus |
| `DetailHeader` | `border-b px-4 py-2.5 bg-white dark:bg-[#171025]` | Unified copy & dispatch capsules | Monospace sequence badges, double-click inline input autofocus |
| `PromptImageContextMenu` | `fixed z-[100050] bg-slate-900/95 backdrop-blur-md` | Rounded-xl dark-glass menu | Viewport boundary clamping, outside click listener |
| `PromptImagePreviewModal` | `fixed inset-0 z-[100050] bg-black/85 backdrop-blur-md` | Floating segmented zoom capsule | Keyboard zoom (`+`, `-`, `0`), escape to close, wheel zoom |
| `FleetMachineRow` | `tr hover:bg-slate-800/30 transition-colors` | IP & Profile pills `rounded-full` | Masked email toggle with screen-reader accessible titles |
| `TelegramGuideModal` | `w-full max-w-2xl rounded-2xl bg-white dark:bg-[#171025]` | Step cards & CLI copy rows | 1-click copy feedback toast, encrypted vault indicator |
