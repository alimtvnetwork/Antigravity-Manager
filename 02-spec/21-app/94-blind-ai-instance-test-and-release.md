# 94 Blind AI Instance Test and Release

Any agent with no prior chat can run this. Read this file, then follow it in order. Do not invent commands. Do not close the IDE that is running this session.

## What you are proving

1. A sandbox can be a full clone of the default IDE, or a new empty instance.
2. Account switch, prompt backup, and prompt restore run on that sandbox only.
3. The CLI and the UI call the same Rust functions. The UI must not shell out to `agm.exe`.
4. Email and Telegram send when they are connected, and the log says `OK` or `FAIL` with the error text.
5. Supabase is checked by reading the stored email back. A missing table is a failure, not a pass.
6. After the flow is green, make a stable release with `npm run bump`.

## Laws

Break any of these and stop. Fix the listed file, then rerun the failed step.

| Law | Pass looks like | If it fails, edit |
| :--- | :--- | :--- |
| L1 Host process | The IDE process you started in, plus `agm-alim.exe` if it was already running, still has the same PID at the end. | `src-tauri/src/modules/instance.rs` `close_instance` and `find_pids_for_data_dir`. Never `taskkill` by image name. |
| L2 Folder owns the PID | The sandbox PID command line contains that instance `data` directory and does not contain the default data directory. | `src-tauri/src/modules/instance.rs` `launch_instance`, `find_pids_for_data_dir`. |
| L3 Clone vs new | `--from default` copies `User/globalStorage/state.vscdb` from the default profile. `--new` does not copy that tree. | CLI: `src-tauri/src/bin/agm.rs` `cmd_test_instance_flow` and `cmd_instances` create. Core: `instance.rs` `copy_instance` and `create_instance_with_account`. UI: `src/components/navbar/InstanceSelector.tsx` `handleCreate`. Tauri: `src-tauri/src/commands/instance.rs` `create_instance`. |
| L4 Same function | UI `invoke('create_instance')` and `invoke('switch_account_to_instance')` land in `instance.rs`. There is no `Command::new("agm")` on those buttons. | `src/services/instanceService.ts`, `src-tauri/src/commands/instance.rs`. |
| L5 Queued prompts | After switch, prompts that were `queued` are still `queued`. The prompt that was `running` is `dispatched` or `running`, not deleted. | `src-tauri/src/modules/backup_prompts_db.rs` `prompt_status_after_restore`. `src-tauri/src/modules/repo_db.rs` `resend_running_commands_for_instance` must skip `queued`. |
| L6 Notify before return | The switch log contains `[Notify] email:` and `[Notify] telegram:` before the process exits. `FAIL` includes the error. A stack trace is in the log, not only on the console. | `src-tauri/src/modules/notification_hub.rs` `deliver_switch_channels` and `record_channel`. `instance.rs` `switch_account_to_instance` must `.await` `notify_account_switched_details`. `src-tauri/src/bin/agm.rs` `main` must call `logger::init_logger`. |
| L7 Supabase read-back | `agm supabase confirm <instance-id>` prints the same email the instance is bound to. `PGRST205` means the table is missing. | `src-tauri/src/modules/supabase_sync.rs` `push_and_read_instance_email`. Schema SQL: `src-tauri/src/modules/supabase_schema.rs`. Apply that SQL in the Supabase SQL editor. Do not treat a missing table as success. |
| L8 Cleanup scope | The test may delete only instance ids that start with `test-cli-flow` or `test-diag`. It must not delete `default` or any other sandbox. | `cmd_test_instance_flow` step 1 in `agm.rs`. Prompt deletes must use those ids only. |

## Shared functions

| Action | Rust function | CLI | UI |
| :--- | :--- | :--- | :--- |
| New empty instance | `instance::create_instance_with_account` | `agm instances create "Name"` | Create dialog, **New empty** |
| Clone default | `instance::copy_instance("default", ...)` | `agm instances create "Name" --from default` | Create dialog, **Clone from default** |
| Switch account | `instance::switch_account_to_instance` | `agm switch <email> --instance <id>` | Account row switch, `switchAccountToInstance` |
| Fast-forward | `auto_switcher::trigger_manual_rotation_for_instance` | `agm instance ff <id>` | Fast-forward button, `fastForwardInstance` |
| Stop one instance | `instance::close_instance` | `agm instance stop <id>` | Stop on that row |
| Supabase check | `supabase_sync::push_and_read_instance_email` | `agm supabase confirm <id>` | Same Tauri path as sync; do not add a second client |

