---
plan: 113-per-instance-running-prompts-and-projects-isolation
subtask: "002"
title: Structured Audit Logging & Prompt Liveness Traceability Probe
domain: backend-rust
depends_on: 001-backend-parametric-tree-and-pid-liveness.md
citations:
  app_spec: ../../../../02-spec/21-app/113-per-instance-running-prompts-and-projects-isolation.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../../02-spec/22-app-issues/20-cross-instance-running-prompts-bleed-rca.md
  parent_plan: ../../113-per-instance-running-prompts-and-projects-isolation.md
target_files:
  - src-tauri/src/modules/repo_db.rs
  - src-tauri/src/modules/instance.rs
status: pending
---

# 002 — Structured Audit Logging & Prompt Liveness Traceability Probe

## 1. Context & Problem Statement
When debugging why specific projects were reported as `[RUNNING]` or `[IDLE]` across instances, developers and autonomous AI agents previously had zero diagnostic visibility into the runtime decision chain. Projects like `Antigravity-Manager` and `spec-builder` were erroneously labeled as running in the `8159` instance because:
1. Workspace storage folders across instances were not traced during project discovery.
2. The decision to mark a project running was silent and opaque, with no log record indicating which PIDs were detected, whether an OS process was alive, or whether a blind recency fallback was triggered.
3. To achieve root cause clarity and permanent maintainability, a structured `[PROMPT_LIVENESS_PROBE]` audit logger must record the exact rationale for every instance and project evaluation.

## 2. Target Files and Symbols
- `src-tauri/src/modules/repo_db.rs`:
  - `log_prompt_liveness_probe(probe: &PromptLivenessProbe)` or inline structured logging helper
  - `compute_project_conversation_tree` liveness decision blocks
  - `detect_running_projects` workspace discovery logs
- `src-tauri/src/modules/instance.rs`:
  - `find_pids_for_data_dir` PID discovery inspection

## 3. Concrete Implementation Steps

### Step 3.1: Define `PromptLivenessProbe` Telemetry Structure in `src-tauri/src/modules/repo_db.rs`
1. Define a structured metadata record capturing the probe parameters:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct PromptLivenessProbe {
       pub target_instance_id: String,
       pub target_instance_name: String,
       pub data_dir: String,
       pub is_instance_alive: bool,
       pub matched_pids: Vec<u32>,
       pub workspace_folders_count: usize,
       pub evaluated_projects_count: usize,
       pub running_projects_count: usize,
   }
   ```
2. Define a per-project evaluation detail record:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ProjectLivenessEvaluation {
       pub project_id: String,
       pub repo_name: String,
       pub repo_path: String,
       pub is_running: bool,
       pub active_tasks_count: usize,
       pub non_idle_convs_count: usize,
       pub rationale: String,
   }
   ```

### Step 3.2: Instrument `compute_project_conversation_tree` with Structured Audit Logs
1. During instance resolution in `compute_project_conversation_tree`:
   - Query instance PIDs via `crate::modules::instance::find_pids_for_data_dir(&inst.data_dir, inst.is_default)`.
   - Record `is_inst_alive = crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid)`.
   - Log the instance-level probe entry:
     ```text
     [PROMPT_LIVENESS_PROBE] instance_id="8159" data_dir="/path/to/data" alive=false pids=[] workspaces=3
     ```
2. During per-project and per-conversation evaluation:
   - Determine the explicit evaluation rationale string:
     - `"INSTANCE_PROCESS_DEAD: instance PID not found or process terminated -> forced idle"`
     - `"WORKSPACE_UNMATCHED: workspace path does not belong to target instance -> excluded"`
     - `"ACTIVE_IN_FLIGHT_TASKS: active prompt present in sqlite -> marked running"`
     - `"CONVERSATION_BUSY: conversation summary status busy with alive instance -> marked running"`
     - `"RECENCY_DISREGARDED_DEAD_PID: recent activity timestamp ignored because process is dead -> forced idle"`
     - `"IDLE: process alive but no in-flight tasks or active conversations -> marked idle"`
   - Output structured log line:
     ```text
     [PROMPT_LIVENESS_PROBE][PROJECT] instance_id="8159" project="spec-builder" is_running=false rationale="INSTANCE_PROCESS_DEAD: instance PID not found or process terminated -> forced idle"
     ```
3. Use both `tracing::info!` (with target `"prompt_liveness_probe"`) and standard output for CLI visibility, ensuring entries appear in headless test runs and diagnostic logs.

### Step 3.3: Zero Sensitive Data Hygiene Gate
1. Ensure the probe NEVER prints:
   - User prompt content or message transcripts (`prompt_content`, `preview_200w`).
   - Account access tokens, session tokens, or API credentials.
   - User email addresses (use masked or hash identifier if necessary).
2. Only log structural metadata: instance IDs, filesystem workspace paths, PID counts, numeric counts, and rationales.

### Step 3.4: Add Unit Test for Probe Formatting and Rationale Generation
1. In `src-tauri/src/modules/repo_db.rs` test suite:
   - Add unit tests verifying that `PromptLivenessProbe` formats expected rationale strings.
   - Assert that an instance with empty PIDs produces `INSTANCE_PROCESS_DEAD` rationale and sets `is_running = false`.
   - Assert that an alive instance with active tasks produces `ACTIVE_IN_FLIGHT_TASKS` rationale and sets `is_running = true`.

## 4. Architectural Constraints & Coding Standards
- **Standardized Formatting**:
  - Keep log prefix strictly as `[PROMPT_LIVENESS_PROBE]` for grep/GitMap automated searchability.
  - Format key-value pairs consistently: `key="value"` or `key=boolean/int`.
- **Coding Guidelines Compliance**:
  - Positive boolean naming (`is_instance_alive`, `is_running`).
  - No bare `unwrap()` or panic-inducing calls in logging helpers.
  - Keep logging helper isolated in a small dedicated function or submodule to obey size tier guidelines.
- **Cross-Platform Compatibility**:
  - Paths formatted safely on Windows (`\`) and Unix (`/`).

## 5. Out of Scope
- Modifying UI components or CSS styles.
- Creating the Markdown RCA document (delegated to Subtask 004).
- Production release bumps or git tagging.

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::repo_db::tests::test_prompt_liveness_probe
```

## 7. Done When
- [ ] Structured `[PROMPT_LIVENESS_PROBE]` log lines are emitted during tree computation.
- [ ] Logs detail target instance ID, data directory, resolved workspace counts, detected PIDs, and explicit evaluation rationales.
- [ ] Zero sensitive data (prompts, tokens, passwords) is leaked in log statements.
- [ ] Unit tests for probe formatting and dead-process rationale pass cleanly.

## 8. Ambiguities & Fallback Defaults
- If an instance has no configured name, fall back to its ID (`inst.id`).
- If an instance data directory does not exist on disk, the probe logs `data_dir_exists=false` and rationalizes `DATA_DIR_NOT_FOUND`.
