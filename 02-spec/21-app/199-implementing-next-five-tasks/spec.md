# Specification 199: Implementing Next Five Tasks (Cross-Platform Compatibility, Maintainer Staging, Published-Tip Rewrite Safety, Headless/CLI Parity & Generic-Fix Preference)

## 1. Overview & Context
This specification defines the architectural contracts and compliance verifiers for the 5 continuous reliability and release engineering tasks in Sequence 199.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.
Selection was cross-checked against every implemented module in `src-tauri/src/modules/release_gate/` and every claimed batch (194, 195, 196, 197, 198) so no task duplicates work another agent owns.

## 2. Five Planned Tasks Breakdown

### Task 1: Cross-Platform Compatibility Guard
- **Module**: `src-tauri/src/modules/release_gate/cross_platform_compat_guard.rs`
- **Specification**:
  - Every code addition and dependency change must be evaluated for seamless cross-platform support (Windows, macOS, Linux).
  - Implement a source scanner that flags platform-locked constructs lacking a `cfg` fallback or cross-platform abstraction: hardcoded drive-letter paths, OS-specific shell invocations, unconditional platform APIs, and path-separator assumptions.
  - Test assertions: verify detection of an unguarded Windows-only path, acceptance of `cfg(target_os)`-gated code, and acceptance of `PathBuf`-based construction.

### Task 2: Maintainer Staging Protocol Guard
- **Module**: `src-tauri/src/modules/release_gate/staging_protocol_guard.rs`
- **Specification**:
  - New features, major refactors, and non-trivial fixes must be confirmed with maintainers and validated on the `beta` branch (with optional preview builds) before merging to `main`; trivial fixes may go direct.
  - Implement a change-set classifier that scores a changeset (files touched, feature flags, schema changes, public surface changes) and reports whether beta staging is required, recommended, or unnecessary, with the deciding factors listed.
  - Test assertions: verify a schema migration changeset requires staging, a one-line typo fix does not, and a multi-module feature is at least recommended.

### Task 3: Published-Tip Rewrite Guard
- **Module**: `src-tauri/src/modules/release_gate/published_tip_rewrite_guard.rs`
- **Specification**:
  - Before rewriting a published branch tip (rebase/force-push), in-flight PRs based on that branch must be inspected and a local rollback ref (`backup/*`) retained, with zero content drift confirmed via diff stat.
  - Implement a rewrite-safety evaluator over a described operation (target branch, open PR count, backup ref presence, drift summary) that blocks unsafe rewrites and names the missing precondition.
  - Test assertions: verify a rewrite with an open PR and no backup ref is blocked for both reasons, and a rewrite with no open PRs plus a backup ref and zero drift is allowed.

### Task 4: Headless & CLI Parity Guard
- **Module**: `src-tauri/src/modules/release_gate/headless_cli_parity_guard.rs`
- **Specification**:
  - GUI configurations and lifecycle operations must maintain functional parity across headless servers, CLI environments, and the desktop GUI; any profile/instance duplication or settings synchronization must pass through identical underlying service routes from both surfaces.
  - Implement a parity analyzer over a registry of operations (name, GUI entry, CLI entry, shared service route) that reports operations missing a surface or bypassing the shared route.
  - Test assertions: verify a GUI-only lifecycle operation is flagged, a CLI-only operation is flagged, and a fully shared operation passes.

### Task 5: Generic-Fix Preference Guard
- **Module**: `src-tauri/src/modules/release_gate/generic_fix_preference_guard.rs`
- **Specification**:
  - Prioritize root-cause, generalized fixes covering entire classes of problems over hardcoded one-off logic, exact string matches, or speculative special cases (model routing via wildcard patterns and prompt sanitization via regex are the canonical examples).
  - Implement a hardcoding detector that scans described match sites (exact-equality chains, literal lookup tables, single-value special cases in routing/sanitization code) and scores whether a pattern/registry generalization is required.
  - Test assertions: verify a five-branch exact-match chain is flagged for generalization, a wildcard pattern matcher passes, and a single deliberate exact match with justification passes.

## 3. Invariants & Rules
- Zero builds and zero tests during the implementation phase (the lead runs the compile gate at integration).
- Strictly repository-relative Git paths.
- Search exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`). Total ban on rg, ripgrep, grep, git grep, Select-String.
- No silent errors: failures route into `AppError` / `record_ignored` with a `// Justification:` comment where an ignore is genuinely safe.
- Anti-collision: this batch is committed as TAKEN before worker dispatch; workers never edit `src-tauri/src/modules/release_gate/mod.rs` (the lead wires module registration at integration).
