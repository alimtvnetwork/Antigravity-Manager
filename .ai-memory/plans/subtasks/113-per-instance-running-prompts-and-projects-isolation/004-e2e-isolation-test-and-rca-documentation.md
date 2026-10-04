---
plan: 113-per-instance-running-prompts-and-projects-isolation
subtask: "004"
title: E2E Per-Instance Isolation Test & Comprehensive State Bleed RCA Documentation
domain: testing-and-spec
depends_on:
  - "001"
  - "002"
  - "003"
citations:
  app_spec: ../../../../02-spec/21-app/113-per-instance-running-prompts-and-projects-isolation.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../../02-spec/22-app-issues/20-cross-instance-running-prompts-bleed-rca.md
target_files:
  - src-tauri/tests/per_instance_prompt_liveness_test.rs
  - 02-spec/22-app-issues/20-cross-instance-running-prompts-bleed-rca.md
status: pending
---

# Subtask 004 — E2E Per-Instance Isolation Test & Comprehensive State Bleed RCA Documentation

## 1. Objectives & Context

1. **End-to-End Isolation Integration Test**:
   - Write a dedicated integration test in `src-tauri/tests/per_instance_prompt_liveness_test.rs` (annotated with `#[ignore]`).
   - The test sets up two distinct mock instances with disjoint data directories and workspace storages:
     - **Instance 1 (Default)**: Hosts project `Antigravity-Manager`.
     - **Instance 2 (8159)**: Hosts project `coding-guidelines`.
   - Verifies:
     - Calling `get_project_conversation_tree` (or `detect_running_projects`) for Instance 1 returns strictly `Antigravity-Manager` and zero traces of `coding-guidelines`.
     - Calling for Instance 2 returns strictly `coding-guidelines` and zero traces of `Antigravity-Manager`.
     - Process liveness gating: `is_running` is strictly `false` if the instance process is not running, regardless of file modification timestamps on disk.
2. **Root Cause Analysis (RCA) Documentation**:
   - Author a complete, grounded 4-part RCA document in `02-spec/22-app-issues/20-cross-instance-running-prompts-bleed-rca.md`.
   - Detail the fundamental architecture flaws that caused cross-instance prompt and project bleeding, including:
     - Global workspace directory aggregation without instance boundary filtering.
     - Un-gated 10-minute recency markers (`age < 600`) overriding real process liveness.
     - Frontend loose name matching and empty-array fallback leaks.

---

## 2. Target Files & Symbols

| File Path | Purpose / Description | Action |
| :--- | :--- | :--- |
| `src-tauri/tests/per_instance_prompt_liveness_test.rs` | Local-only integration test verifying multi-instance isolation and process-gated prompt liveness. | Create new integration test file with `#[ignore]`. |
| `02-spec/22-app-issues/20-cross-instance-running-prompts-bleed-rca.md` | Authoritative 4-part RCA documentation explaining failure mechanisms, remedies, and future AI rules. | Create comprehensive RCA documentation. |

---

## 3. Detailed Specifications

### 3.1 Integration Test Specification (`per_instance_prompt_liveness_test.rs`)

The test must be located in `src-tauri/tests/per_instance_prompt_liveness_test.rs`:

