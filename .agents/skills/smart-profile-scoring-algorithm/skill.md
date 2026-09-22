---
name: smart-profile-scoring-algorithm
description: Multiplicative profile and account candidate scoring algorithm based on active status, tier multipliers, weekly quota, randomized tie-breaking, and pre-activation refresh verification.
---

# Smart Profile & Candidate Account Scoring Algorithm

This skill encapsulates the multiplicative candidate account scoring algorithm for instance and profile rotation:

1. **Active/Used Factor:**
   - Inactive / unused account = 1 point.
   - Active / already used account = 0 points (multiplicative zero elimination).

2. **Tier Multipliers:**
   - Pro tier = multiplied by 3.
   - Ultra tier = multiplied by 5.
   - Free / Standard = multiplied by 1.

3. **Weekly Available Quota Credits:**
   - Scaled by remaining weekly percentage (e.g., 90% available = $\times 90$; 10% available = $\times 10$).

4. **Composite Formula:**
   $$\text{Score} = \text{ActiveStatus}(0 \text{ or } 1) \times \text{TierMultiplier}(1, 3, 5) \times \text{WeeklyQuotaPct}$$

5. **Tie-Breaking:**
   - Randomized directional tie-breaker (A-to-Z or Z-to-A) to prevent hot-spotting when accounts tie in score.

6. **Pre-Activation Live Quota Verification & Demotion:**
   - Live quota refresh on candidate.
   - Recalculate score with fresh quota.
   - If depleted, in-use, or score drifted below threshold, demote to bottom and evaluate next best candidate.
