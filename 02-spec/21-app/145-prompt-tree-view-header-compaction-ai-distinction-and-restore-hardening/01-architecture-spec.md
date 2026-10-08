# 01-architecture-spec: Prompt Tree View Header Compaction, Origin Distinction, Live Results, and Restore Architecture

- **Spec ID:** `145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/01-architecture-spec.md`
- **Application:** Antigravity-Manager (`agm`, GUI, Proxy, Backend)
- **Status:** `APPROVED`
- **Lead Architect:** Antigravity Pairing Agent

---

## 1. Executive Summary

This architecture specification defines the comprehensive overhaul of the Antigravity-Manager Project & Prompt Tree View subsystem. It addresses critical defects observed in user prompt tracking, live execution results visibility, AI instruction vs User prompt distinction, UI header compaction, truncation rendering hygiene, rich clipboard handling (copy with image), and FIFO prompt backup/restoration.

---

## 2. Core Architectural Pillars

### 2.1 3-Tier Prompt Origin Classification & Sub-Point Hierarchy
In Antigravity, conversation transcripts contain three distinct categories of entries:
1. **User Prompts (`USER_PROMPT`):** Direct instructions submitted by the user. These form the primary top-level prompt nodes under each project.
2. **AI Subagent Instructions (`AI_SUBAGENT`):** Autonomous instructions created by subagents (`invoke_subagent`, background worker, codebase researcher). In the UI, these are structured as nested **sub-points** under the parent user prompt or within a dedicated sub-agent collapsible section, with distinct visual badges (`↳ AI Subagent Instruction`).
3. **Non-Prompts (`NON_PROMPT`):** System notifications (`Task id ... completed`, `[Message] timestamp=...`, `<SYSTEM_MESSAGE>`, ephemeral setup alerts, empty placeholder conversations). These are strictly filtered out from the prompt tree root view so they never pollute the prompt history.

### 2.2 Live Results Streaming & Dual Prompt/Output Pane
Previously, `PromptTreeViewModal` only displayed the initial input text. For running prompts or finished runs, the user could not view what the AI actually executed or output.
The new architecture introduces:
- **`latest_response` / `execution_results` Field:** Backend transcripts extraction reads the AI model planner response, thinking summary, tool calls, and final text response.
- **Dual Tab Mode:** `[ Prompt Instruction ]` and `[ AI Results & Tool Outputs ]` tabs in the preview pane. When a prompt is running, the Results pane live-updates with step execution progress.

### 2.3 UI Header & Toolbar Compaction (`AGENTS.md` Capsule Invariant)
Per project rules in `AGENTS.md`, all adjacent actions must be wrapped into contiguous segmented dark-glass pill capsules (`rounded-full`, shared border, subtle divider lines, glass backdrop, tight typography).
- **Modal Header:** Single compact bar with title, instance pill, and a unified action capsule: `[ Backup | Restore | Refresh | Sync | Maximize | Close ]`.
- **Search & Scope:** Single compact top row merging search input and project scope selector.
- **Filter Toolbar:** Single segmented pill bar merging Category (`All` | `User` | `AI`) and Status (`Running` | `Queued` | `Pinned`).
- **Prompt Action Toolbar:** Consolidated 2-capsule layout (Content & Export capsule, Execution & Dispatch capsule) eliminating all loose, disjointed buttons.

### 2.4 Truncation Hygiene & Full Transcript Ingestion
- **Dual-Source Ingestion:** Backend checks `transcript_full.jsonl` whenever `truncated_fields` is present in `transcript.jsonl` to ensure full prompt and result content is loaded without artificial byte cuts.
- **UI Inline Rendering:** Eliminates the jarring banner between the header and preview. Replaces mid-sentence `<truncated N bytes>` tags with sleek inline pill badges (`✂ [X KB omitted in log]`) without line breakage.

### 2.5 Rich Clipboard & Image Export
- **Copy with Image:** Utilizes `ClipboardItem` with `text/html` (embedding inline `<img>` tags or data URIs) and `text/plain` fallback.
- **Export (.md & .json):** Dedicated compact dropdown to export prompt instructions alongside referenced images and execution results.

### 2.6 FIFO Backup & Restore Serialization
- **Backup Format:** Versioned JSON schema capturing projects, conversations, classification (`USER_PROMPT` vs `AI_SUBAGENT`), step counts, images, and results.
- **Restore Engine:** Enforces FIFO ordering when restoring tasks to `.antigravity_resume_task.json` across instance workspaces.
