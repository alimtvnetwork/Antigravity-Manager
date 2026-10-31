# Verification and Root Cause Analysis (RCA) Spec

## 1. 4-Part Grounded Root Cause Analysis

### 1.1 Discrepancy in Prompt Tree Timestamps & Running Turns
- **Symptom**: User IDE shows 16 turns with Turn 16 running and 4 queued messages, but AGM prompt tree showed turns up to T12 and marked running turns as COMPLETED, with inaccurate duration calculations.
- **Root Cause**:
  1. `tree_collect.rs` checked `conversation_summaries.status == "IDLE"` from SQLite without cross-verifying active transcript write activity (`is_recent_active && is_owning_inst_alive`). This caused running conversations to be falsely flagged as completed.
  2. Timestamp delta calculation in `transcript.rs` used the entire conversation lifetime instead of isolating individual turn start/end timestamps.
- **Remediation**:
  1. Enforce live transcript modification supremacy in `tree_collect.rs`.
  2. Isolate each turn's timestamps based on the `USER_INPUT` start time and final assistant response / tool completion time.
  3. Format turn headers with explicit wall-clock times and elapsed durations.

### 1.2 Missing Image Previews in UI
- **Symptom**: Attached images were blank or showed broken image icons in `PromptThumbnailCard`.
- **Root Cause**:
  1. Tauri `default.json` capability configuration lacked `core:asset:default`, preventing WebView2 from serving local file URIs via `convertFileSrc`.
  2. `cmd_files.rs` had `is_sensitive_path` checks that blocked `C:\Users\Administrator`, preventing `read_image_as_base64` from reading images when logged in as Administrator.
- **Remediation**:
  1. Add `core:asset:default` to Tauri capabilities.
  2. Permit legitimate user media paths in `cmd_files.rs` while preserving traversal defenses.
  3. Preload images as Base64 data URLs in `PromptInstructionView.tsx`.

### 1.3 Telegram Machine Identification & GitMap SSH Connectivity
- **Symptom**: AGM identified physical machine `192.168.1.3` as `main` instead of `w1`, and cross-machine prompt sending lacked acknowledgment evidence.
- **Root Cause**:
  1. Windows cloned systems share the hostname `AI-Main`. Hostname lookup is not unique.
  2. AGM did not resolve the local IP against GitMap's cluster configuration (`~/.gitmap/gitmap-ssh-nodes.json`).
- **Remediation**:
  1. Match local IP `192.168.1.3` against GitMap SSH nodes to resolve machine alias to `w1 (collected from gitmap)`.
  2. Ensure Telegram report blocks cleanly format `Machine:` and `Instance:`.
  3. Route cross-machine prompt dispatches over GitMap SSH port 22 with screenshot acknowledgment.

---

## 2. Verification Protocol

1. **Pre-flight Gate**:
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `cargo test --lib telegram_inbound`
   - `npm run build`
2. **Empirical Verification**:
   - Verify prompt tree rendering in React frontend.
   - Test image clipboard copying via `writePureImageToClipboard`.
   - Verify GitMap SSH node resolution.
