# UI & Clipboard Specification: Prompt Tree View, Buttons, and Clipboard Transcoding

**Document ID**: `02-spec/21-app/button-functionality-and-testing-instructions/02-ui-components-and-clipboard-spec.md`  
**Task Slug**: `button-functionality-and-testing-instructions`  
**Status**: APPROVED  
**Date**: 2026-10-20  

---

## 1. UI Layout & Typography Requirements

### 1.1 DetailHeader Title Cleanup
- **Target Component**: `src/components/instances/PromptTreeView/DetailHeader.tsx`
- **Visual Spec**:
  - In the prompt detail header, currently:
    ```tsx
    <div className="flex items-center gap-2">
      <Badge>Turn T40</Badge>
      <Badge>Turn 40 of 40</Badge>
      <Badge>IDLE 2h</Badge>
      <span className="font-semibold">{title}</span>
      <CopyButton />
    </div>
    ```
  - **New Design**:
    - Strip all preceding badges before the title line.
    - Set title style: `text-xl md:text-2xl font-bold text-white tracking-wide flex items-center gap-2`.
    - Place title right at the front, followed by the copy title button.
    - Status badges (e.g. idle/running time) can be placed in the secondary meta row (e.g. near word count / size), not cluttering the title.

### 1.2 Action Buttons Bar
- **Target Component**: `src/components/instances/PromptTreeView/DetailHeader.tsx`
- **Buttons Structure**:
  - `Copy`: Renamed from "Copy All (Include Images)" to clear, concise labels:
    - Default button label: "Copy" (or "Copy Prompt")
    - Dropdown options: "Copy Text", "Copy Prompt with Images (File Links)", "Copy Prompt with Images (Base64)", "Copy All Images".
  - `Send`:
    - Sends prompt turn immediately to active IDE or session.
    - If images exist on the turn, sends both text and image payload (base64 or local paths).
  - `Enqueue`:
    - Enqueues turn to pending queue without immediate execution interruption.
  - `Reset`:
    - Clears SQLite prompt tree cache & forces complete transcript rebuild.
    - Displays spinner and success toast notification upon completion.

---

## 2. Clipboard Transcoding & Multi-Image Copying

### 2.1 Pure Image PNG Writing
- **Target Module**: `src/components/instances/PromptTreeView/clipboard.ts`
- **Implementation**:
  - Convert image URI (local `file:///`, Tauri asset protocol `asset://` or `http://`, or `data:image/...;base64`) into `HTMLImageElement`.
  - Draw to offscreen `<canvas>` with identical natural width and height.
  - Export canvas to `image/png` Blob.
  - Write to clipboard via:
    ```typescript
    await navigator.clipboard.write([
      new ClipboardItem({
        'image/png': pngBlob,
      }),
    ]);
    ```
  - This guarantees OneNote, Antigravity, and Windows desktop apps read native bitmap/PNG data correctly.

### 2.2 Multi-Image Copying ("Copy All Images")
- **Batch Image Aggregator**:
  - If multiple images exist on the prompt turn, provide an action to copy all images.
  - Because browser clipboard API historically supports only one `ClipboardItem` with single `image/png` mime, we also:
    1. Write rich HTML containing all images (`<img src="data:image/png;base64,..."/>`) alongside plain text containing markdown image references.
    2. Write the first image as binary `image/png` to satisfy bitmap-only paste targets (like OneNote).
    3. Copy individual image button: explicitly labeled "Copy" (not "Copy including image").
