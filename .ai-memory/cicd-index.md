# CI/CD Issues — Index

**Version:** 1.2.0
**Updated:** 2026-10-01

---

## Purpose

Tracks every CI/CD validator finding (CODE-RED-*, STYLE-*) encountered during sessions, what fixed it, and what to avoid re-introducing. Source: `linters-cicd/run-all.sh` SARIF output.

---

## Issues

| # | Title | Status | Rules | Date |
|---|-------|--------|-------|------|
| 01 | [Quickstart docs example: boolean negatives + style violations](resolved-issues/02-readme-boolean-negatives-and-style.md) | ✅ Solved | CODE-RED-023, STYLE-001, STYLE-004 | 2026-04-23 |
| 02 | [InstallSection.tsx 337-line component split](resolved-issues/03-installsection-component-split.md) | ✅ Solved | STYLE-005 | 2026-04-23 |
| 03 | [fuzzyMatch.ts: function length + negative guards](resolved-issues/04-fuzzymatch-function-length-negative-guards.md) | ✅ Solved | CODE-RED-004, CODE-RED-012, CODE-RED-023 | 2026-04-23 |
| 04 | [useSearchKeyboard.ts raw `!` operator](resolved-issues/05-usesearchkeyboard-raw-not-operator.md) | ✅ Solved | CODE-RED-023 | 2026-04-23 |
| 05 | [Bulk STYLE-001 / STYLE-004 newline violations across markdown highlighter](resolved-issues/06-markdown-highlighter-newline-violations.md) | ✅ Solved | STYLE-001, STYLE-003, STYLE-004 | 2026-04-23 |
| 06 | [Version drift after `package.json` bump (forgot `npm run sync`)](resolved-issues/07-version-drift-after-package-bump.md) | ✅ Solved | version-drift | 2026-04-23 |
| 07 | [Cross-spec missing-file checker false-positives in `26-spec-outsides`](resolved-issues/08-cross-spec-missing-file-link-checker.md) | ✅ Solved | missing-file | 2026-04-23 |
| 29 | [TypeScript instanceService missing exports & rustfmt](cicd-issues/29-typescript-instance-exports-and-rustfmt-rca.md) | ✅ Solved | tsc, rustfmt | 2026-09-22 |
| 30 | [GUI config empty file parse failure (E9001) & self-healing](cicd-issues/30-gui-config-empty-file-parse-failure-and-self-healing-rca.md) | ✅ Solved | E9001, config | 2026-09-22 |
| 31 | [Fast-forward switch delegation & crash recovery](cicd-issues/31-fast-forward-switch-delegation-rca.md) | ✅ Solved | fast-forward, switch | 2026-09-22 |
| 32 | CI test data dir race & Windows entrypoint failure (RCA file was never committed; see [15](cicd-issues/15-test-isolation-data-dir-env-concurrency-rca.md) and [37](cicd-issues/37-windows-test-manifest-entrypoint-and-rate-limit-rca.md)) | ✅ Solved | concurrency, entrypoint | 2026-09-22 |
| 33 | [MSVC duplicate resource link failure (CVT1100) & release asset decoupling](cicd-issues/33-msvc-duplicate-manifest-and-release-assets-rca.md) | ✅ Solved | msvc, cvtres, release-assets | 2026-09-23 |
| 34 | [Rustfmt auto-switcher unit test argument list failure](cicd-issues/34-rustfmt-auto-switcher-unit-test-rca.md) | ✅ Solved | rustfmt, clippy, multi-os | 2026-09-23 |
| 35 | [macOS universal bundle missing auxiliary `agm` binary](cicd-issues/35-macos-universal-bundle-agm-missing-binary-rca.md) | ✅ Solved | release, macos, universal-bundle | 2026-09-23 |
| 36 | [Windows runner setup-node transient hang & runner concurrency](cicd-issues/36-ci-windows-setup-node-transient-hang-and-runner-concurrency-rca.md) | ✅ Solved | windows, setup-node, concurrency | 2026-09-23 |
| 37 | [Windows test manifest entrypoint & CI rate limit](cicd-issues/37-windows-test-manifest-entrypoint-and-rate-limit-rca.md) | ✅ Solved | windows, manifest, rate-limit | 2026-09-25 |
| 38 | [Auto-switcher `test_is_account_in_use_logic` macOS race](cicd-issues/38-auto-switcher-test-account-in-use-ci-race-rca.md) | ✅ Solved | test-isolation, macos | 2026-09-27 |
| 39 | [Release published with missing platform artifacts](cicd-issues/39-release-published-with-missing-artifacts-rca.md) | 🟡 Planned (plan 86) | release, artifacts | 2026-09-30 |
| 40 | [Recurring rustfmt drift & releases shipping on red CI](cicd-issues/40-recurring-rustfmt-drift-and-releases-shipping-on-red-ci-rca.md) | ✅ Resolved; pre-commit hook + `release.yml` fmt gate (`6b18ddc4`) | rustfmt, pre-commit-hook, release-gate, multi-os | 2026-10-01 |
| 41 | [Test inventory generator crashes on synced manifest schema](cicd-issues/41-test-inventory-generator-schema-mismatch-rca.md) | ✅ Solved | tooling, schema, record-step | 2026-10-01 |

Rows 01–07 above are the older `resolved-issues/` linter series and do not share numbering with `cicd-issues/01-07`. The complete `cicd-issues/` list (01–41) is in [cicd-issues/readme.md](cicd-issues/readme.md).

---

## Recurring Failure Classes (read before committing or tagging)

| Class | RCAs | Enforced guard | Rule |
|-------|------|----------------|------|
| Rustfmt drift | 02, 12, 16, 18, 20, 22, 27, 28, 29, 34, 40 | `.githooks/pre-commit` (via `npm install` / `npm run hooks:install`) | Never `--no-verify`; never put code edits in `npm run bump` commits |
| Test isolation / env concurrency | 01, 04, 06, 14, 15, 17, 38 | none (review only) | Isolated temp data dirs; serialize env mutation; no shared account pools |
| Release shipping on red CI / missing artifacts | 39, 40 | `release.yml` → `verify-release-target` fmt gate | Tag only after CI for that exact SHA is `success`; `cancelled` is not green |
| Hidden failures behind first red step | 40 | none | After a fix, confirm the whole run is green, not only the fixed step |

`gh` defaults to `upstream` (`lbjlaq/Antigravity-Manager`) in this clone. Always pass `-R alimtvnetwork/Antigravity-Manager` when reading CI.

---

## Aggregate stats (last clean run — 2026-04-23, v3.81.0)

- Violations: **0**
- Files scanned: 612
- Lines scanned: 131,966

---

*CI/CD index — v1.1.0 — 2026-04-23*
