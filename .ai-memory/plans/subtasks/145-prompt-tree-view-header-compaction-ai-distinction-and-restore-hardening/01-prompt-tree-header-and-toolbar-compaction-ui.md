# Subtask 01: Prompt Tree Header and Toolbar Compaction UI

**Parent Plan:** [145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening.md](../../145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening.md)  
**Parent Spec:** [01-architecture-and-ui-spec.md](../../../../02-spec/21-app/145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/01-architecture-and-ui-spec.md)  
**Status:** `COMPLETED`  
**Target Component:** `src/components/instances/PromptTreeViewModal.tsx`  
**Lead Specialist:** Antigravity UI & UX Architecture Specialist  

---

## 1. Context & Objectives

The modal header and preview action bars currently suffer from UI clutter, fragmented paddings, and violations of the governing `AGENTS.md` capsule invariant:
- All 6 header actions (Backup, Restore, Refresh, Sync Selector, Fullscreen, Close) were mashed into a single unwieldy bar.
- The Sync Interval selector rendered its own isolated solid background and border inside the capsule.
- View Mode tabs (`preview`, `raw`, `edit`) used non-canonical `rounded-[5px]` buttons.
- The right preview action toolbar had wide suffix selectors, loose margins, and redundant sequence badges.

### Core Objectives:
1. Split the top modal header into **two distinct segmented dark-glass pill capsules** per `AGENTS.md`:
   - **Data Operations Capsule:** `[ Backup | Restore | Refresh | Sync Interval ]` with transparent inline select.
   - **Window Controls Capsule:** `[ Full | Close ]` with subtle divider and rose close hover.
2. Convert View Mode tabs (`preview`, `raw`, `edit`) into a canonical `rounded-full` dark-glass segmented pill capsule.
3. Streamline the confirmation suffix dropdown into a transparent inline selector with shortened labels (`Suffix: None`, `Done?`, `Released?`, etc.).
4. Compact the right-panel action capsules (`Content & Export` and `Workflow & Dispatch`), reducing vertical footprint and padding while maintaining 100% functional parity.
5. Clean up redundant repeated sequence badges (`#P001` repeated across multiple sub-headers).

---

## 2. Detailed Technical Design & Code Modifications

### 2.1 Top Modal Header Dual Capsule Split

In `src/components/instances/PromptTreeViewModal.tsx`, replace the single monolithic header toolbar with two adjacent segmented capsules:

```tsx
{/* Top Right Header Controls: Two Independent Segmented Dark-Glass Capsules */}
<div className="flex items-center gap-2 shrink-0">
    {/* Capsule 1: Data Operations Capsule */}
    <div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#15334d]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#15334d]/80 shadow-2xs">
        {/* Backup Button */}
        <button
            type="button"
            onClick={handleBackup}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d]/90 rounded-l-full transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
            title="Backup Prompts to JSON"
        >
            <Download className="w-3.5 h-3.5 text-indigo-500" />
            <span>Backup</span>
        </button>

        {/* Restore Button */}
        <button
            type="button"
            onClick={handleRestore}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d]/90 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
            title="Restore Prompts from JSON backup file"
        >
            <Upload className="w-3.5 h-3.5 text-emerald-500" />
            <span>Restore</span>
        </button>

        {/* Refresh Button */}
        <button
            type="button"
            onClick={() => loadTree(true, true)}
            disabled={isLoading}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d]/90 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
            title="Refresh Tree"
        >
            <RefreshCw className={cn("w-3.5 h-3.5 text-blue-500", isLoading && "animate-spin")} />
            <span>Refresh</span>
        </button>

        {/* Sync Interval Selector with Transparent Inline Styling */}
        <div className="flex items-center gap-1 pl-2.5 pr-2 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 rounded-r-full">
            <Clock className={cn("w-3.5 h-3.5 text-cyan-500 shrink-0", isAutoSyncing && "animate-spin")} />
            <span className="text-[11px] text-slate-500 dark:text-slate-400 select-none">Sync:</span>
            <select
                value={syncInterval}
                onChange={(e) => {
                    const val = e.target.value as SyncInterval;
                    setSyncInterval(val);
                    setPromptTreeSyncInterval(val);
                }}
                className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-xs font-semibold focus:ring-0 focus:outline-none cursor-pointer pr-1 py-0"
                title="Auto-sync interval timer"
            >
                <option value="15s" className="bg-white dark:bg-[#0c2438]">15s</option>
                <option value="30s" className="bg-white dark:bg-[#0c2438]">30s</option>
                <option value="1m" className="bg-white dark:bg-[#0c2438]">1m</option>
                <option value="2m" className="bg-white dark:bg-[#0c2438]">2m</option>
                <option value="off" className="bg-white dark:bg-[#0c2438]">Off</option>
            </select>
        </div>
    </div>

    {/* Capsule 2: Window Controls Capsule */}
    <div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#15334d]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#15334d]/80 shadow-2xs">
        {/* Fullscreen Toggle */}
        <button
            type="button"
            onClick={() => setIsFullscreen(!isFullscreen)}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d]/90 rounded-l-full transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
            title={isFullscreen ? "Exit Full Screen" : "Full Screen Mode"}
        >
            {isFullscreen ? (
                <>
                    <Minimize2 className="w-3.5 h-3.5 text-blue-500" />
                    <span>Exit</span>
                </>
            ) : (
                <>
                    <Maximize2 className="w-3.5 h-3.5 text-blue-500" />
                    <span>Full</span>
                </>
            )}
        </button>

        {/* Close Button with Rose Dark-Glass Hover */}
        <button
            type="button"
            onClick={onClose}
            className="flex items-center px-2.5 py-1.5 text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-500/10 dark:hover:bg-rose-500/20 rounded-r-full transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
            title="Close Modal"
        >
            <X className="w-4 h-4" />
        </button>
    </div>
</div>
```

