# Architecture Spec: Prompt Tree View, Quota Bar & CLI Enhancements

**Task Slug:** `01-prompt-tree-quota-bar-cli-enhancements`  
**Version:** v4.159.0 (target)  
**Status:** DRAFT / IN SPECIFICATION  
**Author:** Spec Author 01  
**Target Modules:**  
- `src/components/instances/PromptTreeViewModal.tsx`  
- `src/pages/Instances.tsx`  
- `src-tauri/src/modules/repo_db.rs`  
- `src-tauri/src/modules/backup_prompts_db.rs`  
- `src-tauri/src/bin/agm.rs`  
- `src-tauri/src/modules/cli.rs`  

---

## 1. User Request (Lossless Verbatim Ingestion)

The following requirements are cited verbatim from the project backlog instruction:

> **High Priority Instruction**  
>  
> `https://prnt.sc/_dAZ8hP0KPgZ`  
> `https://prnt.sc/LYS8VqaHwMOF`  
>  
> You can see that this is a serious bug where I have my prompts running, but I don't have the results of anything that is running there. I could not see the prompts, and I could not open the prompts in the main window or whatever I have sent. And there are prompts which I have not executed. Can this distinguish what is prompt, what is not prompt? Because some of those are prompt, some of those are not. It's by the AI instruction. So we need to distinguish those, and those could be a sub point or things like that that you need to work on in the UI. I do see that the prompts bit showed up, but still the header display for the prompts tree view, it's very much bad. It does not have the UI/UX. Seems like broken stuff. The menus are not compact, buttons are not compact. I do see that you have most of the text displayed. That's all right. Appreciate it. You should have export button as well, copy button as well. Copy with image. If I do that, can go there. I don't have the image. Images are also there. That's nice. That's good. The copy button is working, but still, it looks broken, especially the header section. That needs to be compacted, the design issue and things like that, you need to work on it. And also there are things which are not prompt, but showing up. You need to work on it. Also, the preview you need to improve. In between, it shows truncated bytes, which we need to fix in terms of the display, I think. Can you help with that? Improve the preview. And also, if a prompt is running, we need to have some indicator that it is a running prompt or in queue prompt. We need to have that. It is not there yet. So I think you need to look back and look deep, the prompt tree view, how the prompts are collecting, taking the prompts backup, restoring those. I think these are still very limited. You need to work on it very hard. Try to find the root cause. Try to write the spec by yourself, in details. Like the testing, UI/UX specs, make it more professional. Then you work on it, make sure that everything is achieved. And also, if a prompt is repeated. For example, in this case, I do see a fix pipeline that is repeated several times, same prompt repeated. In these cases, I think you can make those as a grouping and display it. You need to work on it properly. There is that, and also the UI is still bad. That means the card section of the instances still looks very poor, and you did not work on it yet. So please get cool, and then work on it. And then finally, do end-to-end testing for sure. And then, how do you do the end-to-end testing? You create a new instance, do your testing, do send prompt, see the running prompt, switch the account, see the prompts are running. These are the things that I want you to verify each time. And also check the tree view is compacted, grouped together, and things like that. Is it clear?  
>  
> Do you think we can increase or enhance the `Antigravity Manager (`Antigravity Manager (AGM)`)` CLI methods, commands, and verb following recent changes in the `gitmap`? Can you please make a great plan, like 30 to 40 improvements plans in the CLI level so that user feels comfortable working with CLI? And also make sure that we have the way to open the rest endpoints for most of the tasks that we wanted to do, and we should be able to have a secure pathway in the future. Is it clear? Do you understand the task? Can you please follow through?  
>  
> Also in the latest version, let's say, the progress bar is a bit problematic. What do I mean by that? It has so many dots and checks, and colors are not even blended in. That is one of the problems. So what I want you to do is make sure that you have not more than five round balls. And the colors needs to be blended in, or with the back-end color, when it is on that position. Remember that. And the left-hand side would be a little bit more dark red, with white. And it should have a percentage number when it goes below, let's say, 25%. So those checks which should have the percentage number rather than check. So these are the things I want you to improve in the progress bar section. It does not look much professional, and you did not reduce the space with space in the scroll bar. It is taking too much spaces, and there are lots of white space, as you can see in the screenshot. So I think first you should write for the AI how it could improve the UI. So look into the screenshot, write for the AI so that it can follow the instruction, and then you fix the progress bar screenshot and everything else. Do you understand? And also make sure the theme and hover over effects are more professional. It is still not that professional yet.  

