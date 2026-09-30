# 85 — Instance Switch Prompt Continuity, Update Integrity, UI Fixes, Email Dedupe, and Supabase/Vault Scripts

Status: planned; all open questions resolved 2026-09-30 (see "Decisions Confirmed"); frontend test runner already added
Raised: 2026-09-30
Owner plans: [85](../../.ai-memory/plans/pending/85-ui-tier-badge-priority-highlight-focus-and-stack-trace.md), [86](../../.ai-memory/plans/pending/86-update-integrity-asset-verification-and-release-gates.md), [87](../../.ai-memory/plans/pending/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler.md), [88](../../.ai-memory/plans/pending/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts.md)
Related: [01 unified switch](./01-unified-switch-fast-forward-autoswitch-spec.md), [71 supabase hierarchy](./71-supabase-multi-machine-hierarchy-and-cli-e2e.md), [72 local E2E](./72-local-e2e-instance-switching-and-prompt-restore.md), [73 heartbeat](./73-real-time-running-prompt-heartbeat-instance-e2e-and-ssh-key-management.md), [74 UI/email fixes](./74-ui-email-telegram-fixes-revisit.md), pending plan [75](../../.ai-memory/plans/pending/75-multi-instance-switching-prompt-backup-and-hygiene.md)

This is one intake spec for ten reported problems. They are split into four plans so each plan is a single problem class and each can ship as its own reviewable PR (repo rule: PR scope and grouping). All four plans cite this file.

## User Request (Verbatim)

### Message 1 — problem report (error diagnostic + speech-to-text request)

The four screenshots attached to this message are referenced under "Attachments". Emails in them are blurred by the user's request; no email is reproduced in this repository.

`````text
# Comprehensive Error Diagnostic Report

**Application:** Agm Tool By Alim (v4.108.0)

**Error ID:** 5e6750e2-3b9c-44b0-aaac-2d62b868bf91

**Code:** E9001

**Severity Level:** ERROR

**Captured At:** 2026-09-30T10:13:04.133Z

---

## 1. Error Message & Details

**Message:**
Failed to fetch updates for Stable channel. Please check network/proxy settings.

## 2. Trigger & Location

- **Route / Page:** `/accounts`

- **Trigger Action:** `tauri_invoke`

- **Source:** `tauri.check_for_updates`

## 3. Network / IPC Request

- **Method:** `INVOKE`

- **Endpoint:** `check_for_updates`

## 4. User Interaction Flow

```
div#root "Antigravity Tools
v4.103.0
0
Accounts
#1" [xpath: //*[@id='root']]
```

1. [6:13:02 PM] click on `div` ("Antigravity Tools
v4.103.0
0
Accounts
#1")

## 7. Frontend Stack Trace

- `http://tauri.localhost/assets/index-Lnket4v2.j` at s:535:10047 

**Raw Stack:**
```
Error: Failed to fetch updates for Stable channel. Please check network/proxy settings.
    at dP (http://tauri.localhost/assets/index-Lnket4v2.js:14:243)
    at Object.captureError (http://tauri.localhost/assets/index-Lnket4v2.js:14:1232)
    at ge (http://tauri.localhost/assets/index-Lnket4v2.js:14:13717)
    at async checkForUpdates (http://tauri.localhost/assets/index-Lnket4v2.js:144:219567)
    at async http://tauri.localhost/assets/index-Lnket4v2.js:535:10047
```

## 8. Invocation Chain

1. `http://tauri.localhost/assets/index-Lnket4v2.j`

## 9. Full Context Payload

```json
{
  "source": "tauri.check_for_updates",
  "triggerAction": "tauri_invoke"
}
```

## 10. Raw Error Object (JSON)

