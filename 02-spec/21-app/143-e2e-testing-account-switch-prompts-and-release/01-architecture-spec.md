# Specification 143: E2E Testing for Account Switching, IDE Detection, Prompt Persistence & Minor Release Architecture

## User Request (Verbatim)

> Can you please test out the final implementation and also all the codes in the Gravity Manager that you have committed or created? The reason I'm saying this because we need to test that the functionality is working, because last time I mentioned how the functionality was broken, right? Based on that, we actually created the plans, very detailed plan that using another HyFi AI tool that actually written that what needs to be done and how it needs to be done. I hope that you followed everything and you completed the implementation of the latest plan and everything. Also check the Git has some issue. We need to resolve the Git conflicts. Make sure the Git conflicts are resolved and everything is pushed to the Git nicely. Also, you can check the build using Cargo. If the build is okay, then go for the release. Do a minor version bump and go for the release. Can you please help me with this? I want you to test the end-to-end testing for the account switch, reading the IDE, whichever is running, how many prompts, which prompts are running, which prompts are in queue, what are the projects as last conversation from each IDE. Check all these things can be done from CICD, CLI, and also from the UI. Everything should be working. You need to have the logs audit. You need to enqueue new prompts to see that it is working. We want to do the end-to-end testing as well. First resolve the Git, merge resolve, fix the build issues, make a release, and then do the end-to-end testing here.

---

## 1. System Architecture & Subsystem Interactions

Antigravity-Manager serves as an autonomous multi-instance gateway and profile switcher coordinating desktop AI IDEs, authentication tokens, prompt recovery, and outbound request routing.

```
+----------------------------------------------------------------------------------------------------+
|                                    Antigravity-Manager Gateway                                     |
|  +---------------------------+  +-------------------------------+  +----------------------------+  |
|  |     Desktop UI (React)    |  |       agm Native CLI Suite    |  |     Tauri IPC & Web REST   |  |
|  +-------------+-------------+  +---------------+---------------+  +--------------+-------------+  |
+----------------|--------------------------------|---------------------------------|----------------+
                 |                                |                                 |
                 v                                v                                 v
+----------------------------------------------------------------------------------------------------+
|                                 Core Rust Modules (src-tauri)                                      |
|  +---------------------------+  +-------------------------------+  +----------------------------+  |
|  |     account.rs & quota.rs |  |       auto_switcher.rs        |  |        instance.rs         |  |
|  |   Token & Quota Engine    |  |    Candidate Scoring Engine   |  |     Sandbox Isolation      |  |
|  +-------------+-------------+  +---------------+---------------+  +--------------+-------------+  |
|                |                                |                                 |                |
|  +-------------v-------------+  +---------------v---------------+  +--------------v-------------+  |
|  |         repo_db.rs        |  |     backup_prompts_db.rs      |  |     task_history_db.rs     |  |
|  |  Active Prompts & Projects|  |  Layer 2 Multimodal Vault     |  |   Split SQLite Audit Logs  |  |
|  +---------------------------+  +-------------------------------+  +----------------------------+  |
+----------------------------------------------------------------------------------------------------+
                                                  |
                 +--------------------------------+-------------------------------+
                 |                                                                |
                 v                                                                v
+-------------------------------------------------+  +-----------------------------------------------+
|             Local IDE Ecosystem                 |  |               Workspace Roots                 |
|  - Antigravity IDE (default & sandboxed)        |  |  - .antigravity_resume_task.json              |
|  - Cursor (AppImage / Standalone)               |  |  - .antigravity_goal_prompt.log               |
|  - Windsurf (Standalone)                        |  |  - AGM_INSTANCE_STATUS.md                     |
|  - Conscious PID Matching Engine                |  |  - 3-Tier state.vscdb SQLite Token Store      |
+-------------------------------------------------+  +-----------------------------------------------+
```

---

## 2. Account Switching Pipeline (5 Stages)

The account switching pipeline coordinates safe credential replacement and uninterrupted prompt execution across 5 strictly ordered phases:

```
[ Stage 1: Snapshot ] ---> [ Stage 2: Terminate ] ---> [ Stage 3: Inject ] ---> [ Stage 4: Launch ] ---> [ Stage 5: Restore ]
```