---

## 2. Visual References & Empirical Screenshot Analysis

Visual evidence has been ingested into `assets/screenshots/` and directly informs this specification:

### 2.1 Ingested Visual Evidence

| Screenshot Reference | Target UI Surface | Empirical Observations & Defects Identified |
|----------------------|-------------------|---------------------------------------------|
| `assets/screenshots/01-prompt-tree-quota-bar-cli-enhancements-01.png` | Instance Cards (Top) & Prompt Tree View Modal (Bottom) | 1. **Prompt Tree Inspector Header Clutter**: Right-hand action buttons (`[Copy With Images]`, `[Save Images]`, `[Suffix Dropdown]`, `[Focus IDE]`, `[Send Now]`, `[Enqueue]`, `[Full]`) are rendered as disconnected, loose, square-cornered buttons stacked across multiple wrapped lines.<br>2. **Consecutive Repeated Prompts**: The left tree hierarchy under `gitmap` displays multiple identical consecutive items: `"Fix Pipeline Error And Bump"` (182 stp, 252 stp) and `"Fix Pipeline Error And Bump Version"` (267 stp, 256 stp), cluttering the tree without deduplication or grouping.<br>3. **Missing Running/Queued Visual Cues**: Items in the left tree list have plain text without pulsing status dots or badges indicating active in-flight processing or scheduler queue status.<br>4. **Instance Quota Bars**: 11 unblended checkmark dots cluttering the quota bars. |
| `assets/screenshots/01-prompt-tree-quota-bar-cli-enhancements-02.png` | Antigravity Process Properties & IDE Subagent Dialogue | 1. **Subagent Instructions Mixed with Human Prompts**: The IDE dialogue transcript displays autonomous subagent invocations (`Research Subagents 01 and 02`, `Message from AI Analysis & Safe Removal Architect`, `Spec Author 01`, `Spec Author 02`), which currently leak into prompt discovery as if they were direct human user prompts.<br>2. **Draft & Unexecuted Prompts**: Unexecuted task instructions and draft prompts in `.antigravity_resume_task.json` are indistinguishable from executed conversational turns. |

---

## 3. Architectural Invariants & Strict Boundaries

Per `AGENTS.md` and repository conventions:
1. **Positive Booleans Exclusively**: All new or modified fields across TypeScript and Rust must strictly use positive boolean prefixes:
   - TypeScript: `hasImages`, `isRunning`, `isQueued`, `isGrouped`, `isUserPrompt`, `hasDraft`, `isSubagent`.
   - Rust: `has_images`, `is_running`, `is_queued`, `is_grouped`, `is_user_prompt`, `has_draft`, `is_subagent`.
   - Strictly forbidden: `notRunning`, `nonUserPrompt`, `isNotGrouped`, `unExecuted`, `disableImages`.
2. **Relative Git Paths**: All documentation, schemas, code comments, and UI paths must strictly use relative repository paths (e.g. `src/components/instances/PromptTreeViewModal.tsx`, `assets/screenshots/01-prompt-tree-quota-bar-cli-enhancements-01.png`). Never output absolute paths or `file:///` URIs.
3. **No Git Execution During Planning**: Spec authoring must not invoke any git write commands.
4. **Disjoint Spec Boundaries**: Spec Author 01 strictly owns `01-architecture-spec.md` and `.ai-memory/plans/subtasks/01-prompt-tree-quota-bar-cli-enhancements/01-prompt-tree-and-cli-architecture.md`. Files `02-component-spec.md` and `02-quota-bar-cards-and-roadmap.md` are reserved for Spec Author 02.
5. **UI Dark-Glass Pill Capsule Standard**: Per `AGENTS.md`, adjacent toolbar actions and window controls must be wrapped into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, and dark-glass styling) rather than loose, disjointed buttons.

---

## 4. Dual-Source Prompt Discovery & Classification Engine

### 4.1 Root Cause Analysis (RCA)

