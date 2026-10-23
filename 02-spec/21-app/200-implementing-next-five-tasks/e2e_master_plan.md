# Deferred End-to-End Test Master Plan (Sequence 200)

## 1. Overview & Policy
Per project invariants and user instructions:
- **Zero-Build & Zero-Test Invariant**: Strictly no compilation or unit tests run during implementation.
- All live verification scenarios for Sequence 200 (`Imp-625` through `Imp-629`) are cataloged here for subsequent execution when the user delivers the dedicated testing prompt.

---

## 2. Test Scenarios Matrix

| Test ID | Target Component | Purpose & Verification Flow | Expected Result |
|---|---|---|---|
| `E2E-200-01` | Prefix Stability Sanitizer (`Imp-625`) | Test multi-turn prompt caching prefixes across streaming and non-streaming requests. Strip forbidden headers (`x-goog-api-client`, internal proxy headers), enforce deterministic key serialization order, and verify stable byte-level prefixes across conversation turns. | Removes forbidden headers; preserves essential content and auth headers; outputs deterministic JSON prefix payload suitable for model KV-cache reuse. |
| `E2E-200-02` | Unified Context Structural Aligner (`Imp-626`) | Feed malformed conversation histories containing consecutive duplicate roles (e.g. `user` followed immediately by `user`), empty text parts, and mismatched systemInstruction roles. Verify normalization into strict alternating `user`/`model` sequences. | Merges adjacent same-role messages; strips whitespace-only empty text blocks; ensures valid alternating structure without losing message semantics. |
| `E2E-200-03` | Thinking Cache Target Version Guard (`Imp-627`) | Audit changesets against `SuggestionDeleteThinkingModal.tsx`. Verify that routine releases maintain `SUGGESTION_DELETE_THINKING_STORE = false`, while architecture/schema breaking refactors mandate setting the flag to `true` with `SUGGESTION_TARGET_VERSION` matching manifest SemVer. | Flags violations when schema changes lack thinking cache invalidation; flags routine releases that accidentally leave invalidation prompt enabled. |
| `E2E-200-04` | Job Breakaway Process Spawner (`Imp-628`) | Simulate Windows Job Object execution where `CREATE_BREAKAWAY_FROM_JOB` fails with `os error 5: Access is denied`. Also simulate passing `.bat`/`.cmd` files. Verify automatic multi-stage fallback: target binary → detected base binary → shell dispatch (`cmd.exe /c start`). | Recovers from Access Denied by stripping breakaway flag; replaces direct batch file execution with shell wrapper or base binary; leaves zero orphan processes. |
| `E2E-200-05` | Lock Timeout Escalator (`Imp-629`) | Simulate high concurrency lock contention on split SQLite databases (`active_prompts`, `repo_prompts.db`). Verify stepped backoff escalation (250ms -> 500ms -> 1000ms -> 2500ms -> 5000ms) with jitter, and route exhausted locks into structured `AppError::DatabaseLocked`. | Retries transient lock busy states with exponential jittered delay; routes unrecoverable deadlocks into error pipeline without swallowing errors. |

---

## 3. Automated Execution Commands (Deferred)
When the user executes the test verification suite, run:
```bash
cargo test --package antigravity-manager --lib proxy::handlers::common::tests_prefix_stability_sanitizer
cargo test --package antigravity-manager --lib proxy::handlers::common::tests_unified_context_structural_aligner
cargo test --package antigravity-manager --lib modules::release_gate::tests_thinking_cache_target_version_guard
cargo test --package antigravity-manager --lib modules::instance::tests_job_breakaway_process_spawner
cargo test --package antigravity-manager --lib modules::db_orm::tests_lock_timeout_escalator
```
