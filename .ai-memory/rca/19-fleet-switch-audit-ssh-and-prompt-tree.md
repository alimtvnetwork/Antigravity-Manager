# RCA 19: Fleet, account switch, audit, SSH nodes, and the prompt tree

Read this with RCA 17 and RCA 18 before touching the instance list, the fleet table, Settings → SSH and GitMap, the audit panel, or the prompt tree. Spec: `02-spec/21-app/fleet-switch-audit-ssh-and-prompt-tree/01-architecture-spec.md`.

These are separate systems. An AI that "fixes fleet" by editing the Telegram picker, or "fixes SSH" by writing Supabase rows, breaks a working path.

## Fleet connection (read this first)

The fleet table is not the SSH node list.

| Store | File or table | What it is |
|---|---|---|
| SSH nodes | `ssh_nodes.json` in the data directory | Aliases used to open a shell. Import with `gitmap ssh nodes export-json`. |
| Local fleet cache | `fleet_sync.db` table `FleetNodes` | Heartbeat rows: `node_id`, `node_alias`, `local_ip`, `is_online`, `last_heartbeat_at`. |
| Supabase | `public.nodes` when sync is on | The list the screen actually renders. |

`src/components/instances/fleet/useFleetMachines.ts` does this:

1. Read Supabase config. If `is_sync_enabled` is false, clear the list and show `FleetDisabledState` ("Multi-Machine Fleet Sync is Disabled") with a link to `/supabase`.
2. If sync is on, call `get_fleet_machines`. An empty result shows `FleetEmptyState` ("Shared Cluster") and tells the operator that SSH nodes are configured under Settings, SSH and GitMap.

A machine appears only after it has heartbeated into the fleet store and sync is enabled. Adding an SSH alias does not insert a `FleetNodes` row. A green SSH test does not mean the fleet table will show that machine.

### How an AI fails fleet

- It sees "Shared Cluster" and rewrites SSH import, or it sees the disabled card and adds nodes to `ssh_nodes.json`. The table stays empty.
- It treats `Node-<digits>` as a broken fleet id. That string is the fallback alias when nobody set a name. The display cache for Telegram is `node_display_cache.db` (RCA 18). It is not `FleetNodes`.
- It points the empty state at Supabase and the disabled state at SSH, which is the reverse of the two components.
- It polls fleet in a tight loop. The hook already refreshes every 15 seconds.
- It marks sync enabled in the UI without a reachable Supabase host. Paused projects are NXDOMAIN. That is not a bad API key. See `.ai-memory/rca/10-supabase-sync-failure.md`.

### What to do

- Disabled card: fix Supabase config and `is_sync_enabled`. Do not invent SSH rows.
- Empty Shared Cluster while sync is on: confirm a heartbeat landed in `FleetNodes` (`last_heartbeat_at`, `is_online`) and that `get_fleet_machines` returns it. Then confirm the SSH alias if remote commands are also required.
- Keep both checks. A node can SSH and still be absent from the fleet table.

## Account switcher

RCA 17 is the process bug. This section is the dialog bug on top of it.

`src/pages/instances/SwitchAccountDialog.tsx` sorts accounts by `4h + weekly`, highest first. `fourH` is `ModelQuota.percentage` on the pro model, then the flash model. `weekly` is `QuotaBucket.remaining_fraction * 100`. Null becomes 0 for the sort. The row shows `4h`, `Weekly`, and `Total`.

Switch still has to kill the instance PID before it writes credentials. `close_instance_verified` must return failure when the PID is alive. Windows liveness is `OpenProcess` / `GetExitCodeProcess`, not a sysinfo single-PID refresh. Do not stop the instance printed by `agm instances current`.

### How an AI fails the switcher

- It shows one "Quota: N%" again and drops weekly.
- It sorts by `updated_at` or by email.
- It writes credentials when close returns `Ok(false)`.
- It treats a sysinfo snapshot as proof the process died. On Windows that snapshot keeps an exited PID.
- It leaves the auto-switch threshold at 98 after a test. Put it back to 15.

## Audit log

`get_instance_switch_history` in `src-tauri/src/modules/task_history_db/switch_history.rs` returns `total`, `total_switches`, `records`, `items`, and `recent_switches`. The dialog reads `recent_switches` and `total_switches`. Before that alias existed, the dialog rendered "No Account Switches Recorded" while `records` held the rows.

`src/components/audit/AuditPanel.tsx` maps command ids to sentences. `send_prompt_now` is "Send prompt now". A raw id on the panel means the map is missing that key, not that the command failed.

The list query does not return `payload_json`. Detail loads with `get_task_history_detail` on the info click. Emails on the list are masked.

### How an AI fails the audit

- It binds the modal to `records` only, or only to `recent_switches`, and deletes the other field. Keep both names.
- It puts `payload_json` back on the list query. That makes the list heavy and shows unmasked detail before the click.
- It logs a friendly name that the panel does not map, then reports the audit as broken.
- It treats an empty dialog as an empty database before reading the JSON keys.

