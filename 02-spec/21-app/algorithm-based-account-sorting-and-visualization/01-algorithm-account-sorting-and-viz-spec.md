# Algorithm-Based Account Sorting and Visualization Specification

## 1. Overview & Operational Intent
Antigravity-Manager employs a multi-factor candidate scoring algorithm in both backend (`src-tauri/src/modules/auto_switcher/scoring.rs`) and frontend (`src/services/instanceService/scoring.ts`) to determine candidate account readiness during auto-profile switching, Smart Rotate, and Fast-Forward operations.

Previously, the Accounts table displayed a static `USED` date string (e.g. `18-Oct-26`) in Column 6. This column provided minimal operational value to operators. This specification defines replacing the `USED` column with an interactive **Algorithm Score & Health Progress Bar** that:
1. Calculates and displays the internal account algorithm score / readiness percentage ($0\% - 100\%$).
2. Renders a dynamic, color-ramped progress bar (emerald green for high scores, golden amber for moderate scores, and crimson red for low/penalized scores).
3. Enables bidirectional table sorting on this computed algorithm score (`desc` / `asc`).
4. Provides a rich, dark-glass calculation breakdown hover card detailing every mathematical step (status factor, tier multipliers, weekly quota bottleneck, 4-hour rolling window check, multiplication, division, and final normalized score).

---

## 2. Mathematical Scoring Model & Operations
The scoring engine evaluates each account based on the unified candidate selection algorithm:

### 2.1 Variables & Factors
- $S_{\text{active}} \in \{0, 1\}$: Active status factor. $1$ if idle and eligible; $0$ if currently bound/running in an active IDE instance, disabled, quota forbidden (403), validation blocked, or model rate-limited (429).
- $M_{\text{tier}} \in \mathbb{R}^+$: Subscription tier multiplier. Configurable in settings (`ultra = 4.0`, `pro = 2.0`, `free = 1.0`).
- $Q_{\text{weekly}} \in [0, 100]$: Minimum remaining weekly quota percentage across all quota groups.
- $Q_{\text{4h}} \in [0, 100]$: Minimum remaining 4-hour/5-hour rolling model quota percentage.
- $T_{\text{reset}}$: Timestamp of quota reset boundary. If $T_{\text{reset}} \le T_{\text{now}}$, quota is treated as refreshed ($100\%$).

### 2.2 Mathematical Equations
1. **Primary Pass (Full 4H Window Available)**:
   $$\text{RawScore} = S_{\text{active}} \times M_{\text{tier}} \times Q_{\text{weekly}}$$
   $$\text{NormalizedScore} = \frac{\text{RawScore}}{1000}$$
   $$\text{ReadinessPercentage} = \frac{Q_{\text{weekly}}}{100} \times 100\% = Q_{\text{weekly}}\%$$

2. **Sub-100% Fallback Pass (Proportional Scaling)**:
   When no account has $100\%$ in the 4-hour window, accounts scale proportionally:
   $$\text{RawScore}_{\text{fallback}} = S_{\text{active}} \times M_{\text{tier}} \times \left( \frac{Q_{\text{weekly}} \times Q_{\text{4h}}}{100} \right)$$
   $$\text{NormalizedScore}_{\text{fallback}} = \frac{\text{RawScore}_{\text{fallback}}}{1000}$$
   $$\text{ReadinessPercentage}_{\text{fallback}} = \frac{Q_{\text{weekly}} \times Q_{\text{4h}}}{100}\%$$

3. **Disqualified / In-Use Accounts**:
   $$\text{Score} = 0.0000, \quad \text{ReadinessPercentage} = 0\%$$
   Disqualification reason displayed explicitly (`In Use`, `Disabled`, `Forbidden 403`, `Rate Limited 429`).

---

## 3. UI Component Architecture

### 3.1 Column Replacement in `AccountTable.tsx`
- Replace `th` Column 6 (`USED ↑↓`) with `SCORE ↑↓` (or `ALGORITHM SCORE ↑↓`).
- Column width: `w-[110px]` with `min-w-[100px]`, perfectly fitting within the table layout without disrupting `th:nth-child(4)` and `th:nth-child(5)` 50/50 quota flex.
- Sort toggle: supports `null -> desc -> asc -> null` (defaulting to descending to prioritize top-performing accounts).

### 3.2 Cell Component `AlgorithmScoreCell.tsx`
- Replaces raw date span in `AccountRowContent.tsx`.
- Layout:
  - Top row: Score value (e.g. `85%` or `0.1700`) with high-contrast text badge.
  - Bottom row: Progress bar (`h-1.5` rounded-full track with gradient fill).
- Color Ramping:
  - Score $\ge 70\%$: Green (`from-emerald-500 to-teal-400`, `text-emerald-400`).
  - Score $40\% - 69\%$: Amber (`from-amber-500 to-yellow-400`, `text-amber-400`).
  - Score $< 40\%$: Red (`from-rose-600 to-red-500`, `text-rose-400`).

### 3.3 Interactive Mathematical Tooltip
- Dark-glass card rendered on hover:
  - Header: Account Tier (`PRO 2.0x`) and Algorithm Rank.
  - Formula Box: `Score = (S_active × M_tier × Q_weekly) / 1000`
  - Step 1: Active Eligibility ($S_{\text{active}} = 1$)
  - Step 2: Multiplication ($1 \times 2.0 \times 85 = 170.0$)
  - Step 3: Rolling Window Check ($Q_{4h} = 100\%$, Primary Eligible)
  - Step 4: Division & Normalization ($170.0 \div 1000 = 0.1700$)
  - Step 5: Readiness Percentage ($85.0\%$)

---

## 4. Verification & Testing Matrix
- Pure unit tests for `computeAccountAlgorithmScore` in `src/utils/accountAlgorithmScore.test.ts`.
- Verify sorting by score in `AccountTable.tsx`.
- Verify progress bar visual color ramp (>= 70% green, 40-69% amber, < 40% red).
- Verify tooltip rendering with full arithmetic operations.
- Verify drag-and-drop is paused during score sorting.
