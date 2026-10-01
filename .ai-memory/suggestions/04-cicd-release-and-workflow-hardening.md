# Suggestion 04: CI/CD, Release, and Workflow Hardening

Raised: 2026-10-01
Source: CI/CD recovery session (RCA 40, RCA 41, issue 56), installer RCA 42
Plan: [plans/pending/91-cicd-workflow-deadlines-and-release-ci-gate.md](../plans/pending/91-cicd-workflow-deadlines-and-release-ci-gate.md)

Each item names the failure it prevents. Ordered by risk.

## 1. Pin the Linux runner before 2026-10-19 (deadline)
- **Why:** `ubuntu-latest` moves to Ubuntu 26 on 2026-10-19. The Linux Tauri build installs WebKitGTK and `-dev` packages by name; a rename breaks `Build Tauri App (ubuntu-latest)` and the release Linux job.
- **Do:** set `runs-on: ubuntu-24.04` wherever `ubuntu-latest` appears (`ci.yml`, `release.yml`, `deploy-pages.yml`, `purge-actions-artifacts.yml`, plus `ubuntu-latest` in the build matrices), then test Ubuntu 26 in a separate matrix entry before moving.
- **Credential:** needs the `workflow` scope (granted 2026-10-01; see issue 56).

## 2. Move actions to Node 24-native majors
- **Why:** `actions/checkout@v4` and `actions/setup-node@v4` target Node 20 and run only because of `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24`. Once GitHub removes the override, every job fails at checkout.
- **Do:** bump to the first Node 24 majors; remove the override env var once all actions are native.

## 3. Gate releases on the whole CI result, not only rustfmt
- **Why:** `release.yml` → `verify-release-target` now re-runs `cargo fmt -- --check` (`6b18ddc4`). Clippy errors, compile errors, and frontend build failures are still not re-checked at tag time. `v4.109.1`–`v4.109.4` shipped while CI was red because each push cancelled the in-flight run.
- **Do:** in `verify-release-target`, query the CI run for `github.sha` (`gh run list --commit $SHA --workflow ci.yml --json conclusion`) and fail unless it is `success`. Treat `cancelled` and missing runs as failures (fall back to running the checks inline).

## 4. Stop concurrency cancellation from hiding red builds on `main`
- **Why:** `concurrency: cancel-in-progress` on `main` turns every superseded run into `cancelled`, so a red commit followed by a quick push is never seen.
- **Do:** keep `cancel-in-progress` for PR branches only; on `main`, let runs finish (`cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`).

## 5. Execute tests in CI, at least the fast ones
- **Why:** CI compiles test targets but never runs them, so logic regressions pass CI. AGENTS.md accepts this for speed, but a small subset costs little.
- **Do:** add one Linux job that runs `cargo test --lib` with a time budget, and `npm run test` for the frontend.

## 6. Make chosen Clippy lints deny-level
- **Why:** the crate has no `#![deny]`, no `[lints]` table, and CI passes no `-D warnings`, so Clippy never fails CI. Local Clippy runs out of memory on the 8 GB Windows host, so CI is the only place it runs.
- **Do:** add a `[lints.clippy]` table in `src-tauri/Cargo.toml` with a short deny list (`correctness`, `suspicious`), and widen it gradually.

## 7. Release asset completeness gate
- **Why:** `v4.109.2`–`v4.109.4` published without Windows/Linux assets (RCA 42). The installers now skip partial releases, but the release should not be marked `latest` in the first place.
- **Do:** after all build jobs, assert each platform asset exists before `makeLatest: true`. Tracked in plan 86.

## 8. Stop tracking the per-run cache `.ai-memory/temp/recent-file-changes.json`
- **Why:** it is rewritten by `03-ai-scripts/33-test-inventory-generator.py --record` on every run, shows as modified in every working tree, and has been committed on its own at least 18 times (for example `047fb296`). It blocks clean rebases and adds noise to every diff.
- **Do:** `git rm --cached` it and add it to `.gitignore`, or move it under the gitignored `.ai-memory/temp-agents/`.

## 9. Keep `gh` pointed at the fork
- **Why:** `gh` in this clone defaults to `upstream` (`lbjlaq/Antigravity-Manager`), so `gh run list` reads the wrong repo's runs and reports a false green.
- **Do:** `gh repo set-default alimtvnetwork/Antigravity-Manager` once per clone, and keep `-R alimtvnetwork/Antigravity-Manager` in all scripts.
