# Subtask 03: Telegram Prompt Acknowledgment, Date/Time & GitMap SSH Mesh

## 1. Subtask Metadata
- **ID**: `Task-05`
- **Slug**: `root-cause-analysis-and-end-to-end-testing`
- **Owner**: Worker 03
- **Status**: Ready for Execution
- **Exclusive Owned Files**:
  - `src-tauri/src/modules/telegram_inbound/injection.rs`
  - `src-tauri/src/modules/telegram_inbound/send_flow.rs`
  - `src-tauri/src/modules/telegram_inbound/ui_remote.rs`
  - `src-tauri/src/modules/telegram_inbound/open_shot.rs`
  - `src-tauri/src/modules/supabase_sync/query.rs`
  - `src/components/instances/fleet/useFleetMachines.ts`

---

## 2. Action Items
1. **Add Date and Time to All Telegram Prompt Acknowledgment Envelopes**:
   - In `injection.rs`, `send_flow.rs`, and `ui_remote.rs`, format current UTC and local timestamps.
   - Include:
     `• <b>Dispatched At:</b> {Utc::now().to_rfc3339()} ({Local::now().format("%Y-%m-%d %I:%M:%S %p")})`
     in every returned card so the user can verify the exact dispatch time.
2. **Session 0 Safe Proof Capture**:
   - In `open_shot.rs`, wrap remote screenshot execution with graceful error handling.
   - When remote PowerShell `CopyFromScreen` fails due to Session 0 non-interactive window station, fall back to checking if AGM local proxy is listening or inform the user with a clean status message rather than a crashing error.
3. **GitMap SSH Mesh as Primary Cross-Machine Fleet Backbone**:
   - In `src-tauri/src/modules/supabase_sync/query.rs` and `useFleetMachines.ts`:
     - When Supabase PostgreSQL returns an error or is paused (NXDOMAIN), automatically parse `~/.gitmap/gitmap-ssh-nodes.json` and SSH known hosts.
     - Return the healthy GitMap nodes (`w1`, `main`) so the fleet table and prompt sending operate seamlessly without cloud dependencies.
4. **End-to-End Testing Script**:
   - Create or run an isolated script to verify cross-machine prompt sending and Telegram response formatting.

---

## 3. Verification Criteria
- [ ] Telegram prompt dispatch acknowledgment includes explicit Date and Time.
- [ ] Remote proof shot does not crash or fail the prompt dispatch.
- [ ] Cross-machine communication works over GitMap SSH port 22 even when Supabase is paused.
