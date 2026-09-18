# Phase 2: Content Overlap Audit — Coding Guidelines

**Updated:** 2026-03-31
**Version:** 1.0.0
**Status:** Complete

---

## Overview

Audit of all 5 coding guideline sources + enum spec to identify unique content, duplicates, and contradictions.

---

## Source Summary

| Source | Total Files | Canonical? | Primary Value |
|--------|------------|------------|---------------|
| `01-pre-code-review-guides/` | 3 | No | Practical Go review notes, lazy evaluation, regex, newline styling, code mutation |
| `03-coding-guidelines-spec/` | 60+ | **Yes** | Formal spec with cross-language + 4 language-specific sections |
| `04-coding-guidelines-wpon/` | ~25 | No | WPOnboard format, PHP spacing/imports, response-key-type-inventory |
| `05-coding-guidelines-workflowy/` | 12 | No | TypeScript master guidelines, TS enum definitions |
| `06-enum-spec/` | 11 | No | Go enum remediation plans, migration checklists |

---

## Duplicate Content Map

Content that exists in multiple sources (03 is the canonical version):

| Topic | 01 | 03 | 04 | 05 | 06 |
|-------|----|----|----|----|-----|
| Boolean principles / no-negatives | ✅ (examples) | ✅ (formal) | ✅ (copy) | ✅ (copy) | — |
| Code style / formatting | — | ✅ (formal) | ✅ (copy) | ✅ (copy) | — |
| Cyclomatic complexity | — | ✅ (formal) | ✅ (copy) | ✅ (copy) | — |
| Database naming (PascalCase) | — | ✅ (formal) | ✅ (copy) | ✅ (copy) | — |
| DRY principles | — | ✅ (formal) | ✅ (copy) | ✅ (copy) | — |
| Function naming | — | ✅ (formal) | ✅ (copy) | ✅ (copy) | — |
| Strict typing / no-any | — | ✅ (formal) | ✅ (copy) | ✅ (copy) | — |
| Go enum pattern (byte, iota, Invalid) | — | ✅ (formal) | ✅ (copy) | — | ✅ (copy) |
| Go enum required methods | — | ✅ (formal) | ✅ (copy) | — | ✅ (copy) |
| Go enum folder structure | — | ✅ (formal) | ✅ (copy) | — | ✅ (copy) |
| Go enum validation checklist | — | ✅ (formal) | ✅ (copy) | — | ✅ (copy) |
| Go boolean standards | — | ✅ (formal) | ✅ (copy) | — | — |
| TS enum definitions (6 enums) | — | ✅ (formal) | — | ✅ (copy) | — |
| TS type safety remediation plan | — | ✅ (formal) | ✅ (copy) | ✅ (copy) | — |
| Master coding guidelines | — | ✅ (cross-lang, 997 lines) | ✅ (copy, shorter) | ✅ (TS-only, 294 lines) | — |

---

## Unique Content per Source (NOT in 03)

### 01-pre-code-review-guides — UNIQUE RULES

| # | Topic | File | Value |
|---|-------|------|-------|
| U1 | **Lazy evaluation pattern** | `03-golang-code-review-guides.md` | Full Go lazy evaluation with mutex lock, getter pattern, field dependency chain |
| U2 | **Regex usage guidelines** | `03-golang-code-review-guides.md` | When to/not to use regex, compile in `var`, never in loops |
| U3 | **Newline styling (detailed)** | `03-golang-code-review-guides.md` | 8+ before/after examples for newlines around braces, returns, blocks |
| U4 | **Code mutation avoidance** | `03-golang-code-review-guides.md` | Variable immutability rule, mutex lock patterns, constructor fix examples |
| U5 | **Null pointer safety** | `03-golang-code-review-guides.md` | Pointer return safety, array nil checks, error-before-value pattern |
| U6 | **Nesting resolution patterns** | `03-golang-code-review-guides.md` | 3 methods: extract to function, inverse logic, variable decomposition |
| U7 | **Go defer rule** | `03-golang-code-review-guides.md` | Max 1 defer per function, place at top or bottom |
| U8 | **Go string/slice internals** | `03-golang-code-review-guides.md` | StringHeader/SliceHeader struct details for understanding pass-by-value |
| U9 | **Code Red / Dangerous classification** | `03-golang-code-review-guides.md` | Severity taxonomy for code issues |
| U10 | **Promise/await anti-patterns** | `02-fe-guidelines.md` | Don't return promise then await; use PromiseAll |

### 04-coding-guidelines-wpon — UNIQUE CONTENT

| # | Topic | File | Value |
|---|-------|------|-------|
| U11 | **PHP spacing & import rules** | `07-php-standards/php-spacing-and-imports.md` | 4 PHP-specific rules: blank before if, blank before throw, no backslash imports, reusable log keys via enum |
| U12 | **ResponseKeyType inventory** | `07-php-standards/response-key-type-inventory.md` | 176-case enum inventory with usage counts across 100+ files |
| U13 | **PHP-Go consistency audit** | `07-php-standards/php-go-consistency-audit.md` | Cross-language consistency verification |

