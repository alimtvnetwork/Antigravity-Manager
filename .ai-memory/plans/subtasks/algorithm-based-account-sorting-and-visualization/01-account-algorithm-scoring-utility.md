# Subtask 01: Account Algorithm Scoring Utility & Math Engine

## Objective
Author `src/utils/accountAlgorithmScore.ts` to compute:
1. Normalized candidate score ($0.0000$ to $0.4000+$).
2. Readiness percentage ($0\%$ to $100\%$) for progress bar fill.
3. Color ramping:
   - High ($\ge 70\%$): emerald-500 / green
   - Mid ($40\% - 69\%$): amber-500 / yellow
   - Low ($< 40\%$): rose-500 / red
4. Mathematical formula breakdown:
   - Exact values for $S_{\text{active}}$, $M_{\text{tier}}$, $Q_{\text{weekly}}$, $Q_{\text{4h}}$
   - Step-by-step arithmetic equations:
     - Multiplication: $S_{\text{active}} \times M_{\text{tier}} \times Q_{\text{weekly}} = \text{Raw}$
     - Division: $\text{Raw} \div 1000 = \text{Normalized}$
     - Percentage derivation: $(\text{Normalized} \div \text{MaxTierScore}) \times 100 = \text{Percentage}\%$
   - Disqualification reasons when $S_{\text{active}} = 0$ (In Use, Disabled, 403 Forbidden, 429 Rate Limited).

## Target Files
- `src/utils/accountAlgorithmScore.ts`
- `src/utils/accountAlgorithmScore.test.ts`

## Acceptance Criteria
- Pure, deterministic, zero-allocation calculations.
- Comprehensive unit tests covering Ultra, Pro, Free tiers, in-use accounts, 4h sub-100% fallback, and reset-passed accounts.
