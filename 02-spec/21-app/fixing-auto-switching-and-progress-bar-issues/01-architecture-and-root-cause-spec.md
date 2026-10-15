# Architecture & Root Cause Specification: Fixing Auto-Switching, Prompt Dispatch, Progress Bar Inner Glow, and GitMap Credential / Defender Issues

**Slug**: `fixing-auto-switching-and-progress-bar-issues`
**Date**: 2026-10-15
**Status**: `IN_PROGRESS`

---

## 1. Visual Evidence & Problem Summary

- **Screenshot 1** (`assets/screenshots/fixing-auto-switching-and-progress-bar-issues-01-accounts-progress-switch.png`):
  - Shows the Accounts view in Antigravity Manager v4.200.1 with `#2 Default Copy` selected in the top instance selector.
  - Hovering the account row switch button shows the hardcoded tooltip `"Switch to Antigravity (Classic) (右键选择实例)"` instead of targeting the selected `#2 Default Copy` instance.
  - Progress bars in the Quota column exhibit an excessive outer neon box-shadow halo around the entire rectangular pill container rather than confining the glow strictly inside the 5 circular milestone dots (`#1af18d` green sample).
- **Screenshot 2** (`assets/screenshots/fixing-auto-switching-and-progress-bar-issues-02-prompt-send-agy-error.png`):
  - Shows the Prompt Tree View (`Prompt Tree · Default`) with red banner:
    `Send failed: agy is already running for this workspace (pid 11700); prompt 'p-cc9db537' was not sent again; IDE focused (pid 4856)`
- **Screenshot 3** (`assets/screenshots/fixing-auto-switching-and-progress-bar-issues-03-gitmap-defender-virus.png`):
  - Shows Windows Security flagging `gitmap.exe` in `AppData\Local\gitmap-cli\gitmap.exe` as `VirTool:MSIL/Subti.C` (`Alert level: Severe`, `Status: Active`, `"This program is used to create viruses, worms or other malware"`).

---

## 2. Root Cause Analysis

### 2.1 Root Cause A — Prompt Send / Rerun Blocked by Lingering `agy` Worker PID
- **Location**: `src-tauri/src/modules/repo_db/agy.rs` (`spawn_prompt_via_agy`, `send_prompt_now_for_instance`, `enqueue_prompt_for_instance`).
- **Mechanism**:
  - `get_active_agy_workers()` tracks `"{instance_id}:{repo_path}" -> child_pid`.
  - When `agy -p` is spawned for a workspace, the helper CLI process (`agy.exe`) can remain alive while holding the workspace session or awaiting completion.
  - Any subsequent user click on `Send` (`send_prompt_now_for_instance`), `agm prompt`, or `agm rerun` checks `get_active_agy_workers()` and immediately aborts with `Err("agy is already running for this workspace (pid ...); prompt '...' was not sent again")`.
- **Architecture Fix**:
  - Introduce `spawn_or_replace_prompt_via_agy(prompt: &ActivePrompt, replace_existing: bool) -> Result<(), String>`.
  - When `replace_existing = true` (invoked by `send_prompt_now_for_instance`, `agm prompt`, and `agm rerun`), if an earlier headless `agy` worker PID is still registered for `"{instance_id}:{repo_path}"`, politely terminate that stale headless `agy` helper process (verifying its process name is `agy` / `antigravity-cli` and **never** `Antigravity.exe` IDE), remove the stale map entry, and spawn the new `agy` turn immediately.
  - When `replace_existing = false` (background FIFO queue scheduler `dispatch_queued_prompts`), preserve the non-destructive busy check so queued prompts wait their turn, and add an automatic FIFO drain trigger inside the `agy-monitor` thread as soon as the active `agy` worker exits.
  - Call `invalidate_prompt_tree_cache()` after `send_prompt_now_for_instance` and `enqueue_prompt_for_instance` so the UI reflects `DISPATCHED` / `QUEUED` immediately.

### 2.2 Root Cause B — Accounts Row Switch Button Ignores `activeInstanceId`
- **Location**:
  - `src/components/accounts/AccountTable/cells/ActionsCell.tsx`
  - `src/components/accounts/AccountRow.tsx`
  - `src-tauri/src/modules/instance/switch.rs`
- **Mechanism**:
  - Even when `#2 Default Copy` (`default-copy-5597`) or another instance is selected in the top bar (`activeInstanceId` in `useInstanceStore`), `ActionsCell.tsx` and `AccountRow.tsx` hardcode `title={t('accounts.switch_to_classic')}` and call `onSwitch()` with `targetIde = undefined`.
- **Architecture Fix**:
  - Resolve `activeInstance` from `instances` and `activeInstanceId`.
  - When `activeInstanceId` is set to a non-default instance, render the tooltip as `Switch to <Instance Name> (右键选择实例)` and pass `instance:${activeInstanceId}` to `onSwitch(...)` on left-click.
  - In `src-tauri/src/modules/instance/switch.rs`, ensure switching an account onto a target instance cleanly unbinds any stale/stopped sibling binding or re-verifies the same instance without error.

