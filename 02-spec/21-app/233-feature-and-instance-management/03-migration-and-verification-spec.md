# Migration and Verification Specification: Feature and Instance Management

## 1. Scope of Changes & Migration Strategy
1. **Frontend Non-Breaking Modifications**:
   - Updates Tailwind utility classes on `InstanceCard.tsx` and `InstanceCardActions.tsx`. Zero schema changes or breaking state changes.
   - Integrates `logUiTrace` across user action handlers (`onSwitchAccount`, `handleLaunch`, `handleStop`, `handleRestart`, `handleFastForward`, `openPromptTree`).
   - Upgrades `useUiRemoteControl.ts` to locate elements via `id`, `data-ui-id`, and `data-ui-name`.
2. **Backend Non-Breaking Extensions**:
   - Registers `record_ui_trace` command in `src-tauri/src/commands/mod.rs`.
   - Adds `src-tauri/src/modules/notification_hub/idle_alerts.rs` exposing `dispatch_idle_mode_alert`.
   - Hooks `check_idle_projects_sensor` in `email_watcher/sensors.rs` to call `dispatch_idle_mode_alert` (sending to both Email and Telegram).
   - Updates `src-tauri/src/bin/agm/ui_cmds.rs` to validate and execute dynamic instance actions.

## 2. Verification Test Suite & Quality Gates
### 2.1 Static Checks
- `cd src-tauri && cargo fmt -- --check`
- `cd src-tauri && cargo clippy --all-targets --all-features`
- `npm run build`

### 2.2 CLI & E2E Verification
- Run `agm ui ls instances` to verify catalogued and dynamic instance elements.
- Verify `agm ui click instance-switch-default` executes without unknown ID error.
- Verify `agm errors count` outputs 0 errors.
- Test instance switching and prompt restoring FIFO flow.

## 3. Release Ceremony
- Version bump: `npm run bump minor` (`4.273.1` -> `4.274.0`).
- Update `changelog.md` and `changelog_en.md` with sole attribution to `@aukgit` (`(Thanks to @aukgit)`).
- Update `README.md` and `README_EN.md` with release summary.
- Git commit, tag `v4.274.0`, and push to `origin/main`.
