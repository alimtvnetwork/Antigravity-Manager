# Subtask 01: FastForward Sibling Running Account RCA & Fix (Error E9001)

**Application:** Antigravity Manager (AGM)  
**Task Slug:** `antigravity-manager-agm-codebase-review-and-refactoring`  
**Subtask ID:** `01-fastforward-sibling-rca-and-fix`  
**Status:** Ready / Implemented  
**Spec Reference:** [02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/01-architecture-spec.md](../../../../02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/01-architecture-spec.md)

---

## 1. Problem Statement & Forensic RCA

### 1.1 Error Signature
```
Cannot switch instance to account 'cirajayu11@gmail.com': Account is actively bound to running sibling instance 'Default' on this machine
```

### 1.2 Forensic Root Cause Analysis
1. In `src-tauri/src/modules/auto_switcher/instances.rs`, function `is_instance_binding_stale_or_exhausted` evaluated whether an instance binding could be harvested or re-assigned.
2. The logic checked if the bound account had `q_4h <= 0.0`. If true, it returned `true` (exhausted) without checking `if is_running { return false; }`.
3. When `cirajayu11@gmail.com` had 0% 4-hour window quota on running instance `'Default'`, it was marked stale/exhausted and omitted from `get_active_in_use_account_ids()`.
4. Candidate selection in `src-tauri/src/modules/auto_switcher/selection.rs` (`select_candidate_profiles`) and manual rotation in `manual.rs` saw `cirajayu11@gmail.com` as an available candidate and selected it for another instance.
5. The switch pipeline progressed to `src-tauri/src/modules/instance/switch.rs:59`, where the OS-level running sibling guard detected that instance `'Default'` was actively running with this account, halting the operation with Error E9001.

---

## 2. Technical Implementation Plan

### 2.1 Update `src-tauri/src/modules/auto_switcher/instances.rs`
In `is_instance_binding_stale_or_exhausted`:
- Immediately return `false` if `is_running == true`:
  ```rust
  pub fn is_instance_binding_stale_or_exhausted(
      inst: &crate::models::InstanceConfig,
      is_running: bool,
      stale_timeout_hours: u32,
      target_model: &str,
      now_sec: i64,
  ) -> bool {
      // If actively running, an instance's binding is NEVER stale or available for theft!
      if is_running {
          return false;
      }
      
      let stale_timeout_secs = (stale_timeout_hours.clamp(1, 24) as i64) * 3600;
      let is_stale_time = inst.last_used > 0 && (now_sec - inst.last_used) > stale_timeout_secs;

      if !is_running && is_stale_time {
          return true;
      }

      if let Some(ref acc_id) = inst.bound_account_id {
          if let Ok(acc) = account::load_account(acc_id) {
              let q_4h = calculate_4h_window_quota(&acc, target_model).unwrap_or(100.0);
              if q_4h <= 0.0 {
                  return true;
              }
          }
      }

      false
  }
  ```

### 2.2 Update `src-tauri/src/modules/auto_switcher/selection.rs`
In `select_candidate_profiles`:
- Section 1 and Section 2 candidate loops must explicitly filter out accounts that are bound to or actively running on ANY sibling instance on the host machine:
  ```rust
  let is_bound_to_running_sibling = registry.instances.iter().any(|i| {
      if i.id == current_instance_id {
          return false;
      }
      if !crate::modules::instance::is_instance_running(&i.id, &i.data_dir, i.pid) {
          return false;
      }
      i.bound_account_id.as_deref() == Some(&acc.id)
          || i.bound_email.as_deref().map(|e| e.eq_ignore_ascii_case(&acc.email)).unwrap_or(false)
          || (i.is_default && account::get_current_account_id().ok().flatten().as_deref() == Some(&acc.id))
  });
  if is_bound_to_running_sibling {
      continue;
  }
  ```
- In `select_and_verify_next_best_profile`, add an assertion to verify that the chosen profile is not bound to a running sibling before returning.

### 2.3 Update `src-tauri/src/modules/auto_switcher/manual.rs`
In `trigger_manual_rotation`:
- Expand the `excluded` list to collect all accounts bound to or active on running sibling instances:
  ```rust
  for inst in &registry.instances {
      if inst.id != inst_id && instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid) {
          let active_acc_id = inst.bound_account_id.clone().or_else(|| {
              if inst.is_default || inst.id == "default" {
                  account::get_current_account_id().ok().flatten()
              } else {
                  None
              }
          });
          if let Some(ref acc_id) = active_acc_id {
              if !excluded.contains(acc_id) {
                  excluded.push(acc_id.clone());
              }
              if let Ok(acc) = account::load_account(acc_id) {
                  if !excluded.contains(&acc.email) {
                      excluded.push(acc.email.clone());
                  }
              }
          }
          if let Some(ref email) = inst.bound_email {
              if !excluded.contains(email) {
                  excluded.push(email.clone());
              }
          }
      }
  }
  ```

---

## 3. Invariants & Positive Booleans

- `is_running`: Positive boolean indicating if the target or sibling instance process is alive.
- `is_bound_to_running_sibling`: Positive boolean indicating if an account is associated with a running sibling.
- Never use negative booleans like `is_not_running` or `unbound`.

---

## 4. Verification & Testing

```bash
cd src-tauri && cargo test -p antigravity-tools-lib auto_switcher
cd src-tauri && cargo clippy --all-targets --all-features
```

### Verification Criteria
- [x] Account with `q_4h == 0.0` on a running sibling returns `is_instance_binding_stale_or_exhausted == false`.
- [x] Candidate selection ignores accounts on running siblings regardless of quota score.
- [x] `switch.rs:59` Sibling Local Running Guard is never tripped during automated rotation or fast-forward.
