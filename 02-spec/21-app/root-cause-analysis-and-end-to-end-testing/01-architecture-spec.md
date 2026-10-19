# Architecture Spec: Root Cause Analysis and End-to-End Testing

## User Request (Verbatim)
```markdown
is it done already??

# Root Cause Analysis and End-to-End Testing: high priority instruction, non-negotiable task

I've added the screenshot. You can see I have a running prompt, and the running prompt is very different from what you can show here, show there. I've added two big screenshots so that you can understand in the prompt tree. Your calculation and everything mostly wrong. You need to find the root cause. Find with date, time when you send it, the acknowledgment prompt with image. Image I cannot see yet, so that is a scam. You are performing for the long period of time. I hope you understand and get out of this scam business. If something is not working, tell me that it is not working. It's a pure wastage. You waste my time, not your, but everyone's time. That's not really cool. You need to test it here with end-to-end testing as much as possible. Try to see by using the machine how you can fix it. Also follow through the telegram steps to send prompt from one machine to another. Make sure that it works. Currently, we still cannot connect with two machines where GitMap can.

## task: Root Cause Analysis and End-to-End Testing
## slug: root-cause-analysis-and-end-to-end-testing

# Actionable Items Must Follow Non-Negotiable

1. Git pull BEFORE starting any work — first try `gitmap pull <repo path>` (or run `gitmap pull` in repository folder); if that fails, fall back to regular `git pull` and resolve any merge conflicts so the repository is completely up to date and clean before starting.
2. Write spec under 02-spec/21-app/root-cause-analysis-and-end-to-end-testing/ and enqueue plan task in .ai-memory/plans/root-cause-analysis-and-end-to-end-testing.md (subtasks in .ai-memory/plans/subtasks/root-cause-analysis-and-end-to-end-testing/) first
3. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
4. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
5. Find the root cause of the discrepancies in the prompt tree using date and time stamps.
6. Conduct end-to-end testing to identify and resolve issues.
7. Follow telegram steps to send prompts between machines and ensure connectivity.
8. Verify that GitMap can connect with two machines and troubleshoot any connectivity issues.

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

- /plan first before doing the work to reduce the credits.
- /learn from @[.agents/skills/gitmap] skill to leverage GitMap high-speed search, toolchain discovery, and caching.
- Only add the relative paths, never add the absolute path during your work; this should be respected on the release page and in release notes as well.
```

---

## 1. Architectural Scope & Problem Statement

### 1.1 Problem Domains
1. **Prompt Tree Timing & Turn Calculation Discrepancies**:
   - The user's active IDE shows running prompts and queued turns (e.g. Turn 16 running, 4 queued messages scheduled to send afterwards).
   - In AGM's Prompt Tree View, turn numbers, timestamp durations (e.g. `16h 18m (01:23:45 AM -> 05:41:47 PM)`), and turn statuses can diverge or display inaccurate spans if timestamps are calculated using first-step vs last-turn offsets incorrectly.
2. **Missing Image Previews & Attached Media Failures**:
   - Images attached in prompts or captured as acknowledgments were not rendering in the UI or were failing to load in `PromptThumbnailCard`.
   - Root causes: Tauri asset protocol (`asset://localhost/`) restrictions in WebView2, local filesystem access guards in `src-tauri/src/commands/cmd_files.rs` that blocked `C:\Users\Administrator` paths, and missing Base64 data URL fallbacks.
3. **Cross-Machine Prompt Dispatching & GitMap SSH Connectivity**:
   - AGM supports sending prompts across machines via Telegram bot commands and GitMap SSH port 22 mesh.
   - When communicating between `192.168.1.3` (w1) and `192.168.1.20` (main / w2), the system must correctly resolve machine names using GitMap node configuration (`~/.gitmap/gitmap-ssh-nodes.json` and `GITMAP_CLUSTER_NODES`), append acknowledgments with timestamps and screenshots, and establish reliable bidirectional communication.

---

## 2. Subsystem Architecture & Invariants

### 2.1 Prompt Tree Engine
- **Source of Truth**: `transcript_full.jsonl`, `transcript.jsonl`, and `.antigravity_goal_prompt.log`.
- **Status Computation**:
  - A conversation is marked active if `is_recent_active && is_owning_inst_alive`.
  - The currently executing turn must be flagged with `RUNNING` status and emerald badge.
  - Queued turns waiting to run must be flagged with `QUEUED` status and amber clock badge.
  - Finished turns are marked `COMPLETED`.
- **Timestamp & Duration Formulas**:
  - Turn Start Time: `created_at` timestamp of the `USER_INPUT` turn step.
  - Turn End Time:
    - If `RUNNING`: Current system time (`chrono::Utc::now()`).
    - If `COMPLETED`: Timestamp of the final assistant response or tool completion step in that turn.
  - Elapsed Duration: Format as `Xh Ym Zs` (or `Xm Ys` for < 1 hour).
  - Explicit Wall-Clock Display: `hh:mm:ss AM/PM -> hh:mm:ss AM/PM`.

### 2.2 Image Loading & Security Architecture
- **Capabilities & Permissions**: `src-tauri/capabilities/default.json` grants `core:asset:default` to allow WebView2 to load local media assets via `convertFileSrc`.
- **Path Guard Invariant**: `src-tauri/src/commands/cmd_files.rs` `is_sensitive_path` must distinguish between system-critical folders (e.g. `C:\Windows\System32`) and legitimate user home directories (`C:\Users\Administrator\...`). User workspace assets, screenshots, and uploads must be explicitly permitted.
- **Data URL Fallback**: If WebView2 asset protocol fails or is blocked, the frontend proactively falls back to IPC `read_image_as_base64`, embedding the image as a `data:image/png;base64,...` URL.

### 2.3 Cross-Machine Telegram & GitMap SSH Mesh
- **Node Identification**: Local IP matching against GitMap SSH nodes table (`~/.gitmap/gitmap-ssh-nodes.json`) takes precedence over ambiguous hostname checks.
- **Port 22 SSH Relay**: When dispatching prompts remotely, AGM invokes GitMap SSH mesh commands or `ssh_manager` to route prompts to node `w1` or `main/w2`.
- **Telemetry Card Schema**: Acknowledgment cards sent to Telegram must include:
  - Machine Alias (`• Machine: w1 (collected from gitmap)`)
  - Instance Identifier (`• Instance: <instance_id>`)
  - Exact Date & Time Stamp (`• Timestamp: YYYY-MM-DD HH:MM:SS`)
  - Attached Screenshot Proof (JPEG/PNG buffer).
