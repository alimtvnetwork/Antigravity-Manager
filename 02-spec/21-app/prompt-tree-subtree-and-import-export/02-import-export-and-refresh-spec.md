# Specification: Section-Level Import / Export (JSON, SQLite DB, YAML) with Embedded Base64 Images & Refresh Fix

> **Spec ID:** `02-spec/21-app/prompt-tree-subtree-and-import-export/02-import-export-and-refresh-spec.md`  
> **Parent Task:** `prompt-tree-subtree-and-import-export`  
> **Associated Subtask:** `.ai-memory/plans/subtasks/prompt-tree-subtree-and-import-export/02-import-export-and-refresh.md`  
> **Target Subtask Code:** `TASK-02`  
> **Assigned Worker:** `Worker 02`  
> **Status:** APPROVED & SPECIFIED  
> **Target Modules:**  
> - `src-tauri/src/modules/repo_db/sqlite_export.rs`  
> - `src-tauri/src/modules/repo_db/export.rs`  
> - `src-tauri/src/modules/repo_db/import.rs`  
> - `src-tauri/src/modules/repo_db/tree_phases.rs`  
> - `src-tauri/src/modules/repo_db/tree_collect.rs`  
> - `src-tauri/src/modules/repo_db/tree.rs`  
> - `src-tauri/src/commands/project_cmds.rs`  
> - `src/components/instances/PromptTreeView/usePromptTreeCoreActions.ts`  
> - `src/components/instances/PromptTreeView/usePromptTreeData.ts`  
> - `src/components/instances/PromptTreeView/PromptTreeExportModal.tsx`  
> - `src/components/instances/PromptTreeView/PromptTreeImportModal.tsx`  
> - `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`  
> - `src/components/instances/PromptTreeView/ProjectContextMenu.tsx`  
> - `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`  
> - `src/components/instances/PromptTreeView/PromptNodeContextMenu.tsx`  

---

## User Request (Verbatim)

The following is the verbatim user instruction guiding this specification:

```text
# High Priority Instruction

One more very serious issue in your prompt tree view that usually how it should be is that you have the project name. Inside this, you should have the conversation name, and inside this it would have the prompts sending list. I think this is where you are messing up. And as you can see in my screenshot, I have two different screenshot I have provided. One is your prompt view and another is the actual IDE view. In the IDE it is continuing previous task. That is fine. The name matches. But you don't have a subtree of the prompts that I have sent. That is the problem. And it only shows the last prompt that has been sent, but not the running symbol. It does not have a running symbol which prompts are running at the moment, which are enqueued. You can see there a lot of prompts are in queue which is not showing up here. That is one of the big issues, I think you need to find the root cause by running the code into the machine and make sure that you get the result. At the end it's a very simple task. First write the root cause, find the root cause, and then try to fix it and make sure you write the root cause analysis so that this mistake never happens. And make sure that every one of the prompt tree we could right click and export the full tree item for that section. Import export as a... There should be import export button. If we go into that import export, that would have sub drop down. It would have export as a JSON, export as a SQLite DB, YAML. These are the formats and also same for the import as well. And also make sure that when we export import, we should have the images embedded. So currently you are not getting the... Apparently, the refresh button also does not work. It feels a little bit buggy. And the next thing, you can also check into the logs. I think that would tell you more information. Each prompts are coming as a single item into the tree. I think that you need to fix. And inside the text when you display, the display text needs to be poppins, and the headers needs to be Ubuntu, which is correct at the moment. It looks nice, but also add a copy button besides the header where we can copy the whole thing rather than below. And images does not show up. That is another problem. You cannot detect the image. As you can see in my screenshot, a lot of the prompts are running. Those have images, and you cannot get the images. So images are basically referred to a file when it goes to Antigravity. I hope you know as Antigravity ``Id``, you should know it. Find that and make sure that you find a way to copy the image along with the text, so that when we import as well, either it's a JSON or something, it's a single file, but images should be there somehow, base64, and that would be sent in a way so that the ``Id`` understands it very well. Can you please find the solution for this? We are discussing this for several times. Also, can you please tell me what is C2018 or C17 when I do see in the prompt tree? What does this mean? Is it the number that it is going up, or communication or conversation? Something like this probably you're trying to say here. So we should be able to change the conversation name from this prompt tree view, which I don't see the option. If we right click on the conversation, we should be able to see to rename or double click or F2 should be able to rename the conversation. And also we can right click on a tree and say resend all, let's say, prompts. Or we could do resend last five prompts or resend last 10 prompts. So there should be numbers like resend how many times we want to do, or how many prompts that we wanted to resend again. And also we could have a option like verify last 10 prompts. That means it would write us very much more things like have you completed this? Can you please verify each one of the task has been completed, released, and things like that. So it would add this and then put that prompt and resend back to the IDE and put it to the enqueue. Do you understand this? And we can have a custom number where we could do anything or do any number of the send for that conversation. And also we could create a new conversation from this by cloning this and compacting the memory that it has. And probably we could use the AI from here as well. We could connect with the Antigravity with new project to summarize a little bit, compacting it and creating a new conversation from this. I think you should have all these options. You should include or add these options. And also, we should have a text box inside, like how many we want to send. We could provide a number, let's say 10, 15 that would be resent or probably we can resend all to test out if those are verified and done. Things like that. So these are very critical options. I think you should be able to work on it. So make sure you write it first, and then you start implementing. And make sure the markdown preview is far better, far more enticing. Make sure you work on it. If you have any more question and confusion, we can discuss this later on. Is it clear?

# Actionable Items Must Follow Non-Negotiable

0. Gitmap do `gitmap pull <repo path> or run into the repo path` Make sure we have the latest and conflict resolve before starting.
1. Write spec under 02-spec/21-app/<slug>/ and enqueue plan task in .ai-memory/plans/<slug>.md (subtasks in .ai-memory/plans/subtasks/<slug>/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Find the root cause of the missing prompt subtree and running symbol in the prompt tree view.
5. Implement a right-click export/import feature with JSON, SQLite DB, and YAML formats, ensuring images are embedded.
6. Fix the refresh button functionality and check logs for additional information.
7. Ensure display text uses Poppins font and headers use Ubuntu font, adding a copy button beside the header.
8. Detect and handle images correctly, ensuring they are included in exports as base64.
9. Clarify the meaning of C2018 or C17 in the prompt tree.
10. Implement conversation renaming and prompt resend options in the prompt tree view.
11. Add a feature to verify the last 10 prompts and enqueue verification tasks.
12. Enable conversation cloning and memory compaction, integrating with Antigravity for summarization.
13. Improve markdown preview for better visual appeal.
```

