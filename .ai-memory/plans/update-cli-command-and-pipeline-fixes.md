# Master Plan: Update CLI Command and Pipeline Fixes

**Task Slug**: `update-cli-command-and-pipeline-fixes`  
**Target Release**: `v4.186.0` (Minor Bump)  
**Maintainer Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)  
**Status**: IN_PROGRESS

---

## 1. Problem Statement & Root Cause Synthesis

### Problem 1: `agm update` Crash & Sharing Violation When Tool is Running
- **Root Cause**: On Windows, when `agm update` runs in-process or when `antigravity-manager.exe` is running, executables are loaded with `SEC_IMAGE`, creating strict OS file locks. `update_cmds.rs:380` only delegates if `ui_running` is true, but `is_ui_process_running_excluding` only checks `agm-alim.exe` and `antigravity-tools.exe` (missing `antigravity-manager.exe` and `agm.exe`). In `install.ps1`, `$processNames` and `$lockCandidates` omit `antigravity-manager` and `agm`, and line 289 skips terminating `$parentPid` (the running `agm.exe`). NSIS or PowerShell fails with `Access is denied (os error 5)` / sharing violation `0x20`.
- **Solution**: Universal Windows self-delegation in `update_cmds.rs` to `%TEMP%\agm-updater\agm-update-cli-<pid>.exe` with immediate caller exit (`exit(0)`). Expand process taxonomy and lock candidates across `process.rs` and `install.ps1`.

### Problem 2: Prompt Pipeline Disconnect & Missing Queue Options (CLI & UI)
- **Root Cause**:
  1. `cmd_prompts_send` in `src-tauri/src/bin/agm/prompt_cmds.rs:164` hardcodes fresh UUID `conv-...`, defaults `repo_path` to `"scratch/test-repo"`, and never executes `spawn_prompt_via_agy` or triggers instance dispatch.
  2. `cmd_prompts_queue` hardcodes `session_id: NULL` and lacks conversation flags (`--conversation`, `--existing`, `--new`).
  3. `agm instance prompt` is missing from `instance_cmds.rs`.
  4. `spawn_prompt_via_agy` lacks conversation arguments and fails hard when `agy.exe` is absent without falling back to `.antigravity_resume_task.json` + IDE focus.
  5. `check_and_dispatch_enqueued_prompts` in `queue.rs` marks prompts as `'dispatched'` even when spawn fails, causing prompt loss.
- **Solution**: Add `--conversation <id>`, `--existing`, `--new`, `--repo` (defaulting to `env::current_dir()`) and `qe` alias. Wire CLI execution to `send_prompt_now_for_instance`. Implement Tier 2 graceful degradation when `agy.exe` is absent.

### Problem 3: Prompt Backup & Restore Failure on Account Switch / Auto-Rotation
- **Root Cause**:
  1. `backup_running_prompts` in `backup.rs:65-83` only filters `WHERE status = 'running'`, completely ignoring `in_flight`, `queued`, and `dispatched` prompts.
  2. In `rotation.rs`, prompts backed up from `current_instance_id` remain tagged with `current_instance_id`. Post-switch restore queries `WHERE instance_id = target.instance_id`, finding ZERO prompts.
  3. `auto_resume_recent_prompts` in `auto_resume.rs:90` filters `WHERE status IN ('queued', 'pending')`, skipping `'backed_up'`.
- **Solution**: Transition `status IN ('running', 'in_flight', 'dispatched')` in `backup_running_prompts`. Re-attribute backed-up prompts from `current_instance_id` to `target.instance_id` upon profile rotation. Include `'backed_up'` in `auto_resume_recent_prompts`.

### Problem 4: UI Conversation Target Choice (Existing vs New Conv)
- **Root Cause**: In `PromptTreeViewModal` (`DetailHeader.tsx` and `usePromptTreePromptActions.ts`), there is no inline control to choose between sending/queueing to an existing conversation vs starting a new conversation.
- **Solution**: Add a dark-glass segmented conversation target selector (`targetMode`: `current` vs `new`) to Capsule 2 in `DetailHeader.tsx`. Pass `conversation_id` accordingly in `usePromptTreePromptActions.ts`.

---

## 2. Subtasks Roadmap

| Subtask ID | File | Description | Status |
| :--- | :--- | :--- | :--- |
| **01** | `.ai-memory/plans/subtasks/update-cli-command-and-pipeline-fixes/01-update-cli-self-delegation-fix.md` | Universal Windows self-delegation & process lock candidate expansion | PENDING |
| **02** | `.ai-memory/plans/subtasks/update-cli-command-and-pipeline-fixes/02-prompt-pipeline-and-queue-fix.md` | Prompt pipeline dispatch, queue options, conversation targeting & fallback | PENDING |
| **03** | `.ai-memory/plans/subtasks/update-cli-command-and-pipeline-fixes/03-prompt-backup-and-restore-switch-fix.md` | Pre-switch status backup expansion, cross-instance re-attribution & auto-resume | PENDING |
| **04** | `.ai-memory/plans/subtasks/update-cli-command-and-pipeline-fixes/04-ui-conversation-choice-fix.md` | UI conversation target selector in PromptTreeView header for Send and Queue | PENDING |
| **05** | `.ai-memory/plans/subtasks/update-cli-command-and-pipeline-fixes/05-preflight-release-ceremony.md` | Pre-flight validation, version bump `v4.186.0`, changelog attribution & release tag | PENDING |

---

## 3. Architectural Invariants

1. **Strict Relative Git Paths**: Zero absolute paths in code, specs, plans, or commit messages.
2. **Attribution Invariant**: Release notes and changelogs must strictly attribute `@aukgit` (`(Thanks to @aukgit)`).
3. **No Silent Errors**: All fallible operations must be logged or propagated via `AppError`.
4. **GitMap Atomic Commits**: All commits must use `gitmap cpf "<module> - <summary>"`.
5. **Universal Self-Delegation**: On Windows, updater binary never overwrites its own running process image in-place.
