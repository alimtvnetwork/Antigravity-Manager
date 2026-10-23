# Specification 196: Implementing Next Five Tasks (Wildcard Model Routing, Payload Length Negotiator, Token Budget Decider, SSE Heartbeat Generator & Quota Exhaustion Predictor)

## 1. Overview & Context
This specification defines the architectural contracts and compliance verifiers for the 5 continuous reliability and proxy streaming tasks in Sequence 196.
These tasks originate from `AGENTS.md` code quality invariants (wildcard model routing & regex sanitization) and reverse proxy streaming resilience (payload length negotiation, token budget calculation, SSE keepalive heartbeats, and quota exhaustion prediction).

---

## 2. Five Planned Tasks Breakdown

### Task 1: Wildcard Model Routing & Regex Sanitization Guard (`Imp-610`)
- **Module**: `src-tauri/src/modules/release_gate/wildcard_model_routing_guard.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_wildcard_model_routing.rs`
- **Specification**:
  - Prioritize wildcard patterns (e.g., `gemini-*-flash-*`, `claude-3-5-*`, `gpt-4o-*`) to anticipate future model releases rather than exact string matches.
  - Reject static, hardcoded exact string matches in routing rules that fail to cover future minor version releases.
  - Enforce regex-based pattern matching over static keyword replacement for agent-client prompt sanitization, ensuring full coverage without stripping pipeline system prompts or user queries.
  - Structs: `ModelRouteRule`, `RoutingStrategy`, `ModelMatchReport`, `ModelRoutingViolation`.
  - Functions: `evaluate_model_match(model_name: &str, rules: &[ModelRouteRule]) -> Result<ModelMatchReport, ModelRoutingViolation>`, `lint_routing_code(source: &str) -> Vec<ModelRoutingViolation>`.

### Task 2: Request Payload Length Negotiator (`Imp-611`)
- **Module**: `src-tauri/src/proxy/handlers/common/request_payload_length_negotiator.rs`
- **Test Suite**: `src-tauri/src/proxy/handlers/common/tests_payload_length_negotiator.rs`
- **Specification**:
  - Negotiates whether request payloads should be streamed or buffered based on Content-Length, Transfer-Encoding, and buffer memory limits.
  - Enforces bounded memory allocation ceilings (e.g. 10MB default max) preventing out-of-memory (OOM) vulnerabilities from unbounded chunked transfers.
  - Handles absent Content-Length, chunked transfer encodings, and oversized streaming bodies.
  - Structs: `PayloadLengthDecision`, `TransferEncodingMode`, `PayloadNegotiationConfig`, `NegotiationReport`.
  - Functions: `negotiate_payload_strategy(headers: &HeaderMap, config: &PayloadNegotiationConfig) -> PayloadLengthDecision`.

### Task 3: Token Budget Decider & Thinking Allocation Controller (`Imp-612`)
- **Module**: `src-tauri/src/proxy/handlers/common/token_budget_decider.rs`
- **Test Suite**: `src-tauri/src/proxy/handlers/common/tests_token_budget_decider.rs`
- **Specification**:
  - Calculates dynamic token budgets across upstream completions, allocating headroom between input prompt tokens, thinking/reasoning budget, and `max_output_tokens`.
  - Filters and backfills thinking budgets according to model capabilities (e.g. models requiring thinking budgets vs standard models).
  - Enforces context window safety boundaries, clamping output tokens so total tokens do not exceed the model context window.
  - Structs: `TokenBudgetRequest`, `TokenAllocation`, `ThinkingBudgetDecision`, `BudgetViolation`.
  - Functions: `decide_token_budget(request: &TokenBudgetRequest) -> Result<TokenAllocation, BudgetViolation>`.

### Task 4: SSE Heartbeat Generator & Keepalive Frame Scheduler (`Imp-613`)
- **Module**: `src-tauri/src/proxy/handlers/common/sse_heartbeat_generator.rs`
- **Test Suite**: `src-tauri/src/proxy/handlers/common/tests_sse_heartbeat_generator.rs`
- **Specification**:
  - Emits periodic SSE keepalive comment frames (`:\n\n` or `: ping\n\n`) to prevent reverse proxy and client socket timeouts during extended LLM deliberation and deep thinking phases.
  - Configurable heartbeat intervals (e.g. 15s default, 5s minimum, 60s maximum) with jitter.
  - Tracks time elapsed since last real event frame; only generates heartbeats when idle exceeds the threshold.
  - Structs: `HeartbeatConfig`, `HeartbeatGeneratorState`, `HeartbeatFrameAction`.
  - Functions: `evaluate_heartbeat_tick(state: &mut HeartbeatGeneratorState, now_epoch_ms: u64) -> Option<HeartbeatFrameAction>`.

### Task 5: Quota Exhaustion Predictor & Burn Rate Forecaster (`Imp-614`)
- **Module**: `src-tauri/src/modules/auto_switcher/quota_exhaustion_predictor.rs`
- **Test Suite**: `src-tauri/src/modules/auto_switcher/tests_quota_exhaustion_predictor.rs`
- **Specification**:
  - Extends auto-switcher candidate scoring by calculating token burn rate velocity and forecasting the time horizon until quota exhaustion.
  - Evaluates rolling token consumption across sliding windows (5m, 15m, 1h), predicting exhaustion time and warning before 429 lockout occurs.
  - Provides early-switch signals when burn rate velocity indicates exhaustion within a threshold window (e.g. < 5 minutes remaining).
  - Structs: `UsageObservation`, `BurnRateVelocity`, `ExhaustionForecast`, `ExhaustionSeverity`.
  - Functions: `predict_quota_exhaustion(observations: &[UsageObservation], remaining_quota: u64) -> ExhaustionForecast`.

---

## 3. Invariants & Rules Upheld
- **Zero-Build & Zero-Test Invariant**: Strictly DO NOT build code or run tests during this implementation phase.
- **Strictly Relative Paths**: All references use repository-relative Git paths.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Anti-Collision Commit**: Committed immediately as taken to prevent multi-agent collisions.
