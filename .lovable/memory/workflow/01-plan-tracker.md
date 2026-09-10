# Plan Tracker

**Updated:** 2026-04-24
**Version:** 3.6.0

> Mirrors `.lovable/29-plan.md`. The canonical roadmap lives there.

---

## Completed Plans (recent)

| # | Task | Date |
|---|------|------|
| 23 | Created write-memory prompt v3.3 | 2026-04-16 |
| 24 | FAQ features in code (suppression parsing, baseline flags, TOML) | 2026-04-19 |
| 25 | STYLE-099 SuppressionWithoutReason synthetic finding | 2026-04-19 |
| 26 | Created `99-troubleshooting.md` (CICD) | 2026-04-19 |
| 27 | Performance impl: middle-out walker, `--jobs`, `--check-timeout`, TOOL-TIMEOUT (v3.12.0) | 2026-04-19 |
| 28 | `--version` flag on every check script (v3.13.0) | 2026-04-19 |
| 29 | `02-naming-conventions.md` v3.3.0 — Rule 2 clarification + Rule 9 | 2026-04-19 |
| 30 | `03-schema-design.md` v3.3.0 — §6 Mandatory Descriptive Columns | 2026-04-19 |
| 31 | BOOL-NEG-001 linter (v3.14.0) | 2026-04-19 |
| 32 | Inverted-field codegen tool (Go + PHP + TS) | 2026-04-19 |
| 33 | Cross-linked Rule 9 from boolean-principles + no-negatives (v2.2.0) | 2026-04-19 |
| 34 | `02-naming-conventions.md` v3.4.0 — Rule 8 three-bucket table | 2026-04-19 |
| 35 | `02-naming-conventions.md` v3.5.0 — Rules 10/11/12 (Description/Notes/Comments) | 2026-04-19 |
| 36 | Restructured `.lovable/` to single-file convention; write-memory prompt v1.0.0 | 2026-04-19 |
| 37 | Slug rebrand to `coding-guidelines-v24` (full repo sweep) | 2026-04-24 |
| 38 | Release & Migration UI rewrite (2 cards, no skip-probe variants) — locked | 2026-04-24 |
| 39 | Batch close v4.24.0 — B10 `--strict`, B11 `--split-by severity`, B8 `--total-timeout`, B7 PHP plugins, B2 Playwright spec, 09+10 offline E2E, B6 consistency dates, B5 effective-score waiver, 12 schema §6 alignment | 2026-04-24 |
| 40 | Codegen CI step + BOOL-NEG-001 pipeline smoke + orchestrator flags E2E (closes plan items #05, #01, #11) | 2026-04-25 |

(For dates 2026-04-02 → 2026-04-16, see `.lovable/29-plan.md` Completed Plans Historical.)

---

## Pending Plans

| # | Task | Priority | Notes |
|---|------|----------|-------|
| 01 | ~~Smoke-test BOOL-NEG-001 in `run-all.sh`~~ | — | ✅ Done v4.25.0 — `tests/pipeline/check-bool-neg-001-pipeline.sh` |
| 02 | Go-aware BOOL-NEG-001 variant | Medium | `embed.FS` SQL |
| 03 | Unit tests for BOOL-NEG-001 | Medium | |
| 04 | Round-trip tests for codegen inversion table | Medium | |
| 05 | ~~Wire codegen into CI (`git diff --exit-code`)~~ | — | ✅ Done — CI step "Verify codegen determinism" (.github/workflows/ci.yml) |
| 06 | Linter for missing `Description`/`Notes`/`Comments` | Medium | Rules 10–12 |
| 07 | Strengthen BOOL-NEG-001 with replacement hints | Low | |
| 08 | Cross-link link-checker over `spec/` | Low | |
| 09 | Mobile responsiveness | Low | |
| 10 | Author 8 real Mermaid diagrams (13-generic-cli + 14-update) | Medium | Placeholders only |
| 11 | ~~Real-repo run of orchestrator with `--strict --total-timeout --split-by severity`~~ | — | ✅ Done v4.25.0 — `tests/pipeline/check-orchestrator-flags.sh` |
| 12 | Wire Playwright `landing.spec.ts` into CI | Low | Spec exists |
| 13 | Publish click (button on Lovable UI) | Medium | App ready |

---

*Plan tracker — v3.7.0 — 2026-04-25*
