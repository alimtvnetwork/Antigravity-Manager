# Master Audit Ledger: Smart Instance Process Cache & Prompt Dispatch Fix

- **Slug**: `151-smart-instance-process-cache-and-prompt-dispatch-fix`
- **Created**: 2026-10-10
- **Status**: IN_PROGRESS
- **Lead Architect**: Md. Alim Ul Karim (@aukgit)

---

## User Request (Verbatim)

```text
# High Priority Instruction

Hi there. Look, the sending prompt from the instance section to the IDE does not work. Find the root cause of it, why we cannot send the prompt and why we cannot enqueue the prompt. In both cases, it has no effect so far. Every time it tries to open or reopen the IDE instance, which is very wrong. If the instance is already open, you have to check in the process if this instance is already running. If the instance is already running, you need to cache it so that you can understand if the instance. That means you have to have the smart logic. When we send it or queue the prompt, first, you have to check how many instances are running for that Antigravity Tools Manager. You need to cache those as well. Let's say you find that it is running in the cache, but then again, you try to send and see this PID is closed. Then you have to rerun it and see if it is running again. If it is not running, then and only then you have to reopen and send the prompt. It is not happening right now like this. It is not working at all. You need to work on it, find the root cause, do the end-to-end testing, do things from CLI as well. Currently, the view is nice. I appreciate that, but there are too many tags, which is not necessary in the prompt section. Too many target bracket item, which can be reduced. And too many running items, which is not running, so you need to fix that as well. I hope you respect this and try to fix all this stuff and make a minor bump and release again.

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/<slug>/ and enqueue plan task in .ai-memory/plans/<slug>.md (subtasks in .ai-memory/plans/subtasks/<slug>/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Find the root cause of why the prompt cannot be sent or enqueued from the instance section to the IDE.
5. Implement logic to check if the IDE instance is already running and cache it accordingly.
6. Ensure that if the instance is running in the cache but the PID is closed, rerun it and verify if it is running again before reopening and sending the prompt.
7. Conduct end-to-end testing and perform tasks from the CLI.
8. Reduce unnecessary tags and target bracket items in the prompt section.
9. Fix the issue of too many running items that are not actually running.
10. Make a minor bump and release again.
```

---

## Discrete Deliverables Extraction

| Task ID | Deliverable Description | Status | Target Files |
| :--- | :--- | :---: | :--- |
| **Task-01** | Deep Root Cause Analysis: trace prompt send/enqueue failures to Antigravity IDE and unwarranted reopen loop | IN_PROGRESS | `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/repo_db.rs`, `src/components/instances/PromptTreeViewModal.tsx` |
| **Task-02** | Multi-Instance Process Cache & Closed-PID Two-Tier Re-scan: cache running instances, verify PID vitality, re-scan OS process table upon closed PID, guarantee zero relaunch when running | QUEUED | `src-tauri/src/modules/instance.rs`, `src-tauri/src/commands/instance.rs` |
| **Task-03** | Ghost Running Eradication: filter `auto_resume_recent_prompts` to `queued`/`pending`, eliminate idle conversation bypasses, enforce terminal planner state detection | QUEUED | `src-tauri/src/modules/repo_db.rs` |
| **Task-04** | UI Tag Compaction & Bracket De-Cluttering: strip `[...]` bracket tokens from dual badges, buttons, and row pills; consolidate header badges | QUEUED | `src/components/instances/PromptTreeViewModal.tsx`, `src/components/instances/InstanceTable.tsx` |
| **Task-05** | Host-Shielded E2E Testing, CLI Parity & Minor Bump Release Ceremony (`v4.172.0`) | QUEUED | `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`, `package.json`, `changelog.md`, `changelog_en.md`, `README.md`, `README_EN.md` |

---

## Non-Negotiable Operational Constraints
- Search exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`, `gitmap ps`, `gitmap py`). Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- Strictly relative Git paths in all code, specs, and release documentation.
- Mandatory Multi-Agent Partitioning: `A = 2`, `H = 2` via `invoke_subagent`. Solo execution without spawning subagents is strictly banned.
- Attribution strictly to `@aukgit` (`(Thanks to @aukgit)`).
- Host process shielding: Never kill or restart active Antigravity IDE instances during testing or prompt dispatch.
