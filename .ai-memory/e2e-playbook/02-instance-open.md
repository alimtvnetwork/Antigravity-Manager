# 02 — Instance open end-to-end

## Purpose

Verify that opening an instance from AGM launches (or focuses) the correct
Antigravity IDE window, that the OS shows exactly one window/process for
that instance, and that the window opens the instance's configured
workspace. "Opening" an instance in this app means **launching its IDE
window**: the CLI subcommands are `launch` / `start` / `run`
(`src-tauri/src/modules/cli/instance.rs`); the UI control is the
**"Launch Instance"** button on the instance card
(`src/pages/instances/InstanceCardActions.tsx`). There is **no**
`agm instances open` subcommand — using it must fail as an unknown command.

## Preconditions

- [ ] The `agm` binary under test is the build you intend to test
      (`agm --version`; must match the commit/tag under test). Run one
      command at a time.
- [ ] At least one configured instance exists, verified with
      `agm instances ls`. Record its id, seq number, and name.
- [ ] The IDE executable the instance points at is installed on this
      machine (the instance card shows the effective executable path; the
      launch fails with a clear error if the binary is missing).
- [ ] Prefer an **idle** instance for this test. Do not launch/stop the
      tester's live Default instance without their confirmation.
- [ ] The AGM desktop UI can be opened (`agm open-ui`), or note a CLI-only
      run.

## Key facts (verified in source)

- Native OS window title for an instance is computed by
  `compute_instance_window_title`
  (`src-tauri/src/modules/instance/settings.rs`) and **begins with**
  `#{seq} {name} - ` (e.g. `#1 default - ...`). Use this prefix to confirm
  the *right* window opened.
- On the instance card, a **gray status dot** = idle, a **green pulsing
  dot** = running (`src/pages/instances/InstanceCard.tsx`).
- The row-1 action capsule on the card has, in order, buttons whose tooltips
  read exactly: **"Launch Instance"** (teal Play icon, shown only when the
  instance is NOT running), **"Stop Instance"** (rose Square, shown when it
  IS running), **"Restart Instance on Current Account"**,
  **"Switch Account"**, **"Fast Forward to Next Best"**,
  **"Sync PID and Quota"**.
- While a card action runs, a busy overlay covers the card with a spinner
  and the action label.
- If the instance is already running, `launch` focuses the existing window
  rather than spawning a second process (backend
  `focus_or_launch_instance` in `src-tauri/src/modules/instance.rs`).

## Procedure

### Phase A — baseline

- [ ] **A1.** Run `agm --version`.
      **Expected:** version matches the build under test.
      **Evidence:** the version string.
      **If it fails:** do not continue; find the correct binary first.

- [ ] **A2.** Run `agm instances ls`.
      **Expected:** exit 0; the target instance is listed with its id, seq
      number, name, and running PID (or idle).
      **Evidence:** record id, `#seq`, name, and PID-or-idle.
      **If it fails:** exit 1 or the instance is missing → the instance is
      not registered; create/choose a different one. Do not invent the id.

- [ ] **A3 (safety).** If the target instance is currently running AND it is
      the tester's working IDE, stop and get confirmation before touching
      it. Launching is non-destructive, but stopping is not.

### Phase B — open from the UI

- [ ] **B1.** Run `agm open-ui`.
      **Expected stdout:** `[OK] Antigravity Manager UI launched (PID: ...)`
      or the window was already open and stays open.
      **Evidence:** PID or "already open".
      **If it fails:** spawn error with no fallback → record stderr and
      continue with the CLI path (Phase C).

- [ ] **B2.** In the AGM window, open the **Instances** page
      (the navigation entry that lists the instance cards; if it reads
      differently on your build, record its exact label here and use it for
      every later step).
      **Expected:** one card per instance from A2; the target card shows
      `#<seq>` and its name, with a **gray** status dot (idle).
      **Evidence:** screenshot of the Instances page.
      **If it fails:** card list does not match `agm instances ls` output →
      record the mismatch; the UI data source is stale or the window is an
      older build.

- [ ] **B3.** On the target card, hover the first button of the top action
      capsule.
      **Expected:** the tooltip reads exactly **"Launch Instance"** and the
      icon is a teal Play triangle.
      **Evidence:** screenshot with the tooltip visible.
      **If it fails:** the button slot shows **"Stop Instance"** (rose
      square) instead → the instance is already running; skip to Phase D
      (focus behavior) instead.

- [ ] **B4.** Click **"Launch Instance"**.
      **Expected:** a busy overlay with a spinner and action label covers
      the card briefly, then disappears; the card's status dot turns
      **green and pulsing**; the first capsule slot now shows
      **"Stop Instance"** (rose square) instead of "Launch Instance".
      **Evidence:** note the time the click happened; screenshot after the
      dot turns green.
      **If it fails:**
      - Overlay never clears / spinner stuck → the launch is hung; run
        `agm instances ls` to see whether a PID appeared anyway, then
        inspect `~/.antigravity_tools/logs/app.log`
        (Windows: `%USERPROFILE%\.antigravity_tools\logs\app.log`).
      - An error toast/dialog appears → record its text verbatim.

