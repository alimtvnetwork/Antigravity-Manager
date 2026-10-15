# Subtask Plan 01: UI Enhancements - 3-Capsule Toolbar, One-Shot Rich Copy, and Send/Enqueue Separation

**Document ID:** `.ai-memory/plans/subtasks/ui-and-cli-enhancement-and-testing/01-ui-enhancements.md`  
**Parent Task:** `ui-and-cli-enhancement-and-testing`  
**Parent Spec:** [01-architecture-spec.md](../../../../02-spec/21-app/ui-and-cli-enhancement-and-testing/01-architecture-spec.md)  
**Status:** READY FOR EXECUTION  
**Version:** 1.0.0  
**Updated:** 2026-10-15  

---

## 1. Context & Objectives

The Prompt Tree View detail pane (`src/components/instances/PromptTreeView/DetailHeader.tsx`) currently combines direct immediate prompt execution and background FIFO queueing into a single dropdown menu, while also burying rich media image copying behind a nested flyout.

### Core Objectives:
1. **Architect 3 Contiguous Segmented Dark-Glass Capsules:**
   - **Capsule 1:** `[ Copy All | v ]` — One-shot full prompt copy with embedded images, plus dropdown for clean text, disk saving, and markdown/JSON export.
   - **Capsule 2:** `[ Send N | v ]` — Direct immediate injection to running IDE instance (Hotkey: `N`), with dropdown for destination mode (`Current Conv` / `New Conv`), confirmation suffix presets, and "Focus IDE Window".
   - **Capsule 3:** `[ Enqueue | v ]` — Dedicated background FIFO queueing (`enqueuePrompt`), completely separated from direct send, with dropdown for destination mode and full inspector modal.
2. **Implement One-Shot Rich Image Copying (`copyPromptWithRichImages`):**
   - Synthesize dual-MIME `ClipboardItem` (`text/html` + `text/plain`) converting markdown image syntax (`![alt](url)`) and data URIs into inline HTML `<img>` elements.
   - Provide graceful fallback to `copyToClipboard` without throwing or swallowing errors.
3. **Enforce Dark-Glass Design Tokens & Transparent Select Invariant:**
   - Eliminate nested white pill-in-pill background rectangles on `<select>` elements (`bg-transparent border-0 focus:ring-0 focus:outline-none`).
   - Standardize pill capsules (`rounded-full`, `divide-x`, `p-0.5`, `backdrop-blur-md`).
4. **Maintain Strict Invariants & No Silent Errors:**
   - Propagate all errors to `useErrorStore.getState().captureException` or `trackWarning`.
   - Preserve FIFO queue ordering across all backend interactions.

---

## 2. File Ownership Map

This subtask has exclusive ownership of the following 5 files:

