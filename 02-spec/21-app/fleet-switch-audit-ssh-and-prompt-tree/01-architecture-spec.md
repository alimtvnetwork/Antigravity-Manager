# Fleet, switch, audit, SSH, and prompt tree

The root-cause write-up is `.ai-memory/rca/19-fleet-switch-audit-ssh-and-prompt-tree.md`. Read that file before changing these screens. This spec is the contract.

## Fleet

The instance fleet table lists machines that have heartbeated while Supabase sync is enabled.

- Sync off: `FleetDisabledState` in `src/components/instances/fleet/FleetTableStates.tsx`. The operator opens Supabase settings.
- Sync on and no rows: `FleetEmptyState` ("Shared Cluster"). SSH nodes are a different list, under Settings, SSH and GitMap.
- Heartbeats land in `fleet_sync.db` `FleetNodes` (`src-tauri/src/modules/fleet_sync.rs`).
- SSH aliases land in `ssh_nodes.json`. A passing SSH test does not insert a fleet row.

## Account switch

`SwitchAccountDialog.tsx` orders accounts by the sum of the 4-hour model percentage and the weekly remaining fraction. The row shows 4h, Weekly, and Total.

The switch command closes the instance PID and only then writes credentials. Failure to close is an error. Windows liveness uses the process exit code. See RCA 17.

## Audit

`get_instance_switch_history` returns `recent_switches` and `total_switches` plus the older `records` and `items` keys. The dialog reads the first pair. List payloads omit `payload_json`. `AuditPanel.tsx` maps `send_prompt_now` to "Send prompt now".

## SSH

`test_ssh_node_connection` runs `echo AGM_SSH_OK` for an alias. The client passes `idOrAlias`. `send_remote_ssh_prompt` is the prompt command and requires `instance_name`. Test Connection must not call it.

Import uses `gitmap ssh nodes export-json`.

## Prompt tree

- Send: `send_prompt_now` with instance id, repo path, prompt content, and conversation id.
- Enqueue: `enqueue_prompt`. FIFO. `ORDER BY created_at ASC, id ASC`.
- Copy: AI results JSON is `latest_response` and `execution_results`. Text and markdown include the prompt plus the AI text.
- Export: running-only and queued-only JSON from the tree already bound in the view.

## Conversation rename

Out of scope until an IDE write exists. `lookup_conversation_title` is read-only. A local text field is not a rename.

## Acceptance

- Fleet empty and fleet disabled stay different screens.
- Switch sort and the three quota numbers stay.
- Audit empty state happens only when `recent_switches` is empty.
- Test Connection invokes `test_ssh_node_connection`.
- Send and enqueue stay on different commands.
- No UI string says a conversation was renamed in the IDE.