---

### 2.2 View Mode Tabs Segmented Dark-Glass Capsule

Refactor the View Mode switch container from `rounded-[5px]` to canonical `rounded-full`:

```tsx
{/* View Mode Toggle Tabs (Canonical Segmented Pill Capsule) */}
<div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#15334d]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#15334d]/80 shadow-2xs">
    {/* Preview Tab */}
    <button
        type="button"
        onClick={() => setViewMode('preview')}
        className={cn(
            'flex items-center gap-1 px-3 py-1 text-xs transition-all duration-150 rounded-l-full cursor-pointer',
            viewMode === 'preview'
                ? 'bg-blue-600 text-white font-semibold shadow-2xs'
                : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white font-medium'
        )}
        title="Rich Markdown Preview"
    >
        <Eye className="w-3.5 h-3.5" />
        <span>Preview</span>
    </button>

    {/* Raw Monospace Tab */}
    <button
        type="button"
        onClick={() => setViewMode('raw')}
        className={cn(
            'flex items-center gap-1 px-3 py-1 text-xs transition-all duration-150 rounded-none cursor-pointer',
            viewMode === 'raw'
                ? 'bg-blue-600 text-white font-semibold shadow-2xs'
                : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white font-medium'
        )}
        title="Plain Monospace Raw View"
    >
        <Code className="w-3.5 h-3.5" />
        <span>Raw</span>
    </button>

    {/* Edit Tab */}
    <button
        type="button"
        onClick={() => setViewMode('edit')}
        className={cn(
            'flex items-center gap-1 px-3 py-1 text-xs transition-all duration-150 rounded-r-full cursor-pointer',
            viewMode === 'edit'
                ? 'bg-blue-600 text-white font-semibold shadow-2xs'
                : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white font-medium'
        )}
        title="Editable Textarea Mode"
    >
        <Edit3 className="w-3.5 h-3.5" />
        <span>Edit</span>
    </button>
</div>
```

---

### 2.3 Streamlined Confirmation Suffix Dropdown

Streamline the confirmation suffix selector to blend seamlessly into Capsule 2:

```tsx
{/* Transparent Inline Suffix Selector in Capsule 2 */}
<div className="flex items-center px-2 py-0.5 rounded-l-full">
    <select
        value={confirmationSuffix}
        onChange={(e) => setConfirmationSuffix(e.target.value)}
        className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-[11px] font-semibold focus:ring-0 focus:outline-none cursor-pointer max-w-[95px] truncate pr-1 py-0"
        title="Confirmation suffix appended on Resend"
    >
        <option value="None (Send as is)">Suffix: None</option>
        <option value="Is it done?">Suffix: Done?</option>
        <option value="Is it released?">Suffix: Released?</option>
        <option value="Are you sure about it?">Suffix: Sure?</option>
        <option value="Double check all edge cases">Suffix: Edge cases</option>
        <option value="Verify build and tests">Suffix: Verify tests</option>
    </select>
</div>
```

---

## 3. Step-by-Step Implementation Checklist

1. [ ] **Modal Top Header:**
   - Locate header container in `PromptTreeViewModal.tsx` (~line 2340).
   - Divide the controls into Capsule 1 (Data Operations) and Capsule 2 (Window Controls).
   - Update sync interval selector to transparent inline styling (`bg-transparent border-0`).
2. [ ] **View Mode Switcher:**
   - Locate View Mode toggle container (~line 3052).
   - Replace `rounded-[5px]` with `rounded-full` parent and `rounded-l-full` / `rounded-none` / `rounded-r-full` children.
   - Standardize divider styling with `divide-slate-200/70 dark:divide-[#15334d]/80`.
3. [ ] **Preview Action Toolbars:**
   - Locate Capsule 1 (Content & Export) and Capsule 2 (Workflow & Dispatch) (~line 2775).
   - Apply streamlined transparent suffix selector.
   - Ensure `Send Now` hotkey `N` badge is crisp and readable.
4. [ ] **Redundant Badge Pruning:**
   - Audit right-panel preview title row to ensure sequence code `#P001` and instance trio are displayed cleanly once without repetitive clones.

---

## 4. Acceptance Criteria & Verification

- **AC-01.1:** Top header toolbar clearly renders TWO segmented capsules separated by a visual gap.
- **AC-01.2:** Sync interval selector has NO standalone border or solid white background inside its capsule.
- **AC-01.3:** View Mode tabs use `rounded-full` dark-glass capsule with clean active state highlight.
- **AC-01.4:** Fullscreen and Close buttons function properly and match the dark-glass aesthetic.
- **AC-01.5:** Pre-flight checks (`cargo fmt`, `cargo clippy`, `npm run build`) compile with zero errors.