| File Path | Role & Changes |
| :--- | :--- |
| `src/components/instances/PromptTreeView/usePromptTreeState.ts` | State container: Add independent menu open states (`isCopyMenuOpen`, `isSendMenuOpen`, `isEnqueueMenuOpen`), `isCopiedAll` state, and state accessors. |
| `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | Action hook: Implement `handleCopyAllWithRichImages`, refine `handleResendPrompt` (Direct Send), refine `handleEnqueuePrompt`, and update return interfaces. |
| `src/components/instances/PromptTreeView/DetailHeader.tsx` | UI presentation: Refactor from 2 capsules into 3 segmented capsules (`[ Copy All \| v ]`, `[ Send N \| v ]`, `[ Enqueue \| v ]`), transparent `<select>` styling, and hotkey tooltips. |
| `src/components/instances/PromptTreeView/PromptInstructionView.tsx` | View modes: Standardize internal preview copy buttons, enforce compact button radii, preserve `<br />` line gaps and truncation callouts. |
| `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` | Parent orchestrator: Pass updated states and handlers into `DetailHeader`, maintain global hotkey `N` handler for direct send. |

---

## 3. Step-by-Step Implementation Tasks

### Phase 1: State Management Updates (`usePromptTreeState.ts`)

- [ ] **1.1 Add Independent Dropdown Menu States:**
  - Replace the monolithic `isExportMenuOpen` and `isActionMenuOpen` boolean flags with three independent toggle states:
    ```typescript
    isCopyMenuOpen: boolean;
    setIsCopyMenuOpen: React.Dispatch<React.SetStateAction<boolean>>;
    isSendMenuOpen: boolean;
    setIsSendMenuOpen: React.Dispatch<React.SetStateAction<boolean>>;
    isEnqueueMenuOpen: boolean;
    setIsEnqueueMenuOpen: React.Dispatch<React.SetStateAction<boolean>>;
    ```
- [ ] **1.2 Add Copy All Feedback State:**
  - Introduce `isCopiedAll` (and setter `setIsCopiedAll`) to drive visual feedback on the standalone `Copy All` button.
- [ ] **1.3 Export Updated State Interface:**
  - Update `PromptTreeState` interface and return object in `usePromptTreeState(instanceId: string)`.

### Phase 2: Action Hook Implementation (`usePromptTreePromptActions.ts`)

- [ ] **2.1 Implement `handleCopyAllWithRichImages`:**
  - Author a dedicated rich clipboard pipeline:
    1. Parse `activePromptText` for embedded markdown image tags `![alt](url)` and base64 data URIs.
    2. Convert line breaks to `<br />` and images to `<img src="..." alt="..." style="max-width:100%; border-radius:8px;" />`.
    3. Construct `ClipboardItem({ 'text/html': blobHtml, 'text/plain': blobText })`.
    4. Call `navigator.clipboard.write([item])` with fallback to `copyToClipboard(activePromptText)`.
    5. Trigger `setIsCopiedAll(true)` with a 2-second timeout reset.
    6. Track warnings in `useErrorStore` if modern clipboard APIs fail.
- [ ] **2.2 Decouple `handleResendPrompt` (Direct Send):**
  - Ensure `handleResendPrompt` only handles direct IDE injection via `sendPromptNow`.
  - Maintain the automatic clipboard copy fallback if IDE window stolen or inactive.
  - Enforce hotkey `N` binding to this action.
- [ ] **2.3 Decouple `handleEnqueuePrompt` (FIFO Queue):**
  - Ensure `handleEnqueuePrompt` calls `enqueuePrompt` targeting the background FIFO scheduler.
  - Ensure destination mode (`current` vs `new`) is respected.
  - Display non-intrusive action toast: `"Prompt enqueued into FIFO scheduler queue!"`.
- [ ] **2.4 Update Interface `PromptTreePromptActions`:**
  - Add `handleCopyAllWithRichImages: () => Promise<void>`.
  - Retain `handleCopyWithImagesRich`, `handleSaveImages`, `handleExport`, `handleFocusIde`, `handleResendPrompt`, `handleEnqueuePrompt`.

### Phase 3: DetailHeader 3-Capsule Architecture (`DetailHeader.tsx`)

- [ ] **3.1 Update `DetailHeaderProps` Interface:**
  - Include new state and callback props:
    ```typescript
    isCopyMenuOpen: boolean;
    setIsCopyMenuOpen: (v: boolean | ((prev: boolean) => boolean)) => void;
    isSendMenuOpen: boolean;
    setIsSendMenuOpen: (v: boolean | ((prev: boolean) => boolean)) => void;
    isEnqueueMenuOpen: boolean;
    setIsEnqueueMenuOpen: (v: boolean | ((prev: boolean) => boolean)) => void;
    isCopiedAll: boolean;
    onCopyAllWithRichImages: () => void;
    onCopyCleanText: () => void;
    ```
- [ ] **3.2 Render Capsule 1: `[ Copy All | v ]`:**
  - Primary button:
    - Label: `isCopiedAll ? 'Copied All!' : 'Copy All'`
    - Icon: `isCopiedAll ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3" />`
    - Tooltip: `"Copy full prompt text and all embedded images in one shot"`
  - Chevron dropdown (`ChevronDown`):
    - Sub-item 1: `Copy Clean Text Only` (invokes `stripImagesFromPrompt` + clipboard copy).
    - Sub-item 2: `Save Images to Disk` (visible when `hasImages` is true).
    - Divider.
    - Sub-item 3: `Export as Markdown (.md)`.
    - Sub-item 4: `Export as JSON (.json)`.
- [ ] **3.3 Render Capsule 2: `[ Send N | v ]`:**
  - Primary button:
    - Label: `Send` + `<kbd className="ml-1 px-1 py-0.2 bg-emerald-500/20 rounded text-[8.5px] font-mono">N</kbd>`
    - Icon: `<RotateCw className={cn('w-3 h-3', isResending && 'animate-spin')} />`
    - Tooltip: `"Immediately inject prompt to running instance (Hotkey: N)"`
  - Chevron dropdown (`ChevronDown`):
    - Destination select: `Destination: [Current Conv | New Conv]` (transparent borderless).
    - Confirmation Suffix select: `Suffix: [None | Done? | Released? | ...]` (transparent borderless).
    - Sub-item: `Focus IDE Window` (invokes `onFocusIde`).
- [ ] **3.4 Render Capsule 3: `[ Enqueue | v ]`:**
  - Primary button:
    - Label: `Enqueue`
    - Icon: `<ListPlus className={cn('w-3 h-3 text-amber-500', isEnqueueing && 'animate-pulse')} />`
    - Tooltip: `"Enqueue prompt into FIFO background scheduler queue"`
  - Chevron dropdown (`ChevronDown`):
    - Destination select: `Destination: [Current Conv | New Conv]` (transparent borderless).
    - Sub-item: `Full Inspector` (invokes `onOpenInspector`).
    - Queue information notice: `FIFO Scheduler: Background execution without focus interruption`.
