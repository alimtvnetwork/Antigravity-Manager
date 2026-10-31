# End-to-End Master Test Plan: Sequence 196 (Release Gate & Architectural Invariants)

## 1. Scope & Objectives
This document catalogs the comprehensive end-to-end and integration test suite for the 5 compliance and architectural invariant modules implemented in Sequence 196.
In accordance with the **Zero-Build & Zero-Test Invariant**, live execution is deferred until an explicit testing prompt is provided.
Any AI agent executing deferred end-to-end verification must execute the scenarios detailed below.

---

## 2. Test Cases Specification

### Test Case `E2E-196-01`: Wildcard Model Routing & Future-Proof Rule Verification
- **Target Module**: `src-tauri/src/modules/release_gate/wildcard_model_routing_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_wildcard_model_routing.rs`
- **Objective**: Ensure model routing prioritizes wildcard patterns (`gemini-*-flash-*`, `claude-3-5-*`, `gpt-4o-*`) to anticipate future model releases rather than brittle exact string matches, rejecting hardcoded static matches.
- **Execution Steps**:
  1. Test `is_wildcard_pattern` with globs containing `*` and `?` (e.g. `gemini-*-flash-*`, `claude-3-5-*`, `gpt-4o-?`); assert `true`.
  2. Test `is_wildcard_pattern` with exact strings (e.g. `gemini-1.5-flash-latest`, `claude-3-5-sonnet`); assert `false`.
  3. Test `match_wildcard` against future model names (`gemini-2.5-flash-preview`, `claude-3-5-sonnet-20241022`, `gpt-4o-mini`); assert matches succeed without code changes.
  4. Initialize `evaluate_model_match` with valid wildcard rules; verify `ModelMatchReport` returns `is_wildcard_matched: true` and routes to target adapter.
  5. Attempt to register a rule with `RoutingPatternType::BrittleExact`; assert immediate rejection with `ModelRoutingViolation::BrittleExactMatchForbidden`.
  6. Pass an empty model name; assert rejection with `ModelRoutingViolation::EmptyModelPattern`.
  7. Pass an invalid regex pattern; assert rejection with `ModelRoutingViolation::InvalidRegexPattern`.
  8. Run `lint_routing_code` on source code with exact equality `model == "gemini-1.5-flash"`; assert detection of `BrittleExactMatchForbidden`.
  9. Run `lint_routing_code` on source code with chained `.replace()` calls in prompt sanitization; assert detection of `StaticKeywordSanitizationForbidden`.
- **Expected Outcome**: All unit tests pass, enforcing that model routing accommodates future model releases without brittle hardcoded strings.

---

### Test Case `E2E-196-02`: Agent-Client Prompt Sanitization Regex Guard Verification
- **Target Module**: `src-tauri/src/modules/release_gate/agent_prompt_sanitizer_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_agent_prompt_sanitizer.rs`
- **Objective**: Ensure agent-client prompt sanitization strictly prioritizes regex-based pattern matching over static keyword replacement, ensuring full pattern coverage without stripping pipeline system prompts or user queries.
- **Execution Steps**:
  1. Configure `SanitizerRule::regex` with patterns for bearer authorization tokens and internal debug headers; assert `sanitize_prompt_text` redacts matching tokens while preserving prompt body.
  2. Configure `SanitizerRule::static_keyword`; assert `lint_sanitization_implementation` flags `SanitizationViolation::StaticKeywordSanitizationForbidden`.
  3. Test prompt text containing legitimate system prompts (`<system>`, `System Instruction:`); verify `preserve_system_prompts: true` ensures system directives are preserved.
  4. Test prompt text containing legitimate user queries; verify `preserve_user_queries: true` prevents destructive stripping of user content.
  5. Pass malformed regex pattern to `SanitizerRule::regex`; assert detection of `SanitizationViolation::InvalidRegexPattern`.
  6. Pass code snippet with `source.replace("API_KEY", "")` to `lint_sanitization_implementation`; assert detection of non-compliant static keyword sanitization.
  7. Run linter on clean compiled regex replacement routine; verify 0 violations returned.
- **Expected Outcome**: All unit tests pass, ensuring prompt sanitization is thorough, robust, and non-destructive to system prompts and user queries.

---

