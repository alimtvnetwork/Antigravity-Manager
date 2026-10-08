# Subtask 02 Engineering Spec: Running Detection Hardening & Prompt UI Tag Compaction

**Subtask:** `02-running-detection-hardening-and-tag-compaction.md`  
**Parent Task:** `147-smart-instance-process-cache-and-prompt-dispatch`  
**File Location:** `.ai-memory/plans/subtasks/147-smart-instance-process-cache-and-prompt-dispatch/02-running-detection-hardening-and-tag-compaction.md`  
**Target Modules:**  
- `src-tauri/src/modules/repo_db.rs` (Running detection hardening, transcript terminal state check, timestamp narrowing, stale project purging, `only_running` filter)  
- `src/components/instances/PromptTreeViewModal.tsx` (Compacting `formatDualBadge`, removing redundant role text pills, stripping bracket tokens from buttons)  
**Status:** READY FOR IMPLEMENTATION  

---

## 1. Problem Statement & Subtask Scope

Users reported two prominent user-experience issues in the prompt management interface:
1. **False-Positive "Running" Items**: Conversations and projects remain tagged as `RUNNING` with animated green pulse dots long after the AI agent has finished thinking, generating responses, and executing tools. Stale `running_projects` SQLite entries never expire, loose 10-minute timestamp windows mask completed states, and the `only_running` filter leaks idle projects into view.
2. **Visual Clutter & Excessive Tag Noise**: The prompt list is cluttered with heavy bracketed sequence tags (e.g. `[AGM:P001 | GM:#1]`, `[AGM:C001 | GM:3c5e63cf]`), repetitive role text pills (`User`, `Subagent`, `System`, `Tool`) directly beside the dedicated tier icons, and bracketed button labels (`Show Less [Collapse]`, `Show All ({n}w) [Expand (Full Text)]`).

This subtask delivers:
- **Transcript Terminal State Verification**: Detecting `PLANNER_RESPONSE` / `MODEL` steps with `status: DONE` and zero open tool calls to immediately force conversations to `IDLE`.
- **Narrowed Active Windows**: Reducing timestamp heuristics from 600s/240s down to 60s (unless OS worker process is verified alive).
- **SQLite Pruning of Stale `running_projects`**: Automatically purging rows where `last_detected_at < now - 120`.
- **Hardened `only_running` Filter**: Ensuring projects are only shown in `only_running` mode if they have active running conversations.
- **Tag Compaction**: Formatting dual badges as clean `P001 · #1` and `C001 · <cid>` without square brackets or `AGM:` prefixes, eliminating redundant role text badges, and cleaning button text.

---

## 2. Technical Design: Backend Running Detection Hardening

### 2.1 Transcript Terminal State Verification (`inspect_conversation_transcript`)

In `src-tauri/src/modules/repo_db.rs`, enhance `TranscriptInspection` and `inspect_conversation_transcript`:

```rust
pub struct TranscriptInspection {
    pub step_count: usize,
    pub latest_prompt: Option<String>,
    pub latest_response: Option<String>,
    pub execution_results: Option<String>,
    pub tool_calls_summary: Option<String>,
    pub latest_step_summary: Option<String>,
    pub is_subagent: false,
    pub is_recent_active: bool,
    pub is_non_prompt: bool,
    pub is_terminal_done: bool, // NEW: indicates generation is completed
}
```

In `inspect_conversation_transcript`:
1. Narrow the file mtime window from 240s to 60s:
   ```rust
   let is_recent_active = mtime_epoch > 0 && (now_epoch - mtime_epoch <= 60);
   ```
2. In the reverse-scan of the last 10 lines:
   ```rust
   let mut is_terminal_done = false;

   for line in lines.iter().rev().take(10) {
       if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
           let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
           let s_source = val.get("source").and_then(|v| v.as_str()).unwrap_or("");

           // Bypass telemetry noise
           if s_type == "TOKEN_USAGE" || s_type == "TELEMETRY" || s_type == "HEARTBEAT" {
               continue;
           }

           let s_status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");

           // Inspect for terminal completion
           if s_type == "PLANNER_RESPONSE" || s_source == "MODEL" {
               let tool_calls = val.get("tool_calls").and_then(|tc| tc.as_array());
               let has_in_flight_tools = tool_calls.map(|tc| !tc.is_empty()).unwrap_or(false);

               if (s_status.eq_ignore_ascii_case("DONE") || s_status.eq_ignore_ascii_case("COMPLETED"))
                   && !has_in_flight_tools
               {
                   is_terminal_done = true;
               }
           }
           // ... process tool_calls, thinking, and step summary ...
       }
   }

   // Terminal Supremacy: If the transcript recorded completion, it CANNOT be recent active!
   let effective_recent_active = if is_terminal_done {
       false
   } else {
       is_recent_active
   };
   ```

### 2.2 Narrowing Heuristics in `compute_project_conversation_tree`

In `src-tauri/src/modules/repo_db.rs:4832`:
```rust
let conv_ts = parse_flexible_timestamp(&last_time_str);
let is_recent = conv_ts > 0 && (now - conv_ts <= 60); // NARROWED from 600s to 60s

let is_idle_count = not_fully_idle == 0;
let has_idle_status = status.contains("IDLE")
    || status.contains("COMPLETED")
    || status.contains("FAILED")
    || status.contains("CANCELLED");
let is_explicit_idle = is_idle_count || has_idle_status;

let inspection = inspect_conversation_transcript(base, &cid);

let is_conv_running = if is_explicit_idle || inspection.is_terminal_done {
    false
} else if is_owning_inst_alive && not_fully_idle > 0 && status.contains("RUNNING") && is_recent {
    true
} else if is_owning_inst_alive && inspection.is_recent_active && !has_idle_status {
    true
} else {
    false
};
```