- [ ] **B5 (OS window check).** Look at the OS taskbar / window list.
      **Expected:** a new Antigravity IDE window appears whose title
      **begins with `#<seq> <name> - `** (e.g. `#2 default-copy-5597 - `),
      matching the card you clicked — this is the instance's injected
      `window.title` (`src-tauri/src/modules/instance/settings.rs`).
      **Evidence:** screenshot of the window with its title visible; record
      the full title prefix.
      **If it fails:** a window opens but the title prefix belongs to a
      different instance → **wrong instance opened**, FAIL the case and
      record both prefixes. No window at all → inspect the log tail.

- [ ] **B6 (process check).** Run `agm instances ls`.
      **Expected:** the target instance now shows a running PID (a number),
      and no other instance gained or lost a PID.
      **Evidence:** before/after PID values.
      **If it fails:** no PID after B4/B5 → launch did not register the
      process; check the log. Another instance's PID changed → unexpected
      side effect; record it.

- [ ] **B7 (workspace check).** In the opened IDE window, confirm the
      workspace that is open.
      **Expected:** the workspace root shown in the window (title bar /
      Explorer) matches the instance's configured workspace — compare
      against the projects listed on the instance card
      (`InstanceCardProjects`) or the instance's configured data path.
      **Evidence:** the workspace folder name as shown.
      **If it fails:** the IDE opened a different folder → record the
      expected vs. actual path; the instance's workspace binding is wrong.

### Phase C — open from the CLI (alternative or additional path)

- [ ] **C1.** Run `agm instances launch <id>` (aliases: `start`, `run`).
      **Expected stdout:** `[SUCCESS] Launched instance window for '<id>'.`
      and exit 0.
      **Evidence:** the exact line.
      **If it fails:**
      - `[ERROR] Failed to launch instance '<id>': ...` → record the reason
        verbatim; the common case is a missing IDE executable — verify the
        executable path on the instance card.
      - Hang (no output for 60 s) → kill it and record "HANG"; a spawned
        child may be inheriting stdio (see
        `src-tauri/src/modules/process/launch.rs` notes in
        `.ai-memory/memory/learned/27-e2e-instance-autoswitch-prompt-playbook.md`).

- [ ] **C2.** Repeat B5–B7 (window title prefix, PID, workspace) for the CLI
      launch and tick them off again.
      **If it fails:** follow the same failure branches.

- [ ] **C3 (negative).** Run `agm instances open <id>`.
      **Expected:** `[ERROR] Unknown instance subcommand 'open'.` and exit 1
      (this subcommand does not exist; `launch`/`start`/`run` are the valid
      ones).
      **Evidence:** the error line.
      **If it fails:** it launches something → the CLI gained an `open`
      alias since this was written; record the new behavior.

### Phase D — focus behavior when already running

- [ ] **D1.** With the target instance running (green dot, PID from B6/C),
      run `agm instances launch <id>` again (or click the card's launch
      path in the UI if available).
      **Expected:** the existing IDE window is brought to the front; **no
      second PID** appears in `agm instances ls` (the PID is unchanged).
      **Evidence:** PID before and after (must be identical).
      **If it fails:** a second PID appears → duplicate process spawned;
      record both PIDs. Nothing happens at all (no focus change) → record
      "no focus"; inspect the log.

### Phase E — cleanup (optional, gated)

- [ ] **E1 (gate).** Only stop the instance if the tester confirms it is
      safe (never stop their working Default silently).
- [ ] **E2.** Run `agm instances stop <id>`.
      **Expected stdout:** `[SUCCESS] Stopped instance '<id>'.`
      **Evidence:** the exact line; `agm instances ls` afterwards shows the
      instance idle and the card's dot back to gray.
      **If it fails:** `[ERROR] Failed to stop instance '<id>': ...` →
      record verbatim; verify the PID is actually gone before retrying.

### Evidence to capture

1. CLI transcript: `agm --version`, `agm instances ls` (before/after),
   `agm instances launch <id>` with exit codes and outputs.
2. Screenshot: Instances page — target card idle (gray dot) and running
   (green pulsing dot).
3. Screenshot: the **"Launch Instance"** tooltip on the card.
4. Screenshot: the opened IDE window with its `#<seq> <name> - ` title
   prefix visible.
5. Log excerpt: `app.log` lines around the launch timestamp
   (`~/.antigravity_tools/logs/app.log`; daily rolling).

### PASS/FAIL verdict rule

- **PASS** iff: the instance was launched exactly once per attempt (one new
  PID, no duplicates); the OS window title begins with `#<seq> <name> - `
  matching the target card; the PID in `agm instances ls` matches the
  running card (green dot); the opened workspace matches the instance's
  configured workspace; and the already-running focus case (D1) produced no
  second PID.
- **FAIL** if any of these hold: a window/process opened for the wrong
  instance (title prefix mismatch); a second PID was spawned for an
  already-running instance; no window or no PID appeared; the workspace is
  wrong; or any launch step hung with no output.
- Record the verdict as one line: `02-instance-open: PASS|FAIL — <id>,
  <window-title-prefix>, <pid>, <workspace>, <notes>`.