```json
{
  "id": "5e6750e2-3b9c-44b0-aaac-2d62b868bf91",
  "code": "E9001",
  "level": "error",
  "message": "Failed to fetch updates for Stable channel. Please check network/proxy settings.",
  "createdAt": "2026-09-30T10:13:04.133Z",
  "context": {
    "source": "tauri.check_for_updates",
    "triggerAction": "tauri_invoke"
  },
  "file": "s",
  "line": 535,
  "function": "http://tauri.localhost/assets/index-Lnket4v2.j",
  "stackTrace": "Error: Failed to fetch updates for Stable channel. Please check network/proxy settings.\n    at dP (http://tauri.localhost/assets/index-Lnket4v2.js:14:243)\n    at Object.captureError (http://tauri.localhost/assets/index-Lnket4v2.js:14:1232)\n    at ge (http://tauri.localhost/assets/index-Lnket4v2.js:14:13717)\n    at async checkForUpdates (http://tauri.localhost/assets/index-Lnket4v2.js:144:219567)\n    at async http://tauri.localhost/assets/index-Lnket4v2.js:535:10047",
  "parsedFrames": [
    {
      "function": "http://tauri.localhost/assets/index-Lnket4v2.j",
      "file": "s",
      "line": 535,
      "column": 10047,
      "isInternal": false
    }
  ],
  "endpoint": "check_for_updates",
  "method": "INVOKE",
  "invocationChain": [
    "http://tauri.localhost/assets/index-Lnket4v2.j"
  ],
  "triggerAction": "tauri_invoke",
  "uiClickPath": [
    {
      "element": "div",
      "text": "Antigravity Tools\nv4.103.0\n0\nAccounts\n#1",
      "action": "click",
      "targetId": "root",
      "xpath": "//*[@id='root']",
      "route": "/accounts",
      "id": "qd8be6r",
      "timestamp": 1790763182057
    }
  ],
  "uiClickPathArrow": "div#root \"Antigravity Tools\nv4.103.0\n0\nAccounts\n#1\" [xpath: //*[@id='root']]",
  "route": "/accounts"
}
```

I want you to go through the code base. That's the first thing. Second is that, I think the main issue here is that I really ask the tool before to fix this issue. Several issues are actually at consideration. So first one, let's discuss one problem at a time. So when you see this imager things here, some of the parts are not pro, as you see, but these are pro. Okay? So that is a serious bug. The second thing is that I got a bug with a stack trace, which you can try to fix, I think. Okay, so this is the stack trace. Stack trace. Okay, try to fix that, the stack trace. Now, coming to the more complicated issues, why I'm saying this, because there is an update, okay? And if I try to install this update, it will fail because the CI/CD was not green, and it actually did not put the binaries in the assets. Okay? So I want you to fix that. So if the binary is not there, it shouldn't actually say that this is the update, and also the installer should know beforehand when it is installing. It should also know which, let's say, segment of the assets. Let's say I find the latest one, but latest one does not have the binary asset, so it's going to fail. So I need to have that, let's say, understanding that it's not there before, and then I could let user know. So I'm not installing that. I'm going to install other one because the file is not there. And also the focus button I mentioned several times. The focus button's idea is that I click on it, it will scroll to that position. It cannot. Okay? So that is kind of stupidity. Now, coming to the point, this is also a little bit bad, the instance, how it looks like. What I asked you that when an item is selected, like the selected one, let me find the selected one in the profile. Let's say this one. Okay, this one is the selected one. This has very little distinction. I could not understand that is it really selected or not, and why there is this priority 50 and we cannot change it. Can I double-click and change it? I know. I don't. I cannot. What is the benefit of that? Let me know. And I need to change it. I should be able to double-click and able to change it or to have this feature. And if it is 50, that means it's normal. They do not need to put the priority. If everyone has the same priority, just hide it. If some of those have a different priority, then we show it. So whichever is actually the selected one for this instance, I want this to be highlighted in a different way. Okay? More colors, like bright. It could be yellow line and then dark colors. You can show me two, three observation, like how you could represent this one in different ways. Then I can pick from it. Okay? I've been saying this several times, and it's not there. So update issue, this one, and also the most pressing bug, which I have been sharing with this is the instance switch. Instance switch means that if I make a clone in the instance, okay, either I do clone or not, okay? The problem here is that when the fast forward happens, and if there is a running prompt, it cannot, let's say, detect that running prompt. Cannot. So that is one of the big issues. So I several times said you are going to use the CLI to do that. So your CLI should have the opportunity to back up the running prompt, okay, in a SQLite database for prompt, and then when the switch happens, it shutdowns the Antigravity for that instance, and the instance we have to verify by the instance path. Okay? So with the PID, we will get the instance path, and we see, okay, so this is the instance that we are running with. Okay. All right. So this has been closed and opened. Now on that instance, we need to put the previous running prompt injected and send it to right now. And that needs to be verified by creating different, let's say, different instances, test instances. Okay? So try to make it happen. These are several times I have been saying, and it's not there yet. Also, if you go inside the work directory, there is a repo secret, okay, inside this, there is this Superbase configuration. I want you to run the CLI to open the Superbase configuration into this tool, okay? And share the command that would do that. I hope it makes sense. Okay. So you need to go to the repo secret, Antigravity Manager, I think. This is where the Superbase, Antigravity Event Manager or the manager contains the Superbase information. So I want you to integrate that Superbase DB. I think this is the push settings. If we run the push settings. Let me try to run the push settings, see what happens. I think we need to have two, three commands in there. The first command would be aliasing the machine, the current machine ADS using the map actually. And so the current. For the email import, make sure if the email already exists in the notification, do not re-import the same email. Yeah, I think this is one of the conditional errors that you need to fix. So auto-switch also does not work yet. So you can read the spec, try to understand how the logic works. If you have any confusion, any ambiguity, you can let me know. So first, you don't do anything for now. You just plan this task, okay? And stop it. Once you plan it, you let me know that I planned it, you understand everything. Later on, I will give you the task to complete. Is it understood?

