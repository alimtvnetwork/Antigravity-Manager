# 02-ui-components-spec: UI Modernization, Action Panel Compaction & Performance Engine Specification

- **Spec ID:** `02-spec/21-app/antigravity-manager-and-gitmap-command-enhancements/02-ui-components-spec.md`
- **Application:** Antigravity-Manager (`agm`, React Frontend, Tauri IPC, State Stores)
- **Status:** `APPROVED`
- **Lead Architect:** Antigravity Spec Author 02 (`@aukgit`)

---

## 1. Executive Summary & Architecture Overview

This specification establishes the technical requirements, component designs, state management models, and performance optimizations for the Antigravity-Manager user interface. The primary objective is to modernize navigation, eliminate horizontal toolbar clutter, remove false-empty flashes, accelerate prompt tree rendering through in-memory Stale-While-Revalidate (SWR) caching, and provide seamless access to unified backup/vault services and command-line interfaces.

### 1.1 Core Problems Addressed

1. **Header Navigation Sprawl (`src/components/navbar/NavMenu.tsx`):**
   - Text labels on "Accounts" and "Instances" buttons consume excessive horizontal space, crowding window title controls and smaller displays.
   - Navigation dropdown lacks search filtering, making finding specific views (`Email & Alerts`, `API Proxy`, `Task History`) tedious as the view registry expands.
   - "Check for Updates" button lacks prominent visual separation and status clarity.

2. **Prompt Tree Action Panel Overflow (`src/components/instances/PromptTreeView/DetailHeader.tsx`):**
   - Action Capsule 1 (`Copy Text`, `+ Imgs`, `Save Imgs`, `Export`) spans up to 340px, causing severe wrapping on split-screen or 1080p window layouts.
   - Action Capsule 2 (`Suffix`, `Target Mode`, `Focus IDE`, `Send N`, `Queue`, `Full`) contains redundant text labels for secondary and tertiary actions.

3. **Prompt Tree Loading Latency & False-Empty Flash (`usePromptTreeData.ts`, `TreePanel.tsx`):**
   - The `loadTree` function hardcodes `isForce: true` in its parameters and call sites, completely bypassing backend SQLite cache (`tree_cache`) on every open and auto-sync poll.
   - Opening the Prompt Tree View modal incurs a 300ms–2000ms disk traversal delay.
   - During initial load (`treeData.length === 0`), `TreePanel.tsx` flashes a jarring "No Projects Found in this Profile" empty state card before records arrive.

4. **Instance Section Backup Integration & CLI Parity (`src/pages/instances/InstancesHeader.tsx`, `InstancesStates.tsx`):**
   - Instance management views lack a direct trigger for unified profile backup, export, and encrypted vault restoration.
   - Operators navigating the GUI lack immediate contextual CLI command references (`agm list`, `agm sync`, `agm switch`, `agm clean-restart`).

---

## 2. Header Navigation Modernization

### 2.1 File Location
- Component: `src/components/navbar/NavMenu.tsx`
- Types: `src/components/navbar/constants.ts`

### 2.2 Segmented Pill Navigation: Icon-Only Transformation
The top navigation segmented pill group is streamlined to display icon-only buttons for primary views while retaining `Settings` and the `More Views` dropdown trigger.

#### Visual Hierarchy
```
┌─────────────────────────────────────────────────────────────┐
│ [ (Users) ] [ (Laptop) ] [ (Settings) Settings ] [ (Menu) ▾ ]│
└─────────────────────────────────────────────────────────────┘
      │            │
      ▼            ▼
  Accounts     Instances
  (Vault)      (Runner)
```

#### Detailed Requirements:
1. **Accounts Button:**
   - Display `Users` icon (`w-3.5 h-3.5`).
   - Remove text label span.
   - Apply padding: `px-2.5 py-1.5` (or `h-7 w-8` centered).
   - Retain segmented rounding via `segRounding(accountsItem)`.
   - Tooltip: Dark-glass floating tooltip displaying `Accounts (Profile Vault)`.
   - Active state: `isAccountsActive ? "bg-blue-600 text-white shadow-xs" : "text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d]"`.

