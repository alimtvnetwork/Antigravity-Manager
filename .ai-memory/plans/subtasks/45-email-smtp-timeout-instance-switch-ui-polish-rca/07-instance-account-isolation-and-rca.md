# Subtask: Instance Account Isolation & Token Injection Fix + Detailed RCA

**Target Files:**
- `src-tauri/src/modules/instance.rs`
- `src-tauri/src/modules/integration.rs`
- `02-spec/20-instance-management/01-instance-switching-rca.md`
- `.ai-memory/issues/45-instance-switching-rca.md`

**Action:**
1. In `src-tauri/src/modules/instance.rs`:
   - In `launch_instance`, when launching an instance that has a `bound_account_id`:
     - Load the bound account's credentials.
     - Call `crate::modules::db::inject_token(&account, &instance_state_vscdb_path)`.
     - Ensure the instance's isolated `state.vscdb` (`{data_dir}/User/globalStorage/state.vscdb`) receives the bound account's OAuth token and status before process spawn.
   - In `copy_instance`:
     - If the new cloned instance has `bound_account_id: None`, reset or sanitize the cloned `state.vscdb` so it does not inadvertently inherit the source profile's active user session.
   - In `clone_instance_executable`:
     - Provide true binary copy fallback (`fs::copy`) if hardlink is not sufficient on Windows.
2. In `02-spec/20-instance-management/01-instance-switching-rca.md` & `.ai-memory/issues/45-instance-switching-rca.md`:
   - Write an exhaustive 4-part Root Cause Analysis detailing:
     1. Symptoms & Observed Behavior (both instances showing Ashley EscobarZu).
     2. Code Defect (launch only updated global Windows Credential Manager, leaving instance `state.vscdb` untouched).
     3. Architectural Fix (isolated SQLite `state.vscdb` token injection + session sanitization on clone).
     4. Prevention Guardrails.

**Constraints:**
- Must update the isolated `state.vscdb` inside the instance's custom user data folder.
- Never write credentials into unauthenticated global storage.