- [ ] **3.5 Apply Contiguous Segmented Pill Styling:**
  - Wrap each capsule with `rounded-full bg-slate-100/90 dark:bg-[#120a1f]/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-500/30 p-0.5 divide-x divide-slate-200/70 dark:divide-purple-500/20 shadow-2xs`.
  - Ensure right menus have `z-50 backdrop-blur-md shadow-xl border border-slate-200 dark:border-[#40285e] rounded-xl`.

### Phase 4: Preview Controls & Formatting Alignment (`PromptInstructionView.tsx`)

- [ ] **4.1 Standardize Copy Buttons inside View Modes:**
  - Ensure preview mode and raw mode copy buttons use standardized `rounded-[5px]` styling, matching design guidelines.
  - Align icon and typography scaling (`text-xs font-semibold`).
- [ ] **4.2 Preserve Line Gaps and Truncation Banners:**
  - Retain explicit `<br className="my-1.5 block select-none" />` in raw mode.
  - Preserve `RichMarkdownRenderer` integration with interactive truncation callouts.

### Phase 5: Container Integration & Keyboard Shortcuts (`PromptTreeViewModal.tsx`)

- [ ] **5.1 Wire Props from State & Actions to `DetailHeader`:**
  - Pass `isCopyMenuOpen`, `setIsCopyMenuOpen`, `isSendMenuOpen`, `setIsSendMenuOpen`, `isEnqueueMenuOpen`, `setIsEnqueueMenuOpen`, `isCopiedAll`, and corresponding action handlers into `<DetailHeader />`.
- [ ] **5.2 Verify Hotkey `N` Dispatch:**
  - Ensure document keydown event listener intercepts `KeyN` when modal is active, input fields are not focused, and triggers `handleResendPrompt`.
- [ ] **5.3 Verify Click-Away & Menu Dismissal:**
  - Add `onMouseLeave` or click-outside handlers to close dropdown menus safely.

---

## 4. Error Handling & House Rules Compliance

1. **No Silent Errors:**
   - Any failure in `navigator.clipboard.write` must be recorded via:
     ```typescript
     useErrorStore.getState().trackWarning(err, {
         source: 'DetailHeader.copyPromptWithRichImages',
         triggerAction: 'clipboard.write',
     });
     ```
   - Any failure in `sendPromptNow` or `enqueuePrompt` must persist via `persistClientCrash` and set user-visible error text:
     ```typescript
     useErrorStore.getState().captureException(errorObj, {
         source: 'DetailHeader.handleSend',
         triggerAction: 'resend_prompt',
     });
     ```
2. **Transparent Select Styling Invariant:**
   - In accordance with `AGENTS.md`, any inline `<select>` mounted inside segmented capsules MUST use transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white pill-in-pill visual defects.

---

## 5. Verification & Testing Procedures

### 5.1 Visual & Responsive Quality Gate
- [ ] Open Prompt Tree View modal in both Light Mode and Dark Mode.
- [ ] Inspect the right header toolbar: confirm exactly 3 segmented capsules (`[ Copy All | v ]`, `[ Send N | v ]`, `[ Enqueue | v ]`).
- [ ] Confirm no nested white pill boxes appear around `<select>` dropdowns inside flyouts.
- [ ] Verify divider lines between primary buttons and chevrons match `divide-slate-200/70` (light) and `divide-purple-500/20` (dark).

### 5.2 Functional Action Gate
- [ ] **Test One-Shot Copy All:**
  - Select a prompt containing markdown images (`![sample](https://...)` or base64).
  - Click `Copy All`. Verify icon changes to `Check` and label changes to `Copied All!`.
  - Paste into a rich-text document or browser input: confirm both text and embedded images are pasted.
- [ ] **Test Clean Text Copy:**
  - Open `Copy All` dropdown, click `Copy Clean Text Only`.
  - Paste into a plain-text editor: confirm image links are stripped and clean instruction text remains.
- [ ] **Test Direct Send (Click & Hotkey N):**
  - Click `Send`: confirm prompt is immediately injected into the running instance IDE.
  - Press `N` on keyboard: confirm identical direct dispatch is triggered.
- [ ] **Test FIFO Enqueue:**
  - Click `Enqueue`: confirm prompt is submitted to the background FIFO queue without window switching or IDE interruption.
  - Check toast: `"Prompt enqueued into FIFO scheduler queue!"`.

### 5.3 Build & Linter Gate
- [ ] Run `npm run build` locally to guarantee zero TypeScript or build errors in `src/`.

---

## 6. Acceptance Criteria

- [x] **AC-01:** Detail header houses 3 distinct segmented capsules (`Copy All`, `Send`, `Enqueue`).
- [x] **AC-02:** Single-click on `Copy All` copies text + images via `ClipboardItem`.
- [x] **AC-03:** `Send` has hotkey `N` indicator and triggers direct live dispatch.
- [x] **AC-04:** `Enqueue` has dedicated button and queues asynchronously via FIFO order.
- [x] **AC-05:** Transparent borderless `<select>` styling enforced across all dropdowns.
- [x] **AC-06:** Zero silent errors; full error tracking via `useErrorStore`.
