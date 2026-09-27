# Specification: Idle Sensor Activity Determination, GitMap Telemetry Parity & Notification Typography (Spec 55)

## User Request (Verbatim)

```text
https://prnt.sc/l8PHOl0X3WWo


Here, the status is wrong. It was not in idle. That's the first thing. And below it should also send a few things. For example, when we do the AGM, AGM will tell us which version, which commit hash, just like Git map, okay? Which branch, and what is the last release that it has done. So this type of information like Git map should be embedded, not like what you have right now. So a few things at the end, it will actually show us a table of commands that we can run, how we can run it, and also list of projects which you didn't do it. So you should include that. Because without the project name, how can we run the prompts? And also list of prompts we need. But in this case, your fonts are very small. I requested several times to make the fonts bigger, you didn't do it. So that's disrespectful. I think you focus on this and make sure that these are done properly, and also at the same time, how you did the idleness, how did you check it? So the logic was wrong because it was not in idle, it was actually running. So you need to check and fix that code as well, and then finally do a minor bump and release. And also check the Git map, the Git map to understand there is no issues. Okay? Is it clear?
```

## Visual Reference & Screenshot Ingestion

- Ingested Screenshot Path: `![Idle Notification Screenshot](../../../assets/screenshots/agm-idle-status-fix-01.png)`
- Visual Analysis:
  - Header displays `[v4.75.0 | W3 | 192.168.1.12] AGM TELEMETRY`.
  - Notification incorrectly reported idle state for project `antigravity-manager-d58c5517` while conversation `d58c5517-d8ad-437e-ab7c-e506b0322383` was running.
  - Missing GitMap telemetry: Version, Git commit SHA, branch name, last release tag.
  - Missing executable commands table showing syntax for dispatching prompts and managing node.
  - Missing structured projects list table showing Project Name, Target Identifier (`proj-<id>`), Path, and Status.
  - Missing reference list of prompts.
  - Fonts were noticeably small (11-14px) and difficult to read.

---

## 1. Idleness Detection & Process Activity Sensor Invariant

A workspace / node is strictly **ACTIVE / RUNNING** (and must NEVER trigger an idle alert) if ANY of the following hold true:
1. `conversation_summaries.db`: Any record has `not_fully_idle != 0` OR `status LIKE '%RUNNING%'` OR `last_modified_time` modified within the last 600 seconds.
2. Active Antigravity Processes: Any process with name `Antigravity` or `code` running with Antigravity args and active CPU usage (>0.2%).
3. `repo_prompts.db`: Any prompt record in `active_prompts` with status `'running'` or `'queued'`.
4. Workspace Path & Conversation ID Matching:
   - Path normalization must convert all backslashes to forward slashes, lowercase, and strip trailing slashes.
   - Project ID matching must check both path equivalence AND conversation ID prefix matching (e.g. `antigravity-manager-d58c5517` matches `d58c5517`).

If any project is running or if `is_any_prompt_actively_running()` returns `true`:
- Completely suppress the idle email dispatch.
- Mark the project status as `RUNNING` in all outputs.

---

## 2. GitMap-Style Telemetry Parity

AGM CLI banner (`print_banner()`), `agm version`, and all outgoing email notifications must display the standardized GitMap telemetry block:
```text
  ────────────────────────────────────────────────────────────
  agm binary
  ● Name:           agm (Antigravity-Manager)
  ● Git URL:        https://github.com/alimtvnetwork/Antigravity-Manager
  ● Version:        v4.81.0
  ● Commit SHA:     <commit_hash>
  ● Branch:         <branch>
  ● Last Release:   <last_release>
  ● Database:       <path_to_repo_prompts_db>
  ● Installed path: <executable_path>

  ────────────────────────────────────────────────────────────
  current workspace / node
  ● Node Alias:     <machine_name>
  ● Local IPv4:     <machine_ip>
  ● Dispatched At:  <utc_timestamp>
```

---

## 3. Projects Table, Commands Table & Prompts Reference

Outgoing notifications and CLI help must embed three structured tables:
1. **Active Projects Table**:
   - `Project Name`: Human-readable repository title.
   - `Email Target Identifier`: High-contrast copyable badge `sub: <NODE> | proj-<id>`.
   - `Repository Path`: Normalized full filesystem path.
   - `Live Status`: `RUNNING` (green badge) or `IDLE` (slate badge).
2. **Commands & Execution Table**:
   - `Action / Goal`: Send Prompt, Broadcast, Check Quota, Trigger Auto-Switch, List Prompts.
   - `Email Subject Format`: Exact copyable format.
   - `CLI Equivalent`: Exact `agm` command.
3. **Usable Prompts Reference List**:
   - Standard reusable prompt slugs from `01-prompts/` (e.g., `01-code-review`, `02-test-fix`, `03-feature-plan`).

---

## 4. Substantially Enlarged Typography

Across all email cards and HTML templates:
- **Base Body Text**: `18px` (line-height `1.7`, color `#0f172a`).
- **Card Headings**: `28px` - `32px` bold.
- **Section Headers**: `18px` uppercase bold, letter-spacing `0.06em`.
- **Table Headers**: `16px` bold with `padding: 16px 20px;`.
- **Table Body Cells**: `16px` - `17px` with `padding: 14px 20px;`.
- **Code & Badges**: `16px` monospace with `padding: 6px 12px; font-weight: 700;`.
- **White Hyperlinks**: `color: #ffffff !important; font-size: 16px; font-weight: bold; background: #2563eb; padding: 5px 12px; border-radius: 6px;`.

---

## 5. Verification & Acceptance Criteria

1. `is_any_prompt_actively_running()` returns `true` when conversation is in `CASCADE_RUN_STATUS_RUNNING` or `not_fully_idle = 1`.
2. Projects matching active conversations display `RUNNING` instead of `IDLE`.
3. Idle alert email is suppressed when any prompt or process is actively running.
4. GitMap telemetry (Version, SHA, Branch, Last Release) is embedded in CLI and email notifications.
5. Projects table, commands table, and prompts reference are cleanly rendered with prominent 16-18px typography.
6. GitMap health checks report 0 errors.
7. Minor version bump to `v4.81.0` and release tags pushed cleanly.
