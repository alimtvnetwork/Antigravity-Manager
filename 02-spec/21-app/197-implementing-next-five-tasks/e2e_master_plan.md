# Deferred End-to-End Test Master Plan (Sequence 197)

## 1. Overview & Policy
Per project invariants and user instructions:
- **Zero-Build & Zero-Test Invariant**: Strictly no compilation or unit tests run during implementation.
- All live verification scenarios for Sequence 197 (`Imp-615` through `Imp-619`) are cataloged here for subsequent execution when the user delivers the dedicated testing prompt.

---

## 2. Test Scenarios Matrix

| Test ID | Target Component | Purpose & Verification Flow | Expected Result |
|---|---|---|---|
| `E2E-197-01` | Request Payload Length Negotiator | Inspect Content-Length and Transfer-Encoding headers on small (<1MB), medium (1MB–10MB), chunked, and oversized (>10MB) payloads. | Buffers small payloads, directly streams medium and chunked payloads, and rejects >10MB with HTTP 413. |
| `E2E-197-02` | SSE Heartbeat Keepalive Generator | Simulate extended model deliberation turns (>15s, >30s, >45s) and verify emission of non-disruptive SSE comment frames (`:\n\n` / `: ping\n\n`) and reset upon genuine event activity. | Emits periodic keepalive comment frames at 15s intervals; resets countdown on real tokens; handles backwards clock jumps safely. |
| `E2E-197-03` | Token Budget Decider & Thinking Allocator | Evaluate token allocations across Gemini reasoning models, Claude 3.7 with reasoning, optional thinking, and non-reasoning models (GPT-4o). | Automatically backfills thinking budgets on reasoning models; filters thinking on unsupported models; strictly enforces `max_output_tokens > thinking_budget`. |
| `E2E-197-04` | Quota Exhaustion Velocity Predictor | Feed timestamped token consumption observations into velocity forecaster with varying quota headroom and burn rates. | Correctly classifies exhaustion severity into `Normal`, `Warning`, `CriticalSwitchImminent`, or `Exhausted`; triggers proactive switch signals when runway < 300s. |
| `E2E-197-05` | Auto-Switcher Candidate Dynamic Scorer | Evaluate candidate accounts with mixed quota headroom, burn rate velocity penalties, consecutive throttle counts, active cooldowns, and sibling instance leases. | Accurately disqualifies in-use, inactive, or cooling accounts; penalizes high-burn rate and throttled accounts; correctly ranks highest-runway candidates first. |

---

## 3. Automated Execution Commands (Deferred)
When the user executes the test verification suite, run:
```bash
cargo test --package antigravity-manager --lib proxy::handlers::common::tests_payload_length_negotiator
cargo test --package antigravity-manager --lib proxy::handlers::common::tests_sse_heartbeat_generator
cargo test --package antigravity-manager --lib proxy::handlers::common::tests_token_budget_decider
cargo test --package antigravity-manager --lib modules::auto_switcher::tests_quota_exhaustion_predictor
cargo test --package antigravity-manager --lib modules::auto_switcher::tests_candidate_dynamic_scorer
```
