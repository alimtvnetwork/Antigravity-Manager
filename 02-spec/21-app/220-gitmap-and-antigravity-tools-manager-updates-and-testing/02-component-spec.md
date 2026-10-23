# Specification 220: Component Spec — GitMap and Antigravity Tools Manager Updates and Testing
Sequence: 220
Batch: Steps 311–315
Improvements: `Imp-725` through `Imp-729`

## 1. Component Breakdown

### 1.1 Account Auto-Switch & Prompt Restore Sweep (`src-tauri/src/modules/backup_prompts_db/`)
- **Module**: `restore.rs`
- **Key Function**: `restore_incomplete_prompts_after_switch(target_instance_id: &str) -> Result<RestoreOutcome, AppError>`
- **Behavior**:
  - Sweeps all prompts in `prompt_backups` where status is `'interrupted'`, `'quota_exhausted'`, or un-restored `'backed_up'` associated with the previous profile.
  - Re-attributes ownership to `target_instance_id`.
  - Stages prompts into `active_prompts` with status `'queued'`, preserving original `created_at` timestamps.
  - Enforces atomic transaction rollback if any step fails.

### 1.2 Rerun Randomizer UX & Persistent Execution Engine (`src/components/instances/PromptTreeView/`)
- **Components**: `PromptTreeViewModal.tsx`, `usePromptTreePromptActions.ts`
- **Key Enhancements**:
  - When triggering Rerun, automatically select a randomized nudge template variation from the predefined suite (`verification`, `conciseness`, `architectural`, `test_focused`).
  - Provide inline badge indicating the auto-selected template with a 1-click override option.
  - Support multi-rerun batch dispatching (`count: 1..5`) directly into the FIFO queue.

### 1.3 GitMap Code Quality & CG Execute Standards (`d:/work/gitmap/`)
- **Modules**: `cmdfix/`, `cmdpull/`, `heatmap/`
- **Coding Guideline Conformance**:
  - Convert pointer receivers `*T` on small configuration structs to value semantics.
  - Replace error tuples with structured `*apperror.AppError`.
  - Eliminate nested if ladders in favor of early guard returns.
  - Enforce file cap < 300 lines by decomposing report generation helpers.

## 2. Invariants & Guardrails
- **Segmented Capsule Styling**: Transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) on all inline dropdowns.
- **Strictly Relative Paths**: All recorded references must be repository-relative.
- **No Silent Errors**: Fallible operations must route through `AppError` or record using `record_ignored` with `// Justification:`.
