# Accounts toolbar, audit detail, and one version

## Why the 5H and Weekly buttons are gone

The accounts list already shows the 4-hour quota on the left and the weekly quota on the right. The toolbar buttons only chose which of those two the grid would show, and they used `localStorage` key `accounts_quota_window` (`5h` or `weekly`). That choice did not change the list cells. The buttons are removed. The list and the grid both show the pair. The reset-time column uses the 4-hour reset.

To put the buttons back, restore the `quotaWindow` state in `src/pages/Accounts.tsx`, the two buttons that were above the view switcher, and the `quotaWindow` prop on the table and the grid. Do not add a third quota window.

## Toolbar

One dropdown replaces the All, Pro, Ultra, and Free pills. Default is All. Row icons, left to right: Refresh, Switch, then label, warmup, proxy, and delete, then More. More holds IDE switch, CLI switch, Details, Fingerprint, Export, Show email, and a read-only Cycle tokens line. Cycle tokens stay in `cycle_tokens`. They are not drawn under the weekly bar.

Email masking lives in `src/utils/maskEmail.ts`. The domain is never shown. A stable hash of the local part hides either the first half or the second half with `***`. Show email reveals that row for the session.

## Pro badge

`fetch_quota_with_cache` always calls `loadCodeAssist`. A cached `project_id` is only the fallback when that call returns no project id. `standard-tier` stays unmapped. A refresh can change `subscription_tier`.

## Audit

`AuditAction` is `#[repr(i32)]`: `AddAccount = 1`, `UpdateAccount = 2`, `SwitchAccount = 3`. The list shows title case (`Add Account`). The symbolic name is PascalCase (`AddAccount`). `list_page` does not select `payload_json`. `get_task_history_detail` opens the split file for that id. A switch payload is `from_email`, `to_email`, `prompt_id`, `prompt_text`, `moved_at`, `switch_ok`.

## Version

`scripts/bump-version.mjs` exits 1 if `package.json`, `version.json` (`version` and `Version`), `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` version and window title, or `src-tauri/hooks.nsh` disagree after the rewrite. The header uses Tauri `getVersion()` in a packaged build. Fallbacks import `version.json`.

## Zip handoff

A successful `agm update` writes `~/.antigravity_tools/update-export/agm-update.zip` and still installs. `agm update export-zip` only writes the zip. GitMap copies that remote export zip back into the controller cache after a node update so the next machine can be installed from it.
