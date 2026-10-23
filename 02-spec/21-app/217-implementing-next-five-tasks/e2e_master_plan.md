# End-to-End Master Execution Plan: Sequence 217 (Steps 296–300)

## 1. Scope & Batch Objectives
This document establishes the comprehensive end-to-end and integration verification plan for the 5 architectural, proxy streaming, UI capsule formatting, Split-DB queue allocation, and release governance modules implemented in Sequence 217 (`Imp-710` through `Imp-714`).

In accordance with the **Zero-Build & Zero-Test Invariant** codified in `AGENTS.md` and repository standards, live test runs are strictly deferred during the implementation phase. Any AI agent or developer executing deferred verification must follow the detailed specifications and exact command lines cataloged in this master plan.

The five focus areas verified in this sequence are:
1. **Streaming Thought Signature & Delimiter Validator** (`Imp-710`, Step 296): Validation of SSE streaming chunks with thought blocks (`<thought>`, `</thought>`, `thought: true`), delimiter balancing, and truncation repair.
2. **Segmented Capsule Toggle Switch Styler & Linter** (`Imp-711`, Step 297): Generation of contiguous segmented pill capsule styles (`rounded-full`, shared dark-glass border, subtle dividers) and auditing JSX/HTML markup for uncontained disjoint buttons and nested borderless select defects.
3. **FIFO Prompt Queue Partition & Priority Allocator** (`Imp-712`, Step 298): Multi-instance prompt queue partitioning across SQLite split databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) enforcing strict First-In-First-Out ordering (`ORDER BY created_at ASC, id ASC`) across priority bands.
4. **Transient Sandbox Lifecycle & Cleanup Sentinel** (`Imp-713`, Step 299): Caller IDE active process preservation (exit code 2 fail-fast), tracking of transient test sandboxes (`agm instances test-create`), and guaranteed orphan-free cleanup (`agm instances test-clean`).
5. **Sole Contributor Attribution & No Silent Errors Audit Guard** (`Imp-714`, Step 300): Verification of strict attribution to `@aukgit` (`(Thanks to @aukgit)`), rejection of unauthorized `@...` handles, strictly relative Git path enforcement, and auditing code for the No Silent Errors house rule.

---

## 2. Test Scenarios Matrix

| Test ID | Target Component | Purpose & Verification Flow | Expected Result |
| :--- | :--- | :--- | :--- |
| `E2E-217-01` | Streaming Thought Signature Validator (`Imp-710`) | Push normal text deltas, open `<thought>` tags, thought content, and `</thought>` closing tags across chunks. Push truncated stream and invoke `repair_truncated_thought_block`. | Delimiters tracked accurately, unclosed tags detected, truncated tags repaired cleanly. |
| `E2E-217-02` | Segmented Capsule Toggle Styler (`Imp-711`) | Format multi-option capsule toggle switch config; audit sample JSX markup containing disjoint circular buttons and unstyled select elements. | Formats contiguous `rounded-full` container with subtle dividers; linter flags disjoint buttons and uncontained selects. |
| `E2E-217-03` | FIFO Queue Partition Allocator (`Imp-712`) | Generate partition SQL queries for Urgent and Normal priority bands; verify compound `ORDER BY created_at ASC, id ASC LIMIT N`. Validate item monotonicity check rejects inverted timestamps. | Produces strict FIFO queries, rejects queries containing `DESC` or lacking `id ASC`, confirms monotonic ordering. |
| `E2E-217-04` | Transient Sandbox Cleanup Sentinel (`Imp-713`) | Evaluate eviction safety when target matches caller IDE instance ID without `--force`; evaluate transient sandbox cleanup lifecycle. | Returns fail-fast with exit code 2 when caller IDE is targeted; generates comprehensive purge plan removing sandbox dirs and orphan processes. |
| `E2E-217-05` | Sole Contributor Attribution & Silent Error Guard (`Imp-714`) | Audit changelog / commit text for `@aukgit` vs unauthorized handles; audit Rust and TS source code for bare `Result` discards and bare `catch {}` blocks. | Flags unauthorized handles, validates `@aukgit`, catches bare discards/catches, enforces relative paths. |

---

## 3. Exact Targeted Deferred Test Commands

When an explicit testing prompt is issued, execute the following commands in order:

```bash
# ==============================================================================
# 1. Step 296 (Imp-710): Streaming Thought Signature & Delimiter Validator
# ==============================================================================
cd src-tauri && cargo test --package antigravity-tools-lib --lib proxy::handlers::common::tests_streaming_thought_signature_validator

# ==============================================================================
# 2. Step 297 (Imp-711): Segmented Capsule Toggle Switch Styler & Linter
# ==============================================================================
cd src-tauri && cargo test --package antigravity-tools-lib --lib modules::instance::tests_segmented_capsule_toggle_switch_styler

# ==============================================================================
# 3. Step 298 (Imp-712): FIFO Prompt Queue Partition & Priority Allocator
# ==============================================================================
cd src-tauri && cargo test --package antigravity-tools-lib --lib modules::db_orm::tests_fifo_prompt_queue_partition_allocator

# ==============================================================================
# 4. Step 299 (Imp-713): Transient Sandbox Lifecycle & Cleanup Sentinel
# ==============================================================================
cd src-tauri && cargo test --package antigravity-tools-lib --lib modules::instance::tests_transient_sandbox_cleanup_sentinel

# ==============================================================================
# 5. Step 300 (Imp-714): Sole Contributor Attribution & No Silent Errors Guard
# ==============================================================================
cd src-tauri && cargo test --package antigravity-tools-lib --lib modules::release_gate::tests_sole_contributor_attribution_audit_guard

# ==============================================================================
# 6. Master Pre-Flight Regression Gate
# ==============================================================================
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
npm run build
```

---
*End of Sequence 217 Master Execution Plan.*
