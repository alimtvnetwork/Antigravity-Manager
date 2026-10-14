# End-to-End Testing, Prompt Tree OOM Fix, and Update Asset Gate

## 1. Root Cause of the WebView2 `0xe0000008` Crash and Cursor Freeze

### 1.1 Observed crash
- Opening **Instances → Prompt Tree** in the desktop app caused `msedgewebview2.exe` to fault with exception `0xe0000008` (`STATUS_OOM`, Chromium PartitionAlloc out-of-memory) followed by `Unknown Hard Error`, while `agm-alim.exe` jumped from `71` threads / `33 MB` to `565` threads / `461 MB` within seconds on an `8.7 GB` commit-limit machine, starving Cursor as well.

### 1.2 Exact code loop
1. `src/components/instances/PromptTreeView/usePromptTreeData.ts` listed `archivedProjectIds` and `pinnedProjectIds` in the `useCallback` dependency array of `loadTree`.
2. The modal-open `useEffect` in the same hook listed `loadTree` in its dependency array and unconditionally called `setArchivedProjectIds(getArchivedProjectsForInstance(instanceId))` and `setPinnedProjectIds(JSON.parse(...))`.
3. Because both helpers return a fresh `string[]` reference on every call, each render changed `archivedProjectIds` and `pinnedProjectIds`, which recreated `loadTree`, which re-triggered the modal-open `useEffect`, firing `invoke('get_project_conversation_tree')` in an unbounded render loop.
4. Each IPC call spawned a Tokio `spawn_blocking` worker (`src-tauri/src/commands/instance.rs`), saturating Tokio's 512-thread blocking pool (`71 -> 565` threads in 1 second) with concurrent 48-workspace transcript scans until Windows commit memory was exhausted.

### 1.3 Secondary causes fixed alongside
- `src-tauri/src/modules/repo_db/tree.rs`: uncached `get_project_conversation_tree_cached` calls were not serialized, so concurrent callers each ran a full workspace scan. A static `Mutex<()>` now serializes uncached scans and re-checks `prompt_tree_cache` once the lock is acquired.
- `src-tauri/src/modules/cli/prompts.rs`: `execute_prompts_tree` hardcoded `force: true`, bypassing `prompt_tree_cache` on every CLI tree call; it now defaults to `force: false` unless `--force` or `--refresh` is passed.
- `src-tauri/src/modules/process/launch.rs`: `spawn_with_job_breakaway_fallback` left `stdin`/`stdout`/`stderr` inherited on detached IDE launches, so any piped CLI caller (`agm auto-switch test 98 | ...` or `agm prompts restore`) hung until `Antigravity.exe` exited. Stdio is now nulled before spawning.
- `src-tauri/src/modules/cli/types.rs`: `-r .` stored a literal `"."` in `active_prompts.repo_path`, which resolved to the IDE install directory on relaunch. `CliContext::parse` now canonicalizes `-r`/`--repo`/`--workspace` against the caller's working directory.
- `src-tauri/src/modules/repo_db/detection.rs`: stale-row cleanup in `running_projects` failed with `FOREIGN KEY constraint failed` whenever `active_prompts` referenced the project; the `DELETE` statements now exclude rows referenced by `active_prompts`.

---

## 2. Update Available UI and Platform Asset Gate

### 2.1 Why "Update Available: v4.197.1" appeared while running `v4.197.1`
1. `src/stores/use-update-store.ts` called `check_update_via_script` (`install.ps1 -CheckUpdate`) before `check_for_updates`. `install.ps1` compared the on-disk installed binary (`4.197.0`) against `package.json` on `main` (`4.197.1`) without checking whether the GitHub release had any binary assets attached.
2. `src-tauri/src/modules/update_checker/channels.rs` decided `has_update` by version string comparison alone, without checking whether `updater.json` or the GitHub release contained an asset for the current OS (`.exe`/`.msi` on Windows, `.dmg`/`.app.tar.gz` on macOS, `.AppImage`/`.deb`/`.rpm` on Linux).

### 2.2 Remediation
- **Platform asset verification (`src-tauri/src/modules/update_checker/channels.rs`)**:
  - Parse `platforms` in `updater.json` and `assets` in GitHub releases.
  - Require a matching platform installer asset (`.exe` or `.msi` on Windows; `.dmg` or `.app.tar.gz` on macOS; `.AppImage`, `.deb`, or `.rpm` on Linux) before setting `has_update = true`.
  - In `check_static_url` (`package.json` fallback), verify that the corresponding GitHub release tag has a platform asset before announcing `has_update = true`.
  - In `check_update_via_script` (`src-tauri/src/modules/update_checker/platform.rs`), re-check `compare_versions(&info.latest_version, CURRENT_VERSION)` and platform asset availability so a local build never announces its own version or an asset-less tag.
- **UpdateNotification UI (`src/components/UpdateNotification.tsx` and `src/stores/use-update-store.ts`)**:
  - Prefer `check_for_updates` first in `useUpdateStore`.
  - Render nothing while `updateState === 'checking'` until a verified `has_update` arrives.
  - Use explicit dark-glass surface and button colors (`bg-white/95 dark:bg-[#161926]/95`, `border-emerald-500/40 dark:border-[#1af18d]/40`, primary action `bg-[#1af18d] hover:bg-[#38f59d] text-slate-950 font-semibold`, secondary action `bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-200`).

---

## 3. CLI Parity for UI Trigger, Running Prompts, Resend, and Copy

- `agm open-ui` / `agm instances open-ui [-i <instance>]`: launches or focuses the Antigravity Manager GUI (or opens the target instance IDE when `agm instances open <instance>` is used).
- `agm prompts running [-i <instance>] [--json]`: queries active/running prompts in terminal table format or structured JSON envelope.
- `agm prompts resend [-i <instance>] [N] [--json]` (plus top-level `agm resend`): backs up in-flight prompts and re-dispatches them into `.antigravity_resume_task.json` and `agy`.
- `agm prompts copy [<prompt_id_or_conv_id>] [-i <instance>] [--json]`: copies the selected or latest prompt text for an instance to the system clipboard (and returns the text in `--json` mode), matching the UI standalone prompt copy action.