### Test Case `E2E-196-03`: Pipeline Decoupling & Protocol Normalization Guard Verification
- **Target Module**: `src-tauri/src/modules/release_gate/pipeline_protocol_decoupling_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_pipeline_protocol_decoupling.rs`
- **Objective**: Ensure the gateway pipeline remains strictly decoupled from specific protocols. The four protocols function purely as Gemini adapters restricted to parameter normalization and payload transformation, while the pipeline stage uniformly handles thinking blocks, budgeting, structural alignment, and sanitization.
- **Execution Steps**:
  1. Validate `validate_output_protocol` accepts `"antigravity-gemini"` and `"gemini"`, rejecting foreign outputs (`"anthropic-messages"`, `"openai-chat"`) with `DecouplingViolation::NonGeminiOutputProtocolForbidden`.
  2. Validate `validate_normalized_gemini_json` on valid Gemini payload containing `contents`, `role`, and `parts`; assert validation passes (`Ok(())`).
  3. Validate `validate_normalized_gemini_json` on payload missing `contents`; assert rejection with `DecouplingViolation::PayloadNotNormalizedToGemini`.
  4. Validate `validate_normalized_gemini_json` on raw OpenAI payload containing `"messages"`; assert rejection with `DecouplingViolation::ProtocolLeakInPipeline`.
  5. Validate `validate_normalized_gemini_json` on raw Claude payload containing `"anthropic_version"`; assert rejection with `DecouplingViolation::ProtocolLeakInPipeline`.
  6. Test `validate_architectural_barrier` for `AdapterNormalization` stage: assert premature pipeline operations (`thinking_block_backfill`, `filter_budget`, `prompt_sanitizing`) trigger `PrematurePipelineOperationInAdapter`.
  7. Test `validate_architectural_barrier` for adapter edge-case patch without valid justification; assert `UnjustifiedAdapterModification`. Provide valid `ProtocolDivergenceJustification` (`is_unsolvable_in_pipeline: true`); assert compliance passes.
  8. Run `lint_pipeline_source_code` across pipeline files; detect leaks of `anthropic-version`, `x-openai-organization`, or `role == "function"`.
  9. Run `lint_adapter_source_code` across adapter files; detect premature calls to `backfill_thinking_block` or `filter_thinking_budget`.
- **Expected Outcome**: All unit tests pass, guaranteeing strict protocol decoupling and adherence to the Pipeline First and Backend Fix Strategy invariants.

---

### Test Case `E2E-196-04`: Single Problem Scope & PR Cohesion Verification
- **Target Module**: `src-tauri/src/modules/release_gate/pr_scope_cohesion_verifier.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_pr_scope_cohesion.rs`
- **Objective**: Validate that pull request changesets remain strictly confined to a single problem class (e.g., protocol adapter, database, UI styling, release governance) and that every commit represents an independent, self-contained, individually revertable functional unit.
- **Execution Steps**:
  1. Initialize `PrScopeAuditor` with a changeset confined to a single problem domain (e.g. `src-tauri/src/proxy/`); verify `is_cohesive: true` and 0 scope violations.
  2. Inject mixed concerns touching unrelated subsystem domains (e.g. governance docs + SQLite ORM schema + UI styling + proxy handlers); verify `is_cohesive: false` and detection of `ScopeViolation::CrossDomainContamination`.
  3. Audit individual commits for self-containment: verify commit pairing implementation with its unit test suite passes self-containment check.
  4. Inject commit with dangling dependencies across commits; verify `is_individually_revertable: false` and flag `ScopeViolation::EntangledCommit`.
  5. Audit PR title against touched files; verify divergence (e.g. title "Fix SQLite busy handler" vs touched file `src/components/ThemeToggle.tsx`) flags `ScopeViolation::TitleScopeMismatch`.
  6. Verify report serde JSON serialization and deserialization roundtrip.
- **Expected Outcome**: All unit tests pass, ensuring PR reviewability, isolation of concerns, and clean git history revertability.

---

### Test Case `E2E-196-05`: Contributor Cost Disclosure & Attribution Verification
- **Target Module**: `src-tauri/src/modules/release_gate/contributor_cost_disclosure_linter.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_contributor_cost_disclosure.rs`
- **Objective**: Enforce contributor respect and disclosure invariants per AGENTS.md: disclose costs before merging (affected existing behaviors, unverified paths), preserve authorship via Co-authored-by trailers, and provide file-by-file breakdowns upon rejection.
- **Execution Steps**:
  1. Inspect PR description containing explicit sections for "Affected Existing Behaviors" and "Unverified Paths"; verify `is_compliant: true` and 0 cost disclosure violations.
  2. Submit PR description omitting cost disclosure or leaving empty placeholders (`"None"`, `"N/A"`); verify detection of `CostDisclosureViolation::MissingAffectedBehaviorsDisclosure` and `MissingUnverifiedPathsDisclosure`.
  3. Audit commit history for proxy/squash PRs authored by external contributors; verify presence of valid `Co-authored-by: Name <email>` trailers preserving author attribution.
  4. Audit proxy commit omitting `Co-authored-by` trailer; verify detection of `AttributionViolation::MissingCoAuthorTrailer`.
  5. Validate rejection report generator ensures any rejected PR output includes structured file-by-file accept/drop breakdown and actionable path forward.
  6. Run static linter against sample pull request markdown bodies and verify compliant disclosures pass with 0 warnings.
- **Expected Outcome**: All unit tests pass, ensuring full transparency of implementation costs and strict contributor attribution.

---

## 3. Test Execution Command (Deferred)
When the user explicitly issues the testing prompt:
```bash
cargo test --package antigravity-manager --lib modules::release_gate::tests_wildcard_model_routing
cargo test --package antigravity-manager --lib modules::release_gate::tests_agent_prompt_sanitizer
cargo test --package antigravity-manager --lib modules::release_gate::tests_pipeline_protocol_decoupling
cargo test --package antigravity-manager --lib modules::release_gate::tests_pr_scope_cohesion
cargo test --package antigravity-manager --lib modules::release_gate::tests_contributor_cost_disclosure
```
