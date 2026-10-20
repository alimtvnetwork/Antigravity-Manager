# Prompt Tree View UX, Filtering & Clipboard Specification

**Task Slug**: `telegram-backup-and-fleet-management-enhancements`  
**Location**: `02-spec/21-app/telegram-backup-and-fleet-management-enhancements/02-tree-view-and-clipboard-spec.md`  

---

## 1. Terminal-Style Tree View Hierarchy & Spacing

### 1.1 Top-Level Project Spacing
- Previously, projects were packed tightly with `space-y-0.5`.
- Enhancement: Increase project root spacing to `my-2.5` with distinct card background styling (`bg-slate-800/20 dark:bg-[#120a1f]/40 rounded-lg p-1.5 border border-slate-700/30 dark:border-purple-900/20`), giving each project visual breathing room.

### 1.2 Terminal-Style Indentation & Tree Glyphs
- Child conversations and execution turns must be visibly indented on the right-hand side (`ml-5 pl-3 border-l-2 border-slate-300 dark:border-purple-800/60`).
- Render terminal-style branch connector glyphs (`├──` for intermediate nodes, `└──` for terminal nodes) or crisp SVG branch lines so the visual hierarchy immediately resembles a terminal directory tree (`tree` command).

### 1.3 Global Expand / Collapse Control
- In `TreePanel.tsx` toolbar, provide a toggle button `[ ⊞ Expand All / ⊟ Collapse All ]` to simultaneously expand or collapse all projects and conversation nodes with persistent state.

---

## 2. Filter Bar Enhancements

### 2.1 Attached Images Filter Option
- In `TreePanel.tsx`, add an image filter button `[ 🖼️ With Images ]` to Row 2 alongside `Running`, `Queued`, and `Pinned`.
- When toggled, only projects and conversations containing attached images (`has_images == true` or `media_paths.length > 0`) are displayed.

### 2.2 Auto-Hide Button Sizing Normalization
- Previously, `Hide Auto` used `py-1` and a tall capsule, making it visually out of proportion with adjacent filter pills.
- Normalization: Restyle `Hide Auto` with `px-2 py-0.5 text-[10px]` matching `Running`, `Queued`, `Pinned` in height, border radius, and typography.

### 2.3 Modernized Dark-Glass Sync Interval Dropdown
- Replace the unstyled browser `<select>` in `ModalHeader.tsx` with a custom dark-glass dropdown menu (`bg-[#171025] border border-purple-900/60 shadow-2xl rounded-xl py-1 z-50`).
- Display options (`Off`, `15s`, `30s`, `1m`, `2m`) as styled buttons with checkmarks for the active interval, eliminating raw Windows OS gray dropdown popups.

---

## 3. Pure PNG Composite Image Copying & Queue Status

### 3.1 Composite PNG Canvas Fix
- In `src/components/instances/PromptTreeView/clipboard.ts`:
  - When loading image elements for canvas composition, ensure local paths are always transcoded via `read_image_as_base64` before mounting in `HTMLImageElement` to eliminate CORS canvas tainting (`SecurityError`).
  - Do NOT set `img.crossOrigin = 'anonymous'` on local data URIs.
  - In `copyAllImagesToClipboard`, write to BOTH native `copy_image_to_clipboard` (for Win32 OneNote / Paint / Word) AND `navigator.clipboard.write([new ClipboardItem({'image/png': blob})])` (for browser/webview pasting).

### 3.2 Send Button Queue Status Evaluation
- In `DetailHeader.tsx` and `usePromptTreePromptActions.ts`:
  - Clarify queue status calculation so conversations that are completed or idle do not display a false `QUEUED` indicator when sending prompts.
  - The Send button should display `[ Send N | v ]` with an active state if the conversation is idle/ready, and show queue status only when genuinely waiting in the FIFO prompt queue.
