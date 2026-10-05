# Master Plan: Two-Phase 4-Hour Quota Gate & Hours-Remaining Weekly Scoring Algorithm

**Slug:** `138-two-phase-4h-gate-and-hours-remaining-scoring`  
**Target Version:** v4.158.0  
**Status:** ACTIVE  

---

## 1. Context & Objectives
Implement the user-specified two-phase candidate account selection & scoring algorithm:
1. **Normal Pass (Phase 1)**:
   - If 4h quota is less than 100% and cycle not elapsed: score is strictly `0.0`.
   - If 4h quota == 100% (or period finished): score is `base_weekly_score.floor()`.
   - `base_weekly_score = (tier_multiplier * effective_weekly_pct * (168.0 - hours_remaining)) / 100.0`.
   - Weekly quota < 8% is strictly `0.0`.
   - Sorting order: DESCENDING (highest score selected first).
2. **Fallback Pass (Phase 2)**:
   - When no 100% 4h quota accounts exist (`available_pool.is_empty() && cooldown_pool.is_empty()`):
   - Evaluate available accounts above threshold using partial 4h quota:
     `score = ((q_4h / 100.0) * base_weekly_score).floor()`.
3. **Claude / 3p Buckets**:
   - Retain `TODO(claude)` ambiguity marker as instructed.

---

## 2. Subtasks Breakdown

- [ ] **Subtask 01**: `01-scoring-algorithm-and-fallback.md`  
  Refactor `score_candidate_account`, introduce `calculate_weekly_base_score` and `score_candidate_account_fallback`, and update Step 3 fallback in `select_candidate_profiles` in `src-tauri/src/modules/auto_switcher.rs`.
- [ ] **Subtask 02**: `02-unit-tests-and-verification.md`  
  Update unit tests in `src-tauri/src/modules/auto_switcher.rs` to match the exact mathematical specification and verify edge cases (8% floor, period finished, 4h partial scaling).

---

## 3. Verification & Pre-flight
- Pre-flight quality gate verification
- Atomic GitMap commit
- Minor/patch version release synchronization