In the current architecture (`src-tauri/src/modules/repo_db.rs`), prompt discovery inspects `brain/<cid>/.system_generated/logs/transcript.jsonl` using a blunt matcher:
```rust
if trimmed.contains("\"USER_INPUT\"") || trimmed.contains("\"USER_EXPLICIT\"") {
    // Treat every occurrence as the latest prompt text
}
```
This naive extraction creates three critical defects:
1. **Subagent Task Pollution**: Subagents spawned by `invoke_subagent` write system briefs, subtask role descriptions, and inter-agent directives into `transcript.jsonl` (under `source: "SYSTEM"` or `type: "SYSTEM_MESSAGE"`). Because both user prompts and subagent tasks appear in the same log directory, autonomous worker prompts are rendered as if the human user had typed them.
2. **Unexecuted Draft Conflation**: Prompts staged in `.antigravity_resume_task.json` or pending in memory queues are conflated with fully executed turns in `conversation_summaries.db`.
3. **Missing Semantic Badging**: The UI tree renders all turns identically with a generic `MessageSquare` icon and sequence badge (`[AGM:C001 | GM:#1]`), leaving the user unable to identify human instructions versus autonomous system execution.

### 4.2 Classification Taxonomy

Transcripts and prompt sources are classified into three mutually exclusive categories:

| Category | Identifier Code | Detection Criteria | Visual Badge & Color Token |
|----------|-----------------|--------------------|----------------------------|
| **Human User Prompt** | `USER_PROMPT` | `source == "USER_EXPLICIT"` AND `type == "USER_INPUT"` AND payload does NOT contain `<SYSTEM_MESSAGE>` or `You are ... for task` subagent signatures. | `[USER]` badge<br>`bg-blue-500/10 text-blue-600 dark:text-cyan-400 border border-blue-500/20` |
| **Subagent Instruction** | `SUBAGENT_INSTRUCTION` | `source == "SYSTEM"` OR `type == "SYSTEM_MESSAGE"` OR content matches `invoke_subagent` invocation prompt / role briefing (`You are [Role] for task ...`, `<SYSTEM_MESSAGE>`). | `[SUBAGENT]` badge<br>`bg-purple-500/10 text-purple-600 dark:text-purple-300 border border-purple-500/20` |
| **Unexecuted Draft** | `UNEXECUTED_DRAFT` | Staged in `.antigravity_resume_task.json` or FIFO database queue with status `draft` or `queued`, but without corresponding step execution in `transcript.jsonl`. | `[DRAFT]` badge<br>`bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20` |

### 4.3 Data Model & Schema Extensions

#### Rust Backend (`src-tauri/src/modules/repo_db.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgmPromptKind {
    #[serde(rename = "user_prompt")]
    UserPrompt,
    #[serde(rename = "subagent_instruction")]
    SubagentInstruction,
    #[serde(rename = "unexecuted_draft")]
    UnexecutedDraft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgmConversationNode {
    pub seq_id: usize,
    pub seq_code: String,
    pub gitmap_seq_code: String,
    pub conversation_id: String,
    pub short_id: String,
    pub title: String,
    pub status: String,
    pub is_running: bool,
    pub is_queued: bool,
    pub is_user_prompt: bool,
    pub is_subagent: bool,
    pub has_draft: bool,
    pub prompt_kind: AgmPromptKind,
    pub step_count: usize,
    pub instance_id: String,
    pub instance_seq_num: Option<usize>,
    pub instance_name: Option<String>,
    pub instance_exe_name: Option<String>,
    pub prompt_preview_200w: String,
    pub prompt_tail_snippet: Option<String>,
    pub prompt_word_count: usize,
    pub last_modified: String,
}
```

#### TypeScript Frontend (`src/components/instances/PromptTreeViewModal.tsx`):
```typescript
export type PromptKind = 'user_prompt' | 'subagent_instruction' | 'unexecuted_draft';

