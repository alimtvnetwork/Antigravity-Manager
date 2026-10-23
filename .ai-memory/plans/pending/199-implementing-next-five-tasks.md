# Parent Plan — 199 Implementing Next Five Tasks

Status: IN PROGRESS (TAKEN)
Phase: 1 (claimed & assigned to subagents)
Owner: Agent-199-Lead
Specs: `02-spec/21-app/199-implementing-next-five-tasks/spec.md`
Subtasks: `.ai-memory/plans/subtasks/199-implementing-next-five-tasks/`

## 1. Selected Five Tasks (Claimed & In-Progress)

| Task # | Slug | Module Target | Status | Implementation Details |
|---|---|---|---|---|
| 01 | `cross-platform-compat` | `src-tauri/src/modules/release_gate/cross_platform_compat_guard.rs` | `IN_PROGRESS` | Flag platform-locked constructs (drive-letter paths, OS-specific shells/APIs) lacking a `cfg` fallback or cross-platform abstraction. |
| 02 | `staging-protocol` | `src-tauri/src/modules/release_gate/staging_protocol_guard.rs` | `IN_PROGRESS` | Classify changesets and decide whether maintainer-confirmed `beta` staging is required, recommended, or unnecessary before `main`. |
| 03 | `published-tip-rewrite` | `src-tauri/src/modules/release_gate/published_tip_rewrite_guard.rs` | `IN_PROGRESS` | Block published-tip rewrites lacking in-flight PR inspection, a `backup/*` rollback ref, and zero-drift confirmation. |
| 04 | `headless-cli-parity` | `src-tauri/src/modules/release_gate/headless_cli_parity_guard.rs` | `IN_PROGRESS` | Verify every GUI lifecycle operation has an identical CLI route through the same shared service (and vice versa). |
| 05 | `generic-fix-preference` | `src-tauri/src/modules/release_gate/generic_fix_preference_guard.rs` | `IN_PROGRESS` | Detect hardcoded exact-match chains and literal tables in routing/sanitization code that should be wildcard patterns or registries. |

## 2. Invariants & Rules
- **Zero-Build & Zero-Test Invariant**: workers do not build or run tests during implementation; the lead runs `cargo check --tests` + `cargo fmt -- --check` at integration.
- **Repository-Relative Paths**: exclusively relative paths everywhere, including release notes.
- **Anti-Collision Commit**: committed immediately as taken to prevent multi-agent collisions; workers never edit `src-tauri/src/modules/release_gate/mod.rs` (lead wires registration).
- **Search**: GitMap only (`gitmap aum search`, `gitmap find`, `gitmap cat`); total ban on rg/grep/git grep.
- **No Silent Errors**: failures route into `AppError` / `record_ignored` with `// Justification:` where applicable.

## 3. Collision Check (performed before claiming)
- On-disk `release_gate` modules inventoried: none of the five targets exist.
- Batches 194 (completed), 195 (implemented), 196 (completed), 197 (completed), 198 (in progress: transport-backoff-stepper, transient-sandbox-manager, fifo-prompt-queue-coordinator, account-switch-audit-emitter, rejection-breakdown-generator) reviewed: no overlap with this batch's five.
- `gitmap rc heatmap status`: 0 in-flight actions recorded at claim time.
