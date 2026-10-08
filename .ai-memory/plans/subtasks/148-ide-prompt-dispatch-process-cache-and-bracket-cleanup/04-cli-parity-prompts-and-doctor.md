# Subtask 04: CLI Parity: prompts & doctor

- **Subtask ID**: `148-04`
- **Parent Task**: `148-ide-prompt-dispatch-process-cache-and-bracket-cleanup`
- **Target Files**:
  - `src-tauri/src/modules/cli.rs`
- **Maintainer / Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)
- **Status**: READY_FOR_EXECUTION

---

## 1. Objective

Implement complete headless CLI command handlers for `prompts` (`ls`, `tree`, `send`, `enqueue`, `backup`, `restore`) and `doctor` in `src-tauri/src/modules/cli.rs`. Provide full functional parity with the GUI, supporting human-readable terminal output and structured machine-readable JSON envelopes (`--json`).

---

## 2. Command Syntax & Argument Specifications

### 2.1 Summary of CLI Commands

| Command | Arguments & Options | Description |
|---|---|---|
| `agm prompts ls` | `[-i <instance>] [-r <repo>] [--json]` | List active, queued, or running prompts across instances. |
| `agm prompts tree` | `[-i <instance>] [--only-running] [--json]` | Output hierarchical project conversation tree with clean sequence badges. |
| `agm prompts send` | `<prompt_text> [-i <instance>] [-r <repo>] [--json]` | Dispatch prompt to running instance using Smart Process Cache & clipboard. |
| `agm prompts enqueue` | `<prompt_text> [-i <instance>] [-r <repo>] [--json]` | Enqueue prompt in SQLite database for sequential execution. |
| `agm prompts backup` | `[-i <instance>] [--json]` | Trigger an on-demand snapshot of running prompts to `prompt_backups`. |
| `agm prompts restore` | `<batch_id> [-i <instance>] [--json]` | Restore prompt state from a previous backup batch. |
| `agm doctor` | `[--verbose] [--json]` | Perform comprehensive health probe on processes, paths, SQLite DBs, and CLI tools. |

---

## 3. JSON Output Schemas

All CLI commands supporting `--json` return a standardized `CliEnvelope<T>` envelope:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliEnvelope<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CliErrorPayload>,
    pub meta: CliMetaPayload,
}
```

### 3.1 `agm prompts ls --json` Schema

```json
{
  "success": true,
  "data": {
    "total_prompts": 2,
    "prompts": [
      {
        "id": "prompt-12345",
        "instance_id": "default",
        "repo_path": "git-work/antigravity-manager",
        "prompt_content": "Analyze memory leaks in process scanner",
        "status": "running",
        "created_at": 1728475200,
        "updated_at": 1728475260,
        "is_running": true
      }
    ]
  },
  "meta": {
    "command": "prompts ls",
    "instance_id": "default",
    "timestamp": 1728475265
  }
}
```

### 3.2 `agm prompts tree --json` Schema

```json
{
  "success": true,
  "data": {
    "total_projects": 1,
    "projects": [
      {
        "seq_id": 6,
        "seq_code": "P006",
        "gitmap_seq_code": "GM:#6",
        "project_id": "proj-987",
        "repo_name": "antigravity-manager",
        "repo_path": "git-work/antigravity-manager",
        "instance_id": "default",
        "is_running": true,
        "running_count": 1,
        "queued_count": 0,
        "conversations": [
          {
            "seq_id": 25,
            "seq_code": "C025",
            "gitmap_seq_code": "GM:antigrav",
            "conversation_id": "conv-abcdef123456",
            "short_id": "abcdef12",
            "title": "Fix bracket clutter and process cache",
            "status": "RUNNING",
            "is_running": true,
            "step_count": 14,
            "prompt_preview_200w": "Look, the sending prompt from the instance...",
            "last_modified": "1728475200"
          }
        ]
      }
    ]
  },
  "meta": {
    "command": "prompts tree",
    "instance_id": "all",
    "timestamp": 1728475265
  }
}
```

### 3.3 `agm prompts send --json` Schema

```json
{
  "success": true,
  "data": {
    "prompt_id": "prompt-67890",
    "instance_id": "default",
    "repo_path": "git-work/antigravity-manager",
    "process_status": "reused_active_process",
    "pid": 41209,
    "is_running": true,
    "copied_to_clipboard": true,
    "dispatched_at": 1728475300
  },
  "meta": {
    "command": "prompts send",
    "instance_id": "default",
    "timestamp": 1728475300
  }
}
```

### 3.4 `agm doctor --json` Schema

```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "checks": [
      {
        "name": "registry_integrity",
        "is_passed": true,
        "details": "instances.json valid with 2 registered profiles"
      },
      {
        "name": "split_db_connectivity",
        "is_passed": true,
        "details": "active_prompts and prompt_backups SQLite DBs accessible"
      },
      {
        "name": "antigravity_cli_binary",
        "is_passed": true,
        "details": "Found agy at ~/.local/bin/agy"
      },
      {
        "name": "process_cache_status",
        "is_passed": true,
        "details": "Smart process scanner operational, 1 live IDE instance detected"
      }
    ]
  },
  "meta": {
    "command": "doctor",
    "instance_id": null,
    "timestamp": 1728475350
  }
}
```

---

## 4. Detailed Implementation Steps in `src-tauri/src/modules/cli.rs`

### Step 1: Add Match Arms to CLI Router
In `src-tauri/src/modules/cli.rs` inside `pub fn handle_cli_args() -> bool` (~L229-420):
```rust
// Prompt management commands
"prompts" | "prompt" => {
    handle_prompts_subcommand(&rest_args);
    true
}

