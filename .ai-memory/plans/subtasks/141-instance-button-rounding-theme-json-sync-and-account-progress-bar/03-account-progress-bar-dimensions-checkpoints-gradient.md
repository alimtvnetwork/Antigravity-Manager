# Subtask 03: Account Progress Bar Dimensions, Checkpoints & Gradient

## Objective
Update `src/components/accounts/QuotaProgressBar.tsx` to:
1. Reduce width by 18% (`w-[82%] max-w-[82%]`).
2. Increase height to `h-3.5`.
3. Add 11 checkpoints every 10 points: `[100, 90, 80, 70, 60, 50, 40, 30, 20, 10, 0]`.
4. Implement smooth left-to-right color blend from warning red to prominent vibrant green (`from-rose-500 via-amber-400 via-emerald-400 to-[#1af18d]`).

## Target Files
- `src/components/accounts/QuotaProgressBar.tsx`

## Verification
- Outer container width reduced by 18%.
- Height is `h-3.5` with proportional checkpoint nodes.
- 11 checkpoints are rendered.
- Color gradient flows from red to vibrant green.
