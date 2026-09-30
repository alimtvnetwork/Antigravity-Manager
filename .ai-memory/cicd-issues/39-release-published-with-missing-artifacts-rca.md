# Issue 39: Release published while platform artifacts are missing

Status: open (planned, not fixed)
Raised: 2026-09-30
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Plan: [86](../plans/pending/86-update-integrity-asset-verification-and-release-gates.md)

## 1. Symptom
A GitHub release is public and announced by the updater but lacks installers for one or more platforms.

## 2. Trigger
A matrix build leg fails or is cancelled while the publish job still runs, or `updater.json` is written from guessed file names.

## 3. Root cause
`release.yml` publish job does not require every expected artifact and builds `updater.json` URLs from names it assumes instead of names it uploaded.

## 4. Why it escaped
The publish job was treated as green if it ran; no step fetched the URLs afterwards.

## 5. Fix (planned)
Expected-artifact list defined once; publish fails if any is missing; `updater.json` generated from the uploaded asset list; post-publish job HEADs every URL and fails on non-2xx. See plan 86 subtasks 008-009.

## 6. Prevention
Workflow self-check step; local runner script can replay the check.

## 7. Regression check
Dry run the verification script against a known-good release and a doctored one.
