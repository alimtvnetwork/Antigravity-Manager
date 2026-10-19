# Component & UI Specification: UI and Image Copy Functionality Fixes

**Slug**: `ui-and-image-copy-functionality-fixes`  
**Status**: APPROVED  
**Target Branch**: `main`  
**Author**: Lead Orchestrator  
**Attribution**: strictly `@aukgit`  

---

## 1. Frontend Components & Hook Contracts

### 1.1 `src/components/instances/PromptTreeView/clipboard.ts`
- **`writePureImageToClipboard(blob: Blob): Promise<boolean>`**:
  - Creates a `ClipboardItem({ 'image/png': blob })` with *zero* accompanying text or HTML mime types.
  - Calls `navigator.clipboard.write([item])`.
  - Fallback: If clipboard write fails due to permissions or format mismatch, records warning via `useErrorStore.getState().trackWarning`.
- **`convertImageToValidPngBlob(uriOrPath: string): Promise<Blob>`**:
  - Reads image data via `read_image_as_base64` or `fetch`.
  - Creates an `Image` element and loads the data.
  - Draws onto an offscreen `<canvas width={img.naturalWidth} height={img.naturalHeight}>`.
  - Calls `canvas.toBlob(resolve, 'image/png')` to ensure valid PNG magic bytes regardless of source format (WebP, JPEG, SVG).
- **`copyMultipleImagesToClipboard(imageUrls: string[]): Promise<number>`**:
  - Copies the first image as pure PNG and provides user feedback indicating how many images are available or copies sequentially.

### 1.2 `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
- **Attached Images Gallery Header**:
  - Add an action button group directly inside the header:
    ```tsx
    <div className="flex items-center gap-2">
      <span className="text-xs font-semibold text-slate-700 dark:text-slate-200">
        Attached Images ({images.length})
      </span>
      <button
        type="button"
        onClick={handleCopyFirstImage}
        className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] font-medium bg-purple-500/10 text-purple-600 dark:text-purple-300 border border-purple-500/30 hover:bg-purple-500/20 transition-colors"
        title="Copy pure PNG to clipboard for OneNote / chat"
      >
        <Copy className="w-3 h-3" />
        <span>Copy Image (PNG)</span>
      </button>
    </div>
    ```
  - For each individual image thumbnail card, ensure a 1-click `[ 📋 Copy PNG ]` button is visible on hover.

### 1.3 `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` & `DetailHeader.tsx`
- **Immediate Send vs Enqueue**:
  - `handleSendPromptNow`: Calls `sendPromptNow(targetInstId, repoPath, promptContent, convId)` directly with `media_paths` embedded or preserved.
  - `handleEnqueuePrompt`: Calls `enqueuePrompt(...)` which adds the prompt to the SQLite queue with status `queued`.
  - Distinguish UI buttons:
    - Primary Button: `[ 🚀 Send Now ]` (Executes immediately in running session)
    - Secondary Action: `[ 📥 Enqueue ]` (Appends to queue)
- **Image Forwarding in Send / Rerun**:
  - When calling `sendPromptNow` or `dispatchRerunMessages`, collect all attached `media_paths` and ensure they are appended to the markdown payload (`![image](<path>)`) so the receiving IDE receives the images.

### 1.4 `src/components/instances/PromptTreeView/ConversationNode.tsx`
- **Turn Status Badges beside Prompt Turns**:
  - Inspect each turn's live status (`is_running`, `is_queued`, `step_index`).
  - Render an inline badge next to the turn timestamp/summary:
    - If running: `<span className="px-1.5 py-0.5 rounded text-[9px] font-bold bg-emerald-500/20 text-emerald-400 border border-emerald-500/40 animate-pulse">RUNNING</span>`
    - If queued: `<span className="px-1.5 py-0.5 rounded text-[9px] font-bold bg-amber-500/20 text-amber-400 border border-amber-500/40">QUEUED</span>`
    - If completed: `<span className="px-1.5 py-0.5 rounded text-[9px] font-mono text-slate-400">COMPLETED</span>`

---

## 2. Backend Modules & Telegram Contracts

### 2.1 `src-tauri/src/modules/telegram_inbound/node_label.rs`
- **GitMap Node Alias Resolution**:
  - In `resolve_node_display_alias()`:
    1. If the current alias is empty, `"none"`, or starts with `"Node-"`, read `~/.gitmap/gitmap-ssh-nodes.json`.
    2. Match local IP `192.168.1.3` against the registered node list in the JSON.
    3. If matched (e.g. `w1`), use alias `w1` and set annotation `"(collected from gitmap)"`.
    4. Cache the resolved alias in memory to prevent repeated disk reads.

### 2.2 `src-tauri/src/modules/telegram_inbound/formatting.rs` & `status_cmds.rs`
- **Telegram Multi-Line Layout**:
  - Structure machine and instance status on separate lines:
    ```
    <b>Machine:</b> w1 (192.168.1.3) [collected from gitmap]
    <b>Instance:</b> default
    ```
