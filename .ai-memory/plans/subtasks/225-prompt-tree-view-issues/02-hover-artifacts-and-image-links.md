# Subtask 02: Hover Outline Elimination, 30-Second Caching Debounce, Clickable IDE Image Links & Media Positioning Correctness

- **Subtask ID:** `225-02`
- **Slug:** `hover-artifacts-and-image-links`
- **Component:** Prompt Tree View Node Styling, Caching Engine & Media Presentation Subsystem
- **Status:** `READY_FOR_EXECUTION`
- **Author:** Spec Author 02 (Hover Artifacts, Caching Guard & Image Formatter Specialist)
- **Parent Spec:** [02-spec/21-app/225-prompt-tree-view-issues/02-component-spec.md](../../../../02-spec/21-app/225-prompt-tree-view-issues/02-component-spec.md)
- **Architecture Spec:** [02-spec/21-app/225-prompt-tree-view-issues/01-architecture-spec.md](../../../../02-spec/21-app/225-prompt-tree-view-issues/01-architecture-spec.md)
- **Master Ledger:** [02-spec/21-app/225-prompt-tree-view-issues/00-master-audit-ledger.md](../../../../02-spec/21-app/225-prompt-tree-view-issues/00-master-audit-ledger.md)
- **Primary Files Touched:**
  - `src/components/instances/PromptTreeView/ConversationNode.tsx`
  - `src/components/instances/PromptTreeView/usePromptTreeData.ts`
  - `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
  - `src/components/instances/PromptTreeView/markdown.tsx`
  - `src/components/instances/PromptTreeView/clipboard.ts`
  - `src/components/instances/PromptTreeView/__tests__/imageStandalonePipeline.test.ts`

---

## 1. Problem Statement & Root Cause

The user reported:
> "And also the images. I don't see the images are correctly there. It feels a bit buggy, and the way that you pass the image, that's not really very helpful. The reason why I'm saying is because when you pass the image, you can use the file format for the IDE because you are giving absolute path, and then we can click on the image and view the image. Currently, the path, we cannot view it by clicking on it. That's one issue. So make sure that you pose it correctly so that IDE also display as a file, and we click and view it. That's one thing. The next thing is that please check the correctness. I do believe that your image position or capturing position is not correct. And also same as your conversation name and conversation prompts. These are not correct. You need to do the end-to-end testing here. And also, when I hover over to any conversation, there is a green line appears on top and sides. That actually breaks the UI positioning. That's really terrible. You need to fix it. You don't need to have the green bar on top. If you're refreshing or something like this, you need to do it under the hood. And why you are refreshing when hover over, I don't understand, because in most cases, when you generate this tree, you should have caching data. So why you are refreshing? So you should only refresh in 30 seconds or based on the Refresh button on top... And the tree view will only be updated in next 30 seconds as per the configuration, not before that."

### Key Findings & Defect Categorization:
1. **The "Green Line" on Hover & Layout Jitter:**
   - In `ConversationNode.tsx`, container elements switch border widths or introduce focus outlines on hover/focus, altering box-model sizing and jittering text layout.
   - In `PromptTreeViewModal.tsx`, action notifications (`actionMsg`) insert an emerald status bar (`bg-emerald-500/10 border-b border-emerald-500/20`) above the main tree layout, jumping the whole split-pane down by 36px.
   - Pulsing emerald badges (`border-emerald-500/40 animate-pulse`) lack clipped overflow, creating visual green spikes on neighboring nodes.
2. **Hover Refetching & Interval Thrashing:**
   - Moving the cursor across the tree triggered re-renders that recreated `loadTree`, resetting auto-sync intervals and firing unwanted background fetches.
   - `usePromptTreeData.ts` lacked a 30-second minimum cooldown throttle (`TREE_CACHE_TTL_MS`), causing instant queries against disk transcripts.
3. **Non-Clickable Image Formats in IDEs:**
   - Media paths were provided as plain strings or non-standard paths that IDEs (VS Code, Cursor, Antigravity IDE) cannot detect as clickable hyperlinked files.
   - Required standard: RFC-8089 `[label](file:///<path>)` syntax with a 1-click "Open in IDE" action.
4. **Media Capture & Positioning Inaccuracies:**
   - In multi-turn prompts and subagent trees, screenshots were captured under one turn but incorrectly displayed across all turns or attached to the wrong turn index.
   - Capture position metadata and turn isolation were not strictly enforced during image extraction.

---

## 2. Implementation Contracts & Step-by-Step Execution Plan

### Step 1: Hover Outline Elimination & Stable Styling in `ConversationNode.tsx`
- **Location:** `src/components/instances/PromptTreeView/ConversationNode.tsx`
- **Contract:**
  1. Base row styling: Enforce permanent `border border-transparent` so hover transitions never alter dimensions.
  2. Hover styling: Rely strictly on background transitions (`hover:bg-slate-100/80 dark:hover:bg-[#181126]/70`).
  3. Strip out any `outline-green-*` or `border-emerald-*` classes on the row container.
  4. Selected state: Apply subtle accent border `border-purple-500/40` or left border with zero width delta.
  5. Prompt turn subtree items (lines 408–412): Base state must have `border-l-2 border-transparent`, transitioning to `border-sky-500 bg-sky-500/10` ONLY when `isTurnActive === true`.

```tsx
// Contract: ConversationNode container
className={cn(
    'group flex items-center justify-between rounded-md px-1.5 py-0.5 text-xs cursor-pointer transition-colors duration-100 relative outline-none focus:outline-none focus-visible:!outline-none border border-transparent',
    isSelected && selectedTurnNumber === null
        ? 'border-purple-500/40 bg-gradient-to-r from-violet-600/30 to-purple-600/20 text-white font-medium'
        : isSelected
          ? 'border-purple-500/30 bg-purple-500/15 text-purple-300 font-medium'
          : 'text-slate-600 dark:text-zinc-300 hover:bg-slate-100/80 dark:hover:bg-[#181126]/70'
)}
```

### Step 2: 30-Second TTL Caching Guard in `usePromptTreeData.ts`
- **Location:** `src/components/instances/PromptTreeView/usePromptTreeData.ts`
- **Contract:**
  1. Export constant: `export const TREE_CACHE_TTL_MS = 30_000;`.
  2. Add ref: `const lastLoadTimeRef = useRef<number>(0);`.
  3. In `loadTree()`:
     ```typescript
     const now = Date.now();
     if (!isForce && !isInitialLoad && now - lastLoadTimeRef.current < TREE_CACHE_TTL_MS) {
         return; // Cooldown active; abort background fetch
     }
     lastLoadTimeRef.current = now;
     ```
  4. Auto-sync `useEffect`:
     - Clamp polling interval: `const effectiveInterval = Math.max(TREE_CACHE_TTL_MS, intervalMs);`.
     - Remove volatile dependencies that cause timer churn on hover or mouse movement.
  5. Decouple mouse events: Ensure `onMouseEnter`, `onMouseMove`, and `hover` handlers do not trigger `loadTree`.

### Step 3: Clickable IDE File Links in `PromptInstructionView.tsx` & `markdown.tsx`
- **Locations:**
  - `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
  - `src/components/instances/PromptTreeView/markdown.tsx`
  - `src/components/instances/PromptTreeView/clipboard.ts`
- **Contract:**
  1. `formatIdeClickableFileLink`:
     - Convert local filesystem paths to `file:///` URIs.
     - Return markdown link: `[<filename>](file:///<path>)`.
  2. "Copy Prompt (Local File Links)":
     - Rewrite all image references into `[Screenshot](file:///<path>)` and `![Screenshot](file:///<path>)`.
  3. In `PromptThumbnailCard`:
     - Add 1-click "Open in IDE" button (`ExternalLink` icon) invoking `app.opener().open_path(resolvedPath)` or `open_file_in_ide`.
  4. In `RichMarkdownRenderer`:
     - Render `file:///` links as interactive clickable chips that trigger editor launch on click.

### Step 4: Strict Turn-to-Media Coordinate & Positioning Alignment
- **Location:** `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
- **Contract:**
  1. In `extractImagesFromPrompt`:
     - When `conv.selected_turn_number != null`, strictly extract media from the turn matching `turn_number === conv.selected_turn_number`.
     - Do NOT append media from other turns into the active turn thumbnail strip.
  2. Subagent isolation:
     - Media attached to subagent child runs must be explicitly badged (`Subagent Image`) and rendered only in subagent inspector contexts.
  3. Capture positioning accuracy:
     - Extract coordinates and timestamps from `media_paths` metadata, ensuring screenshots taken during execution correspond 1:1 to their execution step.

---

## 3. Unit & Integration Test Specifications

### 3.1 Unit Test Suite (`src/components/instances/PromptTreeView/__tests__/imageStandalonePipeline.test.ts`)
Add automated test coverage for:
1. `formatIdeClickableFileLink`:
   - Verifies conversion of relative paths to `file:///` markdown links.
   - Verifies conversion of Windows and POSIX paths to valid `file:///` URIs without syntax corruption.
2. `TREE_CACHE_TTL_MS` Cooldown Guard:
   - Verifies that rapid successive calls to `loadTree(false, false)` within 30 seconds execute only once.
   - Verifies that `loadTree(false, true)` (`isForce: true`) immediately bypasses the cooldown guard.
3. Turn-Specific Media Extraction:
   - Verifies that selecting Turn 2 in a 3-turn conversation extracts only Turn 2 media.

---

## 4. Acceptance Criteria Checklist

- [ ] **Zero Hover Outlines:** Hovering over any conversation row displays zero green lines or borders.
- [ ] **Zero Layout Jitter:** Box-model padding and dimensions remain static on hover (0px shift).
- [ ] **30-Second TTL Guard:** Auto-sync and background polling never fire more frequently than every 30,000 ms.
- [ ] **Clickable IDE File Links:** Image links copied or previewed adhere to RFC-8089 `[label](file:///...)` format and open directly in developer IDEs.
- [ ] **Media Positioning Accuracy:** Attached screenshots match the exact prompt turn where they were captured.
- [ ] **Strict Relative Git Paths:** Zero absolute filesystem paths recorded in spec, plan, or code comments.
- [ ] **Unit Tests Green:** All unit test assertions in `imageStandalonePipeline.test.ts` pass cleanly.
