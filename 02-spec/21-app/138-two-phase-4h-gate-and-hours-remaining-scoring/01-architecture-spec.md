# Architecture Spec: Two-Phase 4-Hour Quota Gate & Hours-Remaining Weekly Scoring Algorithm

**Slug:** `138-two-phase-4h-gate-and-hours-remaining-scoring`  
**Version:** v4.158.0 (target)  
**Status:** DRAFT -> READY  

---

## 1. User Requirement (Verbatim)

> "if 4 h time is less than 100% than usually it should be 0  
> unless there is another option  
> if nothing found then we use 4h calculatin to  
> 4h percentage % x score what we have and also less than 8 % in weekly will be count as 0, clear??  
> if so then now write the algo , spec and then implenent please"  
>  
> (Preceding context):  
> "calculation is very wrong  
> kino*** 86% 7h 86×7×3 = 1,806 18 Last  
> Hours Elapsed instead we need hours remaining, hours remaing = 7h = (refil time) to refil again for weekly balance  
> refil time = 7h  
> full weekly hours = 7*24  
> (multiply the four hour percentage%) x 86 x ((7*24 - 7h) / 100) = this one should be the calculation and this one should be the f  
> roki*** 100% 106h 100×106×3 = 31,800 318 1st 🥇  
> (multiply the four hour percentage%) x (100×(full weekly hours - 106h))/100  
> is it clear???"

---

## 2. Algorithm Specification & Mathematical Formulation

### 2.1 Terminology & Inputs
For any candidate account `acc`:
- `q_4h`: 4-hour window quota percentage (0.0 to 100.0).
- `weekly_pct`: weekly quota remaining percentage (0.0 to 100.0).
- `hours_remaining`: refill countdown in hours until weekly quota reset.
- `TOTAL_WEEK_HOURS = 7 * 24 = 168.0`.
- `hours_elapsed = TOTAL_WEEK_HOURS - hours_remaining` (the elapsed cycle duration).
  - Example: `hours_remaining = 7h` -> `hours_elapsed = 168 - 7 = 161h`.
  - Example: `hours_remaining = 106h` -> `hours_elapsed = 168 - 106 = 62h`.
- `tier_multiplier`: Ultra = 5.0, Pro = 3.0, Free = 1.0.

### 2.2 Weekly Quota Threshold (Gemini Only)
- If `weekly_pct < 8.0`, then `effective_weekly_pct = 0.0`.
- Otherwise, `effective_weekly_pct = weekly_pct`.
- Claude / 3p buckets: Excluded with `TODO(claude)` ambiguity marker.

### 2.3 Base Weekly Score ("Score What We Have")
$$\text{weekly\_effective\_score} = \text{effective\_weekly\_pct} \times (168.0 - \text{hours\_remaining})$$
$$\text{base\_weekly\_score} = \frac{\text{tier\_multiplier} \times \text{weekly\_effective\_score}}{100.0}$$

- If `is_period_finished` is true (cycle reset timestamp is in the past), credits refresh to 100%:
  $$\text{weekly\_effective\_score} = 168.0 \times 100.0 = 16800.0$$
  $$\text{base\_weekly\_score} = \frac{\text{tier\_multiplier} \times 16800.0}{100.0} = \text{tier\_multiplier} \times 168.0$$

### 2.4 Phase 1: Primary Selection ("Usually" / Accounts with 100% 4h Quota)
- **Hard Gate**: If `q_4h < 100.0 && !is_period_finished`:
  $$\text{Score} = 0.0$$
  (Disqualified from primary selection; 0.0 score is filtered out).
- If `q_4h >= 100.0 || is_period_finished`:
  $$\text{Score} = \lfloor \text{base\_weekly\_score} \rfloor$$
- Sort order: **DESCENDING** (highest score selected first).

### 2.5 Phase 2: Graceful Fallback ("Unless There Is Another Option / If Nothing Found")
- Triggered **if and only if** `available_pool.is_empty() && cooldown_pool.is_empty()` (no account on the machine has 100% 4h quota).
- In this fallback mode, accounts with partial 4h quota (`q_4h > threshold`) are evaluated:
  $$\text{Score} = \left\lfloor \frac{q_{4h}}{100.0} \times \text{base\_weekly\_score} \right\rfloor$$
- Accounts with `weekly_pct < 8.0` still produce `0.0`.
- Sort order: **DESCENDING** (highest score selected first).

---

## 3. Worked Validation Examples

| Account | 4h Quota | Weekly % | Hours Remaining | Distance ($168 - h$) | Tier | Base Score | Phase 1 Score | Phase 2 Score (if 4h degraded to 50%) | Priority Rank |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **kino\*\*\*** | 100% | 86% | 7h | 161h | Pro (3.0) | $\frac{3 \times 86 \times 161}{100} = 415.38$ | **415** | $\lfloor 0.50 \times 415.38 \rfloor = 207$ | **1st 🥇** |
| **roki\*\*\*** | 100% | 100% | 106h | 62h | Pro (3.0) | $\frac{3 \times 100 \times 62}{100} = 186.0$ | **186** | $\lfloor 0.50 \times 186.0 \rfloor = 93$ | **2nd** |
| **ashi\*\*\*** | 100% | 6% (< 8%) | 20h | 148h | Pro (3.0) | 0.0 (below 8%) | **0** | 0 | **Excluded** |
| **degraded\*\*\***| 50% | 86% | 7h | 161h | Pro (3.0) | 415.38 | **0** (Phase 1 gate) | **207** (Phase 2 fallback) | **Phase 2: 1st** |

---

## 4. Architectural Verification Gates
- **VG-01**: `score_candidate_account` returns `0.0` when `q_4h < 100.0` and `!is_period_finished`.
- **VG-02**: `score_candidate_account` calculates $\lfloor \frac{\text{tier} \times \text{weekly\_pct} \times (168 - \text{rem})}{100} \rfloor$ when `q_4h >= 100.0`.
- **VG-03**: Weekly quota below 8% strictly outputs `0.0`.
- **VG-04**: When `available_pool` and `cooldown_pool` are empty, Step 3 fallback in `select_candidate_profiles` calculates score using $\lfloor \frac{q_{4h}}{100} \times \text{base\_weekly\_score} \rfloor$.
- **VG-05**: All scores are sorted in DESCENDING order (`b.score.partial_cmp(&a.score)`).
- **VG-06**: Rust pre-flight checks pass cleanly.
