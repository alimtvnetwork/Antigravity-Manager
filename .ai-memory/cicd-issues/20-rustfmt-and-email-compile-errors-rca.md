# 4-Part Root Cause Analysis: Rustfmt Discrepancies and Email Module Compilation Errors

> **Version:** 1.0.0
> **Date:** 2026-09-18
> **Failed Runs:** [#35364778178](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35364778178) (CI)
> **Trigger Commit:** `b3327d37`
> **Fixed In:** `HEAD`

---

## 4-Part Root Cause Analysis (RCA)

### 1. Symptom (Why it happened)
In GitHub Actions CI workflow run [#35364778178](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35364778178), all matrix runner platforms (`windows-2025`, `ubuntu-latest`, and `macos-latest`) failed at two distinct pipeline stages:

1. **`Check Rust formatting` (`Check Rust Code` job):**
   Formatting diffs occurred in 7 Rust files:
   - `src-tauri/src/commands/email.rs` (4 diff hunks)
   - `src-tauri/src/modules/auto_switcher.rs` (1 diff hunk)
   - `src-tauri/src/modules/email_inbound.rs` (25 diff hunks)
   - `src-tauri/src/modules/email_io.rs` (5 diff hunks)
   - `src-tauri/src/modules/email_sender.rs` (7 diff hunks)
   - `src-tauri/src/modules/email_vault_db.rs` (4 diff hunks)
   - `src-tauri/src/modules/email_watcher.rs` (4 diff hunks)

2. **`Build Tauri app (debug)` (`Build Tauri App` job):**
   ```text
   error[E0308]: mismatched types
      --> src/modules/email_inbound.rs:324:72
       |
   324 |             let create_res = crate::modules::instance::create_instance(&profile_name);
       |                              ----------------------------------------- ^^^^^^^^^^^^^ expected `String`, found `&String`
   note: function defined here
      --> src/modules/instance.rs:204:8
       |
   204 | pub fn create_instance(name: String) -> Result<InstanceConfig, String> {

   error[E0382]: borrow of moved value: `acc.email`
     --> src/modules/email_io.rs:81:74
      |
   67 |             email: acc.email,
      |                    --------- value moved here
   ...
   81 |             summary.errors.push(format!("Failed to import account '{}'", acc.email));
      |                                                                          ^^^^^^^^^^ value borrowed here after move
   ```

### 2. How it happened & Root Cause
#### How it happened:
- In commits `82a3fbc6`, `56636156`, and `b3327d37`, the remote email automation subsystem was introduced, spanning inbound message parsing, IMAP/SMTP transport, split database vaults, JSON/CSV/Excel serialization, and background watchers.
- In `email_inbound.rs`, `create_instance` was invoked with a borrowed reference `&profile_name` instead of an owned `String`.
- In `email_io.rs`, `acc.email` was moved into the struct initialization of `EmailAccountInput`, making it illegal under Rust borrow checking to reference `acc.email` in the `else` error branch.
- Without a locally installed `cargo fmt` toolchain on the Windows host, multiline method chains, format strings, and match expressions were written with styling variations that differed from standard `rustfmt` formatting defaults.

#### Root Cause:
1. **Ownership / Type Mismatch:** `crate::modules::instance::create_instance` requires `name: String`. Passing `&profile_name` caused type mismatch `E0308`.
2. **Move Before Error Reference:** Moving `acc.email` into `EmailAccountInput` caused `E0382` when constructing the failure diagnostic message in `summary.errors.push(...)`.
3. **Rustfmt Heuristic Discrepancies:** Multiple long method chains and format macros in email modules exceeded the 100-character line length threshold, necessitating standard multiline line-wrapping.

### 3. Resolution
1. **`src-tauri/src/modules/email_inbound.rs`:**
   - Changed `create_instance(&profile_name)` to `create_instance(profile_name.clone())`.
   - Applied all 25 `rustfmt` formatting hunks across prompt injection, CLI execution, IMAP commands, and unit tests.
2. **`src-tauri/src/modules/email_io.rs`:**
   - Cloned `let acc_email = acc.email.clone();` prior to moving `acc.email` into `EmailAccountInput`, allowing `acc_email` to be safely used in error formatting.
   - Applied all 5 `rustfmt` formatting hunks across JSON, CSV, and Excel XML serialization.
3. **`src-tauri/src/commands/email.rs`:**
   - Applied 4 `rustfmt` formatting hunks to error mapping and multi-line message handlers.
4. **`src-tauri/src/modules/auto_switcher.rs`:**
   - Applied multi-line formatting to `notify_workspace_switched`.
5. **`src-tauri/src/modules/email_sender.rs`, `email_vault_db.rs`, `email_watcher.rs`:**
   - Applied all remaining `rustfmt` formatting hunks across email card wrappers, SQL executions, and background telemetry loops.

### 4. Prevention & Learnings
1. **Signature Contract Auditing:** When calling external module functions (`create_instance`), verify ownership semantics (`String` vs `&str` vs `&String`) in the callee definition.
2. **Pre-Move Cloning for Diagnostics:** When a field is moved into a data structure that might fail validation/insertion, clone the diagnostic identifier beforehand.
3. **CI-Aligned Formatting:** Follow strict 100-character line wrapping for method chains, format strings, and struct initializers.
