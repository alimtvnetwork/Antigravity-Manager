# Plan: Fixing Auto-Switching, Prompt Dispatch, Progress Bar Inner Glow, and GitMap Credential / Defender Issues

**Slug**: `fixing-auto-switching-and-progress-bar-issues`
**Status**: `COMPLETED`
**Spec Directory**: `02-spec/21-app/fixing-auto-switching-and-progress-bar-issues/`

---

## Overview

End-to-end root-cause fix and verification across four core areas:
1. **Prompt Dispatch, Enqueue & Rerun (`Antigravity-Manager`)**: Eliminate `Send failed: agy is already running for this workspace` by replacing stale headless `agy.exe` workers on explicit `Send` / `Rerun`, and auto-draining queued FIFO prompts as soon as `agy.exe` exits.
2. **Account Switching, Auto-Switching & Fast-Forward (`Antigravity-Manager`)**: Fix Accounts row switch button tooltip and click handler to target the currently selected instance (`activeInstanceId`), harden instance credential switching, and instrument all switch/prompt actions in the UI/CLI audit recorder (`agm ui record`).
3. **Progress Bar Inner-Dot Glow (`QuotaProgressBar.tsx` & `WaterDrainProgressBar.tsx`)**: Remove outer progress bar track glow (`shadow-none`) and keep crisp internal glow strictly inside the 5 round checkpoint figures (`#1af18d` green sample).
4. **GitMap Credential/SSH Auto-Fix & Windows Defender False Positive (`gitmap`)**: Eliminate `VirTool:MSIL/Subti.C` heuristic triggers (`vssadmin` shadow copy, hidden PowerShell WMI/taskkill, `DefaultPassword`), show explicit credential & SSH commands in `gitmap pa` tree and summary, and make `gitmap fix` / `gitmap fix --all` automatically remediate `wincredman` and convert same-owner (`alimtvnetwork`) HTTPS remotes to SSH (`git@github.com:alimtvnetwork/<repo>.git`).

---

## Subtasks

| # | Subtask File | Status | Description |
| :--- | :--- | :--- | :--- |
| `01` | [01-spec-and-test-plan.md](./subtasks/fixing-auto-switching-and-progress-bar-issues/01-spec-and-test-plan.md) | `COMPLETED` | Save screenshots and write architecture spec + 14 E2E test cases spec. |
| `02` | [02-cli-audit-recorder-and-switch-fixes.md](./subtasks/fixing-auto-switching-and-progress-bar-issues/02-cli-audit-recorder-and-switch-fixes.md) | `COMPLETED` | Add `agm ui record` CLI commands, instrument backend actions, and fix Accounts row switch button instance targeting. |
| `03` | [03-prompt-send-enqueue-rerun-fixes.md](./subtasks/fixing-auto-switching-and-progress-bar-issues/03-prompt-send-enqueue-rerun-fixes.md) | `COMPLETED` | Fix `agy is already running for this workspace` on `Send`/`Rerun` and add automatic FIFO queue drain on `agy` worker exit. |
| `04` | [04-progress-bar-inner-glow-refinement.md](./subtasks/fixing-auto-switching-and-progress-bar-issues/04-progress-bar-inner-glow-refinement.md) | `COMPLETED` | Remove outer track glow and confine crisp glow strictly inside the 5 round checkpoint figures (`#1af18d`). |
| `05` | [05-gitmap-credential-ssh-and-defender-fix.md](./subtasks/fixing-auto-switching-and-progress-bar-issues/05-gitmap-credential-ssh-and-defender-fix.md) | `COMPLETED` | Eliminate `VirTool:MSIL/Subti.C` triggers in `gitmap`, add credential/SSH commands to `gitmap pa`, and auto-convert same-owner HTTPS repos to SSH in `gitmap fix --all`. |
| `06` | [06-e2e-execution-and-git-push.md](./subtasks/fixing-auto-switching-and-progress-bar-issues/06-e2e-execution-and-git-push.md) | `COMPLETED` | Run pre-flight checks, execute all 14 E2E test cases (`TC-01`..`TC-14`), restore auto-switch threshold to `15.0%`, and commit/push. |