2. **Instances Button:**
   - Display `Laptop` icon (`w-3.5 h-3.5`).
   - Remove text label span.
   - Apply padding: `px-2.5 py-1.5`.
   - Retain segmented rounding via `segRounding(instancesItem)`.
   - Tooltip: Dark-glass floating tooltip displaying `Instances (Engine Runner)`.
   - Active state: `isInstancesActive ? "bg-blue-600 text-white shadow-xs" : "text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d]"`.

3. **Accessible Dark-Glass Tooltips:**
   - Utilize accessible `aria-label` and `title` attributes on each link.
   - Implement subtle group-hover tooltip styling:
     ```tsx
     <div className="relative group">
       <Link ... />
       <div className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-1 bg-slate-900/95 dark:bg-slate-950/95 backdrop-blur-md text-white text-[10px] font-medium rounded-md shadow-lg border border-slate-700/60 pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-150 z-[9999] whitespace-nowrap">
         {tooltipText}
       </div>
     </div>
     ```

---

## 3. NavMenu Dropdown Search Filter & Prominent Updates

### 3.1 Real-Time Dropdown View Search
As features grow, finding specific pages requires rapid keystroke filtering. A compact search input is placed immediately above the navigation links inside the dropdown menu.

#### Layout Specification
```
┌──────────────────────────────────────────────┐
│ [ ↓ Check for Updates                      ] │ <- Prominent Top Action
├──────────────────────────────────────────────┤
│ [ 🔍 Filter views...               [ESC]   ] │ <- Real-time Search Input
├──────────────────────────────────────────────┤
│ MORE VIEWS (6)                               │ <- Category Header / Count
├──────────────────────────────────────────────┤
│   ✉  Email & Alerts                        ✓ │
│   ⚡  API Proxy Gateway                      │
│   📜  Task History & Audit Trail             │
│   🔒  Security & Network Guard               │
└──────────────────────────────────────────────┘
```

#### Implementation Logic:
1. **State Management:**
   - State hook: `const [searchQuery, setSearchQuery] = useState('');`
   - Positive boolean indicator: `const isSearching = searchQuery.trim().length > 0;`
   - Input auto-focus when dropdown opens: `menuRef` input autofocus with clean blur on Escape.

2. **Filtering Algorithm:**
   ```typescript
   const filteredDropdownItems = dropdownNavItems.filter((item) => {
       if (!isSearching) return true;
       const query = searchQuery.trim().toLowerCase();
       const matchesLabel = item.label.toLowerCase().includes(query);
       const matchesPath = item.path.toLowerCase().includes(query);
       return matchesLabel || matchesPath;
   });
   ```

3. **Search Input UI Characteristics:**
   - Compact height: `h-7`.
   - Dark-glass background: `bg-gray-100/80 dark:bg-slate-800/80 border border-gray-200/80 dark:border-slate-700/80 rounded-lg`.
   - Embedded `Search` icon (`w-3.5 h-3.5 text-gray-400`).
   - Clear button (`X` icon) rendered conditionally when `isSearching === true`.
   - Empty state when `filteredDropdownItems.length === 0`:
     ```tsx
     <div className="py-4 text-center text-xs text-gray-400">
       No matching views found
     </div>
     ```

4. **Prominent "Check for Updates" Section:**
   - Kept prominently at the very top of the menu, isolated with a bottom divider.
   - Styled with subtle brand accent: `hover:bg-blue-50/70 dark:hover:bg-blue-900/20 text-blue-600 dark:text-blue-400 font-semibold`.
   - Incorporates `Download` icon (`w-4 h-4`) and direct trigger to `useUpdateStore.getState().checkForUpdates(true)`.

---

## 4. Prompt Tree View Action Panel Compaction

### 4.1 File Location
- Component: `src/components/instances/PromptTreeView/DetailHeader.tsx`
- Parent Modal: `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`

### 4.2 Problem & Design Objective
In `DetailHeader.tsx`, horizontal toolbar overflow forces actions onto multiple wrapped rows, degrading vertical space available for prompt inspection. The toolbar is compacted into two tight, contiguous segmented dark-glass capsules without removing any functionality.

### 4.3 Capsule 1: Consolidated Split/Dropdown Copy & Export Button

