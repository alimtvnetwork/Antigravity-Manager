# Specification 197: Implementing Next Five Tasks (Payload Length Negotiator, SSE Heartbeat Generator, Token Budget Decider, Quota Exhaustion Predictor, Candidate Dynamic Scorer)

## 1. Overview & Context
This specification defines the architectural contracts, streaming resilience rules, and auto-switcher candidate scoring engines for 5 continuous reliability tasks in Sequence 197:
1. **Request Payload Length Negotiator** (`Imp-615`): Negotiates `Content-Length` vs chunked transfer encoding, bounding memory allocation (10MB ceiling) to prevent out-of-memory vulnerabilities.
2. **SSE Heartbeat Keepalive Generator** (`Imp-616`): Emits periodic SSE keepalive frames (`:\n\n` or `: ping\n\n`) to prevent socket timeouts during extended LLM reasoning/thinking.
3. **Token Budget Decider & Thinking Allocator** (`Imp-617`): Dynamic headroom calculation between prompt tokens, thinking/reasoning budget, and `max_output_tokens` per AGENTS.md.
4. **Quota Exhaustion Velocity Predictor** (`Imp-618`): Calculates token burn rate velocity and forecasts exhaustion time horizon before HTTP 429 lockouts.
5. **Auto-Switcher Candidate Dynamic Scorer** (`Imp-619`): Dynamically scores switcher candidates incorporating predicted burn rate velocity, consecutive throttles, and cooldown timers into auto-switch candidate ranking.

---

## 2. Five Planned Tasks Breakdown

### Task 1: Request Payload Length Negotiator (`Imp-615`)
- **Module**: `src-tauri/src/proxy/handlers/common/request_payload_length_negotiator.rs`
- **Test Suite**: `src-tauri/src/proxy/handlers/common/tests_payload_length_negotiator.rs`
- **Specification**:
  - Negotiates whether request payloads should be streamed or buffered based on Content-Length, Transfer-Encoding, and memory bounds.
  - Enforces bounded memory allocation ceilings (e.g. 10MB default max) preventing OOM vulnerabilities from unbounded chunked transfers.
  - Structs & Enums: `TransferEncodingMode`, `PayloadLengthDecision`, `PayloadNegotiationConfig`, `NegotiationReport`.
  - Functions: `negotiate_payload_strategy(content_length: Option<u64>, is_chunked: bool, config: &PayloadNegotiationConfig) -> PayloadLengthDecision`, `negotiate_payload_with_report(...) -> NegotiationReport`.

### Task 2: SSE Heartbeat Keepalive Generator (`Imp-616`)
- **Module**: `src-tauri/src/proxy/handlers/common/sse_heartbeat_generator.rs`
- **Test Suite**: `src-tauri/src/proxy/handlers/common/tests_sse_heartbeat_generator.rs`
- **Specification**:
  - Emits periodic SSE keepalive comment frames (`:\n\n` or `: ping\n\n`) to prevent client and proxy TCP socket timeouts during extended deliberation.
  - Tracks elapsed time since last genuine event or ping frame; yields `EmitPing` when interval threshold is exceeded.
  - Handles backwards clock skew safely without panic.
  - Structs & Enums: `HeartbeatConfig`, `HeartbeatGeneratorState`, `HeartbeatFrameAction`.
  - Functions: `evaluate_heartbeat_tick(config: &HeartbeatConfig, state: &mut HeartbeatGeneratorState, now_epoch_ms: u64) -> HeartbeatFrameAction`, `format_sse_comment(comment: &str) -> String`.

### Task 3: Token Budget Decider & Thinking Allocator (`Imp-617`)
- **Module**: `src-tauri/src/proxy/handlers/common/token_budget_decider.rs`
- **Test Suite**: `src-tauri/src/proxy/handlers/common/tests_token_budget_decider.rs`
- **Specification**:
  - Calculates dynamic token budgets across upstream completions, allocating headroom between input prompt tokens, thinking/reasoning budget, and `max_output_tokens`.
  - Filters and backfills thinking budgets according to model capabilities per AGENTS.md.
  - Ensures `effective_max_output_tokens > effective_thinking_budget` to prevent upstream 400 Bad Request errors.
  - Structs & Enums: `ModelThinkingCapability`, `TokenBudgetRequest`, `TokenAllocation`, `BudgetViolation`.
  - Functions: `decide_token_budget(request: &TokenBudgetRequest, capability: &ModelThinkingCapability) -> Result<TokenAllocation, BudgetViolation>`, `resolve_model_thinking_capability(model_name: &str) -> ModelThinkingCapability`.

### Task 4: Quota Exhaustion Velocity Predictor (`Imp-618`)
- **Module**: `src-tauri/src/modules/auto_switcher/quota_exhaustion_predictor.rs`
- **Test Suite**: `src-tauri/src/modules/auto_switcher/tests_quota_exhaustion_predictor.rs`
- **Specification**:
  - Calculates token burn rate velocity and forecasts the time horizon until quota exhaustion.
  - Evaluates consumption velocity across usage observations and triggers preemptive switching when remaining runway falls below threshold (e.g. <= 300s).
  - Structs & Enums: `UsageObservation`, `BurnRateVelocity`, `ExhaustionSeverity`, `ExhaustionForecast`.
  - Functions: `calculate_burn_rate_velocity(observations: &[UsageObservation]) -> BurnRateVelocity`, `predict_quota_exhaustion(observations: &[UsageObservation], remaining_tokens: u64, switch_threshold_seconds: u64) -> ExhaustionForecast`.

### Task 5: Auto-Switcher Candidate Dynamic Scorer (`Imp-619`)
- **Module**: `src-tauri/src/modules/auto_switcher/candidate_dynamic_scorer.rs`
- **Test Suite**: `src-tauri/src/modules/auto_switcher/tests_candidate_dynamic_scorer.rs`
- **Specification**:
  - Dynamically scores switcher candidates incorporating predicted burn rate velocity, consecutive throttles, and cooldown timers into auto-switch candidate ranking.
  - Penalizes accounts with critical quota exhaustion forecasts and boosts accounts with ample remaining runway and low error rates.
  - Structs & Enums: `CandidateScoringFactors`, `ScoredCandidate`, `ScoringViolation`.
  - Functions: `score_candidate_dynamically(factors: &CandidateScoringFactors) -> ScoredCandidate`, `rank_candidates(candidates: &[CandidateScoringFactors]) -> Vec<ScoredCandidate>`.

---

## 3. Invariants & Rules Upheld
- **Zero-Build & Zero-Test Invariant**: Strictly preserved; zero compilers invoked.
- **Repository-Relative Paths**: 100% compliant across all specs, tests, and code.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Anti-Collision Commit**: Enqueued and committed upstream before implementation.
