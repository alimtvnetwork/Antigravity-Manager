# Subtask 05: Candidate Dynamic Scorer
Status: COMPLETED
Sequence: 197
Owner: Subagent-2
Target Module: `src-tauri/src/modules/auto_switcher/candidate_dynamic_scorer.rs`
Test Suite: `src-tauri/src/modules/auto_switcher/tests_candidate_dynamic_scorer.rs`
Objective: Dynamic candidate weighting incorporating predicted burn rate velocity, consecutive throttles, and cooldown timers into auto-switch ranking.

Evidence:
- Module `src-tauri/src/modules/auto_switcher/candidate_dynamic_scorer.rs` created with `CandidateScoringFactors`, `ScoredCandidate`, and `rank_candidates`.
- Test suite `src-tauri/src/modules/auto_switcher/tests_candidate_dynamic_scorer.rs` created covering hard disqualifications, velocity gradients, throttle penalties, and candidate ranking.
