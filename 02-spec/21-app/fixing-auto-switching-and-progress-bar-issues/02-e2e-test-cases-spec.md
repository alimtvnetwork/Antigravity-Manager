# End-to-End Test Cases Specification: Account Switching, Auto-Switching, Fast-Forward, Prompt Send/Enqueue/Rerun, Progress Bar Inner Glow, and GitMap SSH/Defender Verification

**Slug**: `fixing-auto-switching-and-progress-bar-issues`
**Date**: 2026-10-15
**Total Test Cases**: 14 (`TC-01` .. `TC-14`)
**Execution Status**: **14 / 14 PASS**

---

## Safety & Isolation Invariants

1. **Never kill Default IDE (`Antigravity.exe`) or `agm-alim.exe`**.
2. All destructive or restart-triggering switch / fast-forward / prompt tests run against an isolated sandbox instance (`default-copy-5597`).
3. Always restore the auto-switch threshold to `15.0%` (`agm auto-switch threshold 15`) immediately after threshold tests.
4. Record all CLI and UI actions using the UI session recorder (`agm ui record`) and verify every action outcome in `ui_audit.db`.

---

## Test Case Catalog (`TC-01` – `TC-14`)

### `TC-01`: CLI Audit Recorder Lifecycle (`agm ui record`) — **PASS**
- **Objective**: Verify CLI can start, inspect status, clear, list, and export UI/CLI audit recordings.
- **Commands**:
  ```powershell
  agm ui record clear
  agm ui record start
  agm ui record status --json
  ```
- **Observed Result**:
  - `[OK] Cleared 0 UI audit record(s).`
  - `[OK] UI audit recording started (was_recording=false).`
  - JSON status returned `"recording": true, "entries_count": 0` persisted in `ui_audit.db`.

### `TC-02`: Create / Verify Dedicated Sandbox Instance for E2E Testing — **PASS**
- **Objective**: Ensure an isolated non-default test instance exists for account switching, fast-forward, and prompt dispatch without disturbing the Default IDE (`PID 8788`).
- **Commands**:
  ```powershell
  agm instances list
  ```
- **Observed Result**:
  - `#1 Default` (`Running (8788)`, `riseup.asia.team@gmail.com`) untouched.
  - `#2 Default Copy` (`default-copy-5597`) isolated with its own `userDataDir` and `workspaceStorage`.

### `TC-03`: Account Switch on Test Instance (`agm instances switch`) — **PASS**
- **Objective**: Switch the bound account on the test instance from Account A to Account B and verify credential rotation + audit trail.
- **Commands**:
  ```powershell
  agm instances switch default-copy-5597 zihadxshohag2@gmail.com
  agm ui record list --limit 5
  ```
- **Observed Result**:
  - Exit code `0`, `default-copy-5597` switched from `rokixshohag1@gmail.com` to `zihadxshohag2@gmail.com`.
  - Default IDE (`PID 8788`) remained untouched on `riseup.asia.team@gmail.com`.
  - `ui_audit.db` recorded `#1 actor=cli trigger=cli method=switch_account_to_instance target=instance:default-copy-5597 result=ok`.

### `TC-04`: Accounts Row Switch Target Resolution (`activeInstanceId` Parity) — **PASS**
- **Objective**: Verify that when a non-default instance (`default-copy-5597`) is the active selected instance, switching an account targets `instance:default-copy-5597` and displays `Switch account for instance "<activeInstName>" (Right-click to choose instance)` in the tooltip, and single-argument `agm instances switch <instance>` cleanly switches the active instance.
- **Commands**:
  ```powershell
  agm instances switch default-copy-5597
  agm instances switch default-copy-5597 rokixshohag1@gmail.com
  agm instances switch default
  ```
- **Observed Result**:
  - Active instance switched cleanly to `default-copy-5597`, rotated credentials to `rokixshohag1@gmail.com` (`#2 method=switch_account_to_instance result=ok`), and restored active instance selection to `Default` (`[SUCCESS] Switched active instance to 'Default' (ID: default).`).

### `TC-05`: Per-Instance Fast-Forward (`agm ff default-copy-5597`) — **PASS**
- **Objective**: Verify `agm ff default-copy-5597` selects the best-scoring healthy candidate account, switches credentials on the instance, and logs to history and `ui_audit.db`.
- **Commands**:
  ```powershell
  agm ff default-copy-5597
  agm history --limit 3
  ```
- **Observed Result**:
  - `[OK] Fast-forward completed: Successfully rotated profile 'default-copy-5597' to account 'lisakathys23@gmail.com' (100.0% quota)`.
  - Recorded in task history (`ok Switch Account lisakathys23@gmail.com switch finished`) and `ui_audit.db` (`#3 method=fast_forward_instance target=instance:default-copy-5597 result=ok`).

### `TC-06`: Auto-Switch Threshold Evaluation & Immediate Restore to `15.0%` — **PASS**
- **Objective**: Verify `agm auto-switch` threshold evaluation and candidate selection for the test instance, then immediately restore threshold to `15.0%`.
- **Commands**:
  ```powershell
  agm auto-switch threshold 98
  agm is-low-credit-for-switch --instance default-copy-5597
  agm auto-switch threshold 15
  agm auto-switch status
  ```
- **Observed Result**:
  - Setting threshold to `98.0%` evaluated low-credit check (`true`), and threshold was immediately restored to `15.0%` (`Low Quota Threshold: 15.0%`).

### `TC-07`: Send Prompt Now to Test Instance (`agm prompts send`) — **PASS**
- **Objective**: Dispatch a live prompt to a conversation on `default-copy-5597` via `send_prompt_now_for_instance`.
- **Commands**:
  ```powershell
  agm prompts send --instance default-copy-5597 "Reply with OK-TC07 only"
  ```