---

## 1. Executive Summary & Scope Breakdown

The **Prompt Tree View** (`src/components/instances/PromptTreeView/`) in Antigravity Manager (`AGM`) provides visual transparency, lifecycle inspection, and task orchestration across active and background instances. However, two critical operational deficits severely undermine usability:
1. **Lack of Section-Level Portability & Multimodal Image Loss**: While full SQLite database export existed at a basic level, users had no mechanism to export or import scoped subsets of the tree (a single project, a single conversation, or a prompt subtree). Furthermore, existing export routines discarded image assets, causing broken links (`file:///.../media_*.png`) when transferred between different machines or workspaces.
2. **Refresh Button Instability & Selection Whiplash**: Clicking the Refresh button frequently failed to show recent prompts, extinguished active running symbols, or forcefully snapped user focus away from their currently inspected conversation back to the top of the tree.

This specification addresses **Actionable Items 5, 6, and 8** by architecting:
- **Section-Level Import / Export Engine**: Supporting 4 hierarchical scopes across 3 interoperable formats (**JSON**, **SQLite DB**, and **YAML**).
- **Self-Contained Base64 Image Embedding Invariant**: Guaranteeing that all local screenshots, diagrams, and attachments are embedded inline as Base64 data URIs on export and materialized to disk on import, ensuring zero broken links across machines.
- **Root Cause Analysis & Comprehensive Remediation for the Refresh Button**: Eliminating selection whiplash, stale workspace cache, SQLite WAL frame bypass, and the premature 45-second in-flight status cutoff.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                          TASK-02 Architectural Blueprint                               │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│   ┌────────────────────────────────┐            ┌──────────────────────────────────┐   │
│   │   Scoped Export Triggers       │            │   Unified Formats                │   │
│   │   • Full Project Tree          │            │   • JSON  (.json envelope v2.0)  │   │
│   │   • Single Project Node        │───Export──>│   • SQLite Database (.db)        │   │
│   │   • Single Conversation Node   │            │   • YAML  (.yaml / .yml)         │   │
│   │   • Prompt Subtree Node        │            └──────────────────────────────────┘   │
│   └────────────────────────────────┘                             │                     │
│                   │                                              ▼                     │
│                   │                              ┌──────────────────────────────────┐  │
│                   │                              │   Base64 Image Invariant         │  │
│                   │                              │   • Local path -> Base64 URI     │  │
│                   │                              │   • Embedded map / table         │  │
│                   │                              │   • Materialize to disk on import│  │
│                   │                              └──────────────────────────────────┘  │
│                   ▼                                              │                     │
│   ┌────────────────────────────────┐                             ▼                     │
│   │   Refresh Engine 4-Point RCA   │            ┌──────────────────────────────────┐   │
│   │   • Root Cause A: Whiplash Fix │            │   Target UI Modals & Actions     │   │
│   │   • Root Cause B: Cache Scan   │            │   • PromptTreeExportModal.tsx    │   │
│   │   • Root Cause C: WAL Bypass   │            │   • PromptTreeImportModal.tsx    │   │
│   │   • Root Cause D: 600s Cutoff  │            │   • Node Context Menus           │   │
│   └────────────────────────────────┘            └──────────────────────────────────┘   │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Root Cause Analysis (RCA) - Refresh Button & Tree Synchronization