### Stage 1: Snapshot
1. Query active prompts and workspaces assigned to the target `instance_id` in `src-tauri/src/modules/repo_db.rs`.
2. Extract multimodal payload tokens (`extract_image_payload_or_path`) and active prompt text.
3. Transition prompt states in `repo_prompts.db` from `running` to `backed_up`.
4. Create an immutable snapshot entry in `backup-prompts.db` (`prompt_backups` and `backup_batches`).

### Stage 2: Terminate
1. Perform Conscious PID Matching against the target instance profile.
2. Verify host process protection: ensure active operator GUI PIDs and parent agent trees are NEVER targeted.
3. Send graceful termination signals (`WM_CLOSE` on Windows, `SIGTERM` on Unix) to running IDE processes bound to the target instance.
4. Wait for process exit with bounded timeout (5000ms), falling back to force termination (`SIGKILL`) only if processes fail to exit cleanly.
5. Release file locks on SQLite storage databases (`state.vscdb`).

### Stage 3: Inject
1. Read fresh OAuth token credentials and profile configuration from `src-tauri/src/modules/account.rs`.
2. Atomically inject credentials across all 3 SQLite database tiers for the target instance:
   - Tier 1: `<instance_root>/User/globalStorage/state.vscdb`
   - Tier 2: `<instance_root>/AppData/Roaming/Antigravity/User/globalStorage/state.vscdb`
   - Tier 3: `<instance_root>/home/AppData/Roaming/Antigravity/User/globalStorage/state.vscdb`
3. Update `app_storage.json` and standalone token cache files.
4. Seed `.antigravity_resume_task.json` directly into each bound workspace root containing:
   - `prompt_content`: Raw instruction text.
   - `target_model`: Configured routing model.
   - `instance_id`: Canonical instance identifier.
   - `has_images`: Boolean indicating multimodal visual payloads.
   - `timestamp`: UTC ISO 8601 timestamp.
   - `is_reinjecting`: Positive boolean flag `true`.

### Stage 4: Launch
1. Resolve instance binary path and isolated `--user-data-dir` argument flags.
2. Launch IDE instance detached from the manager process with anti-abuse hardware fingerprint spoofing.
3. Record new child process PIDs in active instance state tracking (`useInstanceStore.ts` / `instance.rs`).

### Stage 5: Restore
1. Invoke `resend_running_commands_for_instance()` to transition backed-up prompts from `backed_up` to `dispatched`.
2. Launch or reconnect prompt goal heartbeat daemon (`scripts/prompt_heartbeat_runner.py`).
3. Increment heartbeat iteration counter smoothly to iteration $N+1$ without resetting to 1.
4. Update workspace status documentation (`AGM_INSTANCE_STATUS.md`).
5. Record action entry in split SQLite audit log (`task_index.db`).

---

## 3. Conscious PID Matching & Host Protection Engine

To guarantee zero disturbance to the operator's environment:

### 3.1 Host Protection Invariants
- **Protected PIDs**: The manager detects host execution environments (`agm-alim`, `Antigravity.exe`, `cursor`, `windsurf`) containing current terminal sessions and registers them in a protected set before initiating any process kills.
- **Parent Tree Immunity**: Any ancestor process of the manager or caller subagent is excluded from termination checks.

### 3.2 Conscious PID Identification
- Processes are inspected via OS process tables (`sysinfo` in Rust, WMI/CIM on Windows, `/proc` on Linux).
- Matches require verifying that the process command line explicitly contains the target instance's unique `--user-data-dir` or sandboxed profile directory.
- Processes lacking instance command-line flags are never terminated.

---

## 4. 4-Hour Rolling Quota Scoring & Candidate Evaluation

The intelligent auto-switcher selects candidate profiles using multiplicative candidate scoring and two-phase rate limit evaluation (`src-tauri/src/modules/auto_switcher.rs` and `quota.rs`):

### 4.1 Scoring Formula

$$\text{Candidate Score} = \text{Active Factor} \times \text{Tier Multiplier} \times \text{Effective Quota Percent}$$

Where:
- **Tier Multiplier**: Ultra = `5.0`, Pro = `3.0`, Free/Standard = `1.0`.
- **Active Factor**: `1.0` if active and eligible, `0.0` if exhausted or in cooldown.
- **Effective Quota Percent**: Minimum quota bottleneck across requested models.

