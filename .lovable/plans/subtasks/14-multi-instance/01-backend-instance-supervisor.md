# Subtask 01: Backend Instance Supervisor & Ubuntu Process Isolation

> **Subtask Path:** `.lovable/plans/subtasks/14-multi-instance/01-backend-instance-supervisor.md`
> **Parent Plan:** [.lovable/plans/pending/14-multi-instance-orchestration-and-ubuntu-parallelism.md](../../pending/14-multi-instance-orchestration-and-ubuntu-parallelism.md)
> **Status:** Pending Review

---

## 1. Objectives

1. Define Rust data model `AntigravityInstance` in `src-tauri/src/models.rs`.
2. Implement instance lifecycle manager in `src-tauri/src/modules/instance.rs`:
   - Load and save instances in `instances.json`.
   - Create instance directory structure (`data/`, `extensions/`).
   - Duplicate instance (deep copy of `settings.json` and extension manifests).
   - Delete instance (safety check: fail if currently running).
3. Update `src-tauri/src/modules/process.rs` for selective termination:
   - Match `/proc/<pid>/cmdline` on Linux and `process.cmd()` on Windows for `--user-data-dir`.
   - Never kill sibling instances.
   - Inject `--password-store="basic"` and call `clean_appimage_env()` on Linux.
4. Expose Tauri IPC commands in `src-tauri/src/lib.rs`:
   - `get_instances`
   - `create_instance`
   - `duplicate_instance`
   - `delete_instance`
   - `launch_in_instance`
   - `terminate_instance`

---

## 2. Technical Contracts

```rust
// In src-tauri/src/models.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityInstance {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub profile_path: String,
    pub assigned_account_id: Option<String>,
    pub is_running: bool,
    pub pid: Option<u32>,
    pub last_launched_at: Option<i64>,
}
```

---

## 3. Acceptance Criteria

- [ ] All functions adhere to `<= 15` lines where practical.
- [ ] No `killall` or blanket process termination on Linux or Windows.
- [ ] `instances.json` persists across application reboots.