### 05-coding-guidelines-workflowy — UNIQUE CONTENT

| # | Topic | Value |
|---|-------|-------|
| U14 | **TS-focused master checklist** | Compact 294-line TS/React-specific enforcement checklist (vs 997-line cross-language in 03) |

**Note:** The WorkFlowy master guidelines (05) is a TS-only subset of the cross-language master (03). The 03 version is more complete. The 05 version adds no unique rules but has a slightly different presentation focused on React/TS developers.

### 06-enum-spec — UNIQUE CONTENT

| # | Topic | File | Value |
|---|-------|------|-------|
| U15 | **Naming convention remediation plan** | `96-naming-convention-remediation.md` | Systematic PascalCase fix plan for 25+ files across all CLI specs |
| U16 | **Acronym & JSON tag remediation** | `97-acronym-json-tag-remediation.md` | 50+ files fixed, JSON tag removal rules, acronym conversion (RAG→Rag) |
| U17 | **Spec migration checklist** | `98-migration-checklist.md` | Folder restructuring verification, old→new path mappings |

---

## Contradictions Found

| # | Topic | Source A | Source B | Resolution |
|---|-------|----------|----------|------------|
| C1 | **Function line limit** | 01: "8 lines ideal, 15 worst, 25 max (broken-fix-code)" | 03: "max 15 lines (error lines exempt)" | Use 03's rule (15 max, error lines exempt) — clearer |
| C2 | **File size limit** | 01: "struct/class/file ≤ 120 lines" | 03: "300 lines (Go hard limit 400)" | Use 03's rule (300) — 120 is too restrictive for real code |
| C3 | **Boolean prefixes** | 01: "Is/Has/Can/Should/Does" | 03: "is/has only" | Use 03's list (lowercase, `was` instead of `Does`) |
| C4 | **Max function params** | 01: "max 3, frameworks exempt" | 03: "max 3" (same + options object pattern) | Aligned — 03 adds the options object solution |
| C5 | **Go getter naming** | 01: "Field() not GetField()" | 03: not explicitly stated | Merge 01's explicit getter rule into 03 |
| C6 | **Newline rules** | 01: very detailed (8+ examples) | 03: R4/R5/R10/R12/R13 (rule-based) | Merge 01's examples into 03's rules |
| C7 | **Master guideline version** | 03: v1.2.0 (cross-language, 997 lines) | 05: v3.0.0 (TS-only, 294 lines) | Confusing versions — 05 has higher version but less content |

---

## Broken Cross-References

| # | File | Reference | Issue |
|---|------|-----------|-------|
| B1 | `03-coding-guidelines-spec/00-overview.md` L59 | `../01-general-spec/00-overview.md` | Path doesn't exist under `02-coding-guidelines/` |
| B2 | `03-coding-guidelines-spec/00-overview.md` L60 | `../18-error-resolution/00-overview.md` | Path doesn't exist under `02-coding-guidelines/` |
| B3 | `03-coding-guidelines-spec/00-overview.md` L61 | `../31-generic-enforce/00-overview.md` | Path doesn't exist under `02-coding-guidelines/` |
| B4 | `04-coding-guidelines-wpon/07-php-standards/php-spacing-and-imports.md` L192 | `../02-coding-guidelines/code-style.md` | Should be `../04-coding-guidelines/code-style.md` |

---

## Consolidation Recommendations

### Keep in Consolidated (from 03)

- All 60+ files — this is the canonical source

### Merge into 03 (unique from other sources)

- **U1-U9** from 01: Go-specific practical rules (lazy eval, regex, newlines, mutation, null safety, nesting, defer, internals, severity)
- **U10** from 01: Promise/await anti-patterns → into TS standards
- **U11** from 04: PHP spacing/imports → into `04-php/` section
- **U12** from 04: ResponseKeyType inventory → into `04-php/` as reference
- **U13** from 04: PHP-Go consistency audit → into cross-language

### Archive (duplicates)

- `04-coding-guidelines-wpon/04-coding-guidelines/` — full duplicate of 03's cross-language
- `04-coding-guidelines-wpon/05-golang-standards/` — duplicate of 03's Go section
- `04-coding-guidelines-wpon/06-golang-standards/01-enum-specification/` — duplicate of 03's enum spec
- `05-coding-guidelines-workflowy/` files 02-09 — duplicates of 03's cross-language
- `05-coding-guidelines-workflowy/10-typescript-enums/` — duplicate of 03's TypeScript section

### Keep as Archive Reference (remediation history)

- `06-enum-spec/96-naming-convention-remediation.md` — historical remediation tracking
- `06-enum-spec/97-acronym-json-tag-remediation.md` — completed remediation record
- `06-enum-spec/98-migration-checklist.md` — migration verification record

---

## Cross-References

| Reference | Location |
|-----------|----------|
| Consolidation Plan | `../planned/coding-guidelines-consolidation-29-plan.md` |
| Naming Compliance | `../project/naming-compliance-issues.md` |