### 4.2 4-Hour Rolling Bucket vs 7-Day Weekly Window
- **5-Hour / 4-Hour Rolling Window (`h`)**: Dynamic interactive quota bucket that refills continuously.
- **7-Day Weekly Allocation (`w`)**: Macro budget based on subscription level.
- **Exhaustion Guard**: If weekly quota $\le 0.1\%$, effective quota is clamped to $0\%$ and weekly reset timestamp is used. When weekly budget is healthy, the short-term rolling bucket dictates rotation priority.
- **Period Boundary Reset**: If reset timestamp is in the past (`is_period_finished == true`), quota is evaluated as $100\%$.

---

## 5. IDE Ecosystem Detection & Workspace Introspection

Antigravity-Manager dynamically interrogates installed and running AI IDE ecosystems:

| IDE Ecosystem | Process Names | User Data Directory Pattern | Storage Database Target |
|---|---|---|---|
| **Antigravity IDE** | `Antigravity`, `antigravity-ide` | `instances/<id>/data` | `User/globalStorage/state.vscdb` |
| **Cursor** | `Cursor`, `cursor-bin` | `~/.config/Cursor` | `User/globalStorage/state.vscdb` |
| **Windsurf** | `Windsurf`, `windsurf-bin` | `~/.config/Windsurf` | `User/globalStorage/state.vscdb` |
| **VS Code / OSS** | `code`, `code-oss` | `~/.config/Code` | `User/globalStorage/state.vscdb` |

### 5.1 Project & Conversation Inspection
- Interrogates `workspaceStorage` trees across each detected IDE to extract:
  - Active workspace URI and normalized project name.
  - Active prompt queue status (running vs queued).
  - Last conversation title, preview text, and timestamp from `state.vscdb` protobuf records.
- Parity across interfaces:
  - CLI: `agm status`, `agm observe`, `agm wpr`, `agm running-prompts`, `agm running-projects`, `agm tree`.
  - UI: `Instances.tsx`, `Accounts.tsx`, `MiniView.tsx`.
  - Tauri IPC: `get_running_projects`, `get_active_prompts`, `get_ide_status`.

---

## 6. Prompt Persistence & Multimodal Backup Architecture

Prompt lifecycles are backed across three resilient tiers:

### 6.1 Database Tiers
1. **Layer 1: `repo_prompts.db`**
   - Stores active prompt state machines (`running`, `backed_up`, `dispatched`, `completed`).
   - Tracks per-instance conversation sequences (`agm_conversation_sequences`).
   - Normalizes workspace paths via `normalize_path_for_compare`.
2. **Layer 2: `backup-prompts.db`**
   - High-durability historical backup vault.
   - Persists multimodal image payloads (`images_payload`, `has_images: true`).
   - Enforces deduplication on `(instance_id, conversation_id, repo_path)`.
   - Automatic 24-hour retention cleanup.

### 6.2 Workspace Hand-off: `.antigravity_resume_task.json`
- Placed in project workspace root prior to IDE startup.
- Read by IDE extension or resume watchdog on boot to automatically continue unfinished work.

### 6.3 Real-Time Heartbeat: `.antigravity_goal_prompt.log`
- Detached Python daemon (`scripts/prompt_heartbeat_runner.py`) records continuous heartbeat entries every 5 seconds.
- Continuous iteration tracking: survives profile swaps by resuming at iteration $N+1$.
- Generates live status document `AGM_INSTANCE_STATUS.md`.

---

## 7. Split SQLite Audit Logging Engine

Audit logs use a split SQLite partition model to maintain sub-millisecond query performance (`src-tauri/src/modules/task_history_db.rs`):

- **Master Index (`task_index.db`)**: Maps action sequences, instance IDs, and timestamps to partition databases.
- **Partition Databases (`history-*.db`)**: 500-row auto-rotating SQLite databases storing granular event traces.
- **Action Codes**:
  - `ActionCode::Switch (1)`: Profile / account switch executed.
  - `ActionCode::Restart (2)`: Instance process restarted.
  - `ActionCode::PromptInject (3)`: Prompt injected or resumed.
  - `ActionCode::QuotaFetch (4)`: Remote quota telemetry fetched.
  - `ActionCode::Backup (5)`: Active prompt state backed up.

