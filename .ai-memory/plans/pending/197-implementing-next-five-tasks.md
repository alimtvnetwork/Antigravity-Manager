# Parent Plan — 197 Implementing Next Five Tasks

Status: IN PROGRESS (TAKEN)
Phase: 1 (claim committed & spec authored)
Owner: Agent-197-Lead
Specs: `02-spec/21-app/197-implementing-next-five-tasks/spec.md`
Subtasks: `.ai-memory/plans/subtasks/197-implementing-next-five-tasks/`
Backlog Delta: 55 steps -> 50 steps remaining (-5 steps claimed)

## 1. Selected Five Tasks (Claimed & In-Progress)

| Task # | Slug | Module Target | Status | Implementation Details |
|---|---|---|---|---|
| 01 | `request-payload-length-negotiator` | `src-tauri/src/proxy/handlers/common/request_payload_length_negotiator.rs` | `IN PROGRESS (TAKEN)` | Negotiate Content-Length vs chunked streaming, bounding memory buffers (10MB ceiling) to prevent OOM vulnerabilities. |
| 02 | `sse-heartbeat-generator` | `src-tauri/src/proxy/handlers/common/sse_heartbeat_generator.rs` | `IN PROGRESS (TAKEN)` | Emit periodic keepalive frames (`:\n\n` / `: ping\n\n`) to prevent socket timeouts during extended LLM reasoning. |
| 03 | `token-budget-decider` | `src-tauri/src/proxy/handlers/common/token_budget_decider.rs` | `IN PROGRESS (TAKEN)` | Dynamic headroom calculation between prompt tokens, thinking/reasoning budget, and `max_output_tokens`. |
| 04 | `quota-exhaustion-predictor` | `src-tauri/src/modules/auto_switcher/quota_exhaustion_predictor.rs` | `IN PROGRESS (TAKEN)` | Calculate token burn rate velocity and forecast exhaustion time horizon before HTTP 429 lockouts. |
| 05 | `candidate-dynamic-scorer` | `src-tauri/src/modules/auto_switcher/candidate_dynamic_scorer.rs` | `IN PROGRESS (TAKEN)` | Dynamic candidate weighting incorporating predicted burn rate velocity, consecutive throttles, and cooldown timers into auto-switch ranking. |

## 2. Invariants & Rules Upheld
- **Zero-Build & Zero-Test Invariant**: Strictly preserved; zero compilers invoked.
- **Repository-Relative Paths**: 100% compliant across all modules, tests, specs, and docs.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Anti-Collision Commit**: Enqueued and committed upstream before implementation.