#### Before vs After Architecture
- **Before:** `[Copy Text] | [+ Imgs] | [Save Imgs] | [Export ▾]` (Consumes ~340px)
- **After:** `[ 📋 Copy ] [ ▾ ]` Split Capsule (Consumes ~110px)

```
┌──────────────────────────────────────────────────────────────┐
│ [ 📋 Copy ] [ ▾ ]                                             │
└──────────────────────────────────────────────────────────────┘
               │
               ▼ Dropdown Menu
        ┌──────────────────────────────────────────────────────┐
        │  📋  Copy Plain Text                                │
        │  🖼️  Copy With Rich Images                           │
        │  💾  Save Images to Disk        (Active if hasImages)│
        ├──────────────────────────────────────────────────────┤
        │  📄  Export as Markdown (.md)                        │
        │  📦  Export as JSON (.json)                          │
        └──────────────────────────────────────────────────────┘
```

#### Detailed Behavior:
1. **Primary Button (Left Segment):**
   - Click Handler: Copies plain prompt text (`stripImagesFromPrompt(activePromptText)`).
   - Visual Feedback: Shows `<Check className="w-3 h-3 text-emerald-500" />` and "Copied!" for 2000ms via `isCopiedText`.
   - Styling: `px-2.5 py-0.5 rounded-l-full text-[11px] font-semibold`.

2. **Dropdown Trigger Button (Right Segment):**
   - Click Handler: Toggles `isExportMenuOpen`.
   - Icon: `<ChevronDown className="w-3 h-3 text-slate-400" />`.
   - Styling: `px-1.5 py-0.5 rounded-r-full hover:bg-slate-200/80 dark:hover:bg-[#15334d]`.

3. **Dropdown Flyout Panel:**
   - Position: `absolute right-0 top-full mt-1.5 z-50 w-52 rounded-xl bg-white dark:bg-[#0c2438] border border-slate-200 dark:border-[#15334d] shadow-xl py-1 backdrop-blur-md`.
   - Actions:
     - `Copy Plain Text`: Triggers text copy.
     - `Copy With Rich Images`: Triggers `onCopyWithImages`.
     - `Save Embedded Images`: Triggers `onSaveImages` (conditionally disabled with opacity when `!hasImages`).
     - Divider: `border-t border-slate-100 dark:border-slate-800 my-1`.
     - `Export as Markdown (.md)`: Triggers `onExport('md')`.
     - `Export as JSON (.json)`: Triggers `onExport('json')`.

### 4.4 Capsule 2: Compact Workflow & Execution Toolbar

#### Before vs After Architecture
- **Before:** `[ Suffix ] | [ Target ] | [ Focus IDE ] | [ Send N ] | [ Queue ] | [ Full ]`
- **After:** `[ Suffix ] | [ Target ] | [ ↗ ] | [ ⟳ Send N ] | [ ➕ ] | [ ⛶ ]`

#### Component Layout & Controls:
1. **Confirmation Suffix Selector:**
   - Retain transparent borderless styling: `bg-transparent border-0 text-slate-700 dark:text-slate-200 text-[10.5px] font-semibold focus:ring-0 focus:outline-none cursor-pointer max-w-[85px] truncate pr-1 py-0`.

2. **Conversation Target Mode Selector:**
   - Retain transparent borderless styling: `bg-transparent border-0 text-slate-700 dark:text-slate-200 text-[10.5px] font-semibold focus:ring-0 focus:outline-none cursor-pointer max-w-[90px] truncate pr-1 py-0`.
   - Options: `Current` vs `New Conv`.

3. **Focus IDE Action (Icon-Only):**
   - Icon: `<ExternalLink className="w-3 h-3 text-sky-500" />`.
   - Button text: Removed.
   - Tooltip: `Focus Antigravity IDE Window or Launch Instance`.
   - Padding: `px-2 py-0.5`.

4. **Send Now Action (Primary Dispatch):**
   - Layout: `<RotateCw className="w-3 h-3 text-emerald-500" />` + `Send` + `<kbd className="ml-0.5 px-1 py-0.2 bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 rounded text-[8.5px] font-mono font-bold">N</kbd>`.
   - Hotkey: `N` triggers instant dispatch.
   - Visual spinning state when `isResending === true`.

