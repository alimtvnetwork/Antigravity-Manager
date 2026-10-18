# Specification: Prompt UI/UX, Image Handling & Machine-Agnostic Export

## 1. Executive Summary
This specification defines the architectural and implementation requirements for:
1. Context menu activation in the Prompt Tree View.
2. Distinct UI presentation for conversation sessions versus single prompt turns.
3. Compact button layout, reduced button rounding, and restoration of the Detail button.
4. Dual-mode copy engine (with images vs text only) and image preview lightboxes.
5. Machine-agnostic standalone JSON import/export and modal overlay remediation.

## 2. Topic 1: Tree View Context Menu
- **Fix**: In `ConversationContextMenu.tsx` and `ProjectContextMenu.tsx`, prevent right-click `button === 2` in `handleClickOutside` from closing the menu immediately upon mounting.
- **Actions Required**:
  - `View Details` $\rightarrow$ opens `DetailsModal.tsx`.
  - `Copy Prompt (With Images)` $\rightarrow$ triple-MIME clipboard write.
  - `Copy Prompt (Without Images)` $\rightarrow$ stripped plain text clipboard write.
  - `Copy Image to Clipboard` $\rightarrow$ native PNG blob write.
  - `Export Conversation` $\rightarrow$ opens export modal.

## 3. Topic 2: Conversation View vs Turn Inspector
- When `selectedTurnNumber === null`:
  - Render Conversation Session Dashboard (Session metrics, FIFO enqueued prompts section, interactive turn navigation strip).
- When `selectedTurnNumber !== null`:
  - Render Prompt Turn Inspector with turn breadcrumbs and quick escape button to session dashboard.

## 4. Topic 3: Compact Button Layout & Reduced Rounding
- Group `[ Input | Output | Expand Full ]` into a single right-hand segmented capsule in `DetailTabs.tsx`.
- Replace standalone `rounded-full` buttons across `DetailHeader.tsx`, `DetailTabs.tsx`, and `TreePanel.tsx` with clean `rounded-md` / `rounded-lg` segmented capsules with subtle dividers.
- Restore the `[ Details ]` button in the action bar, forwarding instance, IDE, PID, and turn metadata.

## 5. Topic 4: Dual-Mode Copy & Image Preview Lightbox
- Primary button: "Copy All / Includes Image" using `copyPromptWithRichImages(promptText)`.
- Secondary dropdown option: "Copy without image (text only)" using `stripImagesFromPrompt(promptText)`.
- Inline image thumbnails open `PromptImagePreviewModal.tsx` with pan, zoom, scale-to-fit, and download controls.

## 6. Topic 5: Machine-Agnostic Standalone JSON Export & Overlay Fix
- In `export.rs`: Replace absolute paths with `${workDir}` / `${repoDir}` template anchors.
- Support standalone export (`embed_images: true`) embedding base64 images into `embedded_images: HashMap<String, String>` for full portability across machines.
- In `PromptTreeExportModal.tsx` and `PromptTreeImportModal.tsx`: Elevate z-index to `z-[100050]` and attach `onClick={(e) => e.stopPropagation()}` to prevent outer backdrop traps.
