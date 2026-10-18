# Subtask 01: Prompt Tree Session Overview Collapsible UI, Header Deduplication & Tab Toggle Decoupling

**Document ID:** `.ai-memory/plans/subtasks/ui-ux-redesign-and-bug-fix-instructions/01-ui-ux-overview-collapsible.md`  
**Parent Spec:** [01-ui-ux-and-overview-spec.md](../../../../02-spec/21-app/ui-ux-redesign-and-bug-fix-instructions/01-ui-ux-and-overview-spec.md)  
**Parent Task:** `ui-ux-redesign-and-bug-fix-instructions`  
**Status:** `READY`  
**Target Components:**
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
- `src/components/instances/PromptTreeView/DetailsModal.tsx`
- `src/components/instances/PromptTreeView/DetailTabs.tsx`

---

## 1. Context & Objectives

The Prompt Tree View inspects multi-turn prompt conversations across isolated IDE instances. However, several ergonomics and layout defects currently hamper workflow fluidity:

1. **Session Overview Card Vertical Footprint:** The dashboard card shown when viewing full sessions (`selectedTurnNumber === null`) consumes significant vertical height, forcing users to scroll to read prompt instructions.
2. **Duplicate Identity Header:** Lines 234–247 in `PromptInstructionView.tsx` repeat the conversation sequence code, title, and conversation ID, which are already prominently displayed in `DetailHeader.tsx`.
3. **Ambiguous `C211` / `C231` & `turns` Meaning:** Users lack clarity on what sequence badges (`C211`) and turn counters represent. This metadata belongs in the detailed inspector modal.
4. **Disappearing Expand Toggle on Output Tab:** `hasExpandToggle` in `DetailTabs.tsx` was restricted to `previewTab === 'instruction'`, causing the toggle to vanish when inspecting execution outputs.

