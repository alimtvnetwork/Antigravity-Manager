# 01 — Auto-switch end-to-end

## Purpose

Verify that Antigravity-Manager (AGM) automatically switches the active IDE
profile/account to the next best candidate when quota or focus conditions
trigger it, and that the switch is observable in the UI, in the CLI, and in
the logs. This flow covers the auto-profile-switcher daemon
(`src-tauri/src/modules/auto_switcher/`), its Settings panel
(`src/components/settings/AutoSwitcherSettings.tsx`), and the
`agm auto-switch` CLI (`src-tauri/src/modules/cli/auto_switch.rs`).

Two levels are tested:

1. **Decision-only (safe):** the daemon/CLI evaluates quota and reports its
   rotation decision without changing the live profile.
2. **Real rotation (gated):** the daemon performs an actual account switch.
   This is behind an explicit safety gate — never run it on a machine where
   the active Default instance is the tester's working IDE without their
   confirmation.

## Preconditions

- [ ] The `agm` binary under test is the build you intend to test. Confirm
      with `agm --version` and compare against the commit/tag under test.
      Run one command at a time (this app is memory-sensitive: parallel
      builds plus heavy scans have crashed WebView2 with `0xe0000008`).
- [ ] At least **2 configured instances** on the test machine, verified with
      `agm instances ls`. Record their ids (on the reference host: `default`
      and `default-copy-5597`).
- [ ] At least one of those instances is bound to an account whose quota
      state is known (fresh vs. near-empty), so a rotation decision is
      meaningful.
- [ ] The AGM desktop UI can be opened on this machine (`agm open-ui`), or
      note that the UI steps are skipped (CLI-only run).
- [ ] A scratch notes file is open to record one line per step:
      command, exit code, one observed number/status, PASS or FAIL.

## Notes the tester must not skip

- `agm auto-switch test <percent>` is a **dry decision**. It evaluates and
  reports but must not leave the saved threshold changed.
- A threshold you set yourself (e.g. `98`) **stays** until you set it back.
  The product default is `15.0` (`src-tauri/src/models/config.rs`:
  `low_quota_threshold_percent: 15.0`). End every run with the threshold
  back at `15.0`.
- `--json` output must be pure JSON on stdout. A log line on stdout with
  `--json` is a failure (`src-tauri/src/modules/logger.rs` routes tracing
  to stderr when `--json`/`-j` is passed).
- Do not print tokens, passwords, or prompt bodies in the result block.
  Record ids, counts, timings, and status words only.

## Procedure

### Phase A — baseline (read-only)

- [ ] **A1.** Run `agm --version`.
      **Expected:** a version string matching the build under test.
      **Evidence:** paste the version into the notes file.
      **If it fails:** the binary is stale or missing. If the command is not
      found, check the install location / `PATH`. Do not score later steps
      against an older binary.

- [ ] **A2.** Run `agm instances ls`.
      **Expected:** exit 0; at least 2 profiles listed with ids and running
      PIDs (or idle).
      **Evidence:** record both ids and which one is running.
      **If it fails:** exit 1 or empty list → run
      `agm auto-switch status` (next step) to see whether the whole CLI is
      broken, or check `~/.antigravity_tools/logs/app.log` for startup
      errors (Windows: `%USERPROFILE%\.antigravity_tools\logs\app.log`;
      if the data dir was moved, the `ABV_DATA_DIR` env var or the location
      pointer file takes precedence — see
      `src-tauri/src/modules/account/paths.rs`).

- [ ] **A3.** Run `agm auto-switch status --json`.
      **Expected:** valid JSON on stdout containing `is_enabled`,
      `low_quota_threshold_percent`, `active_instance_id`,
      `active_account_email`, `current_quota_percent`, `last_switch_reason`.
      **Evidence:** record `is_enabled` and `low_quota_threshold_percent`.
      **If it fails:**
      - JSON parse error / log lines on stdout → FAIL the JSON-parity case.
      - Process hangs → kill it, note "hang", then inspect
        `~/.antigravity_tools/logs/app.log` (tail 200 lines) for the last
        auto-switcher entry before the hang.

