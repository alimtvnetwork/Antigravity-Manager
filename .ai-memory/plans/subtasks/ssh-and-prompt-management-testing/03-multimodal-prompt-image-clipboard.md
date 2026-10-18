# Subtask Implementation Plan: Multimodal Prompt Image Clipboard (IDE & OneNote Integration)

> **Subtask ID:** `03-multimodal-prompt-image-clipboard`  
> **Parent Slug:** `ssh-and-prompt-management-testing`  
> **Spec Reference:** [02-spec/21-app/ssh-and-prompt-management-testing/02-component-spec.md](../../../02-spec/21-app/ssh-and-prompt-management-testing/02-component-spec.md)  
> **Target Files:**
> - `src/components/instances/PromptTreeView/clipboard.ts`
> - `src/components/instances/PromptTreeView/PromptImageContextMenu.tsx`
> - `src/components/instances/PromptTreeView/PromptImagePreviewModal.tsx`
> - `src/components/instances/PromptTreeView/DetailHeader.tsx`  
> **Status:** `[QUEUED]`  
> **Author:** `@aukgit`  

---

## 1. Context & Objectives

Prompt turns in Antigravity-Manager frequently contain multimodal assets, such as UI mockups, error screenshots, architecture diagrams, and terminal captures. When developers and operators inspect prompt transcripts, they copy these turns into distinct destination environments:
1. **Developer IDEs & Terminals (VS Code, Cursor, Windsurf)**: Consumers require clean plain-text Markdown (`![Image](data:image/png;base64,...)` or relative path `./.antigravity/attachments/attachment-{hash}.png`). Inserting rich binary objects into code editors corrupts buffer formatting.
2. **Rich Note-Taking & Documentation Apps (Microsoft OneNote, Microsoft Word, Google Docs)**: Consumers require formatted HTML clipboard payloads with embedded Base64 `<img src="data:image/png;base64,...">` tags. If external local paths or binary-only blobs are pasted, OneNote renders broken red-x image placeholders.
3. **Graphics Tools & Web Platforms (Paint, Photoshop, Discord, Gemini Web)**: Consumers require native binary PNG clipboard insertion (`ClipboardItem({ 'image/png': pngBlob })`) to paste raw pixels directly.

The objective of this subtask is to test, verify, and harden the multi-format clipboard pipeline across `clipboard.ts`, `PromptImageContextMenu.tsx`, `PromptImagePreviewModal.tsx`, and `DetailHeader.tsx`.

---

## 2. Architecture & Pipeline Verification

### 2.1 Multi-Format Clipboard Matrix

| Target Destination | Primary Method | MIME Types Provided | Expected Behavior |
|---|---|---|---|
| **VS Code / Cursor Chat Bar** | `copyImageAsMarkdownText` | `text/plain` | Pastes clean markdown tag `![Image](data:image/png;base64,...)` or relative link |
| **Microsoft OneNote / Word** | `copyPromptWithRichImages` | `text/html`, `text/plain`, optional `image/png` | Pastes rich text with embedded Base64 `<img>` tags; images render inline |
| **Paint / Photoshop / Discord** | `writeNativeImageToClipboard` | `image/png` | Pastes raw image pixels directly onto canvas or chat attachment bar |
| **Multi-Image Batch to OneNote** | `copyAllImagesAsRichHtml` | `text/html`, `text/plain` | Formats all images into ordered `<p><img src="..."/></p>` HTML document |

---

### 2.2 Clipboard Resiliency Ladder in `clipboard.ts`

When writing complex multi-MIME payloads to the system clipboard, browser WebView engines enforce strict security permissions and platform restrictions:
1. **Focus Check & Assurance (`ensureDocumentFocus`)**:
   - Calls `window.focus()` and verifies `document.hasFocus()` before attempting `navigator.clipboard.write`.
2. **Stage 1: Triple MIME Write (`image/png` + `text/html` + `text/plain`)**:
   - Asynchronously converts the primary image to a standard PNG binary Blob via `convertImageUriToPngBlob(base64Uri)`.
   - Packages `ClipboardItem({ 'image/png': pngBlob, 'text/html': htmlBlob, 'text/plain': textBlob })`.
3. **Stage 2: Dual MIME Fallback (`text/html` + `text/plain`)**:
   - If the WebView rejects mixed binary and rich HTML types, catches the exception and immediately attempts `ClipboardItem({ 'text/html': htmlBlob, 'text/plain': textBlob })`.
4. **Stage 3: Resilient Plain Text Fallback (`copyToClipboard`)**:
   - If `navigator.clipboard` is unavailable or permissions are denied, falls back to `copyToClipboard(markdownText)` (execCommand / hidden textarea ladder).
   - Never swallows errors: logs any fallback warnings to `useErrorStore.getState().trackWarning`.

---

## 3. Concrete Verification & Test Steps

### Step 1: Verify Single Image Markdown Text Copy (`copyImageAsMarkdownText`)
- **Target Functions:** `copyImageAsMarkdownText` in `src/components/instances/PromptTreeView/clipboard.ts`.
- **Test Scenarios:**
  1. *Prompt Text Input:* Pass markdown containing `![Mockup](data:image/png;base64,iVBORw...)`. Ensure output is exactly the markdown tag string.
  2. *Direct URI Input:* Pass a raw image URI `data:image/png;base64,...`. Ensure output formats as `![Image](data:image/png;base64,...)`.
  3. *Multiple Assets Input:* Pass markdown with 2 image URLs. Ensure output formats as numbered markdown tags `![Image 1](url1)\n\n![Image 2](url2)`.
