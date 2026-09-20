# Issue 45: Instance Account Switching & Same Email Bug RCA

## 1. Symptoms & Observed Behavior
- Multiple running instances of Antigravity displayed identical email addresses (`ashleyescobarzu@gmail.com`).
- Profiles bound to different accounts failed to reflect their respective accounts upon launch.
- Duplicating an existing profile carried over the active login session of the source instance.

## 2. Root Cause
- `launch_instance` in `src-tauri/src/modules/instance.rs` only synced tokens to the system keyring and omitted SQLite token injection into the instance's isolated `{data_dir}/User/globalStorage/state.vscdb`.
- Antigravity reads session data directly from `antigravityUnifiedStateSync.oauthToken` and `antigravityUnifiedStateSync.userStatus` within `state.vscdb`.
- `copy_instance` cloned the source `state.vscdb` without sanitizing auth tokens.
- Non-default instances lacked automatic executable cloning on launch, running identical binary paths.

## 3. Resolution
- Injected `crate::modules::db::inject_token` and `write_service_machine_id` directly in `launch_instance`.
- Added `crate::modules::db::sanitize_session` and called it in `copy_instance` to purge copied credentials.
- Auto-cloned executables (`Antigravity-{instance_id}.exe`) on launch for non-default profiles with copy fallback.
- Added detailed specification in `02-spec/20-instance-management/01-instance-switching-rca.md`.

## 4. Verification
- `cargo fmt --check` verified clean formatting.
- `npx tsc --noEmit` verified zero frontend/type errors.
- Session sanitization and database token injection confirmed in code.