Users reported that *"the refresh button also does not work. It feels a little bit buggy"* and running symbols mysteriously vanished. Comprehensive code analysis across `src/components/instances/PromptTreeView/` and `src-tauri/src/modules/repo_db/` isolated four conjoining root causes:

### 2.1 Root Cause A: Selection Whiplash in `PromptTreeViewModal.tsx`
- **Location:** `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx:499, 509` and `src/components/instances/PromptTreeView/usePromptTreeData.ts:359-363`.
- **Mechanism:** In `PromptTreeViewModal.tsx`, the top-level refresh handler was wired as:
  ```typescript
  onRefreshAll={() => loadTree(true, true)}
  ```
  The signature of `loadTree` in `usePromptTreeData.ts` is:
  ```typescript
  loadTree: (isInitialLoad?: boolean, isForce?: boolean) => Promise<void>;
  ```
  When `isInitialLoad` is `true`, `usePromptTreeData.ts` unconditionally executes `performAutoSelection()`:
  ```typescript
  if (isInitialLoad) {
      const targetArchived = overrideArchivedIds || archivedProjectIdsRef.current;
      const targetPinned = overridePinnedIds || pinnedProjectIdsRef.current;
      performAutoSelection(finalData, targetArchived, targetPinned);
  }
  ```
  `performAutoSelection()` algorithmically seeks the newest running conversation or the first project in the tree, completely overriding the user's current selection. If a user was inspecting Project B, Conversation 3 and clicked Refresh, their view was violently jerked back to Project A.
- **Remediation:** 
  1. All user-triggered and background refresh calls (`onRefreshAll`, `handleRefreshSingleProject`, toolbar reload buttons) MUST pass `loadTree(false, true)` (`isInitialLoad: false, isForce: true`).
  2. In `usePromptTreeData.ts`, the `else` branch preserves the existing selection by ID matching:
     ```typescript
     const currentConv = selectedConversationRef.current;
     const currentProj = selectedProjectRef.current;
     if (currentConv && currentProj) {
         const updatedProj = finalData.find((p) => p.project_id === currentProj.project_id);
         if (updatedProj) {
             setSelectedProject(updatedProj);
             const updatedConv = updatedProj.conversations.find((c) => c.conversation_id === currentConv.conversation_id);
             if (updatedConv) setSelectedConversation(updatedConv);
         }
     }
     ```

### 2.2 Root Cause B: Stale Workspace Cache in `compute_project_conversation_tree`
- **Location:** `src-tauri/src/modules/repo_db/tree.rs:137` and `src-tauri/src/modules/repo_db/tree_phases.rs:31-60`.
- **Mechanism:** `compute_project_conversation_tree` assembled tree nodes by calling `query_tree_projects(target)`, which executed:
  ```rust
  let all_projects: Vec<RunningProject> = list_running_projects().unwrap_or_default();
  ```
  `list_running_projects()` queries the local SQLite table `running_projects` in `repo_prompts.db`. However, `running_projects` is only updated when `detect_running_projects(instance_id)` actively executes disk scans across `~/.antigravity/workspaceStorage` and inspects OS process tables. When users launched new IDE instances or opened new workspace folders, clicking Refresh queried the stale table without scanning disk, causing new projects to remain invisible.
- **Remediation:** 
  Prior to querying `running_projects`, `compute_project_conversation_tree` (or `get_project_conversation_tree_cached`) MUST proactively invoke `crate::modules::repo_db::detect_running_projects(target_instance.unwrap_or("default"))`. This refreshes the `running_projects` table before project node generation.

