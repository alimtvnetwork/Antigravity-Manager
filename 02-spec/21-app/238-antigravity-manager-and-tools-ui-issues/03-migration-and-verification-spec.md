# 238 - Antigravity Manager and Tools UI Issues: Migration & Verification Specification

## 1. Migration Overview
This specification details the testing matrix, automated pre-flight checks, and end-to-end verification steps required to validate all fixes across `Antigravity Manager` and `Antigravity Tools`.

---

## 2. Test Matrix & Verification Checklist

| Test ID | Subsystem | Action / Scenario | Expected Outcome | Status |
|---|---|---|---|---|
| TC-01 | Cargo Installation | Verify `cargo` and `rustc` installed on C: drive | `cargo --version` returns stable version without errors | Pending |
| TC-02 | Immediate CDP Enqueue | Click Enqueue / Rerun with active IDE | Prompt enters IDE input immediately via CDP with Enter | Pending |
| TC-03 | Prompt Tree Enqueue Banner | Enqueue 1 prompt during active run | Tree reflects QUEUED status immediately, DetailHeader shows FIFO banner | Pending |
| TC-04 | FIFO Queue Review Invariant | Inspect prompt review screen | Explains pending FIFO count accurately without confusion | Pending |
| TC-05 | 50x Switch & Restore | Auto-switch account repeatedly | Restores running prompt across iterations without losing context | Pending |
| TC-06 | 5–10s Post-Switch Masking | Trigger account switch in GUI | Prompts masked with privacy shield during 5–10s settling window | Pending |
| TC-07 | First-Open SSH Cluster Sync | Launch app with seed files present | Seeds nodes into `ssh_nodes.db` automatically on startup | Pending |
| TC-08 | Fleet Auto-Refresh | Monitor fleet section without clicks | Refreshes node metrics and status automatically every 30s | Pending |
| TC-09 | Telegram Prompt Enqueue | Enqueue prompt received via Telegram | Dispatches to IDE queue cleanly matching direct UI enqueue | Pending |
| TC-10 | Instance Improvements | Open Improvements inside InstanceCard | Renders inside card, defaults to 20, allows task count options | Pending |
| TC-11 | Auto-Enqueue 20 Improvements | Click Auto-Enqueue Improvements | Submits 20 coding guideline improvement requests to active IDE | Pending |
| TC-12 | Button Capsule Consistency | Inspect DetailHeader Copy & Send | Both capsules render matching `rounded-full` contiguous pill shapes | Pending |
| TC-13 | CLI UI Manipulation | Run `agm ui ls` and `agm ui goto` | Manipulates running GUI windows and routes successfully | Pending |
| TC-14 | Codebase Build Verification | Run frontend build & rust clippy | `npm run build` and `cargo clippy` succeed with 0 errors | Pending |

---

## 3. Pre-flight Verification Commands
1. **Frontend Build**:
   ```bash
   npm run build
   ```
2. **Rust Gate**:
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   cd src-tauri && cargo fmt -- --check
   ```
3. **CLI UI Verification**:
   ```bash
   agm ui ls
   agm ui open --url "/instances"
   ```
