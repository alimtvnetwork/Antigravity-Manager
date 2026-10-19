# Subtask 03: Cross-Machine Telegram Relay & GitMap SSH Mesh

## Status: COMPLETED
- **Assigned Subagent**: Subagent 3 (Cross-Machine Relay Specialist)
- **Target Files**:
  - `src-tauri/src/modules/telegram_inbound/send_flow.rs`
  - `src-tauri/src/modules/telegram_inbound/injection.rs`
  - `src-tauri/src/modules/telegram_inbound/open_shot.rs`

## Problem Analysis
1. User noted: "Currently, we still cannot connect with two machines where GitMap can. Also follow through the telegram steps to send prompt from one machine to another. Make sure that it works."
2. The root cause was identified: in `send_flow.rs::on_machine_picked` and `on_instance_picked`, remote machine callbacks were unconditionally dropped (`if !is_current { return (String::new(), None); }`), completely silencing cross-machine prompt flow whenever the active Telegram bot received a callback for a remote machine.
3. Once the drop guards were removed, the coordinator node cleanly loads instances from the target node, prompts for destination conversation, and delegates remote prompt injection over GitMap SSH (`ssh_manager::exec_via_gitmap_ssh`).

## Acceptance Criteria
- [x] Verify Telegram `/send` flow in `send_flow.rs` and `injection.rs` captures system timestamps, machine identity, and target instance.
- [x] Verify GitMap SSH connectivity between node `w1` (192.168.1.3) and `main`/`w2` (192.168.1.20) — verified 100% green via `agm ssh w1`, `agm ssh main`, and `gitmap ssh exec`.
- [x] Confirm synthetic dispatch card generation in `open_shot.rs` with date/time stamps and acknowledgment proof when operating in headless or remote SSH sessions.
