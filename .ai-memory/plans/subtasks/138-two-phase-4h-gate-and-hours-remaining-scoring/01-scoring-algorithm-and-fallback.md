# Subtask 01: Scoring Algorithm & Fallback Implementation

**Subtask Slug:** `01-scoring-algorithm-and-fallback`  
**Parent Task:** `138-two-phase-4h-gate-and-hours-remaining-scoring`  
**File:** `src-tauri/src/modules/auto_switcher.rs`  

---

## Technical Details
1. Implement `calculate_weekly_base_score(acc: &Account, target_model: &str, now_sec: i64) -> f64`:
   - Computes tier multiplier (Ultra: 5.0, Pro: 3.0, Free: 1.0).
   - Extracts Gemini weekly buckets (`gemini-weekly`, window `weekly`, excluding `3p` and `claude`).
   - If weekly percentage < 8.0%, treats as `0.0`.
   - `hours_elapsed = 168.0 - hours_remaining`.
   - `weekly_effective_score = effective_weekly_pct * hours_elapsed`.
   - If `is_period_finished`, sets `weekly_effective_score = 168.0 * 100.0`.
   - Returns `(tier_multiplier * weekly_effective_score) / 100.0`.
2. Implement `score_candidate_account`:
   - If `q_4h < 100.0 && !is_period_finished`: return `0.0`.
   - Otherwise, return `calculate_weekly_base_score(acc, target_model, now_sec).floor()`.
3. Implement `score_candidate_account_fallback`:
   - If `q_4h <= 0.0`: return `0.0`.
   - Returns `((q_4h / 100.0) * calculate_weekly_base_score(acc, target_model, now_sec)).floor()`.
4. In `select_candidate_profiles` step 3 (fallback):
   - Replace `let score = quota / 100.0;` with `score_candidate_account_fallback(acc, target_model, now_sec)`.
