# Master Plan: Task 109 - Instance OS Window Title Format, Card Grid Alignment, Hover Animation & Prompt Tree Layering Overhaul

## 1. Plan Overview
- **Task ID**: `109-instance-os-title-hover-buttons-and-prompt-layering`
- **Scope**:
  - Subtask 1: Backend OS Window Title Formatting & Settings Injection (`src-tauri/src/modules/instance.rs`).
  - Subtask 2: Frontend Instance Card 6-Column Grid Standardization & Bottom CSS3 Hover Animation (`src/pages/Instances.tsx`).
  - Subtask 3: Frontend Prompt Tree Modal Portal Stacking, Project Pinning/Refresh, Prioritization, Filter Pills, Image Actions & Confirmation Suffix Dropdown (`src/components/instances/PromptTreeViewModal.tsx`).
  - Subtask 4: E2E Verification, Pre-flight Checks & Release Bump (`v4.136.0` -> `v4.137.0`).

---

## 2. Work Breakdown & Subtasks

### Subtask 1: Backend OS Window Title Formatting
- **File**: `src-tauri/src/modules/instance.rs`
- **Goal**: Implement `compute_ide_ending_sequence` and `compute_instance_window_title` to ensure the OS window title starts strictly with `#{seq} {name} - {suffix}` across Windows, macOS, and Linux.
- **Verification**: Ensure `create_instance_with_account`, `copy_instance_with_options`, `rename_instance`, `launch_instance_inner`, and `load_registry` inject and persist this title format into `settings.json`.

### Subtask 2: Frontend Card Grid Standardization & Bottom CSS3 Hover Animation
- **File**: `src/pages/Instances.tsx`
- **Goal**:
  1. Standardize Row 2 buttons into a rigid `grid grid-cols-6 gap-1 w-full`. Use invisible spacer for Default instance slot 6 (`Delete`) to preserve exact horizontal alignment of `[Clone]` and `[Executable]` across all cards.
  2. Relocate color accent from top to bottom with a sleek CSS3 hover animation (`opacity-0 scale-x-95` -> `opacity-100 scale-x-100` on hover) and softened card borders.

### Subtask 3: Prompt Tree View Modal Portal Stacking & Full Feature Overhaul
- **File**: `src/components/instances/PromptTreeViewModal.tsx`
- **Goal**:
  1. Mount modal via `createPortal` to `document.body` with `fixed inset-0 z-[300]`, removing ad-hoc top margins that caused header clipping.
  2. Implement full-screen modal mode (`isFullscreen`) with toggle button.
  3. Add inline `RotateCw` (Refresh) and `Pin` (Pin toggle) buttons on each project row, persisting pinned IDs in `localStorage`.
  4. Multi-tier sorting: Pinned projects first -> Actively running projects -> Recent activity -> Alphabetical; running conversations at top.
  5. Search filter pills: `All`, `Running Only`, `Latest Conv`, `Latest Prompt`, `Pinned`.
  6. Image handling: Copy text only, copy with images, save images to disk.
  7. Confirmation suffix dropdown: Append presets ("Is it done?", "Is it released?", etc.) before resending.
  8. Animated running indicator with elapsed duration and PID.

### Subtask 4: Pre-flight Verification & Minor Release Ceremony
- Run `cd src-tauri && cargo fmt -- --check`.
- Run `npm run build`.
- Execute minor bump (`v4.136.0` -> `v4.137.0`).
- Update `CHANGELOG.md`, `CHANGELOG_EN.md`, `README.md`, `README_EN.md` with strict `@aukgit` attribution.
- Commit via `gitmap cpf` and tag `v4.137.0`.