### Key Deliverables:
- Make the Session Overview Card in `PromptInstructionView.tsx` **collapsible by default** with a compact single-line summary when collapsed.
- Persist collapsed/expanded state in `localStorage` under `agm_prompt_overview_expanded`.
- Style the expand/collapse trigger with a subtle, low-profile purple button using `ChevronDown`/`ChevronUp`.
- Remove duplicate title and conversation ID header (lines 234–247) from `PromptInstructionView.tsx`.
- Enrich `DetailsModal.tsx` with explicit labels explaining `C211` (Conversation #211) and `turns` (User/Assistant interaction turns).
- Decouple `hasExpandToggle` from `previewTab === 'instruction'` in `DetailTabs.tsx`.

---

## 2. Technical Task Decomposition

### Task 1: Collapsible Session Overview in `PromptInstructionView.tsx`

#### 1.1 State Definition & Persistence
Add `isOverviewExpanded` state with lazy initialization from `localStorage`:

```typescript
const [isOverviewExpanded, setIsOverviewExpanded] = useState<boolean>(() => {
    try {
        const saved = localStorage.getItem('agm_prompt_overview_expanded');
        return saved !== null ? JSON.parse(saved) === true : false;
    } catch {
        return false;
    }
});

const handleToggleOverview = () => {
    setIsOverviewExpanded((prev) => {
        const next = !prev;
        try {
            localStorage.setItem('agm_prompt_overview_expanded', JSON.stringify(next));
        } catch (err: unknown) {
            useErrorStore.getState().trackWarning(err, {
                source: 'PromptInstructionView.toggleOverview',
                triggerAction: 'save_overview_state',
            });
        }
        return next;
    });
};
```

#### 1.2 Collapsed Single-Line Summary Bar
When `isOverviewExpanded === false`, render a compact bar (~36px height) displaying:
- Left: Compact status badge (Running / Queued / Idle), turn count (`{totalTurns} turns`), time/duration summary, and enqueued badge if any.
- Right: Low-profile purple expander button:
  ```tsx
  <button
      type="button"
      onClick={handleToggleOverview}
      className="inline-flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-semibold rounded-lg text-purple-700 dark:text-purple-300 bg-purple-50 dark:bg-purple-950/40 hover:bg-purple-100 dark:hover:bg-purple-900/50 border border-purple-200/80 dark:border-purple-800/40 transition-colors cursor-pointer"
      title="Expand full session overview and metrics"
  >
      <span>Session Overview</span>
      <ChevronDown className="w-3.5 h-3.5 text-purple-500 shrink-0" />
  </button>
  ```

#### 1.3 Expanded Overview Card & Header Deduplication
When `isOverviewExpanded === true`:
- Replace lines 234–247 with a clean top bar:
  - Left: Title `Session Overview & Metrics` with BarChart/Terminal icon and optional queued indicator.
  - Right: Purple collapse button with `ChevronUp` (`Collapse Overview`).
- Body: Render the metrics grid (Total Turns, Session Status, Time Range, Duration) and Enqueued Prompts list.

---

### Task 2: Explicit Metadata Labeling in `DetailsModal.tsx`

In `src/components/instances/PromptTreeView/DetailsModal.tsx`:

#### 2.1 Extract Conversation Number & Turns
```typescript
const seqCode = selectedConversation?.seq_code || 'CONV';
const convNumberMatch = seqCode.match(/\d+/);
const convNumberDisplay = convNumberMatch ? convNumberMatch[0] : null;

const turnCount = selectedConversation?.prompts?.length || selectedConversation?.step_count || 1;
```

#### 2.2 Add Explanatory Grid in Conversation Info Section
Directly below the Conversation ID row, add:
```tsx
<div className="grid grid-cols-2 gap-2 pt-2 border-t border-slate-200/50 dark:border-[#40285e]/50">
    <div>
        <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">
            Sequence Code
        </span>
        <div className="flex items-center gap-1.5 mt-0.5">
            <span className="font-mono text-[11px] font-bold px-1.5 py-0.5 rounded-md bg-purple-100 dark:bg-purple-950/60 text-purple-700 dark:text-purple-300 border border-purple-200 dark:border-purple-800/40">
                {seqCode}
                {convNumberDisplay ? ` (Conv #${convNumberDisplay})` : ''}
            </span>
        </div>
        <span className="text-[9.5px] text-slate-400 dark:text-slate-500 block mt-0.5">
            Sequential conversation index in workspace
        </span>
    </div>
    <div>
        <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">
            Interaction Turns
        </span>
        <div className="flex items-center gap-1.5 mt-0.5">
            <span className="font-mono text-[11px] font-bold text-slate-800 dark:text-slate-200">
                {turnCount} {turnCount === 1 ? 'Turn' : 'Turns'}
            </span>
        </div>
        <span className="text-[9.5px] text-slate-400 dark:text-slate-500 block mt-0.5">
            User prompts &amp; AI response cycles
        </span>
    </div>
</div>
```

---

### Task 3: Decouple `hasExpandToggle` in `DetailTabs.tsx`

In `src/components/instances/PromptTreeView/DetailTabs.tsx`:

#### 3.1 Modify `hasExpandToggle`
```typescript
// Before:
const hasExpandToggle = previewTab === 'instruction' && Boolean(
    isTruncated ||
    activePromptText.trim().endsWith('...') ||
    activePromptText.trim().endsWith('…') ||
    showAllWords
);

// After:
const hasExpandToggle = Boolean(
    isTruncated ||
    activePromptText.trim().endsWith('...') ||
    activePromptText.trim().endsWith('…') ||
    showAllWords
);
```

#### 3.2 Verify Segmented Button Group Styling
Verify that when `hasExpandToggle === true`:
- The `Output` button receives `rounded-none`.
- The `Expand Full / Collapse` button receives `rounded-r-md`.
- Active state displays `bg-cyan-500/20 text-cyan-700 dark:text-cyan-300`.

---

## 3. Implementation Diff Details

### 3.1 `PromptInstructionView.tsx`

```diff
@@ -95,6 +95,28 @@
     const [activeTurnPreview, setActiveTurnPreview] = useState<number | null>(null);
+    const [isOverviewExpanded, setIsOverviewExpanded] = useState<boolean>(() => {
+        try {
+            const saved = localStorage.getItem('agm_prompt_overview_expanded');
+            return saved !== null ? JSON.parse(saved) === true : false;
+        } catch {
+            return false;
+        }
+    });
+
+    const handleToggleOverview = () => {
+        setIsOverviewExpanded((prev) => {
+            const next = !prev;
+            try {
+                localStorage.setItem('agm_prompt_overview_expanded', JSON.stringify(next));
+            } catch (err: unknown) {
+                useErrorStore.getState().trackWarning(err, {
+                    source: 'PromptInstructionView.toggleOverview',
+                    triggerAction: 'save_overview_state',
+                });
+            }
+            return next;
+        });
+    };
```

```diff
@@ -231,18 +253,52 @@
             {selectedTurnNumber === null && (
+                !isOverviewExpanded ? (
+                    /* Compact Collapsed Single-Line Summary Bar */
+                    <div className="flex items-center justify-between p-2.5 rounded-xl border border-slate-200/80 dark:border-[#40285e]/80 bg-gradient-to-r from-slate-50 via-white to-purple-50/20 dark:from-[#171025] dark:via-[#120a1f] dark:to-[#1a0e2e] shadow-xs text-xs font-poppins">
+                        <div className="flex items-center gap-2 min-w-0">
+                            {isRunning ? (
+                                <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md text-[9.5px] font-bold bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30">
+                                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-ping" />
+                                    <span>RUNNING</span>
+                                </span>
+                            ) : isQueued ? (
+                                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[9.5px] font-bold bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30">
+                                    <Clock className="w-2.5 h-2.5 text-amber-500" />
+                                    <span>QUEUED</span>
+                                </span>
+                            ) : (
+                                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[9.5px] font-bold bg-slate-200/80 dark:bg-[#251b3a] text-slate-700 dark:text-slate-300 border border-slate-300/60 dark:border-[#40285e]">
+                                    <Check className="w-2.5 h-2.5 text-emerald-500" />
+                                    <span>IDLE</span>
+                                </span>
+                            )}
+                            <span className="font-mono text-[11px] font-bold text-slate-700 dark:text-slate-300">
+                                {totalTurns} {totalTurns === 1 ? 'turn' : 'turns'}
+                            </span>
+                            <span className="font-mono text-[10px] text-slate-400 dark:text-slate-500 hidden sm:inline-block">
+                                · {durationDisplay} ({timestampRange})
+                            </span>
+                            {enqueuedTurns.length > 0 && (
+                                <span className="text-[9.5px] font-mono px-1.5 py-0.5 rounded-md bg-amber-100 dark:bg-amber-950/60 text-amber-800 dark:text-amber-300 border border-amber-300/40">
+                                    FIFO: {enqueuedTurns.length} pending
+                                </span>
+                            )}
+                        </div>
+                        <button
+                            type="button"
+                            onClick={handleToggleOverview}
+                            className="flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-semibold rounded-lg text-purple-700 dark:text-purple-300 bg-purple-50 dark:bg-purple-950/40 hover:bg-purple-100 dark:hover:bg-purple-900/50 border border-purple-200/80 dark:border-purple-800/40 transition-colors cursor-pointer shrink-0 ml-2"
+                            title="Expand full session overview"
+                        >
+                            <span>Session Overview</span>
+                            <ChevronDown className="w-3.5 h-3.5 text-purple-500 shrink-0" />
+                        </button>
+                    </div>
+                ) : (
+                    /* Full Expanded Overview Card */
                     <div className="rounded-xl border border-slate-200 dark:border-[#40285e] bg-gradient-to-br from-slate-50 via-white to-purple-50/20 dark:from-[#171025] dark:via-[#120a1f] dark:to-[#1a0e2e] p-4 shadow-xs space-y-3 font-poppins">
-                        {/* Header: Title and Overall Status */}
-                        <div className="flex items-center justify-between pb-2 border-b border-slate-200/60 dark:border-[#2b1b42]">
-                            <div className="flex items-center gap-2 min-w-0">
-                                <span className="text-[10px] font-mono px-1.5 py-0.5 rounded-md bg-purple-100 dark:bg-purple-950/60 text-purple-700 dark:text-purple-300 font-bold border border-purple-200 dark:border-purple-800/40 shrink-0">
-                                    {selectedConversation.seq_code || 'CONV'}
-                                </span>
-                                <div className="min-w-0">
-                                    <h3 className="text-xs font-bold text-slate-800 dark:text-slate-100 truncate">
-                                        {selectedConversation.title || selectedConversation.short_id || 'Conversation Session'}
-                                    </h3>
-                                    <p className="text-[10px] text-slate-400 dark:text-slate-500 truncate font-mono">
-                                        ID: {selectedConversation.conversation_id}
-                                    </p>
-                                </div>
-                            </div>
+                        <div className="flex items-center justify-between pb-2 border-b border-slate-200/60 dark:border-[#2b1b42]">
+                            <div className="flex items-center gap-2">
+                                <span className="text-xs font-bold text-slate-800 dark:text-slate-100">
+                                    Session Overview & Metrics
+                                </span>
+                                {enqueuedTurns.length > 0 && (
+                                    <span className="text-[9.5px] font-mono px-1.5 py-0.5 rounded-md bg-amber-100 dark:bg-amber-950/60 text-amber-800 dark:text-amber-300 border border-amber-300/40">
+                                        {enqueuedTurns.length} pending
+                                    </span>
+                                )}
+                            </div>
+                            <button
+                                type="button"
+                                onClick={handleToggleOverview}
+                                className="flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-semibold rounded-lg text-purple-700 dark:text-purple-300 bg-purple-50 dark:bg-purple-950/40 hover:bg-purple-100 dark:hover:bg-purple-900/50 border border-purple-200/80 dark:border-purple-800/40 transition-colors cursor-pointer"
+                                title="Collapse session overview"
+                            >
+                                <span>Collapse Overview</span>
+                                <ChevronUp className="w-3.5 h-3.5 text-purple-500 shrink-0" />
+                            </button>
                         </div>
```

---

## 4. Verification Checklist & Acceptance Criteria

- [ ] **Default Collapsed:** Navigating to any conversation initially renders the single-line summary bar (~36px height), giving maximum vertical space to prompt instructions.
- [ ] **State Memory:** Clicking "Session Overview" expands the card. Reloading or switching conversations maintains the user's expanded preference. Clicking "Collapse Overview" folds it and saves `false` to `localStorage`.
- [ ] **Duplicate Header Removed:** The expanded card does NOT render the conversation sequence badge, session title, or conversation ID, which are already in `DetailHeader.tsx`.
- [ ] **`DetailsModal` Labeling:** Opening DetailsModal displays `Sequence Code: C211 (Conv #211)` and `Interaction Turns: N Turns` with explicit explanatory subtext.
- [ ] **Output Tab Expand Toggle:** Switching to Output tab keeps the `Expand Full` toggle visible and active within the segmented capsule.
- [ ] **Lint & Build Pass:** Passes `npm run build` with zero TypeScript errors or linter regressions.