also please refer the images for AI to understand better blur all emails though
`````

### Message 2 — confirmations and additions (speech-to-text)

`````text
Can you please read the code base and the planning mode, how to create a plan inside the AI memory? Can you please follow through and write this plan? And you have some MVUD, as I see, like auto switch require a strict candidate recommendation, things like this. I think you can use the Eras loop, but it would be async operation. You can use the CLI. I think CLI is best. You have CLI, which does it, and there would be another small CLI that would be just checking in every minute or schedule that how soon that it needs to do, right? So two ways that it can be triggered and fixed. Yeah, email. If the notification email is already added, do not add it. Okay, that is confirm. Yes, you need to make the vault, let's say script better. And also I want you to understand that we should have the scripts a little bit better so that it does not do these things. And also I want you to integrate the Supabase things, and that should be also directly as a command in those files, like Supabase, how to run it directly one liner in a PowerShell file, which can be run and make sure that it is working. Does this make sense?
`````

## Extracted Tasks

| ID | Problem (user wording condensed) | Plan | Subtasks |
| :--- | :--- | :--- | :--- |
| T01 | Accounts that are Pro show no PRO badge (serious) | 85 | 003, 004, 005 |
| T02 | Stack trace parsing is wrong (function/file split mid-URL) | 85 | 001, 002 |
| T03 | Update announced but CI was not green and binaries are missing; checker and installer must detect beforehand, skip, tell the user, offer another release | 86 | 001–009 |
| T04 | Focus button must scroll to the selected/current account and does not | 85 | 009 |
| T05 | Selected instance/profile and account rows have no real distinction; bright high-contrast (yellow line, dark colors). Option A chosen | 85 | 007, 008 |
| T06 | `Priority: 50` badge: double-click to edit, hide when it is the default 50, explain the benefit | 85 | 006 |
| T07 | Instance switch / fast-forward loses the running prompt: back up to SQLite via CLI, close only that instance (PID to instance path), switch, relaunch, re-inject, verify with real test instances | 87 | 001–006, 010 |
| T08 | Supabase: import the repo-secrets keys into the tool through the AGM CLI, share the run command, "push settings" (no machine alias) | 88 | 004–007 |
| T09 | Email import must not re-add a notification email that already exists | 88 | 001–003 |
| T10 | Auto-switch does not work; read the spec, raise ambiguity | 87 | 007–009 |
| T11 | (Message 2) Supabase as a runnable one-liner PowerShell file that imports keys through the AGM CLI and is verified. Existing vault PowerShell scripts are fine (follow-up message) | 88 | 006–007 |

## Decisions Confirmed by the User (Message 2)

1. **Auto-switch requires a strict candidate.** A candidate below 100% quota is never selected. The sub-100% fallback is removed.
2. **The CLI is the actor; two triggers.** One CLI command performs the switch. It is invoked by (a) a reactive trigger (quota refresh or UI threshold change, asynchronous) and (b) a small separate scheduled checker that runs about every minute and decides how soon a switch is needed. Both call the same routine.
3. **Email:** if the notification email is already added, do not add it again. Confirmed.
4. **Vault and Supabase (follow-up message, supersedes the earlier "make the vault script better"):** the existing PowerShell scripts are fine. What is wanted is a one-liner PowerShell file that imports the Supabase keys through the AGM CLI, run and verified. The vault path is passed explicitly. No machine alias.
5. **Test runner:** fix it. Done: `npm run test` (`scripts/run-frontend-tests.mjs`, tsx via npx, no new dependency).
6. **Highlight:** Option A. **Focus target:** selected instance's bound account, fallback to the current account. **Native prompt detection:** per-instance directories, DB `dispatched` plus heartbeat. **Branch:** work on `main`.
7. **Working style for future runs:** questions come first, numbered, with the options written inline; after that the run completes end to end (build, tests, live verification) without stopping to ask. Permission to commit and push means do it without waiting.

## Findings (root causes, from reading the code)

Line numbers are from 2026-09-30 and will drift; re-locate by symbol.

### T02 — stack trace
`src/stores/error-store.ts`, `parseStackLine`: the pattern `([^\s(]+)?\s*\(?([^:)]+):(\d+):(\d+)\)?` lets the optional function group swallow a bare URL (`http://tauri.localhost/assets/index-xxx.js:1:2`), then the file group `[^:)]+` captures one character. Result seen in the report: function `…index-Lnket4v2.j`, file `s`. Frames without a function name (URL only) are the trigger.

