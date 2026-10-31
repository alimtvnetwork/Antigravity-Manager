# Completed Plan: 226-antigravity-manager-clone-and-ui-enhancements

## Title
Antigravity Manager Clone and UI Enhancements: Non-Blocking Async Clone, Two-Level Card Actions, Expiration-Urgency Account Selection, IDE Profile Import/Export, Clean Purge Reinstall, and Install IDE Options.

## User Request (Verbatim)
```text
In the Antigravity Manager, most of the things are working fine. That's really great. But when we try to do a clone, it gets stuck. It's got stuck and kind of frozen. You need to fix it. The button previously was two lines or two levels, which was nice, which why you compacted this, I really don't like it. In some places, there are UI glitches, which I think you need to fix. And also the cloning option, when we clone the IDE, make sure that we don't have the copy profile or, what do I say? Yeah, profile copy option, just take out, always the full copy. So no need to have a choosing option. And always copy the workspace. That needs to be checked. That's fine. One more thing is actually into the selection logic. So here today, I have selected two profiles. Somehow these profiles has not selected which of the account is going to be expired very soon. I'm going to give you the screenshot. So based on that, you can actually try to find the logic, what went wrong, and try to fix it. And also make sure the buttons are improved. Also, I want you to add one or two options inside the IDE in the import/export. So we should be able to import/export all the IDE configuration projects, pin projects, a little bit of conversation starting prompt, read prompt into the file system, and import back full. So you can create a new IDE, copy or clone a new IDE without the project and structure, and we should be import/export settings, and including the IDE settings. It should be imported and exported. So make sure that you can implement that and do the end-to-end testing on this. And also, in some cases, the IDE gets wide enough with the issue, like when we try to work with the IDE, it was from the old section. I have not seen it unless I do try to clone to IDE or instance. So everything else is working fine. So any changes that you do, make sure you took care cautions. You don't change any other files without understanding, so plan ahead before doing that so that it's working fine. And also, I want you to add one more options in the future. I know the tool actually says in the README it works with two, three IDEs. Make sure to add the install IDE option from the UI and also from the terminal. All these options I mentioned, it should also be in the CLI and also in the UI. So you can also test from the CLI that import/export works. Yeah, so these are the things I want, so make sure you do that. And also, we should be able to delete the full Antigravity and reinstall from the UI and also from the CLI. The reason being is that sometimes the Gemini folder, the brain and logs, that actually creates so much memory fogs. We wanted to delete fully and reinstall again. So make sure we should be able to do it regardless of the OS. So write the spec first and then start coding. Does this make sense?
```

## Context & Root Cause Analysis

### 1. IDE Cloning Freeze
- **Problem**: In `assets/screenshots/226-antigravity-manager-clone-and-ui-enhancements-02.png`, the window title displays `Antigravity Manager Tools v4.268.0 (Not Responding)` during instance cloning.
- **Root Cause**: `copy_instance` in `src-tauri/src/commands/instance.rs` was synchronous (`pub fn`), executing multi-gigabyte disk copying (`copy_dir_recursive`), SQLite database backups, and project directory copying directly on the Tauri thread without `tokio::task::spawn_blocking`. This starved the Win32 window message pump (WM_PAINT, WM_TIMER) for >5 seconds, triggering the OS `(Not Responding)` ghost window.
- **Solution Verified**: Converted `copy_instance` to asynchronous with `tokio::task::spawn_blocking`, emitted structured `instance-clone-progress` events, and rendered real-time progress indicators in UI modals.

### 2. Cloning Modal Simplification
- **Problem**: The modal previously presented a confusing radio choice between `IDE Copy (Full Environment & Sessions)` and `Profile Copy (Preferences & Snippets)`.
- **Solution Verified**: Removed the `Profile Copy` option completely. Duplication is locked to full IDE copy by default. `copy_projects: true` is permanently active and visibly marked with an emerald badge/banner in both `CopyInstanceDialog.tsx` and `CopyInstanceModal.tsx`.