### 2.3 Root Cause C: SQLite WAL Frame Bypass via `immutable=1`
- **Location:** `src-tauri/src/modules/repo_db/tree_collect.rs:83-93`, `text_utils.rs:206`, `project_queries.rs:111`.
- **Mechanism:** When inspecting `conversation_summaries.db`, code fell back to opening SQLite in immutable mode:
  ```rust
  let uri = format!("file:{}?immutable=1", summaries_db.to_string_lossy().replace('\\', "/"));
  Connection::open_with_flags(&uri, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI)
  ```
  Per SQLite documentation:
  > *"If immutable=1 is specified, SQLite assumes that the database file will never change... In immutable mode, SQLite does NOT check for or read the -wal file."*
  Because the Antigravity IDE runs in WAL (Write-Ahead Logging) mode, recent conversation turns, prompts, and title updates reside inside `conversation_summaries.db-wal`. Opening with `immutable=1` completely bypassed the WAL frames! The Refresh operation read stale, un-checkpointed database bytes from disk, creating the illusion that refresh was "broken".
- **Remediation:** 
  1. Completely ban and remove `immutable=1` fallback across all `repo_db` connection callsites.
  2. Always open connections with:
     ```rust
     let conn = Connection::open_with_flags(
         &summaries_db,
         rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
     )?;
     conn.busy_timeout(std::time::Duration::from_millis(5000))?;
     ```
  3. Configuring `busy_timeout` allows SQLite to wait up to 5000ms for active IDE write transactions to release locks, reliably reading WAL frames without deadlocks or stale data.

### 2.4 Root Cause D: 45-Second Premature In-Flight Transition
- **Location:** `src-tauri/src/modules/repo_db/tree_phases.rs:15-29`.
- **Mechanism:** In `tree_phases.rs`, `transition_stale_inflight_prompts` executed:
  ```rust
  conn.execute(
      "UPDATE active_prompts
       SET status = 'completed', updated_at = ?1
       WHERE status IN ('running', 'in_flight', 'dispatched')
         AND (?1 - updated_at > 45)",
      rusqlite::params![now],
  );
  ```
  Real-world AI tasks (deep reasoning, multi-file code generation, test execution, browser tool calls) routinely exceed 45 seconds. After exactly 45 seconds of running, the backend sweep forcefully marked the prompt as `completed`. When the user clicked Refresh, the running symbol was gone even though the task was still active in the IDE.
- **Remediation:** 
  Raise the cutoff threshold from 45 seconds to **600 seconds (10 minutes)** in `tree_phases.rs:23`. Tasks will only transition to stale after 10 minutes of complete silence, preserving the spinning/pulsing running glyph throughout lengthy generation cycles.

---

## 3. Scoped Export Architecture & Envelope Specification

### 3.1 Scoped Export Hierarchy
Exporting must not be limited to an "all-or-nothing" database snapshot. The export engine must support four granular scopes:

| Scope | Identifier | Description | Context Menu Callout |
|---|---|---|---|
| **Full Project Tree** | `full_tree` | All projects, conversations, prompt turns, sequences, and embedded assets. | Top Toolbar Flyout & Modal |
| **Single Project** | `project` | One project workspace, including all its conversations, prompts, and project sequence. | Right-click Project Node |
| **Single Conversation** | `conversation` | One conversation thread, its sequence metadata, and all constituent prompt turns. | Right-click Conversation Node |
| **Prompt Subtree** | `prompt_subtree` | Selected prompt turns (or a subagent execution subtree) within a conversation. | Right-click Prompt Turn Node |

### 3.2 Two-Tier JSON Envelope Schema (v2.0)
For `.json` exports, the payload follows the canonical two-tier envelope format with embedded metadata, scope declaration, and asset map:

```json
{
  "$schema": "https://agm.internal/schemas/v2/prompt-tree-export.json",
  "version": "2.0",
  "kind": "agm/prompt-tree-section-export",
  "exported_at": "2026-10-17T18:30:00Z",
  "agm_version": "4.221.0",
  "scope": {
    "target_type": "project",
    "project_id": "proj-uuid-1234",
    "project_name": "Antigravity-Manager",
    "conversation_id": null,
    "prompt_ids": null
  },
  "stats": {
    "project_count": 1,
    "conversation_count": 4,
    "prompt_count": 18,
    "image_count": 3,
    "total_image_bytes": 485120
  },
  "data": {
    "projects": [
      {
        "id": "proj-uuid-1234",
        "instance_id": "default",
        "repo_name": "Antigravity-Manager",
        "repo_path": "d:/work/Antigravity-Manager",
        "workspace_storage_path": "...",
        "is_running": true,
        "last_detected_at": 1729150000,
        "conversations": [
          {
            "conversation_id": "conv-uuid-5678",
            "seq_id": 12,
            "title": "Fix prompt tree export",
            "status": "running",
            "is_running": true,
            "last_modified": "2026-10-17T18:25:00Z",
            "prompts": [
              {
                "id": "prompt-uuid-9999",
                "project_id": "proj-uuid-1234",
                "instance_id": "default",
                "repo_path": "d:/work/Antigravity-Manager",
                "prompt_content": "Please analyze this UI layout: ![UI Design](agm-asset://img_9a1f2b3c)",
                "model": "gemini-2.5-pro",
                "session_id": "conv-uuid-5678",
                "status": "running",
                "created_at": 1729151000,
                "updated_at": 1729151010,
                "image_payload": "agm-asset://img_9a1f2b3c",
                "media_paths": ["agm-asset://img_9a1f2b3c"]
              }
            ]
          }
        ]
      }
    ],
    "sequences": {
      "projects": [
        {
          "project_key": "d:/work/Antigravity-Manager",
          "seq_id": 1,
          "project_id": "proj-uuid-1234",
          "repo_name": "Antigravity-Manager",
          "repo_path": "d:/work/Antigravity-Manager",
          "instance_id": "default",
          "updated_at": 1729150000
        }
      ],
      "conversations": [
        {
          "conversation_id": "conv-uuid-5678",
          "seq_id": 12,
          "project_key": "d:/work/Antigravity-Manager",
          "title": "Fix prompt tree export",
          "instance_id": "default",
          "updated_at": 1729150000
        }
      ]
    }
  },
  "embedded_images": {
    "img_9a1f2b3c": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA..."
  }
}
```