export interface AgmConversationNode {
    seq_id: number;
    seq_code: string;
    gitmap_seq_code: string;
    conversation_id: string;
    short_id: string;
    title: string;
    status: string;
    is_running: boolean;
    is_queued: boolean;
    is_user_prompt: boolean;
    is_subagent: boolean;
    has_draft: boolean;
    prompt_kind?: PromptKind;
    step_count: number;
    instance_id: string;
    instance_seq_num?: number;
    instance_name?: string;
    instance_exe_name?: string;
    prompt_preview_200w: string;
    prompt_tail_snippet?: string;
    prompt_word_count: number;
    last_modified: string;
}
```

---

## 5. Prompt Tree View Header Modernization (Segmented Dark-Glass Pill Capsule)

### 5.1 Design Flaws in Existing Header

As shown in `assets/screenshots/01-prompt-tree-quota-bar-cli-enhancements-01.png`:
1. The right side of the inspector header consists of 8 disparate buttons (`[Copy Text]`, `[Copy With Images]`, `[Save Images]`, `[Suffix Dropdown]`, `[Focus IDE]`, `[Send Now]`, `[Enqueue]`, `[Full]`).
2. Each button uses rectangular `rounded-[5px]` padding with conflicting background colors (slate, blue, emerald, purple, sky).
3. The cluster wraps awkwardly across multiple lines on smaller viewports, violating the core rule in `AGENTS.md`:
   > *"When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close), always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons."*

### 5.2 Segmented Dark-Glass Pill Capsule Specification

The inspector toolbar actions are consolidated into two compact segmented pill capsules:

```
+-------------------------------------------------------------------------------------------------------------------------+
| [Copy Text | Copy With Images | Export v | Save Images]   |   [Focus IDE | Send Now (N) | Enqueue | Suffix v | Full]  |
+-------------------------------------------------------------------------------------------------------------------------+
```

#### Visual Styling Specifications:
- **Container Structure**: `inline-flex items-center rounded-full bg-slate-900/60 dark:bg-[#071a27]/80 backdrop-blur-md border border-slate-700/50 dark:border-[#15334d] p-0.5 divide-x divide-slate-700/40 dark:divide-[#15334d]/60 shadow-lg`
- **Capsule Segment**: `inline-flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-300 hover:text-white hover:bg-white/10 dark:hover:bg-[#15334d]/80 transition-all duration-150 cursor-pointer first:rounded-l-full last:rounded-r-full`
- **Active / Primary Segment (`Send Now`)**: `text-emerald-300 bg-emerald-500/15 hover:bg-emerald-500/25 border-emerald-500/30`
- **Hotkey Badge**: `<kbd className="ml-1 px-1 py-0.2 bg-emerald-500/20 text-emerald-300 rounded text-[9px] font-mono font-bold">N</kbd>`

### 5.3 Consolidated Toolbar Action Catalog

| Action | Icon | Keyboard Shortcut | Functionality |
|--------|------|-------------------|---------------|
| **Copy Text** | `<Copy className="w-3.5 h-3.5" />` | `Cmd/Ctrl+C` (when preview focused) | Strips markdown images and data URIs, copies sanitized plain text. Shows `Copied Text!` toast. |
| **Copy With Images** | `<CopyCheck className="w-3.5 h-3.5 text-indigo-400" />` | `Cmd/Ctrl+Shift+C` | Copies full verbatim markdown AND places rich HTML image representations onto the clipboard. |
| **Export JSON/MD** | `<Download className="w-3.5 h-3.5 text-blue-400" />` | — | Dropdown modal/button providing: (1) `Export Prompt (.md)`, (2) `Export Conversation (.json)`. |
| **Save Images** | `<ImageIcon className="w-3.5 h-3.5 text-sky-400" />` | — | Extracts all inline base64 and embedded image assets into a designated folder with prompt sequence prefix. |
| **Focus IDE** | `<ExternalLink className="w-3.5 h-3.5 text-sky-400" />` | — | Invokes `focus_or_launch_instance` to bring target Antigravity IDE instance into foreground. |
| **Send Now** | `<RotateCw className="w-3.5 h-3.5 text-emerald-400" />` | `N` | Immediately injects prompt via `.antigravity_resume_task.json` with active confirmation suffix. |
| **Enqueue** | `<ListPlus className="w-3.5 h-3.5 text-purple-400" />` | — | Enqueues prompt into AGM FIFO background scheduler queue. |
| **Suffix Selector** | `<ChevronDown className="w-3 h-3 text-slate-400" />` | — | Dropdown capsule embedded in pill to select confirmation suffix (`None`, `Is it done?`, etc.). |
| **Full Inspector** | `<Maximize2 className="w-3.5 h-3.5 text-cyan-400" />` | `F` | Opens comprehensive full-screen prompt inspection modal. |

---

## 6. Repeated Prompts Grouping & Deduplication Engine

### 6.1 Problem Statement

In automated multi-turn workflows (e.g. CI/CD fix cycles, test repairs, lint passes), identical or near-identical prompts are repeatedly dispatched (e.g. `"Fix Pipeline Error And Bump"`, `"Fix Pipeline Error And Bump Version"`).  
As observed in `assets/screenshots/01-prompt-tree-quota-bar-cli-enhancements-01.png`, consecutive runs are rendered as separate redundant rows in the tree, forcing users to scroll through hundreds of nearly identical entries.

### 6.2 Grouping & Deduplication Algorithm

In `renderProjectNode` (`src/components/instances/PromptTreeViewModal.tsx`), conversations within each project are processed through a deduplication partitioner:

```typescript
export interface AgmGroupedConversationNode {
    groupKey: string;
    title: string;
    isGrouped: boolean;
    runCount: number;
    latestConversation: AgmConversationNode;
    runs: AgmConversationNode[];
    isRunning: boolean;
    isQueued: boolean;
}

export function groupProjectConversations(
    conversations: AgmConversationNode[]
): (AgmConversationNode | AgmGroupedConversationNode)[] {
    const result: (AgmConversationNode | AgmGroupedConversationNode)[] = [];
    let currentGroup: AgmConversationNode[] = [];

    const flushGroup = () => {
        if (currentGroup.length === 0) return;
        if (currentGroup.length === 1) {
            result.push(currentGroup[0]);
        } else {
            const latest = currentGroup[0]; // sorted DESC by timestamp
            result.push({
                groupKey: `group-${latest.title}-${latest.seq_id}`,
                title: latest.title,
                isGrouped: true,
                runCount: currentGroup.length,
                latestConversation: latest,
                runs: [...currentGroup],
                isRunning: currentGroup.some((c) => c.is_running),
                isQueued: currentGroup.some((c) => c.is_queued),
            });
        }
        currentGroup = [];
    };

    for (const conv of conversations) {
        if (currentGroup.length === 0) {
            currentGroup.push(conv);
            continue;
        }

        const prev = currentGroup[currentGroup.length - 1];
        const isSameTitle = normalizePromptTitle(prev.title) === normalizePromptTitle(conv.title);

        if (isSameTitle) {
            currentGroup.push(conv);
        } else {
            flushGroup();
            currentGroup.push(conv);
        }
    }

    flushGroup();
    return result;
}
```

### 6.3 Grouped Node UI Presentation

When `isGrouped: true`:
1. **Collapsed State**: Renders as a single composite node in the tree list.
   - Shows folder/stack icon (`<Layers className="h-3.5 w-3.5 text-cyan-400" />`).
   - Title displays prompt name (e.g. `"Fix Pipeline Error And Bump"`).
   - Multiplier Badge: `<span className="px-1.5 py-0.2 rounded-full text-[10px] font-bold bg-cyan-500/20 text-cyan-300 border border-cyan-500/30">x4</span>`
   - Total steps badge: Sum of all steps across runs (e.g. `858 stp total`).
   - Expanding chevron: `<ChevronRight />` toggles expansion of child runs.
2. **Expanded State**: Unfolds an indented sub-list containing individual historical runs (`Run #4 - 252 stp`, `Run #3 - 182 stp`, etc.) with timestamps.
3. **Selection Behavior**: Clicking the group header selects the latest run (`runs[0]`) in the inspector by default.

---

## 7. Prompt Running & Queued Activity Indicators

### 7.1 Real-Time Status Indicators

To eliminate user uncertainty regarding task execution state, every conversation node and inspector header provides real-time glowing status cues:

| State | Status Condition | Visual Dot / Animation | Badge Label & Style |
|-------|------------------|------------------------|---------------------|
| **RUNNING** | `is_running == true` AND instance PID active AND prompt actively being generated | Glowing Neon-Green Dot (`bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.8)] animate-pulse`) | `RUNNING`<br>`bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 animate-pulse` |
| **QUEUED** | `is_queued == true` OR status == `"queued"` in FIFO scheduler | Glowing Neon-Amber Dot (`bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.8)] animate-pulse`) | `QUEUED`<br>`bg-amber-500/15 text-amber-400 border border-amber-500/30 animate-pulse` |
| **COMPLETED** | `is_running == false` AND `step_count > 0` | Muted Slate Dot (`bg-slate-400 dark:bg-slate-600`) | `DONE` (shown in details view) |
| **DRAFT** | `has_draft == true` AND unexecuted | Amber Outline Dot (`border border-amber-400`) | `DRAFT`<br>`bg-amber-500/10 text-amber-400 border border-amber-500/20` |

### 7.2 Inspector Header Status Integration

In `PromptTreeViewModal.tsx`, the inspector header displays the live elapsed timer and running badge:
```tsx
{selectedConversation?.is_running && (
    <div className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 text-xs font-bold animate-pulse">
        <span className="w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_6px_#10b981]" />
        <span>RUNNING ({formatDuration(elapsedSeconds)})</span>
    </div>
)}
{selectedConversation?.is_queued && (
    <div className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-amber-500/10 border border-amber-500/30 text-amber-400 text-xs font-bold animate-pulse">
        <span className="w-2 h-2 rounded-full bg-amber-400 shadow-[0_0_6px_#f59e0b]" />
        <span>QUEUED (FIFO #1)</span>
    </div>
)}
```

---

## 8. Truncated Bytes Suppression & High-Fidelity Markdown Preview

### 8.1 Root Cause of Truncated Byte Leaks

During background agent execution and transcript serialization, messages exceeding standard buffer limits append metadata markers:
- `<truncated 4181 bytes>`
- `[truncated 40 lines]`
- Literal escaped newlines `\r\n` or `\n` inside raw JSON strings.

When rendered directly in `RichMarkdownRenderer`, these raw markers appear as ugly artifacts in the prompt instruction text.

### 8.2 Sanitization & Formatting Pipeline

A dedicated prompt sanitizer utility cleanses the markdown payload before rendering:

```typescript
export function sanitizePromptPreview(rawText: string): string {
    if (!rawText) return '';

    let cleaned = rawText
        // 1. Remove raw truncated byte/line markers
        .replace(/<truncated\s+\d+\s+bytes>/gi, '')
        .replace(/\[truncated\s+\d+\s+bytes\]/gi, '')
        .replace(/<truncated\s+\d+\s+lines>/gi, '')
        .replace(/\[truncated\s+\d+\s+lines\]/gi, '')
        // 2. Normalize escaped newlines
        .replace(/\\r\\n/g, '\n')
        .replace(/\\n/g, '\n')
        .replace(/\r\n/g, '\n')
        .replace(/\r/g, '\n')
        // 3. Strip trailing prompt artifact delimiters
        .replace(/```text\s*$/g, '')
        // 4. Compact consecutive excessive blank lines (max 2)
        .replace(/\n{3,}/g, '\n\n');

    return cleaned.trim();
}
```

### 8.3 Paragraph Gap & Spacing Standard

In `RichMarkdownRenderer`:
- Paragraph blocks maintain clean, non-intrusive vertical spacing.
- Empty lines render explicit break tags: `<br className="my-1.5 block select-none" />`.
- Code blocks are styled with dark syntax highlighting, subtle borders (`border-slate-700/60`), and monospaced typography with copy shortcuts.

---

## 9. Enhanced Prompt Actions (Export, Copy, Copy with Images)

### 9.1 Copy Plain Text
- **Action**: Copies cleaned prompt text with all images stripped.
- **Implementation**: Strips markdown syntax `![alt](url)` and base64 payloads using `stripImagesFromPrompt`. Writes plain text to `navigator.clipboard.writeText`.

### 9.2 Copy With Images
- **Action**: Copies rich prompt content preserving all visual assets.
- **Dual-Mime Clipboard Implementation**:
  ```typescript
  export async function copyPromptWithImages(promptText: string): Promise<boolean> {
      try {
          const htmlContent = convertMarkdownToHtmlForClipboard(promptText);
          const blobText = new Blob([promptText], { type: 'text/plain' });
          const blobHtml = new Blob([htmlContent], { type: 'text/html' });
          const item = new ClipboardItem({
              'text/plain': blobText,
              'text/html': blobHtml,
          });
          await navigator.clipboard.write([item]);
          return true;
      } catch (err) {
          // Fallback to plain text verbatim if ClipboardItem HTML is blocked
          await navigator.clipboard.writeText(promptText);
          return true;
      }
  }
  ```

### 9.3 Export Full Conversation / Prompt
- **Export Prompt (Markdown)**: Generates a standalone `.md` file with YAML frontmatter:
  ```markdown
  ---
  sequence_id: "#P001"
  project: "gitmap"
  instance: "default"
  timestamp: "2026-10-08T07:42:47Z"
  steps: 117
  words: 23
  ---
  # Project Structure Memory Analysis

  [Prompt Body Text Here]
  ```
- **Export Conversation (JSON)**: Exports full JSON structure containing all conversational turns, tool calls, subagent spawns, and timestamps.

---

## 10. Antigravity Manager CLI (AGM) 30-40 Command Roadmap & Parity

To satisfy the user request for **30 to 40 improvements in the CLI level** following recent `gitmap` architectural upgrades, AGM CLI (`src-tauri/src/bin/agm.rs` and `src-tauri/src/modules/cli.rs`) is expanded across six cohesive command domains.

### 10.1 Command Domain Matrix (40 Planned Commands & Verbs)

#### Domain A: Prompt Tree & Hierarchy Inspection (Commands 1–7)
1. `agm tree`: Renders active hierarchical tree of running projects, conversations, and prompts.
2. `agm tree all`: Renders all projects (including idle and archived) across all instances.
3. `agm tree --json` (`agm tree -j`): Machine-readable JSON output of full 3-layer tree.
4. `agm tree --grouped`: Displays tree with repeated consecutive prompts grouped into `(xN)` composite nodes.
5. `agm tree --words <N>` (`agm tree -w <N>`): Sets word threshold for prompt previews (default: 200).
6. `agm tree inspect <CID>`: Deep-inspects a single conversation's hierarchy and prompt payload.
7. `agm tree watch`: Live-refreshing terminal UI tree that updates dynamically as prompts execute.

#### Domain B: Prompt Dispatch & Lifecycle Control (Commands 8–15)
8. `agm prompt show <CID>`: Outputs the full prompt instruction of conversation `<CID>`.
9. `agm prompt send <CID> "<text>"`: Immediately dispatches prompt to conversation `<CID>`.
10. `agm prompt send-now "<text>"`: Injects prompt into the current active instance workspace.
11. `agm prompt queue "<text>"`: Enqueues a prompt into the FIFO scheduler queue.
12. `agm prompt queue-list`: Displays all pending prompts in the FIFO scheduler queue.
13. `agm prompt queue-clear`: Flushes all queued prompts.
14. `agm prompt resend <CID>`: Resends the latest prompt with an optional confirmation suffix.
15. `agm prompt cancel <CID>`: Cancels an in-flight prompt execution or unexecuted draft.

#### Domain C: Prompt Import, Export & Backup (Commands 16–21)
16. `agm prompts-export [N] [-f <file.json>]`: Exports recent prompts into split JSON file.
17. `agm prompts-export-md [N] [-d <dir>]`: Exports prompts as individual formatted Markdown files.
18. `agm prompts-import -f <file.json>`: Imports and restores prompt backup batches into `repo_prompts.db`.
19. `agm prompts-backup`: Captures atomic snapshot of all running and recent prompts across instances.
20. `agm prompts-restore`: Restores running prompt state after an IDE or machine restart.
21. `agm prompts-diff <CID1> <CID2>`: Compares prompt text diff between two conversation versions.

#### Domain D: Dual-Source Classification & Transcript Analytics (Commands 22–27)
22. `agm transcript <CID>`: Dumps cleansed transcript steps for a given conversation.
23. `agm transcript filter <CID> --user`: Filters transcript to human `USER_PROMPT` entries only.
24. `agm transcript filter <CID> --subagents`: Filters transcript to `SUBAGENT_INSTRUCTION` turns.
25. `agm subagents list`: Lists all active subagents spawned across running conversations.
26. `agm drafts list`: Shows all unexecuted prompt drafts in `.antigravity_resume_task.json`.
27. `agm drafts prune`: Cleans up stale or orphan prompt drafts across workspaces.

#### Domain E: Multi-Instance & Quota Orchestration (Commands 28–34)
28. `agm instances`: Tabular overview of all instances, PIDs, active profiles, and quota status.
29. `agm quota`: Displays real-time 4H and weekly quota progress bars and reset countdowns.
30. `agm quota --json`: Outputs raw quota metrics for automation scripts.
31. `agm switch <account_email>`: Rotates active account to specified profile.
32. `agm switch-auto [--threshold <pct>]`: Enables auto-switcher daemon with custom quota gate.
33. `agm instance clone <src_id> <new_name>`: Clones instance with workspaceStorage and settings deep merge.
34. `agm instance focus <id>`: Brings instance IDE window to the foreground.

#### Domain F: Process Health, Cache & GitMap Delegation (Commands 35–40)
35. `agm health`: Verifies SQLite database connections, lock states, and instance heartbeats.
36. `agm cache-flush`: Flushes SQLite `prompt_tree_cache` and triggers full tree recomputation.
37. `agm dedup-prompts`: Scans `repo_prompts.db` and consolidates duplicate consecutive entries.
38. `agm gitmap sync`: Synchronizes prompt registry and cluster leases with local GitMap database.
39. `agm gitmap delegate <task_id>`: Delegates long-running execution to GitMap background cluster.
40. `agm rest-gateway start [--port 45123]`: Launches secure local REST API gateway.

---

## 11. Secure REST Endpoints Gateway Architecture

To facilitate programmatic integration by external CLI tools, local automation scripts, and GitMap cluster nodes:

### 11.1 Gateway Protocol & Binding
- **Host & Port**: Bound strictly to `127.0.0.1:45123` (no external interface binding).
- **Authentication**: HTTP Bearer Token generated on startup and stored securely in `~/.antigravity_tools/gateway_token`.
- **CORS Policy**: Restricted strictly to `localhost` and `tauri://localhost`.

