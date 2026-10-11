# Subtask Implementation Plan: Auto-Switcher Candidate Scoring & Fallback Fix

> **Subtask ID:** `03-auto-switcher-candidate-scoring-and-fallback-fix`  
> **Parent Slug:** `auto-switch-and-rotate-fix`  
> **Target Files:** `src-tauri/src/modules/auto_switcher/selection.rs`, `src-tauri/src/modules/auto_switcher/scoring.rs`  
> **Status:** `[DONE]`  
> **Author:** @aukgit  

---

## 1. Context & Objectives

In `v4.162.0`, account rotation succeeded whenever alternative accounts possessed remaining headroom. In subsequent versions, account switching broke completely when all accounts had sub-100% quota (e.g., 90%, 80%, 60%).

### Root Cause:
1. `selection.rs:254`: Step 3 fallback was guarded by `available_pool.is_empty() && cooldown_pool.is_empty()`. If any account had a cooldown or lease, `cooldown_pool` was not empty, so Step 3 was never evaluated.
2. `selection.rs:294`: Step 3 called `score_candidate_account` instead of `score_candidate_account_fallback`. In `scoring.rs:205`, `score_candidate_account` strictly returned `0.0` for accounts with quota $< 100\%$, zeroing out all scores.
3. `selection.rs:388`: If no 100% accounts were found and fallback was bypassed, `select_and_verify_next_best_profile` returned `Ok(None)` and aborted.

---

## 2. Implementation Summary

1. In `src-tauri/src/modules/auto_switcher/selection.rs`:
   - Relaxed Step 3 fallback entry condition to `available_pool.is_empty()`.
   - Replaced candidate scoring call with `score_candidate_account_fallback`.
   - Verified that `select_and_verify_next_best_profile` selects the best candidate exceeding the low quota threshold when 100% accounts are absent.
2. In `src-tauri/src/modules/auto_switcher/scoring.rs`:
   - Verified that `score_candidate_account_fallback` scales scores proportionally by headroom rather than clamping to zero.