### 3.3 SQLite Portable Database Schema (`.db`)
Portable SQLite exports (`src-tauri/src/modules/repo_db/sqlite_export.rs`) must contain all required prompt-tree tables, strict `DELETE` journal mode (no `-wal` or `-shm` sidecars), and an added `embedded_images` table:

```sql
-- Metadata & Scope table
CREATE TABLE IF NOT EXISTS meta (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schema_version TEXT NOT NULL,
    kind TEXT NOT NULL,
    target_type TEXT NOT NULL DEFAULT 'full_tree',
    target_id TEXT,
    created_at TEXT NOT NULL,
    agm_version TEXT NOT NULL,
    notes TEXT
);

-- Core Tree Structure tables
CREATE TABLE IF NOT EXISTS running_projects (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    repo_path TEXT NOT NULL,
    workspace_storage_path TEXT,
    is_running INTEGER NOT NULL DEFAULT 1,
    last_detected_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS active_prompts (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    repo_path TEXT NOT NULL,
    prompt_content TEXT NOT NULL,
    model TEXT,
    session_id TEXT,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    image_payload TEXT
);

CREATE TABLE IF NOT EXISTS agm_project_sequences (
    project_key TEXT PRIMARY KEY,
    seq_id INTEGER NOT NULL UNIQUE,
    project_id TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    repo_path TEXT NOT NULL,
    instance_id TEXT NOT NULL DEFAULT 'default',
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS agm_conversation_sequences (
    conversation_id TEXT PRIMARY KEY,
    seq_id INTEGER NOT NULL UNIQUE,
    project_key TEXT NOT NULL,
    title TEXT NOT NULL,
    instance_id TEXT NOT NULL DEFAULT 'default',
    updated_at INTEGER NOT NULL
);

-- Multimodal Asset Invariant Table
CREATE TABLE IF NOT EXISTS embedded_images (
    image_key TEXT PRIMARY KEY,
    mime_type TEXT NOT NULL,
    base64_data TEXT NOT NULL,
    byte_size INTEGER NOT NULL,
    original_rel_path TEXT,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_export_prompts_instance_status
    ON active_prompts(instance_id, status);
```

### 3.4 YAML Representation (`.yaml` / `.yml`)
YAML export mirrors the structure of the two-tier JSON envelope, leveraging `yaml-rt` or structured serde serialization with clean multiline formatting:

```yaml
version: "2.0"
kind: "agm/prompt-tree-section-export"
exported_at: "2026-10-17T18:30:00Z"
agm_version: "4.221.0"
scope:
  target_type: "conversation"
  conversation_id: "conv-uuid-5678"
stats:
  project_count: 1
  conversation_count: 1
  prompt_count: 5
  image_count: 1
  total_image_bytes: 120400
data:
  projects:
    - id: "proj-uuid-1234"
      repo_name: "Antigravity-Manager"
      repo_path: "d:/work/Antigravity-Manager"
      conversations:
        - conversation_id: "conv-uuid-5678"
          title: "Fix prompt tree export"
          prompts:
            - id: "prompt-uuid-9999"
              prompt_content: "Inspect architecture: ![diag](agm-asset://img_diag1)"
              status: "running"
embedded_images:
  img_diag1: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA..."
```

---

## 4. Base64 Image Embedding & Materialization Invariant