- **Verification Criterion:** Clipboard content retrieved via `navigator.clipboard.readText()` matches expected markdown syntax without extra HTML wrapper tags.

### Step 2: Verify Rich HTML with Embedded Base64 for OneNote (`copyPromptWithRichImages`)
- **Target Functions:** `copyPromptWithRichImages`, `convertImageUriToBase64` in `src/components/instances/PromptTreeView/clipboard.ts`.
- **Test Scenarios:**
  1. Convert markdown with relative image paths or HTTP URLs.
  2. Verify that `convertImageUriToBase64` resolves local files using `convertFileSrc` and renders to an offscreen canvas to produce complete `data:image/png;base64,...` URIs.
  3. Verify HTML output contains:
     ```html
     <!DOCTYPE html><html><head><meta charset="utf-8"></head><body ...>
       <p style="margin:8px 0;line-height:1.6;">...</p>
       <p style="margin:12px 0;"><img src="data:image/png;base64,..." alt="image" style="..." /></p>
     </body></html>
     ```
  4. Verify that pasting into Microsoft OneNote displays the image inline rather than an empty container.

### Step 3: Verify Context Menu Actions in `PromptImageContextMenu.tsx`
- **Component File:** `src/components/instances/PromptTreeView/PromptImageContextMenu.tsx`.
- **Test Scenarios:**
  1. Right-click an attached image inside `PromptInstructionView`. Verify menu mounts at cursor position (`z-[100050]`) with viewport clamping.
  2. Click "Copy Single Image (PNG)": Verify `writeNativeImageToClipboard` is invoked, emerald `Check` icon displays for 1200ms, and menu auto-dismisses.
  3. Click "Copy Image as Text (Markdown)": Verify markdown text is copied and toast notification displays.
  4. Click "Copy All Attached Images": Verify `copyAllImagesAsRichHtml` formats all prompt images into rich HTML.
  5. Press `Escape` or click outside: Verify menu closes immediately.

### Step 4: Verify Lightbox Light & Zoom in `PromptImagePreviewModal.tsx`
- **Component File:** `src/components/instances/PromptTreeView/PromptImagePreviewModal.tsx`.
- **Test Scenarios:**
  1. Trigger preview modal: Verify backdrop blur `bg-black/85 backdrop-blur-md` renders.
  2. Keyboard interactions:
     - Press `+` / `=`: Verifies `zoomScale` increments by 0.25 (capped at 5.0x).
     - Press `-`: Verifies `zoomScale` decrements by 0.25 (minimum 0.25x).
     - Press `0`: Verifies `zoomScale` resets to 1.0x and pan resets to (0,0).
     - Press `Escape`: Verifies modal closes.
  3. Toolbar copy buttons:
     - Click "Copy PNG": Invokes `writeNativeImageToClipboard` and updates button to "Copied PNG!".
     - Click "Copy as Text": Invokes `copyImageAsMarkdownText` and updates button to "Copied Text!".

### Step 5: Verify DetailHeader Copy Capsule in `DetailHeader.tsx`
- **Component File:** `src/components/instances/PromptTreeView/DetailHeader.tsx`.
- **Test Scenarios:**
  1. Open the "Copy" dropdown menu inside the segmented dark-glass capsule.
  2. Test `onCopyCleanText`: Strips images from prompt text and copies plain text.
  3. Test `onCopyAllWithRichImages`: Triggers rich HTML copy.
  4. Test `onCopyImageAsText`: Copies all prompt images as markdown text.

---

## 4. Invariants & Guardrails

- **Zero Absolute File Paths**: All prompt image rewrites for IPC or storage must use relative paths (`./.antigravity/attachments/attachment-{hash}.png`).
- **Focus Verification**: Always call `ensureDocumentFocus()` before accessing `navigator.clipboard` to prevent DOMException failures in Tauri WebViews.
- **No Silent Failures**: All clipboard exceptions must route to `useErrorStore.getState().trackWarning` with source identifiers (`clipboard.writeNativeImageToClipboard`, `PromptTree.copyPromptWithRichImages`).
- **Attribution Invariant**: Strictly attribute `@aukgit` in changelogs and git commits.

---

## 5. Verification Checkpoints

1. **Frontend Compilation Check:**
   ```powershell
   npm run build
   ```
2. **Type Safety Audit:**
   ```powershell
   npx tsc --noEmit
   ```
3. **Manual Verification Protocol:**
   - Launch AGM app (`npm run tauri dev` or inspect running build).
   - Open Prompt Tree View on an instance with multimodal turns.
   - Right-click an attached image and execute:
     - `Copy Single Image (PNG)` -> Paste into Microsoft Paint or Discord -> Verify image renders.
     - `Copy Image as Text (Markdown)` -> Paste into VS Code -> Verify `![Image](...)` text pastes.
     - `Copy All Attached Images` -> Paste into Microsoft OneNote -> Verify rich HTML renders inline.
