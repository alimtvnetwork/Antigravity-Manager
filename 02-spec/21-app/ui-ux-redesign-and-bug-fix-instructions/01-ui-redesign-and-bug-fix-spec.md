# UI/UX Redesign & Bug Fix Specification

**Slug**: `ui-ux-redesign-and-bug-fix-instructions`  
**Status**: In Progress  
**Scope**: PromptTreeView, Session Overview, DetailHeader, DetailTabs, Datetime Formatting, TreePanel Filtering, IDE Liveness Gate, SSH Fleet View

---

## 1. Problem Statement & User Requirements

### 1.1 Collapsible Session Overview Dashboard
- In `PromptInstructionView.tsx`, the Session Overview card (containing turns count, session status, time range, duration, chronological turn chips) takes up substantial vertical space.
- **Requirement**:
  - Collapse by default (`isSessionOverviewExpanded = false`).
  - Provide a subtle purple chevron arrow (`ChevronDown` / `ChevronRight`) that is minimal and non-intrusive.
  - Clicking the arrow toggles the section open/closed.
  - Persist this preference in `localStorage` under `agm_prompt_tree_session_overview_expanded` so it remembers state across turns and sessions.
  - Hide cryptic sequence codes like `C211` and technical "returns" / turns jargon from top view, moving raw diagnostics into the Details modal.

### 1.2 DetailTabs Expand/Fold Button Bug
- In `DetailTabs.tsx`, the `Expand Full` / `Collapse` toggle button disappears when switching between `Input` and `Output` tabs (`previewTab === 'instruction'` vs `results`).
- **Requirement**:
  - The expand/fold toggle button must remain visible and functional across both `Input` and `Output` views.
  - Persist the expanded/collapsed state across tab switches.

### 1.3 DetailHeader Metadata De-duplication
- In `DetailHeader.tsx`, conversation title and sequence ID are duplicated in multiple places (e.g. `UI Redesign And Bug Fixes` and `#C231` repeated in the header and session badge).
- **Requirement**:
  - Clean up repeated headers and IDs.
  - Move full UUIDs (`ID: 81170dc5-...`) and internal technical metadata into the dedicated Details popover/modal.
  - Cleanly format session title and essential status.

### 1.4 Standardized 24-Hour Date/Time Formatting & Relative Time Badges
- Timestamps currently render in raw ISO (`2026-10-18T22:08:28+08:00`) or 12-hour AM/PM formats (`06:50:08 PM`) with meaningless seconds on wide ranges.
- **Requirement**:
  - Display date and time uniformly as `DD-MMM-YYYY HH:mm` (e.g. `18-Oct-2026 21:40`) in 24-hour clock according to local Windows timezone.
  - Omit seconds when the duration or elapsed range is large.
  - Display human-readable relative time badges (e.g. "ran 5 minutes ago", "ran 1 day ago") beneath or adjacent to the timestamp.
  - Maintain UTC0 storage in database/backend and convert only for display. Provide settings configurability.

### 1.5 Tree Panel Empty State & Running/Queued Prompts Surfacing
- In `media_1792330348408_67ed3508.png`, despite 47 projects existing and a prompt currently running (`RUNNING PID: 2188`), the left Tree Panel showed "No projects or conversations found."
- **Requirement**:
  - Trace and fix the root cause in `TreePanel.tsx` / `PromptTreeViewModal.tsx` / `usePromptTreeState.ts`.
  - When filtering or searching, ensure running and queued prompts are always prominently indexed and visible.
  - Ensure status pills (`Running`, `Queued`) accurately reflect active execution states.

### 1.6 Multimodal Prompt Image Visibility
- Multimodal prompt images attached to prompts are not rendering properly in the prompt tree view.
- **Requirement**:
  - Verify image extraction and rendering pipeline in `PromptInstructionView.tsx`, `markdown.tsx`, and `DetailHeader.tsx`.
  - Support copying images as single text (Markdown `![Image](...)` or data URL) for IDEs and rich HTML for OneNote.

### 1.7 IDE Liveness & 5-Second Delay Gate on Send
- When clicking `Send`, if the IDE is not running, launching it and sending immediately causes prompts to be lost because the IDE window isn't ready.
- **Requirement**:
  - Check whether the target IDE process is running.
  - If not running, launch the IDE and wait 5 seconds with polling/readiness checks (window/PID confirmation) before injecting the conversation prompt.

### 1.8 SSH Fleet Account/Email Focus
- In SSH Fleet view, hide administrative usernames and passwords.
- Prominently display node alias, running instance count, and bound account emails. Move raw IP to details.
- Enable cross-instance prompt transmission.

---

## 2. Architecture & Data Flow

```
[PromptTreeViewModal]
   │
   ├── [DetailHeader] (De-duplicated title, status badge, compact Details modal)
   ├── [DetailTabs] (Input / Output tabs, persistent Expand/Fold button across both tabs)
   ├── [TreePanel] (Fix empty-state filter bug, surface Running/Queued prompts, origin filters)
   └── [PromptInstructionView]
         ├── [Session Overview] (Collapsible, subtle purple arrow, persisted in localStorage)
         ├── [Attached Images Gallery] (Visible preview, copy for IDE/OneNote)
         └── [Markdown Renderer] (24h time format, relative badges)
```
