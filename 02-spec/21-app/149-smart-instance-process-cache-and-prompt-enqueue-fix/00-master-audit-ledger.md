# Master Audit Ledger: Smart Instance Process Cache & Prompt Enqueue Fix

- **Slug**: `149-smart-instance-process-cache-and-prompt-enqueue-fix`
- **Specification Path**: `02-spec/21-app/149-smart-instance-process-cache-and-prompt-enqueue-fix/`
- **Plan Path**: `.ai-memory/plans/149-smart-instance-process-cache-and-prompt-enqueue-fix.md`
- **Subtasks Path**: `.ai-memory/plans/subtasks/149-smart-instance-process-cache-and-prompt-enqueue-fix/`
- **Protocol**: `execute-parent-task-with-n-steps-v6` (N = 300, A = 2, H = 2, C = 30)
- **Status**: `[IN PROGRESS]`
- **Created**: 2026-10-09

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

## Discrete Deliverables & Actionable Subtasks

| Subtask ID | Title | Owner | Target Files | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Subtask-01** | Smart Process Cache Unification & Closed-PID Re-Scan | Worker 01 | `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/process.rs` | `[QUEUED]` |
| **Subtask-02** | Prompt Dispatch & Unified FIFO Enqueue Pipeline | Worker 01 | `src-tauri/src/commands/instance.rs`, `src-tauri/src/modules/repo_db.rs`, `src/services/instanceService.ts` | `[QUEUED]` |
| **Subtask-03** | Ghost Running Eradication & Idle State Protection | Worker 02 | `src-tauri/src/modules/repo_db.rs` | `[QUEUED]` |
| **Subtask-04** | Prompt Tree View Tag Compaction & Bracket Noise Stripping | Worker 02 | `src/components/instances/PromptTreeViewModal.tsx`, `src/components/instances/InstanceTable.tsx` | `[QUEUED]` |
| **Subtask-05** | Host-Shielded E2E Tests, CLI Parity & Minor Version Bump | Lead | `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`, `package.json`, manifests | `[QUEUED]` |

---

## Root Causes Catalog

1. **Un-resolved Instance ID in Process Checker**: `is_instance_process_running_smart` fails to match instances when IDs are passed as sequence numbers (e.g. `'1'`), aliases (`'default'`), or case differences, erroneously concluding the instance is offline and launching duplicate windows.
2. **Double-Launch Hazard in Frontend `handleResendPrompt`**: Calling `sendPromptNow` followed by `focusInstanceWorkspace` triggers two independent launch attempts if process detection misses.
3. **Conflicting Duplicate Definitions of `enqueue_prompt`**: `commands/instance.rs` defines `enqueue_prompt` twice with conflicting parameter signatures and types, corrupting IPC calls.
4. **Resurrecting Stale Prompts in `auto_resume_recent_prompts`**: Query lacks status filter and selects old completed prompts, resurrecting them into running tasks.
5. **Excess Bracket Clutter & Badges in Prompt Tree View**: Up to 6 adjacent pills in header and noisy `[AGM:...]` / `[GM:...]` tags clutter UI.
