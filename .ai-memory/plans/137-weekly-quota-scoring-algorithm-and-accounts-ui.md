# Master Plan: 137-weekly-quota-scoring-algorithm-and-accounts-ui

**Status:** IN PROGRESS
**Target Version:** v4.158.0

## Tasks

- [ ] Task-01: Replace weekly quota scoring in `score_candidate_account()` with hours-elapsed algorithm
- [ ] Task-02: Accounts UI color fix + progress bar 18% width compact
- [ ] Task-03 (carry-over 136): Neon green bars, modal header, de-green buttons

## Algorithm Summary

```
hours_elapsed = 168h - hours_remaining_until_weekly_reset
weekly_effective_score = weekly_quota_pct × hours_elapsed
score = (tier_multiplier × weekly_effective_score) / 16800.0
```

Sort ASCENDING → lowest score = most recently reset weekly quota = selected first.
Weekly quota < 8% → treated as 0 (excluded from pool effectively).
Claude/3p buckets: TODO stub only.
