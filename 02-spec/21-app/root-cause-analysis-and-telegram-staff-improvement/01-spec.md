# Spec: Root Cause Analysis and Telegram Staff Improvement

## slug
root-cause-analysis-and-telegram-staff-improvement

## Status
ACTIVE — plan enqueued at `.ai-memory/plans/root-cause-analysis-and-telegram-staff-improvement.md`

## 1. Objective

Three durable outcomes:

1. **RCA memory**: Every significant recent failure and its fix is documented as a
   root-cause analysis in `.ai-memory/` with enough detail that any AI reading it
   will not repeat the mistake. Each RCA names the symptom, the exact root cause,
   how it was found, the fix (file:line, old → new), and the prevention rule.
2. **E2E testing playbook**: A step-by-step, checklist-driven playbook for
   end-to-end testing of AGM — auto-switch, instance open, prompt preview, prompt
   send, entering prompts — so that any AI told "perform the auto-switch end-to-end
   testing" or "perform the instance test testing" can execute without guessing.
3. **Telegram improvements**: New bot commands (`/open`, `/screenshot`, `/send`),
   per-machine targeting, colorful professional help with copy-paste command
   buttons, plus a Supabase health investigation and a voice-command feasibility
   note (analysis only, no implementation).

## 2. Scope

### 2.1 RCA documentation (IN SCOPE)
Document the following recent incidents as individual RCAs under
`.ai-memory/rca/` (new directory; index at `.ai-memory/rca/readme.md`):

| # | Incident | Root cause (one line) |
|---|----------|----------------------|
| 1 | AGM CI red: `cargo fmt` on `run_caught_prompt_task` signature | Long fn signature not wrapped; `cargo fmt --check` gate |
| 2 | AGM CI red: `E0061` on `send_telegram_message_with_markup` | Signature grew a 5th param; 3 call sites in `daemon.rs` missed |
| 3 | gitmap build red: `undefined: os` in `agent_claim_collision.go` | File split dropped the `os` import |
| 4 | gitmap build red: `undefined: io` / `undefined: apperror` in `install_archive_magic.go` | File split dropped `io` + `apperror` imports; unused `bytes` left in sibling |
| 5 | gitmap test red: `TestTopLevelCmdRegistryMatchesAST` (140 missing) | Split dropped `// gitmap:cmd top-level` marker on `constants_commands_extra.go` |
| 6 | gitmap test red: `TestEveryCmdIDHasHelpFile` (zero parsed) | `coverage_test.go` hardcoded old `constants_cli.go` path (now a stub) |
| 7 | gitmap test red (Windows): `TestEffectiveDataDir_ReadOnlyFallback` | POSIX `0555` not enforceable on Windows; needs `t.Skip` |
| 8 | gitmap CI workflow: `test-summary` marks failed SHAs as built | `if: success()` checks job steps, not pipeline; poisoned cache |
| 9 | gitmap CI workflow: E2E "Already validated" fails on re-run | Job-level `working-directory: cli` + skipped checkout = missing dir |
| 10 | AGM `gitmap pa` failure: merge conflict on `fix/ci-gates-green` | Local branch duplicated already-pushed remote fix; stale branch |

Each RCA file MUST contain: symptom (CI log excerpt), root cause, how it was
found (exact commands), the fix (file:line old → new), verification evidence,
and a prevention checklist item.

### 2.2 E2E testing playbook (IN SCOPE)
Write `.ai-memory/e2e-playbook/readme.md` plus per-flow files:

- `01-auto-switch.md` — auto-switch end-to-end: preconditions, steps, expected
  UI states, verification points.
- `02-instance-open.md` — opening an instance from AGM, expected window/process
  state.
- `03-prompt-preview.md` — prompt preview rendering before send.
- `04-prompt-send.md` — entering a prompt, sending to an instance, confirming
  arrival in the IDE conversation.
- `05-checklist.md` — the master checklist every AI must walk before reporting
  "E2E passed".

Each flow: purpose, preconditions (versions, machine state), numbered steps
with exact UI targets, expected result per step, failure branches (what to do
when a step fails), and evidence to capture (screenshot/log).

### 2.3 Telegram improvements (IN SCOPE)
Plan and implement in `src-tauri/src/modules/telegram_inbound/` + `notification_hub/`:

- `/open` — open the AGM UI on a target machine (launch if closed).
- `/screenshot [machine]` — capture the current screen of each (or a named)
  machine and send the image back to Telegram.
- `/send <machine/instance> <prompt>` — two-step flow: first pick the
  destination (machine → instance → conversation), then the prompt text goes
  exactly there. Destination must be explicit before any prompt is accepted.
- `/help` — colorful, professional help (Telegram HTML parse mode), grouped
  commands, copy-paste buttons (inline keyboard with copy-friendly command
  text) so the user can copy a command, edit the prompt part, and send.
- After a `/send`, the bot confirms by navigating to the conversation and
  sending a screenshot snippet as proof of delivery.

### 2.4 Supabase investigation (IN SCOPE)
- Reproduce the reported Supabase failure via the app's Supabase diagnostics.
- Root-cause it (connection, auth, schema, RLS, or client config).
- Document the RCA; fix if the fix is in-repo code (no credential changes).

### 2.5 Voice command (ANALYSIS ONLY — NOT IMPLEMENTATION)
- Feasibility note: Telegram voice → speech-to-text → prompt text → existing
  `/send` flow. Options (Telegram native transcription vs bot-side STT),
  latency/accuracy/cost trade-offs, and a recommended path. No code.

## 3. Out of scope
- Releasing or version-bumping either repo (separate user command).
- Touching `.github/workflows/` (standing no-workflow-changes rule) — workflow
  bugs are documented as RCAs with a recommended fix, not implemented.
- Deleting any repo, file, or branch (standing rule).

## 4. Constraints (non-negotiable)
- Codebase search exclusively via GitMap (`gitmap aum search`, `gitmap find`,
  `gitmap cat`, `gitmap ps`, `gitmap py`, `gitmap llm train`). TOTAL BAN on
  `rg`, `ripgrep`, `grep`, `git grep`, Select-String.
- Relative Git paths only (`02-spec/...`, `.ai-memory/...`, `src-tauri/...`).
  Never absolute paths in files, commits, or release notes.
- Use `gitmap` AI agents to enter data (task DB) where applicable.
- Targeted `git commit <exact paths>` + `git push`. NEVER `gitmap cpf`/`cpb`
  (they run `git add -A` internally and sweep other streams' files).
- Every fallible operation routes errors into the error module (repo
  "No Silent Errors" rule): Rust → `crate::error::record_ignored()` +
  `// Justification:` comment; TypeScript → `trackWarning()`/`captureError`.
- Attribution strictly `(Thanks to @aukgit)`; no other handles anywhere.

## 5. Acceptance criteria
- [ ] `.ai-memory/rca/readme.md` index + 10 RCA files, each with
      symptom/root-cause/how-found/fix/verification/prevention.
- [ ] `.ai-memory/e2e-playbook/` with 5 files; checklist is executable by an
      AI with no prior context.
- [ ] Telegram `/open`, `/screenshot`, `/send`, colorful `/help` with
      copy-paste buttons implemented; `cargo fmt` clean.
- [ ] Supabase RCA written; root cause identified (fix applied if in-repo).
- [ ] Voice feasibility note written (no code).
- [ ] All committed and pushed with targeted commits.