### Phase B — enable daemon and check the Settings panel

- [ ] **B1.** Run `agm auto-switch enable`.
      **Expected stdout:** `[SUCCESS] Auto-profile switcher daemon is now ENABLED.`
      **Evidence:** the exact line.
      **If it fails:** `[ERROR] Failed to save config: ...` → check that the
      config file under the data dir is writable
      (`~/.antigravity_tools/` default). Restore the previous enabled state
      with `agm auto-switch disable` if the test does not need it on.

- [ ] **B2.** Run `agm open-ui`.
      **Expected stdout:** `[OK] Antigravity Manager UI launched (PID: ...)`,
      or the window was already open and stays open.
      **Evidence:** PID or "already open" note.
      **If it fails:** spawn error with no fallback → record stderr; CLI-only
      run is still valid for the remaining CLI steps.

- [ ] **B3.** In the AGM window, open **Settings** and find the section titled
      **"Auto Profile Switcher & Task Resumption"** (exact section title from
      `src/components/settings/AutoSwitcher/SwitcherHeader.tsx`).
      **Expected:** the section shows a toggle switch in the ON position;
      below it a status row reading **"Next evaluation in Ns"** counting down,
      a pill reading **"Stage: Normal"** (or Caution/Critical),
      **"Quota: N%"**, and **"Monitored: N"** (exact labels from
      `src/components/settings/AutoSwitcher/PollingSettings.tsx`).
      **Evidence:** screenshot of this panel.
      **If it fails:** the toggle is OFF after B1 → record; toggle it ON by
      clicking the switch (the peer-checked toggle in `SwitcherHeader.tsx`).
      If the panel is missing entirely → the window is an older build; go
      back to A1.

- [ ] **B4.** In the same panel, locate the **"Low Quota Threshold (%)"**
      slider and read its value (displayed as **"< N%"**).
      **Expected:** the value matches the CLI baseline from A3
      (e.g. `< 15%`).
      **Evidence:** record the number.
      **If it fails:** UI and CLI disagree → record both numbers; do not
      proceed to Phase D until you know which one the daemon actually uses
      (re-run A3 to confirm).

### Phase C — decision-only test (safe, no rotation)

- [ ] **C1.** Run `agm auto-switch test 98` with a 30-second timeout.
      **Expected:** exit 0; a candidate decision or a clear "no candidate"
      message; **no hang** when output is piped.
      **Evidence:** the decision line and the wall-clock time.
      **If it fails:**
      - Still running after 30 s → kill it and record "HANG". This has
        regressed before when spawned children inherited stdio
        (see `src-tauri/src/modules/process/launch.rs` job-breakaway
        notes). Check the log for the evaluation entry and note the
        regression explicitly.
      - `[ERROR] ...` → record the error verbatim; the daemon cannot reach
        quota data (accounts or network).

- [ ] **C2.** Run `agm auto-switch status --json` again.
      **Expected:** `low_quota_threshold_percent` is **unchanged** from A3
      (the `test` action is dry and must not rewrite the threshold).
      **Evidence:** the number.
      **If it fails:** the threshold moved → record before/after; the dry
      path is writing config when it should not.

### Phase D — real rotation (GATED)

> **Safety gate.** Do not run D1–D5 if the active instance is the tester's
> working IDE and they have not confirmed. The gate question is:
> *"Is it acceptable for the active profile to switch accounts right now?"*
> A "no" ends the flow at C2 — the flow still gets a verdict (see below).

- [ ] **D1 (gate).** Tester confirms rotation is acceptable on this machine.
      Record the answer (yes/no) and who gave it.

- [ ] **D2.** Run `agm auto-switch threshold 98`.
      **Expected stdout:** `[SUCCESS] Auto-switch low quota threshold set to 98.0%.`
      **Evidence:** the exact line.
      **If it fails:** `[ERROR] Invalid numeric threshold value` → the arg
      was malformed; retry with a plain number. Any other error → inspect
      `~/.antigravity_tools/logs/app.log`.