### T01 — PRO badge
Badge renders only when `account.quota?.subscription_tier` is set (`AccountTable.tsx`, `AccountRow.tsx`, `AccountCard.tsx`, `Instances.tsx`, `CurrentAccount.tsx`). `src-tauri/src/modules/quota.rs` skips `loadCodeAssist` when a cached `project_id` exists, so tier stays `None`; `models/account.rs::update_quota` preserves a tier only if one existed before. Accounts that were added or refreshed while a project id was cached never get a tier, so a PRO account renders no badge and is indistinguishable from "unknown".

### T03 — update integrity
`update_checker.rs`: updater.json path decides `has_update` on version alone; it has a hardcoded upstream owner in one URL; GitHub API path checks version, not platform assets; errors from all sources collapse into one message mapped to default code `E9001`. `release.yml` publish job guesses artifact names when writing `updater.json` and can publish a release while matrix legs failed. `install.ps1` / `install.sh` walk a version ladder but only learn a release has no binary after a failed download.

### T04 — Focus
`Accounts.tsx::handleFocusActiveAccount` targets the global `currentAccount`, not the account bound to the selected instance; a separate state-driven effect also scrolls, so two mechanisms race and the target row may be off the current page/filter.

### T05 — selected row
`InstanceSelector.tsx` uses `bg-white … ring-2 ring-blue-500/80 border-l-blue-600` for the selected row: white on a dark UI reads as a bright blob, not as "selected", and the accounts table selected row has almost no distinction.

### T06 — priority
`AccountTable.tsx` always renders `Priority: N`; editing exists only inside `AccountDetailsDialog.tsx`. Backend meaning: `proxy/token_manager.rs` sorts accounts by priority, lower number first, inside the same tier; 50 is the neutral default.

### T07 — instance switch
Detection is scoped to the wrong place for non-default instances: `repo_db.rs::discover_running_prompts_from_antigravity` and `detect_running_projects` look at the default/global Gemini/Antigravity state, while a cloned instance keeps its own `data_dir` and `home_dir`. `instance.rs::find_pids_for_data_dir` already maps PID to `--user-data-dir` and is the correct scope source. Backup failures are swallowed, resend can run up to three times from different callers (`instance.rs`, `auto_switcher.rs`, `agm.rs` fast-forward), and nothing waits for IDE readiness before injecting.

