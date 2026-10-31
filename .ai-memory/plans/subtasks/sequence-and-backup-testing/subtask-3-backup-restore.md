# Subtask 3: End-to-End Prompt Backup & Restore Testing with Screenshot IDs

## Objective
Thoroughly test prompt backup and restoration end-to-end:
1. Enqueue tested prompts with unique IDs and screenshot identifiers.
2. Execute backup (`agm prompt backup` / UI export) to persist snapshot into `prompt_backups` or JSON file.
3. Verify that the screenshot of the `Id` is associated with the test prompt.
4. Execute restoration (`agm prompt restore` / UI import) and confirm that the test prompt and screenshot ID are successfully restored.

## Files Involved
- `src-tauri/src/modules/repo_db/dispatch.rs`
- `src-tauri/src/modules/repo_db/detection.rs`
- `src-tauri/src/modules/repo_db/liveness.rs`
- `src-tauri/src/commands/prompt_cmd.rs`
- `src-tauri/src/bin/agm.rs` (`agm prompt backup`, `agm prompt restore`)

## Verification Strategy
- Author an end-to-end test script in PowerShell.
- Queue a designated prompt containing a screenshot reference / ID token.
- Backup prompts to database and file.
- Verify backup records exist.
- Clear/modify active queue row.
- Run restore and assert the prompt and screenshot token exist in active database.
