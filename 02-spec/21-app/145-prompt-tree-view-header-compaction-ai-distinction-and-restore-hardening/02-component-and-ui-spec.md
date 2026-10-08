# 02-component-and-ui-spec: UI Components, Tree Hierarchy, and Modal Layout Specification

- **Spec ID:** `145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/02-component-and-ui-spec.md`
- **Application:** Antigravity-Manager (`agm`, GUI, Proxy, Backend)
- **Status:** `APPROVED`
- **Lead Architect:** Antigravity Pairing Agent

---

## 1. UI Hierarchy & Layout Blueprint

### 1.1 Modal Header Specification
The modal header must be ultra-compact (single line height `h-11`), eliminating multi-line subtitle noise.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [Layers] Prompt Tree View  [#1 · Antigravity.exe · Default]   [Backup|Restore|↻|15s|⛶|✕]│
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Left:** Icon (`Layers`), Title (`Prompt Tree View`), Instance Identity Capsule (`#seq · exe · name`).
- **Right:** Unified segmented pill capsule (`rounded-full`, shared border, subtle divider lines):
  - `Backup` (Download icon)
  - `Restore` (Upload icon)
  - `Refresh` (RotateCw icon, spin on loading)
  - `Sync: [15s|30s|1m|off]` (Clock icon with dropdown)
  - `Fullscreen` (Maximize/Minimize icon)
  - `Close` (X icon with subtle hover red)

---

## 2. Left Panel: Project & Conversation Tree Navigation

### 2.1 Unified Compact Toolbar
Replace 5 vertically-stacked controls with 2 sleek rows:
- **Row 1:** Project scope selector dropdown integrated beside Search input.
- **Row 2:** Single segmented dark-glass capsule for Category (`All` | `User` | `AI`) + Status (`All` | `● Running` | `⏳ Queued` | `📌 Pinned`).

### 2.2 Sub-Point Tree Hierarchy & Visual Distinction
Under each Project:
1. **User Prompt Nodes (Top-level):**
   - Sky pill badge: `[User Prompt]` or `User`
   - GitMap dual sequence badge: `[AGM:C001 | GM:#1]`
   - Running pulse badge (emerald): `● RUNNING (mm:ss)`
   - Queued badge (amber): `⏳ QUEUED`
   - Step count badge: `N stp`
2. **AI Subagent Instructions (Nested Sub-Points):**
   - Indented `ml-4 pl-2 border-l border-purple-500/30`
   - Purple pill badge: `↳ [AI Subagent]` or `↳ Sub-Point`
   - Distinct subagent icon (`Bot` or `Cpu`)
   - Attached to the parent user prompt conversation if spawned by it, or grouped under `AI Instructions` collapsible drawer.
3. **Ghost & Non-Prompt Suppression:**
   - Untitled 0-word conversations and background notifications (`Task id ... completed`, `<SYSTEM_MESSAGE>`) are strictly excluded from the tree.

---

## 3. Right Panel: Detail, Live Results, and Dispatch

### 3.1 Prompt Header Row
- Consolidated single metadata row:
  - Sequence badge `#{seq_code}`
  - Origin badge: `[User Prompt]` (sky) vs `[AI Subagent Instruction]` (purple)
  - Status badge: `● RUNNING (PID: x · mm:ss)` (emerald) or `⏳ QUEUED` (amber)
  - Title and word count
- **Action Toolbar (2 Contiguous Segmented Dark-Glass Capsules):**
  - **Capsule 1 (Content & Export):**
    - `Copy Text` (clean prompt text)
    - `Copy Verbatim` (+ raw markdown)
    - `Copy with Image` (rich HTML clipboard containing `<img>` tags/base64)
    - `Export` dropdown (`.md` / `.json`)
    - `Save Images` (if prompt contains image assets)
  - **Capsule 2 (Workflow & Dispatch):**
    - Suffix selector dropdown (`None`, `Done?`, `Released?`, `Edge cases`)
    - `Focus IDE` / `Open IDE Window` (restores & brings window to foreground with exact repo path)
    - `Send Now` (hotkey: `N`, immediately writes `.antigravity_resume_task.json`)
    - `Queue` (FIFO enqueue)
    - `Full` (Inspector modal)

### 3.2 Dual Pane: [ Prompt Instruction ] vs [ AI Results & Execution Steps ]
- Add tab switch between:
  - `Prompt Instruction`: The prompt text with rich markdown formatting and inline code copy.
  - `AI Results & Tool Execution`: The AI planner response, thinking blocks, tool calls, and execution steps.
- When prompt is RUNNING:
  - Displays live step badge: `Step N · Tool: <name> (<status>)`.
  - Results tab streams in real-time on auto-sync poll.

### 3.3 Truncation Hygiene
- Completely remove the ugly full-width warning banner between header and preview.
- Parse `<truncated N bytes>` into sleek inline badges `✂ [Omitted N bytes]` without breaking sentences or inserting line breaks.
- Full text loaded from `transcript_full.jsonl` whenever possible.