Transferring exported prompt trees between different developer machines, operating systems, or cloud environments routinely breaks image references because file paths (e.g. `C:/Users/Administrator/.gemini/antigravity/brain/...` vs `/home/user/...`) are host-specific.

```
Export Process:
┌─────────────────────┐      ┌─────────────────────────┐      ┌─────────────────────────┐
│ Detected Image Path │ ===> │ Read Bytes & Detect MIME│ ===> │ Base64 URI & Hash Key   │
│ • brain/<cid>/media │      │ • image/png, image/webp │      │ • agm-asset://<hash>    │
│ • attachments/*     │      │ • Read from disk        │      │ • embedded_images Map   │
└─────────────────────┘      └─────────────────────────┘      └─────────────────────────┘

Import Process:
┌─────────────────────────┐      ┌─────────────────────────┐      ┌─────────────────────────┐
│ Read embedded_images    │ ===> │ Decode Base64 Payload   │ ===> │ Materialize onto Disk   │
│ • Iterate keys & URIs   │      │ • Validate MIME & bytes │      │ • ./.antigravity/attach │
│ • Extract image bytes   │      │ • Prevent path traversal│      │ • Rewrite prompt paths  │
└─────────────────────────┘      └─────────────────────────┘      └─────────────────────────┘
```

### 4.1 Asset Discovery & Extraction Rules (Export Phase)
1. **Multi-Source Pattern Matching:**
   For every prompt turn included in the export scope, scan `prompt_content` and `image_payload` using pattern recognition:
   - Markdown image references: `!\[(.*?)\]\((https?:\/\/.*?|file:\/\/.*?|data:image\/.*?;base64,.*?|[^)]+\.(?:png|jpg|jpeg|webp|gif|svg|bmp))\)`
   - HTML `<img>` tags: `<img[^>]+src=["']([^"']+)["']`
   - Gemini / Antigravity media vault paths:
     * `.*[\\/]brain[\\/][^\\/]+[\\/]\.user_uploaded[\\/]media_.*`
     * `.*[\\/]\.antigravity[\\/]attachments[\\/].*`
   - Data URIs: `data:image/(png|jpeg|webp|gif|svg\+xml);base64,[A-Za-z0-9+/=]+`
2. **Disk Reading & Encoding:**
   - If the reference is a local file path (`file://...` or filesystem path), resolve it against the workspace path, instance data directory, or absolute location.
   - Read file bytes. If the file is missing or unreadable, record a warning via `crate::modules::logger::log_warn` and preserve the raw path without crashing the export.
   - Determine MIME type via file extension or image magic header (first 8 bytes).
   - Encode bytes to Base64 data URI format: `data:<mime>;base64,<base64-payload>`.
3. **Asset Key Hashing & Content Normalization:**
   - Compute a deterministic hash key: `img_<hex-hash>` (e.g. `img_8f3c1a2d`).
   - Store the data URI in the `embedded_images` map / table keyed by `img_<hex-hash>`.
   - In the exported prompt text, replace the host-specific absolute path with the portable URI: `agm-asset://img_<hex-hash>`.

### 4.2 Materialization & Disk Anchoring Rules (Import Phase)
1. **Target Attachment Directory:**
   Upon import, locate the destination instance's attachment storage directory:
   `<instance_data_dir>/.antigravity/attachments/` or `<repo_path>/.antigravity/attachments/`.
   If the directory does not exist, create it recursively (`fs::create_dir_all`).
2. **Payload Decoding & Path Sanitization:**
   - For each entry `(image_key, base64_uri)` in `embedded_images`:
     - Strip the `data:<mime>;base64,` prefix.
     - Decode Base64 string to raw binary bytes.
     - Map MIME type to valid extension (`.png`, `.jpg`, `.webp`, `.svg`).
     - Sanitize filename: enforce alphanumeric characters and underscores only (`image_key.ext`), completely preventing path traversal (`../`) attacks.
     - Write decoded bytes to `<target_dir>/<image_key>.<ext>`.
3. **Prompt Reference Rewriting:**
   - Search imported prompts for `agm-asset://<image_key>` references.
   - Rewrite them to the local absolute or workspace-relative path of the materialized file.
   - If the user imports on another machine, the prompt immediately references the local materialized asset with zero broken links!

---

## 5. UI / UX Design & Interaction Flow

In accordance with `AGENTS.md` guidelines, all controls must reuse existing segmented pill capsule conventions (`rounded-full`, shared border, subtle divider lines, dark-glass styling) placed strictly within their most relevant sections.

### 5.1 Right-Click Context Menus on Tree Nodes
Right-click context menus (`ProjectContextMenu.tsx`, `ConversationContextMenu.tsx`, `PromptNodeContextMenu.tsx`) provide contextual export options:

```
┌───────────────────────────────────────┐
│ 📁 Project: Antigravity-Manager       │
├───────────────────────────────────────┤
│ ↻  Refresh Project                    │
│ 📌 Pin / Unpin Project                │
│ 📦 Archive Project                    │
├───────────────────────────────────────┤
│ 📤 Export Section                     │
│    ├─ Export as JSON (.json)...       │
│    ├─ Export as SQLite (.db)...       │
│    └─ Export as YAML (.yaml)...       │
└───────────────────────────────────────┘
```

1. **`ProjectContextMenu.tsx` Options:**
   - `Export Project as JSON...` -> Opens `PromptTreeExportModal` pre-scoped to `target_type: "project"`, `project_id`.
   - `Export Project as SQLite .db...` -> Direct save dialog or modal pre-configured for SQLite.
   - `Export Project as YAML...` -> Modal pre-configured for YAML.
2. **`ConversationContextMenu.tsx` Options:**
   - Added items beneath `Rename Conversation` and `Inspect Transcript`:
   - `Export Conversation as JSON...` -> Pre-scoped to `target_type: "conversation"`.
   - `Export Conversation as SQLite .db...`
   - `Export Conversation as YAML...`
3. **`PromptNodeContextMenu.tsx` Options:**
   - Context menu on individual prompt turns in the subtree:
   - `Export Prompt Turn as JSON...` -> Pre-scoped to `target_type: "prompt_subtree"`.

### 5.2 Top-Level Toolbar Import / Export Flyout & Modals
In `ImportExportMenu.tsx` (mounted inside `ModalHeader.tsx`), clicking `Import / Export` reveals segmented actions:
- **Export Section...** -> Opens `PromptTreeExportModal.tsx`.
- **Import Section...** -> Opens `PromptTreeImportModal.tsx`.
- Quick 1-click actions: `Export Tree as SQLite`, `Export Tree as JSON`, `Import from SQLite`, `Import from JSON`.

#### `PromptTreeExportModal.tsx` Design:
- **Scope Selector:** Segmented radio pills or contextual dropdown:
  * `Full Tree (All Projects)`
  * `Current Project (<Project Name>)`
  * `Current Conversation (<Conversation Title>)`
  * `Selected Prompt (<Prompt Snippet>)`
- **Format Cards (3 Options):**
  * `JSON` (`.json`) - Portable envelope format, web and CLI compatible.
  * `SQLite DB` (`.db`) - Standalone single-file database snapshot with indexes.
  * `YAML` (`.yaml`) - Human-readable structured document.
- **Image Embedding Toggle:**
  * Pill switch: `Embed local images as Base64 data URIs` (Enabled by default).
  * Subtext: `Ensures screenshots and diagrams render correctly across machines without broken links.`
- **Real-Time Live Stats Card:**
  * Displays: `Projects: N | Conversations: N | Prompts: N | Images: N (~X.X MB)`.
- **Primary CTA:**
  * `Export Section` button (with downloading spinner and completion notification).

#### `PromptTreeImportModal.tsx` Design:
- **Drag-and-Drop Zone:**
  * Accepts `.json`, `.db`, `.yaml`, `.yml` files.
  * File picker button: `Browse Export File...`.
- **Validation & Source Telemetry Banner:**
  * Appears immediately upon file selection:
  * Shows detected format, version, scope, project count, prompt count, and embedded image count.
  * Integrity badge: Green check `Verified AGM Export Envelope (v2.0)`.
- **Conflict Resolution Mode:**
  * Segmented pill toggle:
    - `Merge & Upsert (FIFO)` (Default): Preserves existing records, upserts incoming rows enforcing `ORDER BY created_at ASC, id ASC`.
    - `Clean Scope Replace`: Removes existing records matching the imported scope before insertion.
- **Image Materialization Settings:**
  * Checkbox: `Materialize embedded Base64 images to instance attachments folder` (Checked by default).
  * Target path callout: `<instance_data_dir>/.antigravity/attachments/`.
- **Primary CTA:**
  * `Import & Merge Prompts` button.
  * On completion, automatically reloads the tree via non-whiplash `loadTree(false, true)` and highlights imported projects.

---

## 6. Tauri IPC & Service Interface Contracts

### 6.1 Rust Command Contracts (`src-tauri/src/commands/project_cmds.rs`)

