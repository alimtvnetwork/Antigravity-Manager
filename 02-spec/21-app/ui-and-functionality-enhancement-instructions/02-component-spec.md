# Component & UI Specification: Prompt Tree, Markdown, and Modals

## 1. Global Stacking & z-Index Taxonomy

To prevent modals, context menus, and toolbars from disappearing behind the primary modal backdrop (`z-[99999]`), the following strict z-index hierarchy is enforced:

| Component | Selector / Layer | Minimum z-Index |
|---|---|---|
| `PromptTreeViewModal.tsx` | Main backdrop & window container | `z-[99999]` / `z-[100000]` |
| `ConversationContextMenu.tsx` | Native right-click menu | `z-[100050]` |
| `ProjectContextMenu.tsx` | Project right-click menu | `z-[100050]` |
| `PromptImageContextMenu.tsx` | Image right-click menu | `z-[100050]` |
| `DetailsModal.tsx` | Instance / IDE metadata modal | `z-[100050]` |
| `InspectorModal.tsx` | Deep JSON inspector modal | `z-[100050]` |
| `VerificationModal.tsx` | Evidence & verification modal | `z-[100050]` |
| `PromptImagePreviewModal.tsx` | Pan / Zoom image lightbox | `z-[100050]` |
| `PromptTreeExportModal.tsx` | Standalone export modal | `z-[100050]` |
| `PromptTreeImportModal.tsx` | Standalone import modal | `z-[100050]` |

## 2. Conversation Session Dashboard & Enqueued Prompts

When `selectedTurnNumber === null` (a conversation is clicked in the tree):
- Render a distinct **Conversation Session Summary Dashboard** above the instruction view.
- Render total prompt count, active status pill (`RUNNING`, `QUEUED`, `COMPLETED`, `IDLE`), session duration, and last active timestamp.
- Render the **Enqueued Prompts List** with ordering badges (`#1 Enqueued`, `#2 Enqueued`), character count, and preview.
- Render chronological turn pills (`T1`, `T2`, `T3`...) with one-click navigation to specific turns.

## 3. Button Border Radii & Layout Hygiene

- Replace non-conforming `rounded-full` buttons across PromptTreeView components with `rounded-md` (4px/6px) or segmented capsule containers (`rounded-lg`).
- Reposition Input / Output toggle buttons into the right-hand side toolbar adjacent to the "Expand Full" toggle.
- Restore the `Details` button in `DetailHeader.tsx` and pass runtime instance metadata (`instanceExeName`, `instancePid`, `instanceSeqNum`, `executablePath`) from `PromptTreeViewModal.tsx`.

## 4. Multimodal Clipboard & Image Context System

- Dual Copy Actions in Header:
  - `Copy (includes images)`: Triple-MIME clipboard write containing raw PNG binary + rich HTML `<img src="..." />` tags + plain markdown text.
  - `Copy without images`: Clean text-only prompt string with markdown image syntax stripped.
- Image Context Menu:
  - Right-click on any image thumbnail or markdown rendered image allows:
    1. Copy Single Image (binary PNG to clipboard).
    2. Copy All Attached Images (rich HTML to clipboard memory).
    3. View Fullscreen Lightbox (with pan/zoom controls).
    4. Download Image File.
