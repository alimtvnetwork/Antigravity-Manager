# Architecture Spec: Weekly Quota Scoring Algorithm Revision & Accounts UI Refresh

**Slug:** `137-weekly-quota-scoring-algorithm-and-accounts-ui`
**Version:** v4.158.0 (target)
**Status:** IN PROGRESS

---

## User Request (Verbatim)

> "There are several issues in the IDE. First one is that when we are in the accounts mode, the color of the UI needs to be changed. Correct that. There is a logic we need to correct. Previously, we were multiplying the current four-hour quota with the weekly quota. You should have multiplied the weekly quota as well to two or something like this. Whoever has the highest value would usually win. Now, we have to do the calculation in a different way. Rather than just putting the value calculation with the weekly numbers, we are going to do one more calculation from now on. The calculation would be total week's hours. It would be seven into 24, minus how many hours are actually remaining. If something has six days remaining, it would be seven days minus six days, 24 hours. We will get a number of 24, and we will multiply that 24 with that weekly number as well. Not the into two. We will do this calculation, like how many days are remaining. We will first put seven days into 24 hours, then minus whatever the remaining number into 24. That would give us the distance that we have. Using that hour, we will multiply the weekly quota, and then we multiply with the previous algorithm remains as it is, just the weekly quota calculation will be changed. Based on that, confirm your understanding, show your calculation, then update the spec, and start with this algorithm. Whoever has the shortest time would go into the first part. Not who has the highest time or highest credits. If any weekly quota is less than 8%, it would be considered as zero. Weekly quota less than 8% considered to be zero. That's for the Gemini calculation. We can also switch to Claude calculation in the future, but not now. You can put an ambiguity and question mark for the Claude code. We need to work on it. Update the algorithm regarding this. Also make the progress bar a bit compact and reduce width to 18% on both sides please."

---

## Algorithm Confirmation & Worked Examples

### Previous Formula (Current Code — lines 702–737 in `auto_switcher.rs`)

```
score = (tier_multiplier × weekly_quota_percent) / 1000.0
```

Weekly quota is taken directly as a percentage (0–100). Multiplied by tier. Divided by 1000.

### New Formula — User Specification

**Step 1: Compute hours elapsed this week**
```
TOTAL_WEEK_HOURS = 7 × 24 = 168

hours_remaining = hours until weekly quota reset
                = parse_reset_time_to_unix(reset_time) - now_sec
                  (converted to hours)

hours_elapsed = TOTAL_WEEK_HOURS - hours_remaining
```

**Step 2: Apply 8% floor (Gemini only)**
```
effective_weekly_pct = if weekly_quota_pct < 8.0 then 0.0 else weekly_quota_pct
```

**Step 3: Compute weekly effective score**
```
weekly_effective_score = effective_weekly_pct × hours_elapsed
```

**Step 4: Combine with tier multiplier (replacing old formula)**
```
score = (tier_multiplier × weekly_effective_score) / (168.0 × 100.0)
      = (tier_multiplier × effective_weekly_pct × hours_elapsed) / 16800.0
```

The division by `16800.0` (= 168h × 100%) normalizes the score to roughly the same 0.0–1.0+ range as before.

**Sorting: ASCENDING by score** — "whoever has the SHORTEST time goes first."

An account with fewer `hours_elapsed` (reset just happened, weekly quota is fresh) has a LOWER `weekly_effective_score`. Lower score = sorted first = selected first. This prioritizes accounts that recently reset their weekly quota and still have maximum quota available.

### Worked Examples

| Account | Weekly Quota | Days Remaining | hours_remaining | hours_elapsed | effective_weekly_pct | weekly_effective_score | Tier | Score | Priority |
|---------|-------------|----------------|-----------------|---------------|----------------------|----------------------|------|-------|----------|
| A | 86% | 6d 17h = 161h | 161 | 168-161 = **7** | 86 | 86 × 7 = **602** | PRO (3×) | 602×3/16800 ≈ **0.107** | **1st** |
| B | 57% | 4d 23h = 119h | 119 | 168-119 = **49** | 57 | 57 × 49 = **2,793** | PRO (3×) | 2793×3/16800 ≈ **0.499** | 4th |
| C | 100% | 2d 14h = 62h | 62 | 168-62 = **106** | 100 | 100 × 106 = **10,600** | PRO (3×) | 10600×3/16800 ≈ **1.893** | Last |
| D | 6% | 6d 1h = 145h | 145 | 168-145 = **23** | **0** (< 8%) | 0 × 23 = **0** | PRO (3×) | 0/16800 = **0.0** | Skip (zero) |

Account D (6% weekly) → treated as 0 → excluded from pool.
Account A (86% weekly, just reset 7h ago) → goes first.

### Claude Ambiguity Note ❓

The `3p-weekly` bucket maps to Claude weekly quota. The `bucket_id.contains("3p")` check in `auto_switcher.rs` identifies Claude/GPT buckets. The same `hours_elapsed × weekly_pct` formula _could_ apply to Claude quota, but:

- Anthropic quota windows may differ from Google's 7-day cycle
- Claude bucket window identification may need separate handling
- **TODO/Ambiguity**: Apply Gemini-only (`gemini-weekly` bucket) for now; Claude scoring algorithm is undefined and should be stubbed with `// TODO(claude): Define Claude weekly scoring algorithm`

---

## Task Decomposition

