# Deferred End-to-End Test Master Plan (Sequence 198)

## 1. Overview & Policy
Per project invariants and user instructions:
- **Zero-Build & Zero-Test Invariant**: Strictly no compilation or unit tests run during implementation.
- All live verification scenarios for Sequence 198 (`Imp-620` through `Imp-624`) are cataloged here for subsequent execution when the user delivers the dedicated testing prompt.

---

## 2. Test Scenarios Matrix

| Test ID | Target Component | Purpose & Verification Flow | Expected Result |
|---|---|---|---|
| `E2E-198-01` | Transport Backoff Stepper with Jitter (`Imp-620`) | Simulate transient and unrecoverable upstream network failures (DNS resolution failures, TCP resets, connection timeouts, TLS handshake negotiation failures, HTTP 429/502/503/504 gateway statuses vs unrecoverable client errors). Evaluate adaptive exponential progression across `FullJitter`, `EqualJitter`, `DecorrelatedJitter`, and deterministic `None` modes up to configured max attempts and ceiling caps. | Rejects unrecoverable errors immediately without retry; dynamically scales delay with jitter bounds between base and max backoff ceilings; ceases retries and emits exhaustion reason upon exceeding retry budget. |
| `E2E-198-02` | Transient Sandbox Manager & Zero-Orphan Cleanup (`Imp-621`) | Execute lifecycle management for transient test sandboxes (`test-create` / `test-clean`). Probe caller IDE self-preservation barriers against matching instance IDs and active caller PIDs (intercepting termination without `--force`). Create isolated sandboxed profile directories (`.user_data`, `.workspace_storage`) and mock configs, then trigger full sandbox cleanup and stale sandbox sweeps. | Blocks termination of active developer IDE session unless `--force` is supplied; creates completely isolated sandbox file structures; performs complete non-orphan purge removing temporary files while strictly preserving persistent `workspaceStorage` and leaving zero residual processes or lock files. |
| `E2E-198-03` | FIFO Prompt Queue Coordinator (`Imp-622`) | Validate runtime prompt dispatch and restore queries across split SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`). Enforce query ordering validator against inverted LIFO patterns (`ORDER BY updated_at DESC`, `created_at DESC`, `rowid DESC`) and missing ORDER clauses. Test in-memory queue coordinator with out-of-order prompt enqueuing, lease acquisition with TTL expiry, and completion lifecycle. | Strictly intercepts and rejects non-FIFO or descending query ordering; enforces `ORDER BY created_at ASC, id ASC`; processes enqueued prompts in deterministic FIFO arrival sequence; accurately manages lease expirations and state transitions. |
| `E2E-198-04` | Account Switch Audit Emitter (`Imp-623`) | Simulate auto-switch trigger events across all decision classes (`RateLimit429`, `PreemptiveVelocityExhaustion`, `QuotaDepletedZero`, `EndpointHealthDecay`, `CooldownExpiry`, `ManualUserSwitch`). Verify JSON telemetry serialization, payload enrichment with token consumption burn rates, candidate headroom scores, switch latency, and in-memory ring buffer rotation without swallowing errors. | Emits fully structured, auditable JSON switch records into tracing streams and ring buffer; includes comprehensive scoring breakdowns, trigger rationale, and success/failure indicators; strictly avoids silent error absorption. |
| `E2E-198-05` | Rejection Breakdown Generator (`Imp-624`) | Process changeset pull request rejection inputs through compliance validator. Test rejection items containing absolute paths (`C:\...`, `/home/...`), empty guidance, or missing accept/drop classifications. Verify markdown rejection report generation containing summary counts, categorized file breakdowns (Accept / Drop / Partial), specific architectural rationales, and actionable forward guidance. | Strictly rejects absolute filesystem paths, empty file lists, or missing actionable guidance; renders clean, professional GitHub-flavored markdown rejection report adhering to AGENTS.md contributor attribution and respect guidelines. |

---

## 3. Automated Execution Commands (Deferred)
When the user executes the test verification suite, run:
```bash
cargo test --package antigravity-manager --lib proxy::upstream::tests_transport_backoff_stepper
cargo test --package antigravity-manager --lib modules::instance::tests_transient_sandbox_manager
cargo test --package antigravity-manager --lib modules::db_orm::tests_fifo_prompt_queue_coordinator
cargo test --package antigravity-manager --lib modules::auto_switcher::tests_account_switch_audit_emitter
cargo test --package antigravity-manager --lib modules::release_gate::tests_rejection_breakdown_generator
```