### T10 — auto-switch
Candidate selection in `auto_switcher.rs` falls back to sub-100% accounts; the desktop loop in `BackgroundTaskRunner.tsx` exits early when not in Tauri and is separate from the Rust daemon, so behavior differs between GUI and headless; `cooldown_seconds` exists in config and is not honored; the target model differs between code paths.

### T09 — email
No normalization/uniqueness on the recipients table; each import path inserts on its own (`email_vault_db.rs::add_notify_recipient`, `email_io.rs`, `commands/email.rs`, `agm.rs` email commands, `EmailNotificationSettings.tsx`). Spec 74 claimed deduplication; it covered display, not storage.

### T08/T11 — Supabase and vault scripts
- There is no `agm supabase push-settings` subcommand; importing the vault keys takes several manual commands.
- `scripts/setup-supabase.ps1` and `scripts/supabase-setup.ps1` in this repo embed default endpoint keys as parameter defaults and duplicate each other. Keys must come from the vault through the AGM CLI, never from committed defaults.
- Observations about the sibling vault repo scripts (`connect-supabase.ps1` root resolution and unchecked exit codes; `push-settings.ps1` gitmap branch that builds an archive and never uploads it) are recorded in issue 53 for the owner. The user said those scripts are fine, so this plan does not edit them.

## Contracts

### Update check result (Rust `UpdateInfo` and TypeScript mirror)
New fields, all additive: `asset_verified` (bool), `asset_url` (string or null), `candidates` (verified installable versions, newest first), `skipped_versions` (array of `{ version, reason }`), `source_errors` (array of `{ source, reason }`). `has_update` is true only when the resolved version has a verified asset for this platform. The resolved version may be older than the newest tag; the UI states which versions were skipped and why.

### Installer pre-check
`install.ps1 -CheckUpdate` and `install.sh --check-update` print one JSON object: `{ current, latest_tag, resolved_version, asset_url, asset_exists, candidates_tried: [...], skipped_versions: [...] }`. Exit code 0 when a verified candidate exists, non-zero when none does. The ladder verifies the asset (HTTP HEAD) before selecting a version.

### Release workflow gate
Publishing requires every expected platform artifact to exist. `updater.json` is generated only from artifacts that were actually uploaded. A post-publish job re-fetches every URL in `updater.json` and fails the workflow on any non-2xx.

### CLI surface added or changed
| Command | Purpose |
| :--- | :--- |
| `agm accounts refresh-tier [--all] [--json]` | Fetch and persist subscription tier; report unknown count. |
| `agm brp --instance <id> [--json]` | Back up running prompts for one instance; non-zero exit on failure. |
| `agm switch-if-low-credit [--instance <id> \| --all] [--json]` | The single switch actor; takes a lock; idempotent. |
| `agm auto tick [--explain] [--json]` | Small checker: decides whether and how soon a switch is due; invokes the actor when due. |
| `agm auto schedule install \| remove \| status` | Registers the 1-minute tick with Task Scheduler, cron, or a systemd user timer. |
| `agm email dedupe [--apply]` | Reports (default) or merges existing duplicate recipients. |
| `agm supabase push-settings --vault <path> [--json]` | Import the Supabase config and keys from the explicit vault path, sync, test. No alias. |

Every GUI setting introduced or changed here has a config-file field and an environment override (headless parity). Names are chosen in the subtasks.

### Selected-state design options (T05; user picks one, subtask 008 applies it)
Tokens are Tailwind utility families already used in the project.
- **Option A (chosen) — slate with amber rail:** `bg-slate-900` row, 6 px `amber-400` left bar, `text-white`, amber `ACTIVE` pill. Quietest; works in light theme with `bg-slate-100` and the same rail.
- **Option B — dark card with amber frame:** `bg-slate-950`, 2 px `amber-400` border, check icon, soft `shadow-amber-400/30` glow. Strongest separation in long lists.
- **Option C — navy gradient with hairlines:** `bg-gradient-to-r from-slate-900 to-indigo-950`, amber top/bottom hairlines, filled amber dot. Most decorative; best for the popover.
The same choice is applied to the popover selected profile, the Instances page active card, and the accounts `CURRENT` row. Contrast must meet WCAG AA (4.5:1) for text in both themes.