### 2.3 Narrowing Heuristics in `is_prompt_running_for_project`

In `src-tauri/src/modules/repo_db.rs`:
- Line 1888: Query `active_prompts` with `updated_at >= ?3` where `?3 = now - 120` (narrowed from `now - 600`).
- Line 1987: In live conversation summary check, evaluate `now - conv_time <= 60` (narrowed from `now - 600`).

### 2.4 Stale `running_projects` SQLite Row Purging in `detect_running_projects`

In `src-tauri/src/modules/repo_db.rs:626`:
```rust
// Purge stale projects that haven't been detected in the last 120 seconds
let _ = conn.execute(
    "DELETE FROM running_projects WHERE last_detected_at < (?1 - 120)",
    params![now],
);
```

### 2.5 Hardening `only_running` Filter in `compute_project_conversation_tree`

Replace lines 5465-5471 in `src-tauri/src/modules/repo_db.rs`:
```rust
if only_running {
    let has_any_running_conv = conv_nodes.iter().any(|c| c.is_running);
    if !proj_is_running && !has_any_running_conv {
        continue;
    }
}
```

---

## 3. Technical Design: Frontend Tag Compaction & Clutter Elimination

### 3.1 Compacting `formatDualBadge` in `PromptTreeViewModal.tsx`

In `src/components/instances/PromptTreeViewModal.tsx:116`:

```typescript
// Helper to format GitMap dual sequence badge: P001 · #1 or C001 · <cid>
function formatDualBadge(agmCode: string | undefined, defaultAgm: string, gmCode: string | undefined, defaultGm: string): string {
    const rawAgm = (agmCode || defaultAgm).replace(/^AGM:/i, '').trim();
    const rawGm = (gmCode || defaultGm).replace(/^GM:/i, '').trim();
    return `${rawAgm} · ${rawGm}`;
}
```

This transforms:
- `[AGM:P001 | GM:#1]` -> `P001 · #1`
- `[AGM:C001 | GM:3c5e63cf]` -> `C001 · 3c5e63cf`

### 3.2 Removing Redundant Role Text Badges

In `PromptTreeViewModal.tsx`:
1. In conversation row rendering (lines 2133-2145):
   Remove the adjacent role text span:
   ```tsx
   {/* REMOVE THIS REDUNDANT TEXT BADGE */}
   <span className={cn("px-1.5 py-0.2 rounded-full text-[8.5px] ...")}>
       {promptCategory.roleBadge || ...}
   </span>
   ```
   Retain the `IconComponent` (`User`, `Bot`, `Terminal`, `Wrench`) with an informative `title` attribute:
   ```tsx
   <IconComponent
       className={cn('h-3.5 w-3.5 shrink-0', ...)}
       title={promptCategory.tier}
   />
   ```
2. Apply the same clean removal to sub-run child items (lines 2250+).

### 3.3 Stripping Bracket Tokens from Button Labels

In `PromptTreeViewModal.tsx`:
- Line 3405: Change `<span>Show Less [Collapse]</span>` to `<span>Show Less</span>`.
- Line 3410: Change `<span>Show All ({totalWords || activeWordCount}w) [Expand (Full Text)]</span>` to `<span>Show All ({totalWords || activeWordCount}w)</span>`.

---

## 4. Implementation Steps: Subtask 02

### Step 1: Update `inspect_conversation_transcript`
- **File:** `src-tauri/src/modules/repo_db.rs`
- **Actions:**
  1. Add `is_terminal_done` to `TranscriptInspection`.
  2. Implement `is_terminal_done` detection in reverse-scan loop.
  3. Narrow mtime check to 60s.
  4. Force `effective_recent_active = false` if `is_terminal_done` is true.

### Step 2: Harden Running Heuristics in `repo_db.rs`
- **File:** `src-tauri/src/modules/repo_db.rs`
- **Actions:**
  1. Update `compute_project_conversation_tree`: check `inspection.is_terminal_done`, narrow `is_recent` to 60s.
  2. Fix `only_running` filter at lines 5465-5471.
  3. Add `DELETE FROM running_projects WHERE last_detected_at < (?1 - 120)` in `detect_running_projects`.
  4. Update `is_prompt_running_for_project` TTL checks (600s -> 60s/120s).

### Step 3: Refactor `formatDualBadge` and Remove Redundant Tags
- **File:** `src/components/instances/PromptTreeViewModal.tsx`
- **Actions:**
  1. Update `formatDualBadge` to return `${rawAgm} · ${rawGm}`.
  2. Remove redundant `<span className="px-1.5 py-0.2 rounded-full text-[8.5px] ...">` role pills.
  3. Clean button labels for "Show Less" and "Show All".

---

## 5. Verification & Testing Protocol

1. **Terminal Status Verification**:
   - Inspect a project whose agent just finished a task (`PLANNER_RESPONSE` with `status: DONE`).
   - Trigger `agm prompt tree` or open Prompt Tree View modal.
   - Verify that the conversation displays `IDLE` (no pulsing green dot).
2. **Only Running Filter Verification**:
   - Query tree with `only_running: true`.
   - Verify that completed/idle projects are completely excluded.
3. **UI Visual Audit**:
   - Open Prompt Tree View.
   - Verify dual badges render cleanly as `P001 · #1` and `C001 · <cid>`.
   - Verify no redundant "User" / "Subagent" text pills appear next to tier icons.
   - Verify buttons read "Show Less" and "Show All (150w)" without bracket noise.
