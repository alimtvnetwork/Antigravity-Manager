# Specification: Idle Notification Logic Fix & AGM GitMap Parity (Spec 43)

## User Request (Verbatim)

```text
https://prnt.sc/l8PHOl0X3WWo

Here, the status is wrong. It was not in idle. That's the first thing. And below it should also send a few things. For example, when we do the AGM, AGM will tell us which version, which commit hash, just like Git map, okay? Which branch, and what is the last release that it has done. So this type of information like Git map should be embedded, not like what you have right now. So a few things at the end, it will actually show us a table of commands that we can run, how we can run it, and also list of projects which you didn't do it. So you should include that. Because without the project name, how can we run the prompts? And also list of prompts we need. But in this case, your fonts are very small. I requested several times to make the fonts bigger, you didn't do it. So that's disrespectful. I think you focus on this and make sure that these are done properly, and also at the same time, how you did the idleness, how did you check it? So the logic was wrong because it was not in idle, it was actually running. So you need to check and fix that code as well, and then finally do a minor bump and release. And also check the Git map, the Git map to understand there is no issues. Okay? Is it clear?
```

## Visual Reference & Screenshot Ingestion

- Ingested Screenshot Path: `![Idle Notification Screenshot](../../../assets/screenshots/idle-email-notification.png)`
- Screenshot Analysis:
  - Header displays `[v4.75.0 | W3 | 192.168.1.12] AGM TELEMETRY`.
  - Body asserts "There are no active prompts currently running in the following active projects..." when project `antigravity-manager-d58c5517` was in fact actively executing a prompt with `status = 'CASCADE_RUN_STATUS_RUNNING'` and `not_fully_idle = 1`.
  - Missing GitMap telemetry: commit hash, branch name, last release tag.
  - Missing commands cheat-sheet table and full project details.
  - Font size is small and hard to read on mobile and desktop email clients.

---

## 1. Architectural Architecture & Root Cause Fix

### 1.1 Root Cause of False "Idle" State
In `src-tauri/src/modules/email_watcher.rs`, `check_idle_projects_sensor` called:
```rust
let active_prompts = match crate::modules::repo_db::list_backed_up_prompts() { ... };
```
`list_backed_up_prompts()` executes `SELECT ... FROM active_prompts WHERE status = 'backed_up'`. It ONLY counts prompts that have been manually snapshotted via backup commands. It completely ignored live Antigravity execution state in `~/.gemini/antigravity/conversation_summaries.db`, where `not_fully_idle = 1` or `status = 'CASCADE_RUN_STATUS_RUNNING'`.

### 1.2 Fixed Idleness Invariant
A project or workspace is **ACTIVE/RUNNING** (and NOT idle) if:
1. `conversation_summaries.db` contains any conversation associated with that workspace having `not_fully_idle != 0` OR `status.contains("RUNNING")`.
2. OR the conversation was modified within the last 120 seconds with an active prompt payload.
3. OR in-flight prompt entries exist in `repo_prompts.db` (`active_prompts`).

If ANY project currently has active prompts running, the idle sensor must:
- NOT declare the overall workspace idle.
- Exclude running projects from the idle project candidate list.
- If all monitored projects have active prompts running, completely suppress sending the idle notification email.

---

## 2. GitMap-Style Telemetry Parity

Just like GitMap, AGM must expose:
- `VERSION`: e.g. `v4.79.0`
- `COMMIT_HASH`: Short git commit hash (e.g., `33622a2e`)
- `BRANCH`: Active branch name (e.g., `main`)
- `LAST_RELEASE`: Previous/active release tag (e.g., `v4.78.0`)

This metadata is compiled or determined at runtime via `build.rs` and injected into:
1. Terminal CLI banner & `agm version` / `agm status`.
2. All outgoing email notifications (Idle notification, switch notification, update alert).

---

## 3. Email Layout, Typography & Command Tables

### 3.1 Typography Standards
- Font family: `'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif`.
- Base body text: `16px` with line-height `1.6`.
- Headings: `24px` - `28px` bold.
- Command code blocks & tables: `14px` - `15px` with high contrast.
- Hyperlinks: Pure white (`color: #ffffff !important; text-decoration: underline; font-weight: 600;`).

### 3.2 Actionable Commands Table
At the end of the notification, render a clear HTML table:
| Command / Action | Syntax / Email Format | Purpose |
| :--- | :--- | :--- |
| **Send Prompt to Project** | `Subject: sub: <NODE> \| proj-<project_name>`<br>`<prompt_text>` | Dispatch prompt to specified workspace |
| **List Running Projects** | `Subject: cmd: <NODE> \| agm running-projects` | Inspect live project status |
| **List Prompts** | `Subject: cmd: <NODE> \| agm running-prompts ls` | Inspect recent prompts and queue |
| **Trigger Auto-Switch** | `Subject: cmd: <NODE> \| agm switch-if-low-credit` | Check quota and rotate to highest account |

### 3.3 Complete Project List
Instead of bare slugs, display a clean table listing:
- Project Name
- Full Repository Path
- Current Execution State (`RUNNING` / `IDLE`)
- Conversation ID / Last Activity

---

## 4. Acceptance Criteria & Verification Gates
1. Idleness logic correctly detects running conversations (`CASCADE_RUN_STATUS_RUNNING` / `not_fully_idle == 1`).
2. Email notification includes GitMap metadata (version, commit hash, branch, last release).
3. Email template incorporates enlarged Ubuntu typography and white links.
4. Commands table and project listings are cleanly rendered.
5. All pre-flight checks pass (`cargo fmt`, `cargo clippy`, `npm run build`).
6. Minor version bump and release workflow passes on `gitmap pe`.
