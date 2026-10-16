# Root-Cause Analysis Index (`.ai-memory/rca/`)

Every significant recent CI/CD and tooling failure is documented here as a
standalone RCA, written so that any AI reading it will not repeat the
mistake. Each file follows the same contract: **Symptom → Root cause →
How found (exact commands) → Fix (file:line, old → new) → Verification
evidence → Prevention checklist item**.

Read these before touching CI, doing a file split, changing a shared
function signature, or re-running a failed pipeline.

## Index

| RCA file | Incident | One-line cause |
|---|---|---|
| [01-agm-cargo-fmt-instance-rs.md](./01-agm-cargo-fmt-instance-rs.md) | CI `Check Rust Code` failed on all runners: `cargo fmt` diff in `instance.rs` | a hand-written fn signature longer than rustfmt's 100-column limit was committed without running `cargo fmt` |
| [02-agm-telegram-e0061.md](./02-agm-telegram-e0061.md) | CI Clippy `error[E0061]`: `send_telegram_message_with_markup` takes 5 arguments, 4 supplied | new 5th parameter (`disable_preview: bool`) added to the function but three call sites in `daemon.rs` were not updated |
| [03-agm-paquirk-branch-conflict.md](./03-agm-paquirk-branch-conflict.md) | `gitmap pa` on gitmap-v28 failed with merge conflicts | stale local branch `fix/ci-gates-green` held a private duplicate of the fix already pushed as `83939b8ae`; `pa` tried to merge the two copies |
| [04-gitmap-split-imports.md](./04-gitmap-split-imports.md) | CI `Build (...)` failed on all 6 platform jobs: `undefined: os`, `undefined: apperror`, `undefined: io`, `"bytes" imported and not used` | file-split commit moved functions into new files without carrying their import lines, and left a stale import in the trimmed donor file |
| [05-gitmap-split-marker.md](./05-gitmap-split-marker.md) | `TestTopLevelCmdRegistryMatchesAST` failed: 140 registry entries with no matching AST declaration | split moved const blocks into `constants_commands_extra.go` without the `// gitmap:cmd top-level` marker the AST collector requires |
| [06-gitmap-coverage-test-locator.md](./06-gitmap-coverage-test-locator.md) | `TestEveryCmdIDHasHelpFile` failed: "parsed zero Cmd* constants — locator broke?" | helpdoc coverage test hardcoded the path `constants/constants_cli.go`, which the split gutted into a 6-line stub with zero `Cmd*` constants |
| [07-gitmap-windows-readonly-skip.md](./07-gitmap-windows-readonly-skip.md) | `TestEffectiveDataDir_ReadOnlyFallback` failed on `windows-latest` only | test relied on POSIX `chmod 0555` to make a directory unwritable; Windows does not enforce POSIX mode bits on directories, so the fallback never triggered |
| [08-gitmap-workflow-poisoned-cache.md](./08-gitmap-workflow-poisoned-cache.md) | re-running CI on a failed SHA short-circuits everything green via the `ci-passed-<sha>` cache | `test-summary` job's `if: success()` mark step checks only its own job's steps (not the pipeline), so failed pipelines still write the "passed" cache entry |
| [09-gitmap-workflow-e2e-shortcircuit.md](./09-gitmap-workflow-e2e-shortcircuit.md) | E2E jobs fail on re-run at step "Already validated": bash cannot start in `.../cli` | E2E job's `defaults.run.working-directory: cli` still applies to the no-op step, but the checkout is skipped when `already-built == 'true'`, so `cli/` does not exist |
| [10-supabase-sync-failure.md](./10-supabase-sync-failure.md) | Supabase health check shows "Unreachable" / DNS failure for both configured endpoints | free-tier Supabase/Lovable projects are paused cloud-side, so `<project_ref>.supabase.co` is unpublished from DNS — not a code defect |
| [11-telegram-sendflow-e0015.md](./11-telegram-sendflow-e0015.md) | CI failed on the Telegram program commit: `error[E0015]` at `send_flow.rs:41` | `HashMap::new()` is not const — cannot initialize a `static` with it; wrapped in the repo's `once_cell::sync::Lazy` pattern |
| [12-telegram-sendflow-not-send.md](./12-telegram-sendflow-not-send.md) | CI failed on the E0015 fix commit: `future cannot be sent between threads safely` | `std::sync::MutexGuard` held across `.await` in `consume_send_prompt`; restructured to scope the lock in a block returning owned data |
| [13-admin-ui-emitter-trait.md](./13-admin-ui-emitter-trait.md) | v4.200.0 tag shipped with `error[E0599]`: `tauri::Emitter` trait not in scope in `admin_ui.rs` | one-line trait import missing; fixed and released as v4.200.1 without rewriting the published tag |
| [14-telegram-cmd-callback-double-slash.md](./14-telegram-cmd-callback-double-slash.md) | Telegram send/find/credits buttons silently did nothing | `cmd:/send` hit the daemon's second dispatch site which didn't strip `/` → `//send` → lookup `None` → silent no-op; unified both sites on `normalize_cmd_callback_payload()` |
| [15-telegram-picker-callback-64byte-cap.md](./15-telegram-picker-callback-64byte-cap.md) | Send-flow picker callbacks could exceed Telegram's 64-byte callback limit | `send:cman:{m}|{i}` payload and unconditionally-pushed local machine label; shortened to bare `send:cman` + `machine_entry_usable()` guard |
| [16-telegram-unknown-cmd-callback-silence.md](./16-telegram-unknown-cmd-callback-silence.md) | Unresolvable `cmd:` button taps failed silently | added user-visible "Unknown button action" fallback notice |
| [17-instance-pid-stop-switch-and-audit-trail.md](./17-instance-pid-stop-switch-and-audit-trail.md) | List showed a bare number, Stop left the IDE running, audit trail was empty | `1632` is `inst.pid`; close returned Ok after dropping that PID; the dialog read `recent_switches` while the query returned `records` |

## Standing notes

- RCAs 08 and 09 are **documentation only**: their recommended fixes are
  blocked by the standing no-workflow-changes rule (never edit
  `.github/workflows/` from a subagent — that needs the user's explicit
  decision).
- RCA-10's fix is a user action (unpause the projects in the Supabase/Lovable
  dashboard), not code.
- Source incidents: 01–07 from the 2026-10-14 dual-release/CI saga
  (AGM v4.197.0, gitmap v6.550.0); 08–10 written 2026-10-15.
