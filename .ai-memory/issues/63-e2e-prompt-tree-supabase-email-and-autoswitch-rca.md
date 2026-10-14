# 63 — Prompt tree OOM, CLI hangs, update toast, auto-switch 15%, Supabase vault, email import

Full procedure and the checklist live in `.ai-memory/memory/learned/27-e2e-instance-autoswitch-prompt-playbook.md`. Read that file when the user asks for end-to-end testing, instance testing, or auto-switch testing. Instance testing includes auto-switch, opening the instance, the prompt preview, and entering a prompt.

## What broke

1. Opening the Prompt Tree crashed WebView2 with `0xe0000008` and froze Cursor. `agm-alim.exe` went from about 71 threads to 565 threads.
2. `agm prompts restore` and a piped `agm auto-switch test 98` hung until the IDE exited. One restore produced no JSON for 78 seconds.
3. The Update Available toast appeared for the version already running, because the tag had no Windows installer yet.
4. Auto-switch defaults and leftover test values were 25%, 12%, or 98%. A test that set 98% could rotate the live account on the next tick.
5. Supabase status looked like a bad credential. The local file held mock URLs and the host typo `pezjuuddecbyrmqxytrv`. The real cloud names returned `NXDOMAIN` because the free-tier projects are paused.
6. Email import inserted `devorg.bd@gmail.com` once per import.

## Why

1. `loadTree` depended on fresh `archivedProjectIds` and `pinnedProjectIds` arrays, so the modal effect called `get_project_conversation_tree` forever. Uncached scans were also unsynchronized, and the CLI tree path forced a cache bypass.
2. Detached IDE launches inherited the caller's stdin, stdout, and stderr.
3. Update detection compared version strings and the on-disk binary against `package.json`, without requiring an `.exe` / `.msi` / `.dmg` / `.AppImage` asset.
4. The struct default and the settings default were corrected to 15%, but migration only rewrites saved `25.0` and `12.0`. A test value of 98 stays until the test sets 15 again.
5. Seed and load-json skipped an endpoint id that already existed, so mock rows were never replaced. DNS failure is a paused project, not a wrong anon key.
6. `notify_recipients` had no unique index on `LOWER(TRIM(email))`.

## Fix

Refs inside `loadTree`. A mutex plus a second cache read in `src-tauri/src/modules/repo_db/tree.rs`. CLI tree `force: false` unless `--force` or `--refresh`. `Stdio::null()` in `src-tauri/src/modules/process/launch.rs`. Canonicalize `-r`. Foreign-key-safe deletes in `running_projects`. Platform asset gate in `src-tauri/src/modules/update_checker/channels.rs` and `platform.rs`. Defaults `15.0` in `src-tauri/src/models/config.rs` and `src/components/settings/AutoSwitcher/switcherTypes.ts`. Supabase URL normalize, mock purge, and dedupe. Email recipient upsert plus `idx_notify_recipients_email_lower`. Password-only `agm email import --ask-password --json`.

## Proof

v4.198.0. CI `38556786736` success. Release `38556790966` success. Tree cold about 2402 ms, warm about 183 ms, threads 51–54. Restore 162 ms after the stdio fix. Auto-switch returned to 15.0 after a 98 read-back. Supabase file has two real endpoints. Email list has one recipient. `agm open-ui` confirms the process PID only. It does not confirm that Settings or Supabase is the visible screen.
