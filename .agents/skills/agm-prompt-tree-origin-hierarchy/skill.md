---
name: agm-prompt-tree-origin-hierarchy
description: Specialized skill for managing the 4-tier prompt origin classification, hierarchical conversation tree rendering with branch glyphs, interactive dark-glass truncation callouts, and deep transcript inspection in Antigravity-Manager.
---

# AGM Prompt Tree Origin Hierarchy & Deep Transcript Inspection

Governs prompt origin categorization, subagent hierarchical tree construction, interactive omission banners, and full transcript resolution in Antigravity-Manager (`src/components/instances/PromptTreeView/`, `src-tauri/src/modules/repo_db.rs`, and `AGENTS.md`).

---

## 1. 4-Tier Prompt Origin Classification

The prompt inspection engine classifies prompts and conversation steps into 4 mutually exclusive origin tiers:

| Tier | Category Badge | Color / Style | Icon | Description & Matching Rules |
|---|---|---|:---:|---|
| **`USER_PROMPT`** | `USER` | Sky (`bg-sky-500/15 text-sky-700 dark:text-sky-300 border-sky-400/30`) | `User` | Direct developer requests, top-level instructions, explicit prompts. |
| **`SUBAGENT_INSTRUCTION`** | `SUBAGENT` | Purple (`bg-purple-500/15 text-purple-700 dark:text-purple-300 border-purple-400/30`) | `Bot` | Delegated subagent tasks matching `invoked by a caller agent`, subagent role tags, or spawned child sessions. |
| **`SYSTEM_MESSAGE`** | `SYSTEM` | Zinc (`bg-zinc-500/15 text-zinc-700 dark:text-zinc-300 border-zinc-400/30`) | `Terminal` | Environment preambles, `<conversation_transcript>`, `<artifacts>`, or runtime system initialization directives. |
| **`TOOL_OUTPUT`** | `TOOL` | Amber (`bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-400/30`) | `Wrench` | Execution results matching `[Tool Result]`, `{"tool_call_id":`, `<tool_response>`, or code output blocks. |

---

## 2. Hierarchical Tree Construction & Branch Glyphs

Subagent conversations and secondary child steps must never be rendered as a flat list. They must nest hierarchically beneath their parent user prompt:

```
[#P001] Main Refactoring Task (USER_PROMPT)
  ├── ↳ [Bot: Codebase Researcher] Scan module dependencies (SUBAGENT_INSTRUCTION)
  │     └── [Wrench] grep results for proxy/pipeline (TOOL_OUTPUT)
  └── ↳ [Bot: Test Runner] Execute local clippy and unit gates (SUBAGENT_INSTRUCTION)
```

- **Branch Glyphs**: Render tree connector lines using standard Unicode tree characters (`├──`, `└──`, and `↳`).
- **Parent-Child Linkage**: Linked via `parent_conversation_id`, subagent caller metadata, or temporal invocation bounds.

---

## 3. Interactive Truncation Callouts

Raw truncation markers (e.g., `<truncated 120 lines>` or `<truncated 45000 bytes>`) must NEVER be rendered as plain unstyled text.

### Implementation: `TruncatedContextCallout.tsx`
- Parses byte and line count markers dynamically.
- Formats sizes human-readably (e.g., `1.2 MB`, `45 KB`, `120 lines`).
- Renders an interactive, dark-glass omission banner:
  ```
  ⚡ [Omitted 45 KB transcript context · Click to expand] [Expand Full ⌵]
  ```
- Clicking the banner opens the complete un-truncated content modal or expands the log chunk inline.

---

## 4. Deep Transcript Inspection Invariant

When reading and parsing conversation transcripts from IDE storage or disk:

1. **Full Log Precedence**: Always prioritize `transcript_full.jsonl` over `transcript.jsonl`.
2. **Reverse-Scan Depth ($\ge 10$ lines)**: When determining active task state or final tool status, scan backwards through at least 10 lines.
3. **Filter Noise**: Bypass trailing `TOKEN_USAGE`, `HEARTBEAT`, and `TELEMETRY` events to reach genuine `tool_calls` and assistant `thinking` blocks.

---

## 5. Key Frontend Components

| File Path | Core Role |
|---|---|
| `src/components/instances/PromptTreeView/tierClassification.ts` | Regex & pattern classification logic resolving the 4 tiers with confidence scores. |
| `src/components/instances/PromptTreeView/TruncatedContextCallout.tsx` | Dark-glass interactive omission callout widget. |
| `src/components/instances/PromptTreeView/TreePanel.tsx` | Visual tree renderer applying indentation and branch glyphs. |
| `src/components/instances/PromptTreeView/ConversationNode.tsx` | Node row widget rendering category badges, status icons, and timing metadata. |
| `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` | Master modal dialog coordinating selection, inspection, and action dispatching. |