5. **Enqueue Action (Icon-Only):**
   - Icon: `<ListPlus className="w-3.5 h-3.5 text-purple-500" />`.
   - Button text: Removed.
   - Tooltip: `Enqueue Prompt into FIFO Scheduler Queue`.
   - Padding: `px-2 py-0.5`.

6. **Full Inspector Action (Icon-Only):**
   - Icon: `<Maximize2 className="w-3 h-3 text-blue-500 dark:text-cyan-400" />`.
   - Button text: Removed.
   - Tooltip: `Open Full-Screen Conversation Inspector`.
   - Padding: `px-2 py-0.5 rounded-r-full`.

---

## 5. Prompt History Loading Performance & SWR Caching

### 5.1 Root Cause Analysis: The `isForce: true` SQLite Cache Bypass

#### Identified Bug:
In `src/components/instances/PromptTreeView/usePromptTreeData.ts`:
```typescript
// Buggy implementation:
const loadTree = useCallback(
    async (
        isInitialLoad: boolean = true,
        isForce: boolean = true, // <- Bug: defaults to true!
        overrideArchivedIds?: string[],
        overridePinnedIds?: string[]
    ) => {
        ...
        const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
            instanceId: instanceId || undefined,
            maxWords: 2000,
            onlyRunning: false,
            force: isForce, // <- Always true, bypassing backend SQLite cache!
        });
```
Furthermore, the modal open effect in `usePromptTreeData.ts` called:
```typescript
loadTree(true, true, latestArchived, latestPinned); // Hardcoded force=true
```
And the auto-sync interval timer called:
```typescript
loadTree(false, true); // Hardcoded force=true
```

#### Remediation:
1. Update `loadTree` signature: default `isForce` to `false`.
2. When the modal opens: invoke `loadTree(true, false)`. Backend SQLite index is queried directly, resolving in `<15ms`.
3. Auto-sync interval: invoke `loadTree(false, false)`.
4. Manual refresh: Only explicit user clicks (modal header refresh button, single project refresh button) pass `isForce: true`.

---

### 5.2 Module-Level In-Memory SWR (Stale-While-Revalidate) Cache Architecture

#### Cache Lifecycle Specification:
To achieve an instant **0ms perceived render** when opening the Prompt Tree View modal:

```
[ Operator clicks "Prompts" ]
             │
             ▼
   Does memory cache exist?
        /         \
    [YES]         [NO]
      │             │
      ▼             ▼
Render cached    Display Shimmering
tree in 0ms!     Skeleton Loader
      │             │
      └──────┬──────┘
             │
             ▼
   Background fetch (`force: false`)
             │
             ▼
   Update UI & SWR cache smoothly
   (Zero visual layout shift)
```

#### Cache Store Design in `usePromptTreeData.ts`:
```typescript
interface SwrCacheEntry {
    data: AgmProjectTreeNode[];
    timestamp: number;
}

// Module-level dictionary persisting across modal unmounts
const promptTreeSwrCache = new Map<string, SwrCacheEntry>();
```

#### SWR Execution Flow:
1. **Cache Key:** `const cacheKey = instanceId || 'default';`
2. **On Modal Mount (`isOpen === true`):**
   - Check `promptTreeSwrCache.get(cacheKey)`.
   - If cached entry exists:
     - Call `setTreeData(cached.data)`.
     - Execute `performAutoSelection(cached.data, latestArchived, latestPinned)`.
     - Mark `hasCachedData = true`.
     - Do NOT trigger blocking skeleton (`setIsLoading(false)`). Instead, trigger subtle `setIsAutoSyncing(true)`.
   - If no cached entry exists:
     - Call `setIsLoading(true)`.
3. **Background Revalidation:**
   - Dispatch `invoke('get_project_conversation_tree', { instanceId, maxWords: 2000, onlyRunning: false, force: false })`.
   - On response:
     - Update in-memory cache: `promptTreeSwrCache.set(cacheKey, { data, timestamp: Date.now() })`.
     - Update React state: `setTreeData(data)`.
     - Re-run selection preservation logic to avoid resetting user's actively selected conversation.
     - Clear loading and syncing flags.