| ID | Task | Target Files | Worker |
|----|------|-------------|--------|
| Task-01 | New weekly quota scoring algorithm in `score_candidate_account()` | `src-tauri/src/modules/auto_switcher.rs` | Worker 02 |
| Task-02 | Accounts UI color change + progress bar width compact (18% both sides) | `src/pages/Accounts.tsx`, `src/components/accounts/AccountTable.tsx` | Worker 01 |
| Task-03 (Carry-over from 136) | Restore neon green progress bars, modal header seq/profile/exe, de-green instance buttons | `src/components/accounts/QuotaProgressBar.tsx`, `src/components/common/WaterDrainProgressBar.tsx`, `src/components/instances/PromptTreeViewModal.tsx`, `src/pages/Instances.tsx` | Worker 01 |

---

## Implementation Spec for `score_candidate_account()` (lines 668–738)

### Weekly Score Extraction Helper

Add a private helper function:

```rust
/// Compute hours elapsed since the weekly quota cycle started.
/// Returns `hours_elapsed` as f64, or None if reset_time cannot be parsed.
fn compute_weekly_hours_elapsed(bucket: &crate::models::quota::QuotaBucket, now_sec: i64) -> Option<f64> {
    let reset_ts = parse_reset_time_to_unix(&bucket.reset_time)?;
    let remaining_secs = reset_ts.saturating_sub(now_sec).max(0);
    let remaining_hours = remaining_secs as f64 / 3600.0;
    const TOTAL_WEEK_HOURS: f64 = 168.0; // 7 × 24
    let elapsed = (TOTAL_WEEK_HOURS - remaining_hours).max(0.0).min(TOTAL_WEEK_HOURS);
    Some(elapsed)
}
```

### Revised `score_candidate_account()` Weekly Section (replace lines 701–737)

```rust
    // 2. Weekly quota: hours-elapsed-weighted scoring (Gemini buckets only)
    // Algorithm: weekly_effective_score = weekly_quota_pct × hours_elapsed_this_week
    // "Whoever has the shortest time goes first" → sort ASCENDING → lower score = prioritized
    // Weekly quota < 8% → treated as 0 (below viable threshold).
    // TODO(claude): Claude/3p weekly scoring algorithm undefined — skip 3p buckets for now.
    const TOTAL_WEEK_HOURS: f64 = 168.0;
    const WEEKLY_ZERO_THRESHOLD: f64 = 8.0;
    const SCORE_NORMALIZER: f64 = TOTAL_WEEK_HOURS * 100.0; // 16800.0

    let mut weekly_effective_score = TOTAL_WEEK_HOURS * 100.0; // Default: maximum (worst = prioritized last)

    if let Some(quota_data) = acc.quota.as_ref() {
        let mut gemini_weekly_scores: Vec<f64> = Vec::new();

        if let Some(ref groups) = quota_data.quota_groups {
            for g in groups {
                for b in &g.buckets {
                    let win = b.window.to_lowercase();
                    let bid = b.bucket_id.to_lowercase();
                    let is_gemini_weekly = (win.contains("week") || bid.contains("week"))
                        && !bid.contains("3p")    // TODO(claude): exclude 3p/Claude buckets
                        && !bid.contains("claude");
                    if is_gemini_weekly && (0.0..=1.0).contains(&b.remaining_fraction) {
                        let pct = (b.remaining_fraction * 100.0).round();
                        let eff_pct = if pct < WEEKLY_ZERO_THRESHOLD { 0.0 } else { pct };
                        let hours_elapsed = compute_weekly_hours_elapsed(b, now_sec)
                            .unwrap_or(TOTAL_WEEK_HOURS);
                        gemini_weekly_scores.push(eff_pct * hours_elapsed);
                    }
                }
            }
        }

        if !gemini_weekly_scores.is_empty() {
            // Use minimum bottleneck across multiple Gemini weekly buckets
            weekly_effective_score = gemini_weekly_scores.into_iter().fold(f64::INFINITY, f64::min);
        } else {
            // Fallback: legacy model percentage × half-week elapsed (assume mid-cycle)
            let valid_models: Vec<_> = quota_data.models.iter()
                .filter(|m| {
                    let n = m.name.to_lowercase();
                    !n.contains("claude") && !n.contains("3p")
                })
                .collect();
            if !valid_models.is_empty() {
                let sum: i32 = valid_models.iter().map(|m| m.percentage).sum();
                let avg_pct = (sum as f64 / valid_models.len() as f64).round();
                let eff_pct = if avg_pct < WEEKLY_ZERO_THRESHOLD { 0.0 } else { avg_pct };
                weekly_effective_score = eff_pct * (TOTAL_WEEK_HOURS / 2.0); // assume mid-cycle
            }
        }
    }

    // 3. Reset boundary: if period has finished, will refresh to 100% → assign score = 0 (best priority)
    if is_period_finished {
        weekly_effective_score = 0.0;
    }

    let active_factor = 1.0;
    (active_factor * tier_multiplier * weekly_effective_score) / SCORE_NORMALIZER
```

---

## UI Spec: Accounts Mode Color + Progress Bar Width

### Progress Bar Compact Width
- Currently the 4H quota column occupies too much of the row.
- Reduce progress bar container max-width (both left sidebar label and right time/percent side) from current to **18% each**.
- Specifically in `AccountTable.tsx` or `AccountRow.tsx`: find `min-w-[*]` and `max-w-[*]` on the label and time columns, reduce to `18%` or `w-[18%]`.

### Accounts Mode UI Color
- From the screenshot: The accounts table has teal/cyan progress bars + dark background.
- The neon green `#1af18d` restore (from task 136) applies to the 4H quota bar.
- The weekly quota bar should remain red/orange/amber when low.
- Header row background: change from `bg-slate-900` heavy to `bg-[#040d14] dark:bg-[#040d14]` for a deeper, sharper dark.
- Table row alternating backgrounds: reduce opacity/contrast for a cleaner look.