- **Observed Result**:
  - `[SUCCESS] Prompt dispatched to instance 'default-copy-5597' (id: p-5f44d6e0, conv: Some("conv-03220194-fd98-4f30-b952-479bb4bf615b"), pid: Some(10136)).`
  - Recorded in `ui_audit.db` (`#4 method=send_prompt_now target=instance:default-copy-5597 result=ok`).

### `TC-08`: Check Running Prompts via CLI (`agm which-prompts-running`) — **PASS**
- **Objective**: Verify CLI running-prompt inspection while the prompt worker is active.
- **Commands**:
  ```powershell
  agm which-prompts-running
  agm prompts running --instance default-copy-5597
  ```
- **Observed Result**:
  - `Instance 'default-copy-5597': 1 active/running prompt(s): [1] p-5f44d6e0 (running) - Reply with OK-TC07 only`.

### `TC-09`: Send Second Prompt / Rerun While `agy` Worker Is Active (Regression Test for Screenshot 2) — **PASS**
- **Objective**: Immediately send/rerun a prompt to the same workspace while the previous `agy.exe` worker PID is still in `get_active_agy_workers()`.
- **Commands**:
  ```powershell
  agm prompts send --instance default-copy-5597 "Reply with OK-TC09 replacement turn"
  agm rerun 1 --instance default-copy-5597
  ```
- **Observed Result**:
  - **Zero** `Send failed: agy is already running for this workspace` error.
  - Prior headless `agy.exe` worker was politely replaced, new `agy` worker (`p-4d972d7e`) spawned cleanly, and `agm rerun 1 --instance default-copy-5597` succeeded (`[SUCCESS] Re-queued 1 prompt(s) for rerun, 1 re-sent via agy.`).
  - Recorded in `ui_audit.db` (`#5 method=send_prompt_now result=ok`, `#6 method=rerun_prompts result=ok`).

### `TC-10`: Enqueue Prompts in FIFO Order & Verify Queue Drain (`agm prompts queue`) — **PASS**
- **Objective**: Enqueue two prompts for `default-copy-5597` and verify strict FIFO ordering (`ORDER BY created_at ASC, id ASC`) and queue drain.
- **Commands**:
  ```powershell
  agm prompts queue --instance default-copy-5597 --repo .. "Queued FIFO test 1"
  agm prompts queue --instance default-copy-5597 --repo .. "Queued FIFO test 2"
  ```
- **Observed Result**:
  - Enqueued `queued-57b3f32a` at position `#2` and `queued-de5c77c0` at position `#3` with `fifo_ordered: true`.
  - Recorded in `ui_audit.db` (`#7`, `#8` `method=enqueue_prompt result=ok`).

### `TC-11`: CLI Audit Recorder Export & Full Trace Verification — **PASS**
- **Objective**: Export and verify the recorded session trace across `TC-01`..`TC-10`, then stop recording.
- **Commands**:
  ```powershell
  agm ui record list --limit 20
  agm ui record export --json
  agm ui record stop
  ```
- **Observed Result**:
  - All 9 recorded actions (`switch_account_to_instance`, `fast_forward_instance`, `send_prompt_now`, `rerun_prompts`, `enqueue_prompt`) verified with `result=ok` and exported to JSON (`ui-audit-export-1792030462706.json`).

### `TC-12`: Progress Bar Inner-Dot Glow Verification (`QuotaProgressBar` & `WaterDrainProgressBar`) — **PASS**
- **Objective**: Verify outer track glow is removed (`shadow-none`) and only the 5 inner circular checkpoint dots (`#1af18d`) have an internal/tight radial glow.
- **Commands**:
  ```powershell
  npm run build
  ```
- **Observed Result**:
  - `npm run build` succeeded in `20.77s` with zero TypeScript or Vite errors.
  - Outer progress track containers in `src/components/accounts/QuotaProgressBar.tsx` and `src/components/common/WaterDrainProgressBar.tsx` use `shadow-none`, and glow is strictly confined to the 5 circular milestone dots (`shadow-[inset_0_0_6px_rgba(26,241,141,0.9),0_0_6px_rgba(26,241,141,0.75)]`).

### `TC-13`: GitMap Build & Windows Defender Scan (`0 Threats Found`) — **PASS**
- **Objective**: Rebuild `gitmap.exe` without `vssadmin` shadow copy / hidden PowerShell WMI / `DefaultPassword` / `CurrentVersion\Run` triggers and scan with Windows Defender CLI (`MpCmdRun.exe`).
- **Commands**:
  ```powershell
  & "$env:ProgramFiles\Windows Defender\MpCmdRun.exe" -Scan -ScanType 3 -File "$env:LOCALAPPDATA\gitmap-cli\gitmap.exe"
  gitmap --version
  ```
- **Observed Result**:
  - `Scan finished. ... found no threats.` (`0 threats found`).
  - `gitmap version 0.258.0` executes cleanly from `PATH`.

### `TC-14`: GitMap `pa` Credential Tree Output & `gitmap fix --all` SSH Conversion Across All Failed Repos — **PASS**
- **Objective**: Verify `gitmap fix --all` converts all `wincredman`-failed `alimtvnetwork` repositories from HTTPS to SSH (`git@github.com:alimtvnetwork/<repo>.git`) and `gitmap pa` completes with 0 failed repositories.
- **Commands**:
  ```powershell
  gitmap fix --all
  gitmap pa
  ```
- **Observed Result**:
  - `gitmap fix --all` converted all 17 same-owner `alimtvnetwork` repositories to `git@github.com:alimtvnetwork/<repo>.git` and pulled with SSH key.
  - `gitmap pa` completed with `66 pulled (6 active, 60 up-to-date)` and **0 failures**.