---

### 5.3 Elimination of False-Empty Flash in `TreePanel.tsx`

#### Problem:
When `treeData` is initially `[]` before backend response arrives, `TreePanel.tsx` evaluates:
```tsx
{activeProjects.length === 0 && archivedProjects.length === 0 ? (
    treeData.length === 0 ? (
        <div className="py-16 text-center space-y-3 px-6">
            <Folder className="w-6 h-6 text-slate-400" />
            <div className="text-sm font-semibold">No Projects Found in this Profile</div>
            ...
        </div>
    ) : ...
)}
```
This renders a false-negative "No Projects Found" flash for 100ms–500ms even when dozens of projects exist.

#### Remediation:
1. **Extend `TreePanelProps`:**
   ```typescript
   export interface TreePanelProps {
       // ... existing props
       isLoading: boolean;
   }
   ```
2. **Wire `isLoading` in `PromptTreeViewModal.tsx`:**
   ```tsx
   <TreePanel
       isLoading={isLoading}
       treeData={treeData}
       ...
   />
   ```
3. **Render Skeleton Shimmer when Loading:**
   In `TreePanel.tsx`, when `isLoading && treeData.length === 0`, render three animated skeleton project cards instead of the empty state:
   ```tsx
   {isLoading && treeData.length === 0 ? (
       <div className="space-y-2 p-1 animate-pulse">
           {[1, 2, 3].map((n) => (
               <div key={n} className="rounded-lg border border-slate-200/60 dark:border-[#15334d]/60 p-2.5 space-y-2 bg-slate-50/50 dark:bg-[#0c2438]/40">
                   <div className="flex items-center gap-2">
                       <div className="w-4 h-4 rounded bg-slate-200 dark:bg-slate-800" />
                       <div className="h-3.5 w-32 rounded bg-slate-200 dark:bg-slate-800" />
                       <div className="ml-auto h-3 w-10 rounded bg-slate-200 dark:bg-slate-800" />
                   </div>
                   <div className="pl-6 space-y-1.5">
                       <div className="h-3 w-48 rounded bg-slate-200/70 dark:bg-slate-800/70" />
                       <div className="h-3 w-36 rounded bg-slate-200/70 dark:bg-slate-800/70" />
                   </div>
               </div>
           ))}
       </div>
   ) : activeProjects.length === 0 && archivedProjects.length === 0 ? (
       // Render real empty state only when NOT loading
       ...
   )}
   ```

---

## 6. Instance Section Backup/Restore & CLI Documentation

### 6.1 Unified Backup Modal Action in `InstancesHeader.tsx`

#### UI Placement:
In `src/pages/instances/InstancesHeader.tsx`, add an "Import / Export" action pill inside Segmented Group 1 (or adjacent to Settings & Sync):

```
┌────────────────────────────────────────────────────────────────────────┐
│ [ Running 3 of 4 ] [ ⟳ Refresh ] [ 🔄 Sync All ] [ 💾 Import / Export ] │
└────────────────────────────────────────────────────────────────────────┘
```

#### Action Button Specification:
- Label: `Import / Export`
- Icon: `<FolderArchive className="w-3.5 h-3.5 text-purple-500" />` (or `<ShieldCheck className="w-3.5 h-3.5 text-purple-500" />`)
- Tooltip: `Unified Backup: Export accounts, configurations, or restore encrypted archives`
- Click Handler: Opens `UnifiedBackupModal` via `api.setIsBackupModalOpen(true)`.

### 6.2 Page API & Modal Wiring
1. **Extend `InstancePageApi` (`src/pages/instances/instancePageTypes.ts`):**
   ```typescript
   export interface InstancePageApi {
       // ... existing properties
       isBackupModalOpen: boolean;
       setIsBackupModalOpen: (v: boolean) => void;
       backupInitialTab: 'export' | 'import';
       setBackupInitialTab: (tab: 'export' | 'import') => void;
   }
   ```
2. **State in `useInstancePage.ts` (`src/pages/instances/useInstancePage.ts`):**
   - Declare state: `const [isBackupModalOpen, setIsBackupModalOpen] = useState(false);`
   - Declare tab: `const [backupInitialTab, setBackupInitialTab] = useState<'export' | 'import'>('export');`
   - Expose in returned `InstancePageApi` object.