The UI file is `src/services/instanceService.ts`. It uses `invoke(...)`. If you find `agm.exe` spawned from a React click handler, delete that spawn and call `invoke` instead.

## Repeatable test

Run this whenever you need a fresh proof. It is safe to repeat. It removes only previous `test-cli-flow*` and `test-diag*` sandboxes.

Record the host PIDs first. On Windows:

```powershell
Get-Process Cursor, agm-alim, Antigravity -ErrorAction SilentlyContinue |
  Select-Object Id, ProcessName, StartTime
```

Save that list. If a later command would stop one of those PIDs, abort.

### A. Clone from default

```text
agm test-instance-flow --help
agm instances create "Blind Clone Default" --from default
agm test-instance-flow --from default
```

`test-instance-flow` with no flag is the same as `--from default`.

Pass when the log says `Cloning instance from 'default'` and the new data directory contains `User/globalStorage/state.vscdb`. The default IDE PID from the list above is unchanged.

### B. New empty instance

```text
agm instances create "Blind New Empty"
agm test-instance-flow --new
```

Pass when the log says `Creating a new empty instance` and that data directory is much smaller than the clone. `--new` together with `--from` must exit with an error and create nothing.

### C. Switch, prompts, notify

The flow in A already switches the sandbox, backs up one running prompt and two queued prompts, restores them, and switches back. Read the stdout.

Pass when all of these are true:

- Two different sandbox PIDs appear, both with `--user-data-dir` under that instance folder.
- `[Notify] email: OK` or a `FAIL` that names the SMTP error. Silent skip is a fail if email is enabled.
- `[Notify] telegram: OK` or a `FAIL` that names the Telegram error. Silent skip is a fail if the bot is enabled.
- Prompt lines show the queued prompts still `queued`.
- The host PIDs from the first list are still alive.

Logs live in `%USERPROFILE%\.antigravity_tools\logs\app.log.<date>` and in the command stdout. Search for `[Notify]` and `[Prompt]`.

### D. Supabase

```text
agm supabase test
agm supabase confirm <sandbox-id>
```

Pass when `test` prints `PASS` and `confirm` prints `supabase: OK` with the sandbox email. If the error contains `PGRST205`, open `src-tauri/src/modules/supabase_schema.rs`, apply the `nodes` and `instance_profiles` SQL in the Supabase SQL editor, then rerun `confirm`. Do not invent a new table name.

### E. UI uses the same functions

Open the running AGM window. Instances menu, Create.

1. Choose **Clone from default**, name it `UI Clone Default`, create it.
2. Choose **New empty**, name it `UI New Empty`, create it.

`list_instances` must show both. The clone directory must contain `state.vscdb` copied from the default profile. The empty one must not be a copy of that tree. The click path is `InstanceSelector.handleCreate` to `createInstance` to Tauri `create_instance` to `instance::copy_instance` or `instance::create_instance_with_account`.

Switching an account in the UI must call `switch_account_to_instance` and produce the same `[Notify]` lines in `app.log`.

## Where to fix

Do not patch around a failure in a new script. Change the function in the table above, rebuild `agm`, and rerun only the failed letter.

```text
cargo build --manifest-path src-tauri/Cargo.toml --bin agm
```

Set `CARGO_TARGET_DIR` to `src-tauri/target` so the build uses the existing cache. Full `npm run build` and `cargo clippy` stay in the release pre-flight, not in every retry.

## Release

Do this only after A through E pass and the host PIDs are still alive. If any law failed, do not bump and do not tag.

Follow `docs/release_guide.md` and `AGENTS.md`.

1. `git checkout main` and `git pull origin main`. The tree must be clean.
2. Pre-flight on the commit you will tag:
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `npm run build`
3. Stable patch: `npm run bump patch`. This is the release bump script. It updates the manifests and the changelog skeleton together. Do not edit version files by hand.
4. Fill the new changelog section in `CHANGELOG.md` and `changelog_en.md`. The only GitHub handle allowed is `@aukgit`, written as `(Thanks to @aukgit)`.
5. Copy the stable summary into `README.md` under `## 📝 更新日志` and `README_EN.md` under `## 📝 Changelog`.
6. Commit those release files. Push `main`. Do not force-push.
7. Wait until GitHub Actions for that commit is green. A cancelled run is not green.
8. Tag with the exact changelog heading, including the `v` prefix: `git tag vX.Y.Z && git push origin vX.Y.Z`.
9. Do not tag a beta from `main`. Pre-releases use `beta` and `npm run bump beta`.

If the bump script refuses because the version is not higher, stop and read the error. Do not override it.