## SSH node section

Test Connection must call `test_ssh_node_connection`. The command is `src-tauri/src/commands/ssh.rs`. It runs `echo AGM_SSH_OK` through `exec_ssh_command`. The frontend is `sshService.testSshNodeConnection` and passes `{ idOrAlias: alias }`.

`send_remote_ssh_prompt` is a different command. Its payload requires `instance_name`. The broken client sent `instance_target` and omitted `instance_name`. Telegram and the error modal then showed E9001: `invalid args payload for command send_remote_ssh_prompt: missing field instance_name`.

Nodes are imported by `import_ssh_nodes_from_gitmap`, which calls `gitmap ssh nodes export-json` (fallback `gitmap ssh nodes --json` in `src-tauri/src/modules/ssh_manager/sync.rs`). Records live in `ssh_nodes.json`, keyed by alias.

### How an AI fails SSH

- It "fixes" Test Connection by calling `send_remote_ssh_prompt` again.
- It adds `instance_name: "default"` to a connection test. A test has no instance.
- It expects the fleet table to fill because the SSH test printed `AGM_SSH_OK`.
- It renames `Node-*` only in the Telegram string and leaves `ssh_nodes.json` and `FleetNodes.node_alias` disagreeing. Display cache is separate from the alias used as the SSH key.
- It logs the node password or the bot token into the RCA.

## Prompt tree

Send from the tree goes through `sendPromptNow` in `src/services/instanceService/lifecycle.ts`, IPC `send_prompt_now`, with `instanceId`, `repoPath`, `promptContent`, and `conversationId`. Enqueue is the FIFO queue (`enqueue_prompt`), not a second send. `agy` does not take `--resume` or `--user-data-dir` for this path. Send must pass the conversation id and surface stderr when the process exits non-zero.

Copy and export, added in `bdf62306`:

- Copy AI results as JSON (`latest_response` and `execution_results` only).
- Copy prompt plus AI as text or markdown.
- Export running prompts and export queued prompts as JSON from the tree already loaded (`downloadPromptTreeStatusJson` in `usePromptTreeCoreActions.ts`).

The export binds the loaded tree with `bindLoadedPromptTree`. If that bind did not run, the file is an empty project list and the button still looks successful.

Hover on the instance table needs `group` on the `<tr>` in `InstanceTable.tsx`. `InstanceRowCells.tsx` then turns the email and the quota figures yellow. A class on the cell alone does not see the row hover.

### How an AI fails the prompt tree

- It wires Send to `enqueue_prompt` or Enqueue to `send_prompt_now`.
- It copies the whole conversation, including the user prompt, into "AI results JSON".
- It exports running rows with a new SQL query that uses `ORDER BY updated_at DESC`. Dispatch queues stay `ORDER BY created_at ASC, id ASC`.
- It adds a copy button whose handler is not passed through `PromptTreeViewModal`. A function that exists only inside the hook does nothing on screen.

## Renaming a conversation

There is no command that renames an Antigravity conversation from the prompt tree or from Telegram. `lookup_conversation_title` in `src-tauri/src/modules/repo_db/text_utils.rs` only reads `conversation_summaries.db`. `AgmConversationNode.title` is the title already stored for the tree. Nothing writes a new title back into the IDE.

An AI fails this by adding a text box that updates React state, or a SQLite `UPDATE` in AGM's own database, and then reporting that the IDE was renamed. The IDE title changes only when the IDE's own summary row changes. Until a write path exists, the picker and the tree must show the title they can read, and the id stays the selection key.

Do not invent `rename_conversation` and do not point Telegram "type it manually" at a rename.

## Checklist

- [ ] Fleet: say whether the card is the disabled state or the empty Shared Cluster. Name which store you will change.
- [ ] Do not make SSH import the thing that fills `get_fleet_machines`.
- [ ] Switch dialog still shows 4h, Weekly, and Total, best score first.
- [ ] Switch still refuses to write credentials while the instance PID is alive.
- [ ] Audit JSON still has `recent_switches` and `total_switches`.
- [ ] Test Connection still invokes `test_ssh_node_connection` with `idOrAlias`.
- [ ] Prompt send and enqueue stay on their own IPC names.
- [ ] No new code claims conversation rename works.
- [ ] FIFO queries stay `ORDER BY created_at ASC, id ASC`.
- [ ] Do not stop `default` or the instance from `agm instances current`.
- [ ] Put the auto-switch threshold back to 15 after a test.

## Must-follow

1. Name the store before editing. SSH alias, fleet heartbeat, and Supabase node are three keys.
2. A successful `echo AGM_SSH_OK` proves the shell. It does not prove a fleet row.
3. Close the PID, then write credentials. Never the other way around.
4. Keep the audit field names the dialog already reads. Add a name if you must. Do not remove `recent_switches`.
5. Connection tests do not send a prompt and do not require `instance_name`.
6. Leave conversation titles read-only until a real IDE write exists.
7. One file owner per change. `send_flow.rs` is not the fleet table.