3. **Mount in `InstanceAttachedModals.tsx` (`src/pages/instances/InstanceModals.tsx`):**
   ```tsx
   <UnifiedBackupModal
       isOpen={api.isBackupModalOpen}
       onClose={() => api.setIsBackupModalOpen(false)}
       initialTab={api.backupInitialTab}
   />
   ```

---

### 6.3 Contextual CLI Help Cards & Operator Shortcuts

#### Placement:
Rendered in `src/pages/instances/InstancesStates.tsx` (as `InstancesCliHelpCard`) and mounted at the bottom of `src/pages/Instances.tsx`.

#### Content & Command Reference Matrix:

| Category | Command | Description | Hotkey / Notes |
| :--- | :--- | :--- | :--- |
| **Lifecycle** | `agm list` | View active instances, PIDs, and running tasks | Real-time status |
| **Sync** | `agm sync` | Synchronize process PIDs and account quotas | Automated multi-process audit |
| **Switch** | `agm switch <id>` | Rotate profile, transfer credentials, and resume tasks | Atomic handoff |
| **Vault** | `agm backup` | Export encrypted profile backup archive | AES-256-GCM |
| **Vault** | `agm restore <file>` | Restore accounts and configurations from archive | Safe merge |
| **Hygiene** | `agm clean-restart` | Terminate orphaned processes and clean lockfiles | Resolves port & lock conflicts |
| **Prompt** | `agm prompts list` | Inspect historical and active prompts across profiles | FIFO ordered |

#### Visual Design:
- Collapsible dark-glass container: `bg-white/60 dark:bg-[#0c2438]/60 backdrop-blur-md rounded-xl border border-gray-200/80 dark:border-[#15334d] p-4 shadow-sm`.
- Monospace command tokens with copy-to-clipboard interactions.
- Contextual tip regarding Antigravity process sandboxing and isolated Electron storage.

---

## 7. Architectural Invariants & Rules

1. **Strictly Relative Git Paths:**
   - All references, imports, and documentation must use strictly relative Git paths (e.g. `src/components/navbar/NavMenu.tsx`).
   - Absolute local paths (`d:\...`) and `file:///` URIs are forbidden.

2. **Positive Boolean Naming:**
   - Strictly positive boolean identifiers: `isAccountsActive`, `isInstancesActive`, `isSearching`, `hasCachedData`, `isLoading`, `isBackupModalOpen`, `isMenuOpen`, `isCopiedText`.
   - Never use double negatives (e.g. `isNotLoading`, `hasNoCache`).

3. **Total Ban on Git CLI Commands:**
   - The execution agent must NEVER run `git add`, `git commit`, `git status`, or `git push`. All version control operations are handled externally.

4. **Transparent Select Styling (AGENTS.md Invariant):**
   - Any `<select>` elements mounted inside segmented pill capsules must use `bg-transparent border-0 focus:ring-0 focus:outline-none` to eliminate white pill-in-pill visual defects.

5. **No Silent Errors:**
   - Fallible async operations (update checks, clipboard writes, Tauri IPC calls) must route failures through `useErrorStore.getState().trackWarning` or `captureError`.

---

## 8. Verification & Pre-flight Gate

1. **Compilation:**
   - `npm run build` must compile with 0 TypeScript diagnostics and 0 JSX syntax errors.
2. **Layout & Styling:**
   - Inspect navigation bar on widths from 768px to 1920px. Ensure Accounts and Instances icon buttons align seamlessly with Settings and More Views.
   - Inspect Prompt Tree header action panel. Ensure no line wrapping occurs when Prompt Tree modal is opened on 1080p display.
3. **Performance & SWR Verification:**
   - Initial modal open shows shimmer skeleton (no "No Projects Found" flash).
   - Subsequent modal open displays existing project tree in 0ms without skeleton flicker.
   - Auto-sync poll verifies `force: false` SQLite cache hit.
4. **Unified Backup Integration:**
   - Clicking "Import / Export" in Instances header opens `UnifiedBackupModal`.
