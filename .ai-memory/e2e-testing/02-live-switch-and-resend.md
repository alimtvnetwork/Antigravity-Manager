# 02 — Live switch and prompt resend (v4.201.0)

Executed 2026-10-15 against `default-copy-5597`. Commit `f9b7d218`. Evidence came from `app.log`, the instance `agy` cli log, `repo_prompts.db`, and `ui_audit.db`.

Why the AI ran this: earlier auto-switch and post-switch send failed in practice. The run found five causes. Do not reintroduce them.

1. `agy -p` reads `antigravity-oauth-token` under the instance `.gemini/antigravity-cli` tree, not only `jetski-standalone-oauth-token`. Missing file: `Print mode: silent auth failed`, then a 60s OAuth hang.
2. HTTP 403 `VALIDATION_REQUIRED` on quota summary must set `validation_blocked`. `fetchAvailableModels` can still show 100%.
3. The idle queue must not select `backed_up`. That status belongs to post-switch restore. A scheduler that takes it mid-switch marks the prompt dispatched against a dead worker.
4. Restore must run in the same process as the switch, after relaunch is verified. A 5-second background sleep dies when the CLI exits.
5. Conversation id `-`, `p-*`, `proj-*`, and `queued-*` must not be passed as `--conversation`.

## Prepare

- [ ] Use the copy instance. Confirm Default's PID and do not kill it.
- [ ] `agm ui record start`

## TC01–TC06 isolation, switch, ff, audit

- [ ] `agm ui record status` shows recording.
- [ ] `agm ff -i default-copy-5597 --force`
  - Pass: skips accounts with `validation_blocked = true`. On this run it skipped `lisakathys23@gmail.com`.
  - Pass: selected a healthy candidate (`zihadxshohag2@gmail.com` or `rokixshohag1@gmail.com` on this host).
  - Pass: Default IDE PID is unchanged.
- [ ] `agm ui record list` then `agm ui record stop`. Pass: the ff action is in the audit export.
- [ ] `agm ui record clear` only after you copied the evidence you need.

## TC07–TC11 send, rerun, enqueue, resend after switch

- [ ] Send a short prompt to the copy only. The run used text `Reply with E2E_AUTO_RESEND_OK only` and got id `p-84d2e1bd`.
- [ ] `agm prompts running -i default-copy-5597` shows that id running or in flight. Do not paste the body in the report.
- [ ] `agm ff -i default-copy-5597 --force` while that prompt is in flight.
- [ ] In the app log for that day, confirm these lines, in order:
  - [ ] in-flight prompts moved from `running` to `backed_up` for `default-copy-5597` before switch
  - [ ] credentials injected for the new account
  - [ ] relaunch verified for `default-copy-5597`
  - [ ] `Dispatched agy execution` for the same prompt id
  - [ ] `STAGE_5_RESTORE` restored and dispatched that prompt
- [ ] In the copy instance `agy` cli log, confirm:
  - [ ] `silent auth succeeded`
  - [ ] authenticated as the account ff selected
  - [ ] `sending message` with a real conversation id (not `-`)
- [ ] Fail if the log says `Print mode: auth timed out` or `Conversation - not found`.
- [ ] Fail if the prompt is marked `dispatched` before relaunch verified.

## TC12–TC14 host tools (only if the user asked for fleet health)

- [ ] `gitmap pa` finishes with zero pull failures.
- [ ] `gitmap fix` does not rewrite unrelated repos. On the recorded run, `gitmap fix --all` converted same-owner remotes to SSH. Do not repeat `--all` unless the user asked.
- [ ] Windows Defender custom scan is optional and was clean on that host. Skip it unless the user asked.

## Stop

- [ ] Copy instance is the one that rotated. Default PID unchanged.
- [ ] Threshold still 15.0 if you touched it.