### Priority (T06) — behavior and benefit
Lower number is tried first when several accounts are in the same tier and have quota; default 50 means "no preference". A badge is useful only when it differs from the default, so: hide at 50, show as a compact badge when different, double-click to edit inline (1–100, Enter saves, Esc cancels), reuse the existing `updateAccountPriority` command. Benefit: pin a preferred account to the front of rotation, or push a risky one to the back, without removing it.

## Acceptance Criteria

1. A PRO account always shows a PRO badge after one refresh; accounts whose tier is truly unknown show a distinct neutral state, never an absent badge.
2. A stack frame line that is only a URL yields function `<anonymous>` and the full file path; the regression is covered by an automated test.
3. The update banner never offers a version whose platform asset is missing; the user is told which versions were skipped and why; `agm update` and both installers resolve the same version.
4. A release workflow cannot publish with a missing artifact or an `updater.json` URL that returns non-2xx.
5. Focus scrolls to, and briefly highlights, the account bound to the selected instance, across pagination and filters, using one mechanism.
6. Selected instance/profile and account rows are unmistakable in both themes using the option the user chose.
7. `Priority` is hidden at 50, visible otherwise, double-click editable.
8. Switching or fast-forwarding an instance with a running prompt: the prompt is backed up to SQLite for that instance, only that instance's processes are closed, credentials switch, the IDE relaunches and is ready, the prompt is injected exactly once, and verification reports success or a typed failure. Proven by an E2E run with two real test instances that are removed afterwards.
9. Auto-switch selects only 100% candidates; the reactive trigger and the scheduled tick call the same actor; `agm auto tick --explain` states why a switch did or did not happen; GUI and headless behave the same.
10. Adding or importing an existing notification email (any case or surrounding whitespace) does not create a row; the import summary shows how many were skipped.
11. `agm supabase push-settings` and the one-liner PowerShell file import the vault config and keys through the AGM CLI, sync, and test; the run is verified with real output and exit code 0. No secret is printed, committed, or hardcoded. No alias is involved.
12. Scripts written in this work check exit codes, never swallow errors, and never embed keys.

## Verification Matrix

| Area | Local (targeted) | CI |
| :--- | :--- | :--- |
| Rust | `cd src-tauri && cargo fmt -- --check`; `cargo clippy --all-targets --all-features`; targeted `cargo test <module>` | compile tests, full run |
| Frontend | `npm run test`; `npm run build` | build |
| Scripts | run the one-liner live against the vault JSON, twice, plus once with a missing path | none |
| Switch E2E | `scripts/test-instance-e2e.ps1` (local only, opt-in) | not run |

## Risks and Mitigations

- Real instance E2E closes Antigravity processes: matching is by `--user-data-dir` of the test instance only, never by process name.
- Scheduled tick can overlap a running switch: the actor takes a lock file with timeout.
- Dedupe migration could merge rows wrongly: default is report-only; `--apply` is explicit.
- Release gate could block a legitimate release when one platform is intentionally skipped: gate reads the expected-artifact list from one place in `release.yml`.

## Attachments

Four screenshots were supplied in chat. They are not committed: at least one shows a visible account email, and this repo must not contain emails. If the executor needs them, ask the user for redacted copies and store them as `assets/screenshots/85-<name>.png` (lowercase-hyphen).
- Accounts table: rows lacking the PRO badge, `Priority: 50` on every row, weak selected-row distinction, Focus button.
- Instances/Profiles popover: selected profile drawn as a solid white row.
- Accounts rows: `CURRENT` row and priority badge.
- Popover with "Duplicate profile" tooltip: action icons on the selected row.

## Release Policy

No release, version bump, changelog entry, or release-notes edit occurs on any individual task. A release happens only when an entire plan is finished and the user explicitly asks for it. Work is on `main` (resolved ambiguity `06`).