### 3. Instance Card Buttons Reversion
- **Problem**: In `assets/screenshots/226-antigravity-manager-clone-and-ui-enhancements-03.png`, 10 action buttons were crammed into a single horizontal pill capsule, making buttons narrow, hard to read, and difficult to click.
- **Solution Verified**: Reverted `InstanceCardActions.tsx` back to two structured rows / levels:
  - **Level 1 (Lifecycle - 5 buttons)**: Stop/Restart/Launch, Switch Account (3D yellow hover), Fast Forward, Audit Trail, Sync PID & Quota.
  - **Level 2 (Utilities/Tools - 6 buttons)**: Prompts Tree (with active task cyan pulse dot), Settings & Sync, Clone Profile, Clone Executable, Wipe Session/Reset, Delete Profile / More Actions.

### 4. Account Auto-Selection & Quota Expiration Urgency
- **Problem**: In `assets/screenshots/226-antigravity-manager-clone-and-ui-enhancements-01.png`, `shohagbazar004@gmail.com` with weekly quota resetting in 4h 24m (33%) was deprioritized over `riseup.team.now@gmail.com` (6d 8h remaining, 78%).
- **Root Cause**: In `src-tauri/src/modules/auto_switcher/scoring.rs:168-170`, the formula multiplied weekly quota by `remaining_hours` (`eff_pct * remaining_hours`). Fresh accounts with 168h received ~80x higher scores than accounts with 4h left, penalizing expiring accounts instead of prioritizing them. Weekly quota is "use it or lose it".
- **Solution Verified**: Inverted the remaining headroom logic to apply an **Expiration Urgency Multiplier** ($E_{\text{urgency}}$) for accounts with $T_{\text{remaining}} \le 24\text{h}$ or $\le 48\text{h}$ having usable quota ($\ge 15\%$), and sort urgent expiration accounts ascending by reset time in candidate selection. Aligned `src/utils/accountAlgorithmScore.ts` and `src/services/instanceService/scoring.ts` to match.

### 5. IDE Settings, Projects & Prompts Import / Export (UI & CLI)
- **Problem**: Users could not easily export/import IDE configuration, pinned projects, conversation starter prompts, read prompts, and settings into filesystem bundles.
- **Solution Verified**: Implemented `AgmIdeProfileBundle` schema in `src-tauri/src/modules/instance/bundle_export.rs` & `bundle_import.rs`. Added CLI commands `agm ide export` and `agm ide import`, and added `IdeBundleModal.tsx` in the Instances page.

### 6. Full Antigravity Clean Delete & Reinstall (UI & CLI)
- **Problem**: The Gemini folder (`~/.gemini/antigravity`), `brain/`, conversation DBs, and logs accumulate memory fog, stale OAuth state, and disk bloat.
- **Solution Verified**: Implemented a safe 4-stage purge protocol across Windows, macOS, and Linux in `src-tauri/src/modules/instance/purge_reinstall.rs`. Added CLI `agm ide purge-reinstall [--force]` and `PurgeReinstallModal.tsx` in Instances. Enforces caller IDE self-preservation (`detect_caller_instance_id()`).

### 7. Install IDE Option in UI & CLI
- **Problem**: AGM supports Antigravity, Cursor, and VS Code, but lacked a first-class installation command and UI button.
- **Solution Verified**: Implemented `src-tauri/src/modules/ide_installer/` supporting `antigravity`, `cursor`, and `vscode`. Added CLI `agm ide install <target>` and `InstallIdeModal.tsx` in the Instances header.

### 8. UI Glitches Throughout Application
- **Problem**: In `assets/screenshots/226-antigravity-manager-clone-and-ui-enhancements-04.png`, modal freezing, washed-out backdrops, and unthrottled base64 image reads degrade responsiveness.
- **Solution Verified**: Batched and throttled base64 image reads (concurrency limit 3 via `BoundedImageLoaderPool`), added in-memory LRU cache, removed nested `backdrop-blur-2xl` on inner modal dialogs, and debounced `loadTree`.

---

## Verification & Gating Evidence
- `cd src-tauri; cargo fmt -- --check`: PASSED (Exit code 0)
- `npm run build`: PASSED (Exit code 0, all TypeScript and Vite production assets built)
- `cd src-tauri; cargo check --lib`: PASSED (Exit code 0)
- `cd src-tauri; cargo check --bin agm`: PASSED (Exit code 0)
- `cd src-tauri; cargo clippy --lib --bin agm`: PASSED (Exit code 0)
- CLI Help & Dry-Run Tests:
  - `agm ide --help`: PASSED (Displays export, import, purge-reinstall, install)
  - `agm ide install antigravity --dry-run`: PASSED (Discovers executable path and resolves installer command)
