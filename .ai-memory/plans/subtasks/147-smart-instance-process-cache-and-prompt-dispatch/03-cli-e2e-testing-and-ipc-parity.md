---
plan: 147-smart-instance-process-cache-and-prompt-dispatch
subtask: "03"
title: CLI Prompt Verbs Parity, Safe Python E2E Verification Suite & Host PID Shielding
domain: cli/e2e-testing/ipc
target_files:
  - src-tauri/src/bin/agm.rs
  - src-tauri/src/modules/cli.rs
  - 03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py
  - 02-spec/21-app/147-smart-instance-process-cache-and-prompt-dispatch/02-component-and-cli-spec.md
status: pending
---

# 03 — CLI Prompt Verbs Parity, Safe Python E2E Verification Suite & Host PID Shielding

## 1. Context & Objectives

To achieve 100% operational parity between the Antigravity-Manager GUI and terminal environments, CLI prompt commands (`agm prompt send`, `agm prompt queue`, `agm prompt running`, and `agm instance status`) must utilize the exact same smart process cache and liveness verification pipeline as the GUI.

### Core Objectives:
1. **CLI Verbs Implementation & Parity:**
   - Verify and enhance CLI verbs in `src-tauri/src/bin/agm.rs` and `src-tauri/src/modules/cli.rs`:
     * `agm prompt send <instance_id> <prompt_text>`: Check process cache and OS table before deciding whether to dispatch directly or trigger launch.
     * `agm prompt queue <instance_id> <prompt_text>`: Store in FIFO backlog with status `queued` (`ORDER BY created_at ASC, id ASC`).
     * `agm prompt running [--instance <id>]`: Return active prompts filtered against false positives.
     * `agm instance status <instance_id>`: Query cached PID, verified OS process state, and live metrics.
2. **Automated E2E Verification Harness:**
   - Author `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py` covering all 5 core test cases.
3. **Safety Gate & Host PID Protection:**
   - Implement strict PID tracking before test execution to shield existing developer/host IDE windows from termination or state corruption.

---

## 2. CLI Command Implementation Specification

### 2.1 Argument Parsing & Dispatch Wiring (`src-tauri/src/modules/cli.rs`)
Extend `CliContext` to parse positional and flag parameters cleanly:
```rust
impl CliContext {
    pub fn parse(args: &[String]) -> Self { ... }

    /// Resolves target instance: checks positional instance argument first,
    /// then `--instance` flag, falling back to default active instance.
    pub fn resolve_target_instance(&self) -> Result<String, String> { ... }
}
```

### 2.2 CLI Verbs in `src-tauri/src/bin/agm.rs`

#### 1. `agm prompt send <instance_id> <prompt_text>`
```rust
fn cmd_prompts_send(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx.resolve_target_instance().unwrap_or_else(|_| "default".to_string());
    let prompt_text = ctx.positional_args.join(" ");

    // 1. Check Smart Process Cache & OS Liveness
    let is_alive = instance::is_instance_process_alive(&target_instance);

    if !is_alive {
        // Double-check system process table to prevent false cache miss
        let pids = instance::find_pids_for_data_dir(&inst_data_dir, is_default);
        if pids.is_empty() {
            // Truly dead -> Launch instance and register new PID
            let _ = instance::launch_instance(&target_instance);
        } else {
            // Re-cache active PID
            let _ = instance::record_instance_pid(&target_instance, pids[0], &inst_data_dir);
        }
    }

    // 2. Dispatch prompt payload directly without relaunching
    let active_prompt = repo_db::dispatch_prompt_to_instance(&target_instance, &prompt_text)?;
    
    if ctx.json_output {
        CliEnvelope::ok("prompt send", Some(target_instance), active_prompt).print_and_exit();
    }
}
```

#### 2. `agm prompt queue <instance_id> <prompt_text>`
```rust
fn cmd_prompts_queue(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx.resolve_target_instance().unwrap_or_else(|_| "default".to_string());
    let prompt_text = ctx.positional_args.join(" ");

    // Enqueue prompt into FIFO table with status 'queued'
    let queued_item = repo_db::enqueue_prompt_for_instance(&target_instance, &prompt_text)?;

    if ctx.json_output {
        CliEnvelope::ok("prompt queue", Some(target_instance), queued_item).print_and_exit();
    }
}
```

#### 3. `agm prompt running [--instance <id>]`
```rust
fn cmd_prompts_running(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx.resolve_target_instance().unwrap_or_else(|_| "default".to_string());

    // Query running prompts with false-positive elimination
    let running = repo_db::list_verified_running_prompts(&target_instance)?;

    if ctx.json_output {
        CliEnvelope::ok("prompt running", Some(target_instance), running).print_and_exit();
    }
}
```

#### 4. `agm instance status <instance_id>`
```rust
fn cmd_instances_status(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx.resolve_target_instance().unwrap_or_else(|_| "default".to_string());

    let status = instance::get_instance_deep_status(&target_instance)?;

    if ctx.json_output {
        CliEnvelope::ok("instance status", Some(target_instance), status).print_and_exit();
    }
}
```

---

## 3. Automated Python E2E Verification Harness Design

Script Target: `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`

### 3.1 Host Process Protection Safety Gate
Before running tests, the script inventories all active Antigravity and IDE processes across the host OS:

```python
def inventory_host_protected_pids() -> set[int]:
    """Capture all existing IDE PIDs across the OS to ensure they are NEVER touched."""
    protected = set()
    for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
        try:
            name = (proc.info['name'] or '').lower()
            if 'antigravity' in name or 'code' in name or 'electron' in name:
                protected.add(proc.info['pid'])
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
    print(f"[SAFETY GATE] Shielded {len(protected)} host IDE processes from test mutations.")
    return protected
```

### 3.2 Five Test Cases Implementation Structure

```python
class TestPromptDispatchAndProcessCache:
    @classmethod
    def setUpClass(cls):
        cls.shielded_pids = inventory_host_protected_pids()
        cls.test_instance_id = "test-e2e-cache-147"
        setup_sandbox_instance(cls.test_instance_id)

    @classmethod
    def tearDownClass(cls):
        cleanup_sandbox_instance(cls.test_instance_id)
        # Assert no shielded PIDs were killed or altered
        verify_shielded_pids_unaltered(cls.shielded_pids)

    def test_01_smart_process_cache_hit_prevents_relaunch(self):
        """TC01: Verify sending prompt preserves PID and never relaunches IDE."""
        ...

    def test_02_process_death_double_check_and_graceful_relaunch(self):
        """TC02: Verify relaunch triggers only if PID is confirmed dead in OS table."""
        ...

    def test_03_prompt_enqueue_ipc_and_fifo_storage(self):
        """TC03: Verify queueing persists with status 'queued' in FIFO order."""
        ...

    def test_04_false_positive_running_elimination(self):
        """TC04: Verify idle or completed conversations evaluate to idle."""
        ...

    def test_05_ui_tag_compaction_verification(self):
        """TC05: Verify UI contains compacted badges and no redundant uppercase tags."""
        ...
```

---

## 4. Verification & Quality Gates

Run the test harness:
```bash
python 03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py
```

### Acceptance Invariants:
- All 5 test cases pass cleanly with exit code 0.
- Zero shielded host PIDs killed, terminated, or disrupted.
- CLI JSON output strictly matches `CliEnvelope<T>` schema.
- Tree view UI code adheres to compacted badge format (`P001 · #1`, `C001 · <cid>`).
