# Specification: Implementing Next Five Tasks (Sequence 217)
Sequence: 217
Steps: 296–300
Improvements: `Imp-710` through `Imp-714`
Slug: `217-implementing-next-five-tasks`

## 1. Executive Summary
This specification defines five production-grade architectural components and release verification guards designed to uphold the non-negotiable standards of `AGENTS.md`:
1. `Imp-710` (Step 296): **Streaming Thought Signature & Delimiter Validator** (`proxy/handlers/common/streaming_thought_signature_validator.rs`)
2. `Imp-711` (Step 297): **Segmented Capsule Toggle Switch Styler & Linter** (`modules/instance/segmented_capsule_toggle_switch_styler.rs`)
3. `Imp-712` (Step 298): **FIFO Prompt Queue Partition & Priority Allocator** (`modules/db_orm/fifo_prompt_queue_partition_allocator.rs`)
4. `Imp-713` (Step 299): **Transient Sandbox Lifecycle & Cleanup Sentinel** (`modules/instance/transient_sandbox_cleanup_sentinel.rs`)
5. `Imp-714` (Step 300): **Sole Contributor Attribution & No Silent Errors Audit Guard** (`modules/release_gate/sole_contributor_attribution_audit_guard.rs`)

---

## 2. Invariants & Rules Enforced
- **Strictly Relative Git Paths**: All file paths and references must be repository-relative (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`). Zero Windows drive letters (`D:\`, `C:\`) or root paths allowed.
- **Strict FIFO Ordering**: Every database query across prompt databases must strictly enforce `ORDER BY created_at ASC, id ASC`.
- **Minimalist & Contextual UI**: Capsule grouping with `rounded-full`, shared border, subtle dividers, and transparent borderless selects.
- **Caller IDE Self-Preservation**: Exit code 2 fail-fast when targeting caller IDE without `--force`.
- **No Silent Errors**: Zero unrouted errors or bare catches across Rust and TypeScript.
- **Zero-Build & Zero-Test Policy**: Strictly zero builds and tests during authoring; deferred E2E tests cataloged in master plan.

---

## 3. Detailed Component Architecture

### 3.1 Task 1 (`Imp-710`): Streaming Thought Signature & Delimiter Validator
- **Module**: `src-tauri/src/proxy/handlers/common/streaming_thought_signature_validator.rs`
- **Domain**: Reverse Proxy Gateway & Gemini Pipeline Normalization
- **Purpose**: Validates SSE streaming payloads containing thinking blocks. Ensures `<thought>` and `</thought>` tags, `thought: true` attributes, and delta buffers maintain balanced delimiters and do not leak internal marker tokens across SSE frames.
- **Data Structures**:
  - `ThoughtTagState`: Inactive, OpenTagEncountered, InsideThoughtBlock, ClosingTagEncountered.
  - `ThoughtValidationFinding`: Balanced, UnclosedThoughtTag { open_epoch_ms: u64 }, UnexpectedClosingTag, MismatchedThoughtFlag.
  - `ThoughtStreamValidationReport`: is_compliant: bool, total_thought_bytes: usize, findings: Vec<ThoughtValidationFinding>, sanitized_payload: String.
- **Core Functions**:
  - `validate_thought_stream_chunk(delta: &str, current_state: &mut ThoughtTagState) -> ThoughtStreamValidationReport`
  - `repair_truncated_thought_block(text: &str) -> String`

### 3.2 Task 2 (`Imp-711`): Segmented Capsule Toggle Switch Styler & Linter
- **Module**: `src-tauri/src/modules/instance/segmented_capsule_toggle_switch_styler.rs`
- **Domain**: UI / UX Minimalist Design System
- **Purpose**: Audits and generates JSX/HTML layout tokens for contiguous segmented toggle capsules. Enforces `rounded-full`, shared dark-glass container borders, subtle divider lines, and transparent borderless inline select dropdowns.
- **Data Structures**:
  - `CapsuleSegmentOption`: id: String, label: String, is_selected: bool, is_disabled: bool, aria_label: String.
  - `CapsuleStyleConfig`: size_tier: String (sm, md, lg), is_dark_glass: bool, has_dividers: bool.
  - `SegmentedCapsuleFormatting`: container_classes: String, segment_classes: Vec<String>, divider_classes: String, formatted_markup: String.
  - `CapsuleAuditFinding`: DisjointButtonsDetected, MissingRoundedFull, BorderlessSelectMissing, Compliant.
  - `CapsuleAuditReport`: is_compliant: bool, findings: Vec<CapsuleAuditFinding>, recommendations: Vec<String>.
- **Core Functions**:
  - `format_segmented_capsule_toggle(options: &[CapsuleSegmentOption], config: &CapsuleStyleConfig) -> SegmentedCapsuleFormatting`
  - `audit_capsule_markup_compliance(html_or_jsx: &str) -> CapsuleAuditReport`

### 3.3 Task 3 (`Imp-712`): FIFO Prompt Queue Partition & Priority Allocator
- **Module**: `src-tauri/src/modules/db_orm/fifo_prompt_queue_partition_allocator.rs`
- **Domain**: Split-DB / SQLite Prompt Queue Engine
- **Purpose**: Allocates prompt partitions across multi-instance SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`). Enforces strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC` across partition buckets and priority levels.
- **Data Structures**:
  - `QueuePriorityBand`: Urgent, Normal, Background.
  - `QueuePartitionBucket`: partition_id: String, instance_id: String, priority: QueuePriorityBand, limit: usize.
  - `AllocatedPromptItem`: id: i64, created_at: String, prompt_text: String, priority: QueuePriorityBand, partition_id: String.
  - `PartitionAllocationReport`: total_allocated: usize, items: Vec<AllocatedPromptItem>, query_sql: String, is_fifo_compliant: bool.
  - `PartitionAllocationError`: InvertedLifoOrderingDetected, MissingTieBreakerId, InvalidPartitionLimit.
- **Core Functions**:
  - `build_partition_query_sql(bucket: &QueuePartitionBucket) -> Result<String, PartitionAllocationError>`
  - `verify_allocated_partition_monotonicity(items: &[AllocatedPromptItem]) -> bool`

### 3.4 Task 4 (`Imp-713`): Transient Sandbox Lifecycle & Cleanup Sentinel
- **Module**: `src-tauri/src/modules/instance/transient_sandbox_cleanup_sentinel.rs`
- **Domain**: Multi-Instance & Process Sandboxing
- **Purpose**: Tracks transient sandbox test instances (`agm instances test-create`), verifies caller active IDE process preservation (`detect_caller_instance_id()`), and enforces complete zero-orphan remnant cleanup (`agm instances test-clean`).
- **Data Structures**:
  - `TransientSandboxState`: Created, Active, PendingCleanup, Purged, OrphanRemnantsDetected.
  - `SandboxInstanceDescriptor`: sandbox_id: String, instance_dir: String, created_epoch_ms: u64, is_caller_ide: bool.
  - `SandboxEvictionVerdict`: TargetIsCallerIdeFailFast { exit_code: i32 }, SafeToPurge, AlreadyCleaned.
  - `SandboxCleanupReport`: sandbox_id: String, directories_removed: Vec<String>, processes_terminated: Vec<u32>, verdict: SandboxEvictionVerdict, is_fully_purged: bool.
- **Core Functions**:
  - `evaluate_sandbox_eviction_safety(descriptor: &SandboxInstanceDescriptor, caller_instance_id: Option<&str>, force: bool) -> SandboxEvictionVerdict`
  - `plan_transient_sandbox_cleanup(descriptor: &SandboxInstanceDescriptor) -> SandboxCleanupReport`

### 3.5 Task 5 (`Imp-714`): Sole Contributor Attribution & No Silent Errors Audit Guard
- **Module**: `src-tauri/src/modules/release_gate/sole_contributor_attribution_audit_guard.rs`
- **Domain**: Release Governance & Quality Discipline
- **Purpose**: Release gate linter that verifies strict sole attribution to `@aukgit` (`(Thanks to @aukgit)`), detects unauthorized third-party handles (`@...`), verifies strictly relative paths, and audits code additions for the No Silent Errors house rule.
- **Data Structures**:
  - `AttributionFinding`: ValidAukgitAttribution, UnauthorizedHandle(String), MissingAttributionTrailer.
  - `SilentErrorFinding`: BareResultDiscarded { file: String, line: usize }, BareCatchWithoutTracking { file: String, line: usize }, DocumentedIgnoredResult.
  - `ReleaseGovernanceReport`: is_compliant: bool, attribution_findings: Vec<AttributionFinding>, silent_error_findings: Vec<SilentErrorFinding>, absolute_path_count: usize, summary: String.
- **Core Functions**:
  - `audit_release_attribution(text: &str) -> Vec<AttributionFinding>`
  - `audit_silent_error_patterns(source_code: &str, file_name: &str) -> Vec<SilentErrorFinding>`
  - `generate_governance_report(text: &str, code_sources: &[(&str, &str)]) -> ReleaseGovernanceReport`

---

## 4. Subagent Allocation Plan (`A = 3`, `H = 2`)
- **Worker 01**: Tasks 1 & 2 (`Imp-710`, `Imp-711`)
  - Files:
    - `src-tauri/src/proxy/handlers/common/streaming_thought_signature_validator.rs`
    - `src-tauri/src/proxy/handlers/common/tests_streaming_thought_signature_validator.rs`
    - `src-tauri/src/modules/instance/segmented_capsule_toggle_switch_styler.rs`
    - `src-tauri/src/modules/instance/tests_segmented_capsule_toggle_switch_styler.rs`
- **Worker 02**: Tasks 4 & 5 (`Imp-713`, `Imp-714`)
  - Files:
    - `src-tauri/src/modules/instance/transient_sandbox_cleanup_sentinel.rs`
    - `src-tauri/src/modules/instance/tests_transient_sandbox_cleanup_sentinel.rs`
    - `src-tauri/src/modules/release_gate/sole_contributor_attribution_audit_guard.rs`
    - `src-tauri/src/modules/release_gate/tests_sole_contributor_attribution_audit_guard.rs`
- **Worker 03**: Task 3 (`Imp-712`) & Master E2E Plan
  - Files:
    - `src-tauri/src/modules/db_orm/fifo_prompt_queue_partition_allocator.rs`
    - `src-tauri/src/modules/db_orm/tests_fifo_prompt_queue_partition_allocator.rs`
    - `02-spec/21-app/217-implementing-next-five-tasks/e2e_master_plan.md`

---

## 5. Execution Model
- Zero builds and zero test execution during coding.
- Pre-flight and test verification strictly deferred to the designated master plan.
