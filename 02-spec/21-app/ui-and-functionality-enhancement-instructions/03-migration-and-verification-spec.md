# Migration and Verification Specification: UI and Functionality Enhancement Instructions

**Slug:** `ui-and-functionality-enhancement-instructions`  
**Status:** `ACTIVE`  
**Version:** `1.0.0`  
**Author:** `@aukgit`

---

## 1. Migration Strategy & Zero Regression Safeguards

### 1.1 Multi-Instance Running Prompt Isolation Verification
- **Verification Criterion:**
  - Launch an active prompt in Instance #2 (`default-copy-5597`).
  - Query running project nodes via IPC: Instance #1 (`default`) MUST NOT report `hasActiveTask = true` and MUST NOT render a pulsing green "Prompt" badge on its instance card.
  - Instance #2 MUST render the pulsing green "Prompt" badge.

### 1.2 Button Rounding & Layout Compaction Verification
- **Verification Criterion:**
  - In `PromptTreeViewModal`, inspect rendered DOM classes.
  - Action buttons must use `rounded-md` / `rounded-[5px]`; zero `rounded-full` classes on prompt view mode, action pills, or category tabs.
  - View mode toggle `[Input | Output | Expand Full]` must sit contiguous on the right-hand side of the secondary header.

### 1.3 Context Menu Verification
- **Verification Criterion:**
  - Right-clicking any conversation item in the left-hand sidebar opens the unified context menu at cursor coordinates.
  - Menu options dispatch accurately on the target conversation.
  - Fast mouse-wheel scroll does not prematurely dismiss the menu.

### 1.4 Standalone JSON Export & Overlay Verification
- **Verification Criterion:**
  - Exporting a conversation with "Standalone Mode (Embed Images)" produces a JSON envelope with base64 embedded images and `${workDir}` paths.
  - Opening the Export or Import modal displays a readable backdrop without turning completely black.
  - Clicking on the outer modal scrollbar does not close the dialog.

### 1.5 Updater & Pinned Version Verification
- **Verification Criterion:**
  - Running updater with a GitHub release URL (`https://github.com/alimtvnetwork/Antigravity-Manager/releases/tag/v4.226.3`) parses version `v4.226.3` correctly without 404.
  - `install.ps1` detects the active installed version from `agm-ui.exe` rather than reporting `v4.190.0`.
  - Shortcuts correctly launch `agm-ui.exe`.

### 1.6 Fleet Mesh Table Verification
- **Verification Criterion:**
  - With Supabase sync disabled, the Fleet Mesh table renders configured remote nodes (e.g. `w2`, `w3`, `main`) showing their IP, alias, and an offline status badge instead of hiding the entire section.