### 2.3 Root Cause C — Missing CLI Audit Recorder Controls (`agm ui record`) & Backend Action Instrumentation
- **Location**:
  - `src-tauri/src/bin/agm/ui_cmds.rs`
  - `src-tauri/src/proxy/server/admin_ui.rs`
  - `src-tauri/src/modules/audit/`
- **Mechanism**:
  - The UI interaction recorder (`src-tauri/src/modules/audit/`) and top-bar `REC` badge exist, but `agm ui` lacks CLI commands (`start`, `stop`, `status`, `list`, `tail`, `clear`, `export`) to control and inspect the audit session from the terminal, and backend commands (`switch_account`, `switch_account_to_instance`, `fast_forward_instance`, `send_prompt_now`, `enqueue_prompt`) do not log their execution results (`ok` / `error`) into `ui_audit.db`.
- **Architecture Fix**:
  - Add `agm ui record <start|stop|status|list|tail|clear|export> [--json]` in `src-tauri/src/bin/agm/ui_cmds.rs` with both REST (`/api/ui/record/*` in `admin_ui.rs`) and direct SQLite (`<data-dir>/ui-audit/ui_audit.db`) support, plus persistent cross-process recording state in `ui_audit.db` (`audit_meta` table) so CLI and GUI share recording state seamlessly.
  - Instrument `switch_account`, `switch_account_to_instance`, `fast_forward_instance`, `send_prompt_now`, and `enqueue_prompt` to record audit events whenever recording is active.

### 2.4 Root Cause D — Progress Bar Outer Track Glow Bleeding Outside the Pill
- **Location**:
  - `src/components/accounts/QuotaProgressBar.tsx`
  - `src/components/common/WaterDrainProgressBar.tsx`
- **Mechanism**:
  - `QuotaProgressBar.tsx` applies `clamped >= 95 && "shadow-[0_0_12px_rgba(26,241,141,0.8)]"` to the outer progress bar track `<div>`.
  - `WaterDrainProgressBar.tsx` applies `clamped >= 50 ? 'shadow-[0_0_10px_rgba(26,241,141,0.75)]' : ''` to the outer track `<div>`.
  - Checkpoint circles also use wide outer drop-shadows that bleed outside the pill container.
- **Architecture Fix (Per Spec `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/02-ui-ux-progress-theming-spec.md`)**:
  - Remove all outer box-shadow classes from the progress bar track container (`shadow-none`).
  - Keep the 5 quintile round checkpoint figures (`[100, 80, 60, 40, 20]`) neatly inside the pill height using `#1af18d` (`rgb(26, 241, 141)`) and apply a crisp internal/tight radial glow (`shadow-[inset_0_0_6px_rgba(255,255,255,0.9),0_0_3px_rgba(26,241,141,0.55)]`) strictly to the filled round checkpoint figures.

### 2.5 Root Cause E & F — GitMap Credential/SSH Auto-Remediation & Windows Defender `VirTool:MSIL/Subti.C`
- **Location**: External `gitmap` repository (`cli/cmdchromeprofile/chromeprofile_vss_windows.go`, `cli/fspath/lockcheck_windows.go`, `cli/cmdos/os_autologin_windows.go`, `cli/cmdpull/pull_remediation_fallback.go`, `cli/cmdpull/pull_efficient_render.go`, `cli/cmdpull/pull_remediation.go`, `cli/cmdfix/fix_cmd.go`).
- **Mechanism**:
  1. **Defender False Positive (`VirTool:MSIL/Subti.C`)**: `chromeprofile_vss_windows.go` invokes `vssadmin create shadow` / `vssadmin delete shadows /quiet` and references `GLOBALROOT\Device\HarddiskVolumeShadowCopy`; `lockcheck_windows.go` invokes hidden PowerShell `Get-CimInstance Win32_Process` and `taskkill /F`; `os_autologin_windows.go` manipulates `Winlogon` `DefaultPassword`.
  2. **Missing Credential/SSH Commands in `gitmap pa` & `gitmap fix`**: `ResolveDualPullRemediationHints` did not include `isWincredmanFailure(msg)` in `resolveStateOrAuthDualHints`, causing all 14 `wincredman`-failed repos to display only generic `gitmap fix <repo>` / `gitmap status <repo>` hints. And `gitmap fix` / `gitmap fix --all` only ran `git pull --no-rebase` over HTTPS without fixing `wincredman` or converting same-owner (`alimtvnetwork`) HTTPS remotes to SSH (`git@github.com:alimtvnetwork/<repo>.git`).
- **Architecture Fix**:
  1. Remove `vssadmin` shadow copy execution/strings, hidden PowerShell WMI/taskkill strings, and `DefaultPassword` registry strings so Windows Defender scans `gitmap.exe` with `0 threats found`.
  2. Update `pull_remediation_fallback.go` and `pull_efficient_render.go` to display explicit credential & SSH commands per repo and a unified single command at the bottom of `Failed Repositories`.
  3. Update `pull_remediation.go` and `fix_cmd.go` so `gitmap fix` and `gitmap fix --all` automatically run credential store remediation, convert same-owner HTTPS remotes to SSH (`git@github.com:<owner>/<repo>.git`), and pull cleanly over SSH.