// System diagnostic doctor command
"doctor" | "diagnose" | "health" => {
    handle_doctor_subcommand(&rest_args);
    true
}
```

### Step 2: Implement `handle_prompts_subcommand`
- Parse sub-action (`ls`, `tree`, `send`, `enqueue`, `backup`, `restore`).
- Extract flags (`--instance` / `-i`, `--repo` / `-r`, `--json` / `-j`, `--only-running`).
- Route to appropriate backend handlers:
  - `ls`: query `active_prompts` table in `repo_db`.
  - `tree`: call `repo_db::get_project_conversation_tree(target_inst, 50, only_running)`.
  - `send`: invoke `repo_db::dispatch_prompt_now(inst_id, repo_path, prompt_text, None)`.
  - `enqueue`: invoke `repo_db::enqueue_prompt(inst_id, repo_path, prompt_text, "gemini-pro", None)`.
  - `backup`: invoke `backup_prompts_db::backup_active_running_prompts_for_instance(inst_id, None)`.
  - `restore`: invoke `backup_prompts_db::restore_running_prompts_for_instance(inst_id, batch_id, None)`.

### Step 3: Implement `handle_doctor_subcommand`
- Inspect core subsystems:
  1. Registry file permissions and JSON validity.
  2. Data directory existence for all registered instances.
  3. `agy` CLI binary resolution via `crate::modules::process::get_antigravity_cli_executable_path()`.
  4. SQLite split-DB connections (`repo_prompts.db`, `backup_prompts.db`).
  5. Process scan table verification (`scan_and_cache_all_running_instances()`).
- Format output as clean ASCII checklist for humans, or structured JSON when `--json` is supplied.

### Step 4: Update Help Renderers
- Add `prompts` and `doctor` sections to `print_all_cli_help()`.
- Add dedicated `print_prompts_cli_help()` and `print_doctor_cli_help()`.

---

## 5. Invariants & Constraints

- **Strict Relative Git Paths**: All paths cited in code and documentation must be relative (e.g. `src-tauri/src/modules/cli.rs`).
- **Positive Booleans**: Exclusively use positive boolean identifiers (`is_json`, `is_running`, `is_passed`, `has_valid_pid`).
- **Exit Code Fidelity**: Return exit code `0` on success, exit code `1` on error.
- **Cross-Platform Console Output**: Support Windows parent console attachment (`attach_parent_console()`) and standard Unix terminal pipelines.

---

## 6. Verification & Pre-flight Checklist

```bash
# Verify Rust compilation and linting
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::cli
```

---

## 7. Done When

- [ ] `agm prompts ls` lists prompts with human and `--json` support.
- [ ] `agm prompts tree` outputs conversation tree with `#6`, `P006`, and `C025` sequence codes.
- [ ] `agm prompts send` dispatches prompt via Smart Process Cache.
- [ ] `agm prompts enqueue` appends prompt to SQLite queue.
- [ ] `agm prompts backup` and `agm prompts restore` operate without errors.
- [ ] `agm doctor` audits system health and reports status cleanly.
- [ ] All CLI tests pass with zero clippy warnings.
