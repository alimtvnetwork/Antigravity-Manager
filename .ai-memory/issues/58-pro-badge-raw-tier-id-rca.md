# Issue 58: A Pro account shows no Pro badge because the raw tier id is not a known label

Status: partial. Fetch-time normalization landed in v4.119.0. The cached-project skip in issue 47 is still open. `standard-tier` is intentionally not mapped to PRO.
Raised: 2026-10-02
Related: [47](./47-pro-badge-missing-when-tier-not-fetched-rca.md)
Spec: [02-spec/21-app/95-accounts-columns-reinject-and-zip-update.md](../../02-spec/21-app/95-accounts-columns-reinject-and-zip-update.md)

## 1. Symptom
An account that is Pro in the service shows no Pro badge in the accounts table. The UI draws the badge only when `subscription_tier` is a known label such as `PRO`, `ULTRA`, or `FREE`.

## 2. Trigger
A quota fetch that receives a Google tier id, or a later read of a tier string already stored on the account.

## 3. Root cause
Two separate gaps produce the same missing badge.

The gap this issue records: `loadCodeAssist` can return a paid tier whose id, name, slug, or quota tier is not the literal `PRO`. Before v4.119.0 that raw string was stored as-is. The badge compared it to the known labels and drew nothing. `standard-tier` is one such raw id. It was not relabeled PRO, because no captured payload has shown that id on an account already known to be Pro.

The gap that remains issue 47: `fetch_quota_with_cache` in `modules/quota.rs` still skips `loadCodeAssist` when `project_id` is already cached and returns `subscription_tier = None`. Normalization never runs on that path. A refresh of an account that already has a project id can wipe or skip the tier.

## 4. Why it escaped
The badge has no unknown state, so a raw id and a missing tier look the same as Free. Issue 47 described only the cache skip. A reader who fixed only that skip would still store `premium` or `advanced` and show no badge.

## 5. Fix
On the path that does call `loadCodeAssist`, `finalize_subscription_tier` runs `normalize_subscription_tier`. Ids that contain ultra or helium become `ULTRA`. Ids that contain free or starter become `FREE`. Ids that contain pro, premium, or advanced become `PRO`. A known label is stored. An unrecognized id is stored raw and logged as `Unrecognized subscription tier id '...'; not mapped to PRO`. `standard-tier` stays unrecognized. Rows already saved are not rewritten until the next fetch that actually calls `loadCodeAssist`.

## 6. Prevention
Do not map `standard-tier` to PRO until a captured `loadCodeAssist` payload shows that id on a known Pro account. Log the raw id. Keep issue 47 open until a cached `project_id` still returns a tier.

## 7. Regression check
`cargo test models::quota` covers `standard-tier` staying unrecognized. A Pro account whose last fetch skipped `loadCodeAssist` will still have no badge until issue 47 is fixed and that account is refreshed.
