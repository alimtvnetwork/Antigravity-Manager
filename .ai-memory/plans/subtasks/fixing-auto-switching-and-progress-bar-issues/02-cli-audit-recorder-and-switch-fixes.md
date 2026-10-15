# Subtask 02: CLI Audit Recorder (`agm ui record`) & Account Switch Fixes

- Add `agm ui record <start|stop|status|list|tail|clear|export> [--json]` in `src-tauri/src/bin/agm/ui_cmds.rs`.
- Persist recording state across CLI/GUI processes in `ui_audit.db` (`audit_meta` table) and expose `/api/ui/record/*` in `src-tauri/src/proxy/server/admin_ui.rs`.
- Instrument `switch_account`, `switch_account_to_instance`, `fast_forward_instance`, `send_prompt_now`, and `enqueue_prompt`.
- Fix `src/components/accounts/AccountTable/cells/ActionsCell.tsx` and `src/components/accounts/AccountRow.tsx` to target `activeInstanceId` and display the active instance name in the tooltip.
