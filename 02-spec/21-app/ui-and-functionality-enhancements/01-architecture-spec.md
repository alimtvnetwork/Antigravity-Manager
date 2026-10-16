# UI and functionality enhancements

## User request

Test Connection on SSH and GitMap failed with E9001: `send_remote_ssh_prompt` missing `instance_name`. The switch-account dialog must show a combined total, the weekly number, and the 4-hour number, with the best account first. Card hover should brighten the email and the hour and percent.

## What the running v4.210.0 build does

`src/services/sshService.ts` called `send_remote_ssh_prompt` with `instance_target`. The Rust command `send_remote_ssh_prompt` requires `instance_name` (`src-tauri/src/modules/ssh_manager/types.rs`). Test Connection is not a prompt. It must call `test_ssh_node_connection` (`src-tauri/src/commands/ssh.rs`).

## What the working tree does

- `src/services/sshService.ts` `testSshNodeConnection` invokes `test_ssh_node_connection` with `idOrAlias`.
- `src/pages/instances/SwitchAccountDialog.tsx` sorts by 4-hour plus weekly and prints `4h`, `Weekly`, and `Total`.
- `src/pages/instances/InstanceCard.tsx` turns the email and quota labels yellow on hover.

## Still open

Conversation rename from the prompt tree, copy of prompt plus AI results as text and markdown, and a check that a cloned exe is refreshed when the IDE binary changes. Those are not part of this commit.