```rust
//! Local-only End-to-End Integration Tests for Per-Instance Project & Prompt Isolation
//! Run manually with: cargo test --test per_instance_prompt_liveness_test -- --ignored

use antigravity_tools_lib::models::instance::InstanceConfig;
use antigravity_tools_lib::modules::repo_db::{
    detect_running_projects, get_project_conversation_tree_cached,
};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test per_instance_prompt_liveness_test -- --ignored"]
async fn test_two_distinct_instances_return_isolated_project_sets() {
    // 1. Create temporary directory structure for Instance 1 (Default)
    let temp_dir_inst1 = tempdir().expect("Failed to create tempdir for inst1");
    let ws_storage_inst1 = temp_dir_inst1.path().join("User").join("workspaceStorage");
    let ws_proj1 = ws_storage_inst1.join("ws_antigravity_manager");
    fs::create_dir_all(&ws_proj1).expect("Failed to create ws_proj1 dir");
    fs::write(
        ws_proj1.join("workspace.json"),
        r#"{"folder": "file:///d:/work/Antigravity-Manager"}"#,
    ).expect("Failed to write ws_proj1 workspace.json");

    // 2. Create temporary directory structure for Instance 2 (8159)
    let temp_dir_inst2 = tempdir().expect("Failed to create tempdir for inst2");
    let ws_storage_inst2 = temp_dir_inst2.path().join("User").join("workspaceStorage");
    let ws_proj2 = ws_storage_inst2.join("ws_coding_guidelines");
    fs::create_dir_all(&ws_proj2).expect("Failed to create ws_proj2 dir");
    fs::write(
        ws_proj2.join("workspace.json"),
        r#"{"folder": "file:///d:/work/coding-guidelines"}"#,
    ).expect("Failed to write ws_proj2 workspace.json");

    // 3. Register mock instance configurations
    // Instance 1: "default"
    // Instance 2: "inst-8159"

    // 4. Assert: Instance 1 yields ONLY Antigravity-Manager
    // Assert: Instance 2 yields ONLY coding-guidelines
    // Assert: Neither contains the other's workspace projects
}

#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test per_instance_prompt_liveness_test -- --ignored"]
async fn test_stopped_instance_projects_are_never_marked_running() {
    // Verify that when an instance has no active OS process (PID = 0 or dead PID),
    // its projects and conversation nodes are strictly marked is_running = false,
    // regardless of recent last_modified timestamps.
}
```

---

### 3.2 Root Cause Analysis Document (`20-cross-instance-running-prompts-bleed-rca.md`)

The document must follow the repository's 4-part RCA standard:
- **Title**: `20 — Cross-Instance Project and Running Prompt State Bleed RCA`
- **Part 1: Executive Summary & Failure Symptoms**:
  - Screenshot analysis: Instance Card #1 (Default) and Card #2 (8159) mirror identical project lists.
  - Card #1 displays `spec-builder` and `coding-guidelines` as running.
  - Card #2 displays `Antigravity-Manager` and `spec-builder` as running.
- **Part 2: Deep-Dive Root Causes (4 Core Flaws)**:
  1. *Global Candidate Directory Aggregation*: `compute_project_conversation_tree` iterated across all candidate directories on the system without filtering conversations to the workspace owning instance.
  2. *Un-Gated 10-Minute Recency Heuristic*: Conversations updated within 600 seconds (`age < 600`) were unconditionally tagged `is_running: true` without checking whether an instance process was actually alive.
  3. *Loose Frontend Name Matching*: `Instances.tsx` matched projects via `node.instance_name === inst.config.name`.
  4. *Frontend Empty Array Fallback Trap*: `PromptTreeViewModal.tsx` contained `const finalData = relevant.length > 0 ? relevant : data;`, actively leaking all global projects into instances that had zero projects.
- **Part 3: Comprehensive Architecture Remedies**:
  - Parametric `instance_id` filtering in `get_project_conversation_tree`.
  - Process PID verification requirement for `is_running`.
  - Elimination of loose name matching in frontend.
  - Elimination of empty fallback traps.
- **Part 4: Future AI Directives & Non-Negotiable Rules**:
  - Never use loose string name matches for instance entities.
  - Never fall back to global collections when an entity filter evaluates to empty.
  - Never declare a process entity "RUNNING" based purely on timestamp recency without OS process confirmation.

---

## 4. Verification Commands

```bash
# Verify Rust compilation and clippy
cd src-tauri && cargo clippy --all-targets --all-features

# Verify test syntax and compilation (without running ignored tests)
cd src-tauri && cargo test --test per_instance_prompt_liveness_test --no-run

# Run local ignored integration test manually
cd src-tauri && cargo test --test per_instance_prompt_liveness_test -- --ignored
```

---

## 5. Done When Checklist

- [ ] `src-tauri/tests/per_instance_prompt_liveness_test.rs` created and annotated with `#[ignore]`.
- [ ] Integration test verifies workspace isolation and process-gated prompt liveness between two mock instances.
- [ ] `02-spec/22-app-issues/20-cross-instance-running-prompts-bleed-rca.md` authored with complete 4-part RCA breakdown.
- [ ] Cargo test compiles cleanly with `cargo test --test per_instance_prompt_liveness_test --no-run`.