---

## 8. Strict Release Invariants & 15-Location Version Synchronization Matrix

When executing a minor release from `4.160.0` to `4.161.0`:

### 8.1 Non-Negotiable Release Invariants
1. **Attribution Invariant**: Attribution must strictly credit `@aukgit` (`(Thanks to @aukgit)`). No other `@` user handles may appear in release changelogs or notes.
2. **Dual README Synchronization**: Stable minor releases MUST update release summaries in both `README.md` (`## 📝 更新日志`) and `README_EN.md` (`## 📝 Changelog`). Updating changelogs alone is strictly forbidden.
3. **Atomic 15-Target Synchronization**: All 15 configuration targets must be synchronized via `scripts/bump-version.mjs` (`npm run bump minor`):

| # | Target File | Path | Key Synchronized |
|---|---|---|---|
| 1 | Root Manifest | `package.json` | `"version": "4.161.0"` |
| 2 | NPM Lockfile | `package-lock.json` | Root package `"version"` (2 locations) |
| 3 | Version Record | `version.json` | `"version": "4.161.0"`, `"Version": "4.161.0"` |
| 4 | Cargo Manifest | `src-tauri/Cargo.toml` | `[package]` version |
| 5 | Tauri Conf | `src-tauri/tauri.conf.json` | `"version": "4.161.0"`, window title |
| 6 | Cargo Lockfile | `src-tauri/Cargo.lock` | `agm-alim` package version |
| 7 | Homebrew Cask | `Casks/antigravity-tools.rb` | `version "4.161.0"` |
| 8 | Chinese README | `README.md` | Title version & Version badge |
| 9 | English README | `README_EN.md` | Title version & Version badge |
| 10 | MiniView Widget | `src/components/layout/MiniView.tsx` | App version fallback string |
| 11 | Settings Page | `src/pages/Settings.tsx` | App version fallback string |
| 12 | Installer Manifest | `releases-manifest.json` | `latest_version`, download URLs |
| 13 | Chinese Changelog | `CHANGELOG.md` | Auto-inserted release header |
| 14 | English Changelog | `CHANGELOG_EN.md` | Auto-inserted release header |
| 15 | NSIS Hook | `src-tauri/hooks.nsh` | Installer DisplayName version string |

4. **Thinking Cache Toggle**: Maintain `SUGGESTION_DELETE_THINKING_STORE = false` in `src/components/common/SuggestionDeleteThinkingModal.tsx` for routine releases.

---

## 9. Verification Gates & Traceability Matrix

| Requirement Item | Specification Section | Subtask File | Verification Command |
|---|---|---|---|
| Git Conflict Resolution | Section 8 | Subtask 01 (`01-git-conflict-and-cargo-build.md`) | Working tree verification & clean status |
| Cargo Compilation Gate | Section 1, 8 | Subtask 01 (`01-git-conflict-and-cargo-build.md`) | `cargo clippy --all-targets --all-features` |
| Minor Version Bump (`4.161.0`) | Section 8 | Subtask 02 (`02-minor-version-bump-and-release.md`) | `npm run bump minor` |
| 15-Target Manifest Sync | Section 8 | Subtask 02 (`02-minor-version-bump-and-release.md`) | Manifest audit across 15 files |
| Strict `@aukgit` Attribution | Section 8 | Subtask 02 (`02-minor-version-bump-and-release.md`) | Changelog handle scan |
| README Changelog Sync | Section 8 | Subtask 02 (`02-minor-version-bump-and-release.md`) | Header & content match in `README.md` & `README_EN.md` |
| Account Switch E2E | Section 2, 3 | Subtask 03 (`03-ide-and-account-switch-e2e.md`) | Multi-instance account switch test |
| IDE Detection & Projects | Section 5 | Subtask 03 (`03-ide-and-account-switch-e2e.md`) | `agm status`, `agm observe`, `agm wpr` |
| Prompt Queue & Resume Task | Section 6 | Subtask 03 (`03-ide-and-account-switch-e2e.md`) | Seed prompt & verify `.antigravity_resume_task.json` |
| Split SQLite Audit Logs | Section 7 | Subtask 03 (`03-ide-and-account-switch-e2e.md`) | Query `task_index.db` & `history-*.db` |
