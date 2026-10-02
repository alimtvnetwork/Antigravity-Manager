# 95 Accounts Columns, Prompt Re-push, and Update Zip

## Tasks

1. The four row actions Refresh, Details, Fingerprint, and Export sit in one dropdown. The menu paints above the sticky actions cell.
2. The quota header offers Gemini or Claude. The left cell is that model's 4-hour quota. The right cell is that model's weekly quota.
3. When every visible account has the same priority, the priority chip is hidden. Double-click edits the priority in place. A different priority shows the number.
4. A Pro account shows the Pro badge. The fetch path stores `PRO`, `ULTRA`, or `FREE` when the Google tier id is known.
5. CLI switch, the Accounts UI, and auto-switch re-push the prompt that was running. Email and Telegram already send. Queued prompts stay queued.
6. `agm update export-zip` downloads this platform's release zip once and does not install it. GitMap's AGM fleet zip sends that file and the remote machine extracts it with PowerShell or a shell script.

## Laws

| Law | Pass | Fix |
| :--- | :--- | :--- |
| One model, two windows | Left cell label is `4h`. Right cell label is `Weekly`. Both use the selected family. | `src/components/accounts/AccountTable.tsx` |
| Uniform priority hidden | All accounts at the same priority show no chip. One different value shows the number. Enter saves through `updateAccountPriority`. | `AccountTable.tsx`, `src/pages/Accounts.tsx` |
| Pro badge | Stored tier is `PRO` when the raw id contains pro, premium, or advanced. An unknown id is logged and not relabeled. `standard-tier` stays unmapped until a captured payload shows it on a known Pro account. | `src-tauri/src/modules/quota.rs`, `src-tauri/src/models/quota.rs` |
| One inject after ready | `launch_instance` during a switch does not restore or dispatch. The switch waits until the instance process is up, then restores once. `dispatched` is written only after `spawn_prompt_via_agy` returns true. A failed spawn leaves `backed_up` and logs the prompt id. | `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/integration.rs` |
| Zip does not install | `agm update export-zip` writes a zip path and does not replace the running GUI. | `src-tauri/src/bin/agm.rs` |
| GitMap reuses the zip | `gitmap uaz agm` sends that zip. Windows runs PowerShell `Expand-Archive`. Linux and macOS run `unzip`. | `D:\work\gitmap\cli\cmdupdate\update_fleet.go` |

## Switch inject

`switch_account_to_instance` and `on_account_switch` are the inject sites. Auto-switch calls those functions. A later auto-switch resend only retries rows that are still `backed_up`. It does not mark a backup restored on its own.
