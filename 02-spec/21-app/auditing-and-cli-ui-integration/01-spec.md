# Spec: Auditing and CLI-UI Integration

## slug
auditing-and-cli-ui-integration

## Status
ACTIVE — plan enqueued at `.ai-memory/plans/auditing-and-cli-ui-integration.md`

## 1. Objective

Two durable capabilities plus release:

1. **UI interaction audit recording**: when recording is started, every UI
   interaction (click, navigation, input, scroll) is logged with the method
   name invoked, the UI target (page / component / element id), how it was
   triggered, and a timestamp — forming an audit trail usable to reproduce
   and fix bugs. Extends the instance audit-trail concept from spec 104 to
   all UI interactions.
2. **CLI drives the UI**: the `agm` CLI gains commands to open the UI,
   navigate to any page / settings page / settings subpage, list (`ls`) the
   UI element tree of the current view, scroll to and focus any element —
   with help text that tells the user exactly what to do.

Then: Telegram follow-up enhancements, and a minor release with CI green.

## 2. Background (existing specs)
- `02-spec/21-app/104-instance-audit-trail-and-pid-quota-sync.md` — instance
  action audit trail modal (switch history, 4-step lifecycle). The new
  interaction recorder generalizes this: instead of only switch lifecycle
  steps, record every UI interaction while recording is on.
- `.agents/skills/agm-task-history-and-audit/skill.md` — task history/audit
  skill (consult for conventions).

## 3. Scope

### 3.1 Interaction audit recorder (IN SCOPE)
- A recorder service (Rust backend + frontend hook) with start/stop/clear/
  export. While active, each interaction appends an audit entry:
  `timestamp, actor (user/cli/telegram), method_name, ui_target
  (route + component + element id), trigger (click/keyboard/programmatic),
  params, result (ok/error + message)`.
- Entries persisted (SQLite, per spec-104 storage conventions) and viewable
  in an Audit modal/panel; export to JSON/JSONL for bug reports.
- Recording indicator in the UI while active. No silent failures — recorder
  errors go to the error module.

### 3.2 CLI-UI bridge (IN SCOPE)
New `agm` CLI commands (in `src-tauri/src/bin/agm/`, new file
`ui_cmds.rs` following the existing `*_cmds.rs` pattern):
- `agm ui open` — open the AGM UI (launch if closed, focus if running).
- `agm ui goto <route>` — navigate to any page / settings page / settings
  subpage (e.g. `settings`, `settings/supabase`, `instances`, `accounts`).
  Unknown routes list the valid ones instead of failing silently.
- `agm ui ls [selector]` — list the UI element tree of the current view
  (route, components, element ids, focusable elements). This is the "ls on
  any UI".
- `agm ui scroll <element-id|selector>` — scroll the view so the element
  is visible, then focus it.
- `agm ui click <element-id>` — activate an element (same path as a user
  click, so the audit recorder logs it with `actor=cli`).
- `agm ui --help` — help text describing every subcommand with examples.
  The help must tell the user what to do, not just list flags.

### 3.3 Telegram follow-ups (IN SCOPE)
- Wire the new UI capabilities into the Telegram bot where sensible
  (`/open` already exists from the previous program — extend with
  `/ui goto`, `/ui ls` style passthrough or bot-native equivalents).
- Any gaps found in the previous program's `/send`, `/screenshot`, `/help`
  (e.g. from the E0015 / not-Send fixes) get hardened.

### 3.4 Release (IN SCOPE — explicitly requested)
- Minor version bump via the repo's release flow, changelog, release notes.
- CI watched until green on the release commit; release published.

## 4. Out of scope
- Touching `.github/workflows/` (standing rule) — document, don't implement.
- Deleting repos/files/branches (standing rule).
- The macro-ahk-v55 clone/chat/read is a parallel workstream (see plan).

## 5. Constraints (non-negotiable)
- Codebase search exclusively via GitMap. TOTAL BAN on rg/ripgrep/grep/
  git grep/Select-String.
- Relative Git paths only; lowercase filenames.
- Targeted `git commit <exact paths>` + `git push`. NEVER `gitmap cpf`/`cpb`.
- No silent errors (error module + justification comments).
- Attribution strictly `(Thanks to @aukgit)`.
- `cargo fmt` clean; release flow per repo convention (`npm run bump minor`
  etc. — coordinator to confirm from repo files).
