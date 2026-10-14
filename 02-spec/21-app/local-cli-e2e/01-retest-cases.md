# Local CLI retest cases

Run these on the machine that holds the accounts. Use the `agm` binary built from the commit under test (`cargo build --bin agm -j 1` in `src-tauri`). The installed `agm` and the open manager window can be an older build. Do not score a case against that older binary.

Do not print tokens, passwords, or prompt bodies. Do not close the Default IDE or `agm-alim.exe`. Do not switch all vault accounts through Default. Do not leave the auto-switch threshold at 98.

## Prepare

1. `agm instances ls`. Record the two ids. On this host they are `default` (running) and `default-copy-5597` (idle unless a test started it).
2. `agm accounts ls`. Record the count. Do not copy the table into a chat.
3. Note the account bound to Default and the account bound to the copy. Call them Default-account and Copy-account in the result block.
4. Read `~/.gemini/google_accounts.json` field `active` only. It must match Default-account. That file is the shared login. A copy switch must not change it.

## Cases

### P1 — Tree

1. `agm prompts tree -i default` (running only).
2. `agm tree all --words 1 --json`.

Pass: the running-only tree may print no projects when every project is idle. The `all` tree exits 0 and the JSON `data` array has the project count (this host: 48). Do not paste prompt text.

### P2 — Enter a prompt on Default

From `src-tauri` (so `.git` is not in the working directory and the command does not `git pull`):

`agm prompt <C-seq> --instance default "Review the latest change in this repo."`

Pick `<C-seq>` from the `all` tree for the Antigravity-Manager project on `default` (this host used `C197`).

Pass: exit 0, stdout contains `[SUCCESS]`, and a new `agy.exe` command line contains `--conversation` and `-p` and does not contain `--resume` or `--user-data-dir`.

Fail: exit 0 when stderr says `agy did not accept the prompt`. That path must exit 1.

### P3 — Enter the same prompt on the copy

`agm prompt <same C-seq> --instance default-copy-5597 "Review the latest change in this repo."`

Pass: exit 0, `[SUCCESS]`, instance id in the line is the copy, and that `agy.exe` also has `--conversation` and no `--resume`.

### S1 — Switch the idle copy, then put it back

1. `agm account switch #2 <one other vault account>`
2. `agm instances ls`
3. `agm account switch #2 <Copy-account>`
4. `agm instances stop #2`
5. Read `active` in `~/.gemini/google_accounts.json` again.

Pass: both switches print `[SUCCESS]` and `Relaunch verified`. After step 2, Default still has the same PID and Default-account. The copy shows the other account. After step 4 the copy is Idle and bound to Copy-account. `active` is Default-account the whole time.

Fail: the shared login changes to the copy's account. Fail: Default's PID changes. Fail: the close log says the copy's own `Antigravity-<copy-id>.exe` was left running because it "belongs to another instance", and `instances ls` still shows that PID after `stop`.

Do not run `agm account switch` against Default. That closes the IDE the user has open.

### S2 — Vault size

`agm accounts ls` still lists the same count as Prepare (this host: 46). Switching the copy must not delete accounts.

### E1 — Email password stays in the vault

`agm email ls`

Pass: one account and one recipient. The output has no password and no token.

### B1 — Auto-switch threshold

1. `agm auto-switch threshold 98` prints `98.0%`.
2. `agm auto-switch threshold 15` prints `15.0%`.
3. Stop. Do not run `agm auto-switch test 98` against the live Default IDE.

`agm auto-switch status --json` does not include the threshold field. The success line of the `threshold` command is the proof.

## Last run (2026-10-14)

```text
Binary: src-tauri/target/debug/agm.exe after this commit. Version string may still say an older hash until build.rs reruns.
P1: PASS. Running-only tree empty. tree all JSON data length 48. Cold about 1.6s.
P2: PASS. exit 0. agy pid started. --conversation present. --resume absent. --user-data-dir absent.
P3: PASS. Same flags on a second agy pid. Instance in the success line was the copy.
S1: PASS after the shared-login fix. First switch of the copy rewrote ~/.gemini; the restore switch wrote the Default account back and relaunch verified. stop #2 left the copy Idle. Default PID unchanged.
S2: PASS. 46 accounts before and after.
E1: PASS. One SMTP account, one recipient, no password printed.
B1: PASS. Set to 98.0% then back to 15.0%. Daemon is_running false.
Window Send button: NOT CLICKED. The open manager is the installed v4.198.0 GUI, not this debug build.
```
