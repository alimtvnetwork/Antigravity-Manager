# Subtask Implementation Plan: UI Modernization, Compaction & Performance Engine

> **Subtask ID:** `02-ui-frontend-modernization`  
> **Parent Slug:** `antigravity-manager-and-gitmap-command-enhancements`  
> **Spec Reference:** `02-spec/21-app/antigravity-manager-and-gitmap-command-enhancements/02-ui-components-spec.md`  
> **Target Files:**
> - `src/components/navbar/NavMenu.tsx`
> - `src/components/instances/PromptTreeView/DetailHeader.tsx`
> - `src/components/instances/PromptTreeView/usePromptTreeData.ts`
> - `src/components/instances/PromptTreeView/TreePanel.tsx`
> - `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
> - `src/pages/instances/InstancesHeader.tsx`
> - `src/pages/instances/instancePageTypes.ts`
> - `src/pages/instances/useInstancePage.ts`
> - `src/pages/instances/InstanceModals.tsx`
> - `src/pages/instances/InstancesStates.tsx`
> - `src/pages/Instances.tsx`  
> **Status:** `[QUEUED]`  
> **Author:** `@aukgit`  

---

## 1. Context & Objectives

The Antigravity-Manager user interface serves as the mission-control dashboard for multiple isolated AI coding agent profiles, prompt scheduling pipelines, and upstream proxy gateways.

As the platform has expanded, several frontend friction points have emerged:
1. **Header Sprawl:** Navigation buttons ("Accounts", "Instances") consume horizontal room, crowding other desktop elements. The dropdown menu lacks real-time filtering for rapidly locating sub-views.
2. **Prompt Tree Action Panel Overflow:** The right-hand action capsules in `DetailHeader.tsx` frequently wrap on medium resolutions, pushing prompt text below the viewport.
3. **Prompt Tree Load Delays & False-Empty Flash:** Hardcoded `isForce: true` in `usePromptTreeData.ts` bypasses backend SQLite indexing on every load, causing unnecessary disk reads. Furthermore, opening the modal briefly flashes an empty state ("No Projects Found") before records arrive.
4. **Instance Backup & CLI Accessibility:** Operators lack a quick action pill to trigger encrypted backups and imports directly from the Instances page, as well as an accessible reference for essential `agm` CLI commands.

This subtask provides an exhaustive, step-by-step implementation plan to resolve these bottlenecks through surgical React component refactoring, SWR in-memory caching, and cohesive dark-glass UI styling.

---

## 2. Concrete Implementation Steps

### Step 1: NavMenu Navigation Modernization (`src/components/navbar/NavMenu.tsx`)

#### 1.1 Icon-Only Accounts & Instances Buttons
- **File:** `src/components/navbar/NavMenu.tsx`
- **Objective:** Convert "Accounts" and "Instances" into compact icon-only buttons while preserving `Settings` with text and the `More Views` dropdown trigger.
- **Code Changes:**
  - In `NavMenu.tsx`, modify the Accounts and Instances `<Link>` components:
    ```tsx
    {/* 1. Accounts direct button (Icon-only with tooltip) */}
    {accountsItem && (
        <div className="relative group">
            <Link
                to="/accounts"
                draggable="false"
                className={cn(
                    "flex items-center justify-center px-2.5 py-1 text-xs font-semibold transition-all duration-150 cursor-pointer select-none",
                    segRounding(accountsItem),
                    isAccountsActive
                        ? "bg-blue-600 text-white shadow-xs"
                        : "text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d]"
                )}
                aria-label="Accounts (Profile Vault)"
            >
                <Users className={cn("w-3.5 h-3.5 shrink-0", isAccountsActive ? "text-white" : "text-blue-600 dark:text-blue-400")} />
            </Link>
            <div className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 bg-slate-900/95 dark:bg-slate-950/95 backdrop-blur-md text-white text-[10px] font-medium rounded-md shadow-lg border border-slate-700/60 pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-150 z-[9999] whitespace-nowrap">
                Accounts (Profile Vault)
            </div>
        </div>
    )}

    {/* 2. Instances direct button (Icon-only with tooltip) */}
    {instancesItem && (
        <div className="relative group">
            <Link
                to="/instances"
                draggable="false"
                className={cn(
                    "flex items-center justify-center px-2.5 py-1 text-xs font-semibold transition-all duration-150 cursor-pointer select-none",
                    segRounding(instancesItem),
                    isInstancesActive
                        ? "bg-blue-600 text-white shadow-xs"
                        : "text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d]"
                )}
                aria-label="Instances (Engine Runner)"
            >
                <Laptop className={cn("w-3.5 h-3.5 shrink-0", isInstancesActive ? "text-white" : "text-teal-600 dark:text-teal-400")} />
            </Link>
            <div className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 bg-slate-900/95 dark:bg-slate-950/95 backdrop-blur-md text-white text-[10px] font-medium rounded-md shadow-lg border border-slate-700/60 pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-150 z-[9999] whitespace-nowrap">
                Instances (Engine Runner)
            </div>
        </div>
    )}
    ```

---

### Step 2: NavMenu Dropdown Search Filter & Prominent Updates (`src/components/navbar/NavMenu.tsx`)

#### 2.1 State Management & Filtering Logic
- **File:** `src/components/navbar/NavMenu.tsx`
- **Add state:**
  ```typescript
  const [dropdownSearch, setDropdownSearch] = useState('');
  const isDropdownSearching = dropdownSearch.trim().length > 0;
  ```
- **Filter items:**
  ```typescript
  const filteredDropdownNavItems = dropdownNavItems.filter((item) => {
      if (!isDropdownSearching) return true;
      const query = dropdownSearch.trim().toLowerCase();
      return item.label.toLowerCase().includes(query) || item.path.toLowerCase().includes(query);
  });
  ```

#### 2.2 Dropdown Header Search Input & Update Button
- In `isMenuOpen` container, insert the real-time search field between the update section and item list:
  ```tsx
  {/* Update Section - Prominent at the top */}
  <div className="px-1.5 pb-1.5 mb-1 border-b border-gray-100 dark:border-slate-800">
      <button
          type="button"
          onClick={handleCheckUpdates}
          className="w-full px-3 py-2 text-left text-xs flex items-center gap-2.5 rounded-lg hover:bg-blue-50/80 dark:hover:bg-blue-900/25 transition-colors cursor-pointer text-gray-700 dark:text-gray-300 group"
          title={t('nav.check_for_updates_title', 'Check for a new version now')}
      >
          <Download className="w-4 h-4 shrink-0 text-blue-600 dark:text-blue-400 group-hover:animate-bounce" />
          <span className="font-semibold text-blue-600 dark:text-blue-400">{t('nav.check_for_updates', 'Check for Updates')}</span>
      </button>
  </div>

  {/* Real-time Search Input */}
  <div className="px-2 py-1 border-b border-gray-100 dark:border-slate-800">
      <div className="flex items-center gap-1.5 px-2 py-1 bg-gray-100 dark:bg-slate-800 rounded-lg text-xs">
          <Search className="w-3.5 h-3.5 text-gray-400 shrink-0" />
          <input
              type="text"
              value={dropdownSearch}
              onChange={(e) => setDropdownSearch(e.target.value)}
              placeholder="Search views..."
              className="w-full bg-transparent border-0 focus:outline-none text-[11px] text-gray-800 dark:text-gray-200"
          />
          {isDropdownSearching && (
              <button
                  type="button"
                  onClick={() => setDropdownSearch('')}
                  className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
              >
                  <X className="w-3 h-3" />
              </button>
          )}
      </div>
  </div>
  ```

---

### Step 3: DetailHeader Action Panel Compaction (`src/components/instances/PromptTreeView/DetailHeader.tsx`)

#### 3.1 Capsule 1: Split/Dropdown Copy & Export Consolidation
- **File:** `src/components/instances/PromptTreeView/DetailHeader.tsx`
- Replace separate `Copy Text`, `+ Imgs`, `Save Imgs`, and `Export` buttons with a consolidated split button capsule:
  ```tsx
  {/* Capsule 1: Consolidated Content & Export Split Button */}
  <div className="relative flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs shrink-0">
      {/* Primary Click: Copy Clean Text */}
      <button
          type="button"
          onClick={() => {
              const cleanText = stripImagesFromPrompt(activePromptText);
              navigator.clipboard.writeText(cleanText);
              setIsCopiedText(true);
              setTimeout(() => setIsCopiedText(false), 2000);
          }}
          className="flex items-center gap-1 px-2.5 py-0.5 rounded-l-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
          title="Copy clean prompt text (excluding embedded images)"
      >
          {isCopiedText ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3 text-slate-500 dark:text-slate-400" />}
          <span>{isCopiedText ? 'Copied!' : 'Copy'}</span>
      </button>

      {/* Dropdown Chevron for Rich Copy & Export Options */}
      <button
          type="button"
          onClick={() => setIsExportMenuOpen(!isExportMenuOpen)}
          className="flex items-center px-1.5 py-0.5 rounded-r-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d] text-[11px] transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
          title="More copy and export options"
      >
          <ChevronDown className="w-3 h-3 text-slate-400" />
      </button>

      {/* Flyout Menu */}
      {isExportMenuOpen && (
          <div
              className="absolute right-0 top-full mt-1.5 z-50 w-48 rounded-xl bg-white dark:bg-[#0c2438] border border-slate-200 dark:border-[#15334d] shadow-xl py-1 animate-in fade-in zoom-in-95 duration-100"
              onMouseLeave={() => setIsExportMenuOpen(false)}
          >
              <button
                  type="button"
                  onClick={() => {
                      onCopyWithImages();
                      setIsExportMenuOpen(false);
                  }}
                  className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#15334d] text-left cursor-pointer transition-colors"
              >
                  <Copy className="w-3.5 h-3.5 text-indigo-500" />
                  <span>Copy with Rich Images</span>
              </button>

              {hasImages && (
                  <button
                      type="button"
                      onClick={() => {
                          onSaveImages();
                          setIsExportMenuOpen(false);
                      }}
                      className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#15334d] text-left cursor-pointer transition-colors"
                  >
                      <ImageIcon className="w-3.5 h-3.5 text-blue-500" />
                      <span>Save Images to Disk</span>
                  </button>
              )}

              <div className="border-t border-slate-100 dark:border-slate-800 my-1" />

              <button
                  type="button"
                  onClick={() => {
                      onExport('md');
                      setIsExportMenuOpen(false);
                  }}
                  className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#15334d] text-left cursor-pointer transition-colors"
              >
                  <FileText className="w-3.5 h-3.5 text-cyan-500" />
                  <span>Export as Markdown (.md)</span>
              </button>
              <button
                  type="button"
                  onClick={() => {
                      onExport('json');
                      setIsExportMenuOpen(false);
                  }}
                  className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#15334d] text-left cursor-pointer transition-colors"
              >
                  <Code className="w-3.5 h-3.5 text-amber-500" />
                  <span>Export as JSON (.json)</span>
              </button>
          </div>
      )}
  </div>
  ```

#### 3.2 Capsule 2: Compact Workflow Actions
- Retain transparent borderless `<select>` for `confirmationSuffix` and `conversationTargetMode`.
- Compact `Focus IDE` to icon-only `<ExternalLink className="w-3 h-3 text-sky-500" />` with tooltip.
- Keep `Send Now` with `<RotateCw /> Send <kbd>N</kbd>`.
- Compact `Queue` to icon-only `<ListPlus className="w-3.5 h-3.5 text-purple-500" />` with tooltip.
- Compact `Full` to icon-only `<Maximize2 className="w-3 h-3 text-blue-500" />` with tooltip.

---

### Step 4: Performance Engine & SWR Caching (`usePromptTreeData.ts`, `TreePanel.tsx`)

#### 4.1 Fix `isForce: true` SQLite Cache Bypass
- **File:** `src/components/instances/PromptTreeView/usePromptTreeData.ts`
- Change signature default:
  ```typescript
  const loadTree = useCallback(
      async (
          isInitialLoad: boolean = true,
          isForce: boolean = false, // Changed from true to false!
          overrideArchivedIds?: string[],
          overridePinnedIds?: string[]
      ) => {
  ```
- In modal open `useEffect`:
  ```typescript
  // Pass isForce = false to utilize backend SQLite tree_cache
  loadTree(true, false, latestArchived, latestPinned);
  ```
- In auto-sync interval `useEffect`:
  ```typescript
  // Pass isForce = false for background polls
  loadTree(false, false);
  ```

#### 4.2 Module-Level In-Memory SWR Cache in `usePromptTreeData.ts`
- Define module cache above hook:
  ```typescript
  interface PromptTreeSwrEntry {
      data: AgmProjectTreeNode[];
      timestamp: number;
  }
  const promptTreeSwrCache = new Map<string, PromptTreeSwrEntry>();
  ```
- In `useEffect` on modal open:
  - If `promptTreeSwrCache.has(instanceId || 'default')`:
    - Immediately call `setTreeData(cached.data)`.
    - Run `performAutoSelection(cached.data, latestArchived, latestPinned)`.
    - Mark `isAutoSyncing(true)` instead of `isLoading(true)`.
  - Fetch fresh data in background; on success, update both state and cache.

#### 4.3 Replace False-Empty Flash in `TreePanel.tsx`
- **File:** `src/components/instances/PromptTreeView/TreePanel.tsx`
- Add `isLoading: boolean` to `TreePanelProps`.
- In `PromptTreeViewModal.tsx`, pass `isLoading={isLoading}` to `<TreePanel />`.
- In `TreePanel.tsx`:
  - When `isLoading && treeData.length === 0`: render animated shimmering project skeletons.
  - When `!isLoading && treeData.length === 0`: render the "No Projects Found in this Profile" empty card.

---

### Step 5: Instance Section Backup/Restore & CLI Documentation (`InstancesHeader.tsx`, `InstancesStates.tsx`)

#### 5.1 Extend Instance Page State & Types
- **File:** `src/pages/instances/instancePageTypes.ts`
  - Add to `InstancePageApi`:
    ```typescript
    isBackupModalOpen: boolean;
    setIsBackupModalOpen: (v: boolean) => void;
    backupInitialTab: 'export' | 'import';
    setBackupInitialTab: (tab: 'export' | 'import') => void;
    ```
- **File:** `src/pages/instances/useInstancePage.ts`
  - Declare state hooks:
    ```typescript
    const [isBackupModalOpen, setIsBackupModalOpen] = useState(false);
    const [backupInitialTab, setBackupInitialTab] = useState<'export' | 'import'>('export');
    ```
  - Return in API object.

#### 5.2 Mount Modal in `InstanceAttachedModals.tsx`
- **File:** `src/pages/instances/InstanceModals.tsx`
- Import `UnifiedBackupModal` from `../../components/modals/UnifiedBackupModal`.
- Mount in `InstanceAttachedModals`:
  ```tsx
  <UnifiedBackupModal
      isOpen={api.isBackupModalOpen}
      onClose={() => api.setIsBackupModalOpen(false)}
      initialTab={api.backupInitialTab}
  />
  ```

#### 5.3 Add Action Pill in `InstancesHeader.tsx`
- **File:** `src/pages/instances/InstancesHeader.tsx`
- Add button in Segmented Group 1:
  ```tsx
  <button
      type="button"
      onClick={() => {
          api.setBackupInitialTab('export');
          api.setIsBackupModalOpen(true);
      }}
      className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
      title="Unified Backup: Export profiles, configurations, or restore encrypted archives"
  >
      <FolderArchive className="w-3.5 h-3.5 text-purple-500" />
      <span>Import / Export</span>
  </button>
  ```

#### 5.4 Contextual CLI Help Card in `InstancesStates.tsx` & `Instances.tsx`
- **File:** `src/pages/instances/InstancesStates.tsx`
- Implement `InstancesCliHelpCard`:
  - Outlines essential CLI commands: `agm list`, `agm sync`, `agm switch <id>`, `agm backup`, `agm restore`, `agm clean-restart`.
  - Explains background worker architecture and multi-profile isolation.
- Mount at the bottom of `src/pages/Instances.tsx`.

---

## 3. UI Invariants & Styling Guidelines

- **Dark-Glass Capsules:** Contiguous pill styling (`rounded-full`, shared border, subtle divider lines, backdrop blur) per `AGENTS.md`.
- **Transparent Selects:** All inline `<select>` elements inside pills must use `bg-transparent border-0 focus:ring-0 focus:outline-none`.
- **Positive Booleans:** Identifiers must strictly maintain positive polarity (`isAccountsActive`, `isSearching`, `hasCachedData`, `isLoading`, `isBackupModalOpen`).
- **No Silent Errors:** Any error condition must route through `useErrorStore.getState().trackWarning(e, { source, triggerAction })`.
- **Zero Git Commands:** Execution must strictly avoid `git add`, `git commit`, `git status`.

---

## 4. Verification Checkpoints

1. **Compilation Gate:**
   - Execute `npm run build` to confirm 100% type safety and clean JSX bundle generation.
2. **Navigation Bar Verification:**
   - Verify Accounts and Instances appear as icon-only buttons with hover tooltips.
   - Verify Settings remains intact with text label.
   - Verify search input in dropdown filters views dynamically.
3. **Prompt Tree Action Panel Verification:**
   - Open Prompt Tree View modal. Verify Capsule 1 renders as a compact split button.
   - Confirm clicking "Copy" copies text and shows "Copied!".
   - Confirm clicking chevron reveals rich copy and export options.
   - Confirm Capsule 2 actions fit without wrapping.
4. **Performance & SWR Cache Verification:**
   - Confirm initial open shows shimmer skeleton (no "No Projects Found" flash).
   - Close and reopen modal: verify tree loads in 0ms from in-memory SWR cache.
   - Confirm auto-sync runs with `force: false`.
5. **Unified Backup Integration:**
   - In Instances header, click "Import / Export". Confirm `UnifiedBackupModal` opens and operates correctly.
