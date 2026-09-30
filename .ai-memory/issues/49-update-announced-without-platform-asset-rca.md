# Issue 49: Update announced although the release has no binary

Status: open (planned, not fixed)
Raised: 2026-09-30
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Plan: [86](../plans/pending/86-update-integrity-asset-verification-and-release-gates.md)

## 1. Symptom
The app announces a new version; installing fails because the release assets are missing (CI was not green).

## 2. Trigger
A tag was pushed and a release page exists while one or more build legs failed.

## 3. Root cause
`update_checker.rs` decides `has_update` by version comparison only (updater.json, GitHub API, static fallback). Nothing verifies that an asset for this platform exists. `release.yml` guesses artifact names when writing `updater.json` and can publish with missing matrix output. Installers learn a release has no binary only after a failed download. All source failures collapse into one message mapped to default code `E9001`. One URL also hardcodes a non-origin owner.

## 4. Why it escaped
The happy path was tested; partial publish was never simulated. Spec 20 fixed installer fork inversion but not asset presence.

## 5. Fix (planned)
Resolver that walks releases newest to oldest and selects the first with a verified platform asset; aggregated source errors with a registered code; UI shows skipped versions; installers pre-check with JSON output; release gate and post-publish URL verification. See plan 86.

## 6. Prevention
Post-publish verification job; fixture-based resolver tests; installer `-CheckUpdate` contract.

## 7. Regression check
Resolver unit tests with fixtures: newest release without asset is skipped; all releases without asset gives `has_update = false` plus reasons.