```rust
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportScope {
    pub target_type: String, // "full_tree" | "project" | "conversation" | "prompt_subtree"
    pub project_id: Option<String>,
    pub conversation_id: Option<String>,
    pub prompt_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub embed_images: bool,
    pub format: String, // "json" | "sqlite" | "yaml"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStats {
    pub path: String,
    pub project_count: usize,
    pub conversation_count: usize,
    pub prompt_count: usize,
    pub image_count: usize,
    pub total_image_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub conflict_mode: String, // "merge" | "replace"
    pub materialize_images: bool,
    pub target_instance_id: Option<String>,
}

#[tauri::command]
pub async fn export_prompt_tree_section(
    scope: ExportScope,
    options: ExportOptions,
    destination_path: Option<String>,
) -> Result<ExportStats, AppError>;

#[tauri::command]
pub async fn export_prompt_tree_section_bytes(
    scope: ExportScope,
    options: ExportOptions,
) -> Result<(String, String, ExportStats), AppError>; // (data_b64, filename, stats)

#[tauri::command]
pub async fn validate_prompt_tree_import(
    source_path: Option<String>,
    data_b64: Option<String>,
) -> Result<ExportStats, AppError>;

#[tauri::command]
pub async fn import_prompt_tree_section(
    source_path: Option<String>,
    data_b64: Option<String>,
    options: ImportOptions,
) -> Result<usize, AppError>; // Returns count of imported prompts
```

### 6.2 Frontend Interface Contracts (`usePromptTreeCoreActions.ts`)

```typescript
export interface ExportTreeSectionParams {
    scope: {
        target_type: 'full_tree' | 'project' | 'conversation' | 'prompt_subtree';
        project_id?: string;
        conversation_id?: string;
        prompt_ids?: string[];
    };
    format: 'json' | 'sqlite' | 'yaml';
    embed_images: boolean;
}

export interface ImportTreeSectionParams {
    filePath?: string;
    dataB64?: string;
    conflictMode: 'merge' | 'replace';
    materializeImages: boolean;
    targetInstanceId?: string;
}

export interface PromptTreeCoreActions {
    // Existing actions...
    handleOpenExportModal: (preselectedScope?: ExportTreeSectionParams['scope']) => void;
    handleOpenImportModal: () => void;
    executeSectionExport: (params: ExportTreeSectionParams) => Promise<ExportStats>;
    executeSectionImport: (params: ImportTreeSectionParams) => Promise<number>;
    isExporting: boolean;
    isImporting: boolean;
}
```

---

## 7. Concrete Acceptance Criteria & Verification Matrix

| ID | Criterion | Verification Method | Expected Outcome |
|---|---|---|---|
| **AC-01** | **Section Scoped Export (JSON)** | Right-click a project node, select "Export Section as JSON", complete export. | Generates valid JSON envelope v2.0 containing only that project, its conversations, and prompts. |
| **AC-02** | **Section Scoped Export (SQLite)** | Right-click a conversation node, export as SQLite `.db`. | Creates single-file `.db` containing only records belonging to that conversation, with `embedded_images` table. |
| **AC-03** | **Section Scoped Export (YAML)** | Export conversation as YAML from `PromptTreeExportModal`. | Produces valid YAML document parseable by `yaml-rt`, with proper indentation and embedded image keys. |
| **AC-04** | **Base64 Image Extraction** | Export a prompt containing local screenshot `![img](file:///.../media_1.png)`. | Output JSON/YAML/SQLite contains `embedded_images` entry with Base64 data URI; prompt content replaces local path with `agm-asset://...`. |
| **AC-05** | **Base64 Image Materialization** | Import an export bundle with embedded images into a clean instance. | Decodes Base64 payloads and writes them to `<target_dir>/.antigravity/attachments/img_*.png`; prompt displays materialized image without broken links. |
| **AC-06** | **Refresh Selection Preservation** | Select Conversation 3 in Project B; click Refresh button. | `loadTree(false, true)` executes; Project B and Conversation 3 remain selected and active in the detail pane. |
| **AC-07** | **WAL Frame Synchronization** | Start prompt in IDE; verify `.db-wal` has frames; click Refresh in AGM. | AGM opens connection with read-only + `busy_timeout(5000)` without `immutable=1`; latest prompt turn is displayed. |
| **AC-08** | **600s In-Flight Retention** | Simulate prompt running for 90 seconds; click Refresh in AGM. | Status remains `'running'`; running spinning glyph is visible (cutoff is 600s, not 45s). |
| **AC-09** | **FIFO Merge on Import** | Import SQLite or JSON bundle into existing database with overlapping timestamps. | Prompts are merged using `ORDER BY created_at ASC, id ASC`; strict FIFO order is preserved. |
| **AC-10** | **Error Handling & Visibility** | Attempt to import a corrupted non-database file or invalid JSON. | Throws structured `AppError::Config`; caught and recorded via `useErrorStore.getState().trackWarning`; user-friendly error banner displayed in modal. |