- [ ] **D3.** Run `agm auto-switch run`.
      **Expected stdout:**
      `[*] Triggering immediate auto-switch evaluation across instances...`
      followed by
      `[SUCCESS] Auto-switch rotation triggered: <reason>`
      (with the threshold at 98%, a rotation decision is expected).
      **Evidence:** the full reason string.
      **If it fails:**
      - `[INFO] Quota healthy across all monitored instances. No rotation required.`
        at 98% → suspicious; record it as FAIL for the rotation case (the
        threshold clearly should have triggered an evaluation).
      - `[ERROR] Auto-switch evaluation failed: ...` → record verbatim and
        inspect the log tail.

- [ ] **D4.** Run `agm auto-switch status --json`.
      **Expected:** `active_instance_id` and/or `active_account_email` now
      point at the rotation candidate (different from A3), and
      `last_switch_reason` is set.
      **Evidence:** before/after values.
      **If it fails:** no change → the rotation claimed success but wrote
      nothing; inspect the log and the instance's bound account
      (`agm instances ls` / the instance card in the UI).

- [ ] **D5.** In the AGM window, open Settings → **"Auto Profile Switcher &
      Task Resumption"** and also the Instances page.
      **Expected:** the panel's status row and the instance card's bound
      account show the new account; the affected instance card still shows a
      running state if it was running before.
      **Evidence:** screenshots of both.
      **If it fails:** UI does not reflect the CLI → refresh the window;
      if still stale, record the mismatch.

- [ ] **D6 (UI manual path).** In the Settings panel, scroll to the
      **"Test Failover:"** row ("Manually rotate the IDE to the next best
      profile candidate right now.") and click **"Rotate Now"**.
      **Expected:** the button label changes to **"Rotating..."** while it
      runs, then a feedback line appears with the result message (a green
      check icon on success, a red alert icon starting with "Error" on
      failure — see `src/components/settings/AutoSwitcher/ManualRotation.tsx`).
      **Evidence:** the feedback message text.
      **If it fails:** button stays "Rotating..." → note the hang; "Error"
      feedback → record the message verbatim and inspect the log.

- [ ] **D7 (restore).** Run `agm auto-switch threshold 15`, then
      `agm auto-switch status --json`.
      **Expected:** `[SUCCESS] Auto-switch low quota threshold set to 15.0%.`
      and the status read-back shows `15.0`.
      **Evidence:** the read-back number.
      **If it fails:** value is not `15.0` → this is a hard FAIL; leaving a
      test threshold live makes the daemon rotate on the next tick.

### Final verification (UI state + logs)

- [ ] Threshold is back at `15.0` (D7 or C2 read-back).
- [ ] `agm auto-switch status` (text form) shows the
      **"AGM Auto-Profile Switcher Status"** block with
      **Daemon Status: RUNNING (Active)**,
      **Low Quota Threshold: 15.0%**, and (if Phase D ran)
      **Last Switch Reason: <reason>**.
- [ ] The day's `app.log` under the data-dir `logs/` folder
      (`~/.antigravity_tools/logs/app.log`, daily rolling) contains the
      rotation evaluation and its reason around the D3 timestamp.

### Evidence to capture

1. CLI transcript: every command above with exit code and output.
2. `agm auto-switch status --json` before (A3) and after (D4/D7).
3. Screenshot: Settings → "Auto Profile Switcher & Task Resumption" panel.
4. Screenshot: Instances page showing the affected instance card.
5. Log excerpt: `app.log` lines spanning the D3 evaluation.

### PASS/FAIL verdict rule

- **PASS** iff all of these hold: every executed step is ticked with its
  expected output observed (not inferred from source); no step hung; the
  dry `test` did not change the saved threshold; the saved threshold reads
  `15.0` at the end; and, if Phase D ran, `last_switch_reason` is set and
  the log contains the rotation record.
- **FAIL** if any executed step misses its expected output, any step hangs,
  the threshold is not `15.0` at the end, or the gate answer (D1) was "no"
  while a real rotation was nevertheless attempted.
- If D1 was "no", the verdict is **PASS (decision-only)** provided Phases
  A–C all passed and the threshold is unchanged; record "real rotation not
  attempted (gate declined)" in the notes.