### 11.2 Endpoint Specifications

| Endpoint | Method | Description | Request Payload | Response |
|----------|--------|-------------|-----------------|----------|
| `/api/v1/tree` | `GET` | Returns full 3-layer project/conversation/prompt tree. | Query: `?words=200&running=false&grouped=true` | `AgmProjectTreeNode[]` |
| `/api/v1/prompt/dispatch` | `POST` | Dispatches or enqueues a prompt to a target instance. | `{"instance_id": "default", "prompt": "...", "suffix": "None", "enqueue": false}` | `{"status": "dispatched", "task_id": "..."}` |
| `/api/v1/quota` | `GET` | Retrieves live quota metrics across all accounts. | None | `QuotaReport` |
| `/api/v1/instances` | `GET` | Lists all registered instances and process states. | None | `InstanceConfig[]` |

---

## 12. Quality Gates & Verification Checklist

Prior to approving implementation of these architecture specifications:

- [ ] **Dual-Source Classification**: Verify that `USER_PROMPT`, `SUBAGENT_INSTRUCTION`, and `UNEXECUTED_DRAFT` are cleanly separated in transcript parsing and displayed with distinct badges.
- [ ] **Dark-Glass Pill Capsule**: Verify that all 8 prompt inspector actions are contained within unified segmented pill capsules (`rounded-full`, shared border, dark-glass style per `AGENTS.md`).
- [ ] **Prompt Grouping**: Verify that consecutive identical prompts are grouped into a single expandable node with a multiplier badge (e.g. `(x4)`).
- [ ] **Status Indicators**: Verify that pulsing green/amber dots and badges appear on running and queued conversations.
- [ ] **Truncated Bytes Suppression**: Verify that raw byte count markers and escaped newlines are stripped from markdown previews.
- [ ] **Prompt Actions**: Verify that `Copy Text`, `Copy With Images`, and `Export JSON/MD` operate correctly without errors.
- [ ] **AGM CLI 40-Command Roadmap**: Verify that all 40 commands are cataloged, documented, and architecturally aligned with GitMap verbs.
- [ ] **Positive Boolean Discipline**: Verify 100% adherence to positive booleans (`hasImages`, `isRunning`, `isQueued`, `isGrouped`, `isUserPrompt`).
- [ ] **Relative Git Paths**: Verify all file references are relative Git paths.
