# AI Instructions: Profile Switch Crash & Complete Process Termination

## Master Execution Workflow

### Step 1: Complete App Termination on Window Close
1. In `src-tauri/src/modules/process/mod.rs` (or `lifecycle.rs`), add `pub fn exit_app_fully(app_handle: &tauri::AppHandle)`.
2. In `src-tauri/src/lib/setup_app.rs:256`, remove `api.prevent_close()` and call `crate::modules::process::exit_app_fully(window.app_handle())`.
3. In `src-tauri/src/lib/run.rs:396`, remove `api.prevent_exit()` and call `crate::modules::process::exit_app_fully(app_handle)`.
4. In `src-tauri/src/commands/cmd_window.rs:58`, call `crate::modules::process::exit_app_fully(&window.app_handle())`.

### Step 2: Multi-Stage Process Spawning Ladder & Switch Resilience
1. In `src-tauri/src/modules/instance/launch.rs`, strengthen Windows process spawning fallback:
   - Handle Stage 1, Stage 2 (base executable fallback), Stage 3 (`cmd.exe /s /c start`), Stage 4 (PowerShell `Start-Process`).
2. In `src-tauri/src/modules/instance/switch.rs:294-314`, wrap relaunch so that if launch fails after credentials are injected, the switch function logs a warning and returns `Ok(())` instead of propagating an error.
3. In `src/services/instanceService/lifecycle.ts:189`, ensure error handling does not open modal if call succeeds.

### Step 3: Telegram Rich Formatting, Copy Button & Standalone Fleet Identity
1. In `src-tauri/src/modules/notification_hub/telegram_alerts.rs`:
   - Use HTML formatting with clean sections and prompt pre blocks.
   - Attach Telegram inline keyboard with `copy_text` button.
   - Include Fleet Identity Card.
2. In `src-tauri/src/modules/telegram_inbound/processing.rs`:
   - Support `/fleet`, `/instances`, `/accounts`, `/whoami`.

### Step 4: Preflight & Release Ceremony
1. Run `cargo fmt -- --check`.
2. Synchronize version via `node scripts/bump-version.mjs minor` (`v4.186.0` -> `v4.187.0`).
3. Update `changelog.md`, `changelog_en.md`, `README.md`, `README_EN.md` attributing `@aukgit` (`(Thanks to @aukgit)`).
4. Stage and commit via `gitmap cpf`.
5. Tag and push.
