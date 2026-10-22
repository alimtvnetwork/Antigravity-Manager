# Subtask 02: Fix Account Selection Bugs (Reduced Quota & In-Use Collisions)

## Objective
Repair the root causes of account reselection bugs where Fast-Forward selects an account that is already in use or has reduced 4-hour quota ($< 100\%$).

## Files to Modify
- Backend:
  - `src-tauri/src/modules/auto_switcher/scoring.rs`
  - `src-tauri/src/modules/auto_switcher/selection.rs`
  - `src-tauri/src/modules/auto_switcher/manual.rs`
  - `src-tauri/src/modules/auto_switcher/instances.rs`
- Frontend:
  - `src/services/instanceService/scoring.ts`
  - `src/stores/useInstanceStore/smart.ts`

## Key Fixes
1. **Backend Scoring Gate (`scoring.rs`)**:
   - `score_candidate_account`: strictly check `if q_4h < 100.0 { return 0.0; }`. Remove the `!is_period_finished` loophole that allowed accounts with $<100\%$ quota to receive full scores.
2. **Backend Selection Gate (`selection.rs`)**:
   - Do NOT artificially override `quota = 100.0` when `is_period_finished` is true if the account actually has recorded quota $< 100.0$.
   - Include current instance's `bound_account_id` AND `bound_email` into `effective_exclusions`.
   - Ensure running sibling instances' `bound_account_id` and `bound_email` are both added to exclusions.
3. **Backend Manual Rotation (`manual.rs`) & In-Use Discovery (`instances.rs`)**:
   - In `manual.rs`: ensure target instance's `bound_email` and running siblings' `bound_email` are properly resolved and added to `excluded`.
   - In `instances.rs`: ensure `get_active_in_use_account_ids()` also resolves `inst.bound_email` to account ID and tracks email exclusions.
4. **Frontend Scoring & Candidate Selection (`scoring.ts` & `smart.ts`)**:
   - In `scoring.ts: calculateMultiplicativeScore`: strictly return `0` if `fourHourQuotaPercent < 100 || activeFactor === 0`.
   - In `scoring.ts: rankSmartCandidates`: filter `primaryEligible` strictly with `q4h >= 100`.
   - In `smart.ts: smartRotateProfileAccount`: check both `bound_account_id` and `bound_email` when collecting `activeInUseAccountIds`. Include current instance account by ID and email.
