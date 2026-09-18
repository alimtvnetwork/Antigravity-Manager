# Naming & Reference Compliance Issues

**Updated:** 2026-03-31
**Version:** 2.0.0
**Status:** ✅ All Resolved

---

## Overview

All naming convention violations and cross-reference path issues have been resolved across all 6 consolidation phases.

---

## File Naming Violations — ✅ FIXED (Phase 1)

| # | Original | Renamed To |
|---|----------|-----------|
| 1 | `Conditional Guidelines.md` | `01-conditional-guidelines.md` |
| 2 | `FE Guidelines.md` | `02-fe-guidelines.md` |
| 3 | `Golang Code Review Guides.md` | `03-golang-code-review-guides.md` |

## Folder Naming Violations — ✅ FIXED (Phase 1)

| # | Issue | Fix Applied |
|---|-------|------------|
| 1 | Duplicate prefix `05-` in `04-coding-guidelines-wpon/` | Renamed `05-typescript-standards/` → `08-typescript-standards/` |

## Missing Required Files — ✅ CREATED (Phase 1)

7 overview and consistency report files created for legacy folders.

## Cross-Reference Issues — ✅ FIXED (Phase 5 + Post-QA)

| Phase | Action |
|-------|--------|
| Phase 4 | Fixed 9 internal broken refs in PHP audit file |
| Phase 5 | Verified 0 internal broken refs in consolidated spec |
| Post-QA | Converted 29 external refs (to specs outside module) to plain text with `<!-- external -->` comments |

**Result:** Zero broken markdown links across all 85 files.

---

## Cross-References

| Reference | Location |
|-----------|----------|
| Consolidation Plan | `../done/coding-guidelines-consolidation-29-plan.md` |
