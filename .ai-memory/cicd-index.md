# CI/CD Issues — Index

**Version:** 1.1.0
**Updated:** 2026-04-23 (session 2)

---

## Purpose

Tracks every CI/CD validator finding (CODE-RED-*, STYLE-*) encountered during sessions, what fixed it, and what to avoid re-introducing. Source: `linters-cicd/run-all.sh` SARIF output.

---

## Issues

| # | Title | Status | Rules | Date |
|---|-------|--------|-------|------|
| 01 | [Quickstart docs example: boolean negatives + style violations](resolved-issues/01-readme-boolean-negatives-and-style.md) | ✅ Solved | CODE-RED-023, STYLE-001, STYLE-004 | 2026-04-23 |
| 02 | [InstallSection.tsx 337-line component split](resolved-issues/02-installsection-component-split.md) | ✅ Solved | STYLE-005 | 2026-04-23 |
| 03 | [fuzzyMatch.ts: function length + negative guards](resolved-issues/03-fuzzymatch-function-length-negative-guards.md) | ✅ Solved | CODE-RED-004, CODE-RED-012, CODE-RED-023 | 2026-04-23 |
| 04 | [useSearchKeyboard.ts raw `!` operator](resolved-issues/04-usesearchkeyboard-raw-not-operator.md) | ✅ Solved | CODE-RED-023 | 2026-04-23 |
| 05 | [Bulk STYLE-001 / STYLE-004 newline violations across markdown highlighter](resolved-issues/05-markdown-highlighter-newline-violations.md) | ✅ Solved | STYLE-001, STYLE-003, STYLE-004 | 2026-04-23 |
| 06 | [Version drift after `package.json` bump (forgot `npm run sync`)](resolved-issues/06-version-drift-after-package-bump.md) | ✅ Solved | version-drift | 2026-04-23 |
| 07 | [Cross-spec missing-file checker false-positives in `26-spec-outsides`](resolved-issues/07-cross-spec-missing-file-link-checker.md) | ✅ Solved | missing-file | 2026-04-23 |

---

## Aggregate stats (last clean run — 2026-04-23, v3.81.0)

- Violations: **0**
- Files scanned: 612
- Lines scanned: 131,966

---

*CI/CD index — v1.1.0 — 2026-04-23*
