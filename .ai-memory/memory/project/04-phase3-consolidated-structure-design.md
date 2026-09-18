# Phase 3: Consolidated Guideline Structure Design

**Updated:** 2026-03-31
**Version:** 1.0.0
**Status:** Complete

---

## Overview

Design for the consolidated AI-optimized coding guideline. Uses `03-coding-guidelines-spec/` as the base, merging unique content from all other sources, and adding AI-specific anti-hallucination and quick-reference sections.

---

## New Folder Structure

```
02-spec/02-coding-guidelines/
├── 00-overview.md                          # Updated with AI scoring, full module map
├── 01-cross-language/
│   ├── 00-overview.md
│   ├── 01-issues-and-fixes-log.md
│   ├── 02-boolean-principles.md
│   ├── 03-casting-elimination-patterns.md
│   ├── 04-code-style.md
│   ├── 05-cross-spec-contradiction-checks.md
│   ├── 06-cyclomatic-complexity.md
│   ├── 07-database-naming.md
│   ├── 08-dry-principles.md
│   ├── 09-dry-refactoring-summary.md
│   ├── 10-function-naming.md
│   ├── 11-key-naming-pascalcase.md
│   ├── 12-no-negatives.md
│   ├── 13-strict-typing.md
│   ├── 14-test-naming-and-03-structure.md
│   ├── 15-master-coding-guidelines.md       # EXISTING: 997-line cross-language master
│   ├── 16-lazy-evaluation-patterns.md       # NEW: from 01 (U1)
│   ├── 17-regex-usage-guidelines.md         # NEW: from 01 (U2)
│   ├── 18-code-mutation-avoidance.md        # NEW: from 01 (U4)
│   ├── 19-null-pointer-safety.md            # NEW: from 01 (U5)
│   ├── 20-nesting-resolution-patterns.md    # NEW: from 01 (U6)
│   ├── 21-newline-styling-examples.md       # NEW: from 01 (U3) — supplements 04-code-style.md
│   ├── 97-acceptance-criteria.md
│   ├── 98-changelog.md
│   └── 99-consistency-report.md
├── 02-typescript/
│   ├── (existing files unchanged)
│   └── 09-promise-await-patterns.md         # NEW: from 01 (U10)
├── 03-golang/
│   ├── (existing files unchanged)
│   ├── 05-defer-rules.md                    # NEW: from 01 (U7)
│   ├── 06-string-slice-internals.md         # NEW: from 01 (U8)
│   └── 07-code-severity-taxonomy.md         # NEW: from 01 (U9)
├── 04-php/
│   ├── (existing files unchanged)
│   ├── 08-spacing-and-imports.md            # NEW: from 04 (U11)
│   ├── 09-response-key-type-inventory.md    # NEW: from 04 (U12)
│   └── 10-php-go-consistency-audit.md       # NEW: from 04 (U13)
├── 05-rust/
│   └── (existing files unchanged)
├── 06-ai-optimization/                      # NEW SECTION
│   ├── 00-overview.md
│   ├── 01-anti-hallucination-rules.md
│   ├── 02-ai-quick-reference-checklist.md
│   ├── 03-common-ai-mistakes.md
│   └── 99-consistency-report.md
├── 97-acceptance-criteria.md
└── 99-consistency-report.md
```

---

## New File Designs

### 06-ai-optimization/01-anti-hallucination-rules.md

**Purpose:** Explicit rules that prevent AI from generating non-compliant code. Each rule has a forbidden pattern, the correct pattern, and the reasoning.

**Structure:**

```markdown

# Anti-Hallucination Rules

## How to Use

AI agents MUST check every code generation against these rules before output.

## Rule Categories

### Category 1: Naming Hallucinations

- AH-N1: Never generate `ID`, `URL`, `API` — always `Id`, `Url`, `Api`
- AH-N2: Never generate snake_case variables — always camelCase
- AH-N3: Never generate camelCase JSON keys — always PascalCase
- AH-N4: Never generate `GetField()` in Go — use `Field()` for getters
- AH-N5: Never generate `Unknown` as enum zero value — always `Invalid`

### Category 2: Type Safety Hallucinations

- AH-T1: Never generate `any` or `interface{}` in business logic
- AH-T2: Never generate `fmt.Errorf()` — use `apperror.Wrap()`
- AH-T3: Never generate multi-return Go functions — use `Result[T]`
- AH-T4: Never generate type assertions in business code — use concrete structs
- AH-T5: Never generate `(T, error)` return — use `apperror.Result[T]`

### Category 3: Boolean Hallucinations

- AH-B1: Never generate `!func()` — use semantic inverse (`isInvalid()`)
- AH-B2: Never generate booleans without `is`/`has`/`can`/`should`/`was` prefix
- AH-B3: Never generate negative names (`isNotReady`) — use positive (`isPending`)
- AH-B4: Never generate boolean function parameters — use options objects

### Category 4: Structure Hallucinations

- AH-S1: Never generate nested `if` — flatten with early returns
- AH-S2: Never generate functions >15 lines body
- AH-S3: Never generate functions with >3 parameters
- AH-S4: Never generate files >300 lines
- AH-S5: Never omit blank line before `return`/`throw` after statements

### Category 5: Error Handling Hallucinations

- AH-E1: Never access `.Value()` without prior `HasError()` check
- AH-E2: Never use raw `error` in Go struct fields — use `*AppError`
- AH-E3: Never swallow errors silently — always log, return, or propagate
- AH-E4: Never use `\Throwable` in PHP — use `use Throwable;` import

### Category 6: Enum Hallucinations

- AH-EN1: Never use `string` as Go enum type — use `byte`
- AH-EN2: Never compare enums with `===` in PHP — use `isEqual()`
- AH-EN3: Never use magic strings for status comparisons — use enum constants
- AH-EN4: Never generate explicit `json:` tags on Go structs — PascalCase is default

Each rule includes: ❌ Forbidden pattern | ✅ Required pattern | 📖 Why
```

### 06-ai-optimization/02-ai-quick-reference-checklist.md

**Purpose:** A condensed, machine-parsable checklist that AI can validate code against in <30 seconds.

**Structure:**

```markdown

# AI Quick Reference Checklist

## Pre-Output Validation (check every code block)

### Naming (5 checks)

- [ ] Variables: camelCase (`pluginSlug`, not `plugin_slug`)
- [ ] Types/Classes/Components: PascalCase (`SnapshotManager`)
- [ ] JSON/API keys: PascalCase (`"PluginSlug"`, not `"pluginSlug"`)
- [ ] Abbreviations: First-letter-only (`Id`, `Url`, `Api`, `Http`)
- [ ] Booleans: `is`/`has`/`can`/`should`/`was` prefix

### Structure (5 checks)

- [ ] Zero nested `if` — early return pattern
- [ ] Function body ≤15 lines (error handling exempt)
- [ ] Function params ≤3 (use options object for 4+)
- [ ] File size ≤300 lines
- [ ] Blank line before `return`/`throw` when preceded by statements

### Go-Specific (6 checks)

- [ ] Enums: `byte` type, `Invalid` zero value, `iota`
- [ ] Returns: single `Result[T]`, never `(T, error)`
- [ ] Errors: `apperror.Wrap()`, never `fmt.Errorf()`
- [ ] No `any`/`interface{}` in business logic
- [ ] No type assertions — use concrete structs
- [ ] No explicit `json:` tags (unless `omitempty` or `-`)

### PHP-Specific (4 checks)

- [ ] No `\Throwable` — use `use` import
- [ ] Enum comparison via `isEqual()`, not `===`
- [ ] Blank line before `if` when preceded by statements
- [ ] Log keys: camelCase; DB keys: PascalCase

### TypeScript-Specific (4 checks)

- [ ] No `any` — use explicit types
- [ ] Enum: PascalCase + `Type` suffix (`StatusType`)
- [ ] No magic strings — use enum constants
- [ ] `isDefined()` / `isDefinedAndValid()` — no raw null checks

### Error Handling (3 checks)

- [ ] `HasError()`/`hasError()` before `.Value()`/`.value()`
- [ ] No silent error swallowing
- [ ] Struct error fields: `*AppError` (Go), `Throwable` (PHP)
```

### 06-ai-optimization/03-common-ai-mistakes.md

**Purpose:** Real examples of mistakes AI commonly makes when generating code for this project, with corrections.

**Sections:**
1. Top 10 most common AI mistakes (with before/after)
2. Language-specific traps (Go: multi-return, PHP: backslash, TS: any)
3. Pattern recognition guide (how to identify when AI is about to hallucinate)

---

## Content Merge Plan (what goes where)

### From 01-pre-code-review-guides

| Unique Item | Target File | Action |
|-------------|-------------|--------|
| U1: Lazy evaluation | `01-cross-language/16-lazy-evaluation-patterns.md` | Extract from Go review, formalize |
| U2: Regex usage | `01-cross-language/17-regex-usage-guidelines.md` | Extract, add cross-language context |
| U3: Newline styling | `01-cross-language/21-newline-styling-examples.md` | Extract 8+ examples, link to 04-code-style |
| U4: Code mutation | `01-cross-language/18-code-mutation-avoidance.md` | Extract, add immutability rules |
| U5: Null pointer safety | `01-cross-language/19-null-pointer-safety.md` | Extract Go examples, add PHP/TS guards |
| U6: Nesting resolution | `01-cross-language/20-nesting-resolution-patterns.md` | Extract 3 methods with examples |
| U7: Go defer rule | `03-golang/05-defer-rules.md` | Extract, formalize |
| U8: String/slice internals | `03-golang/06-string-slice-internals.md` | Extract Go internals |
| U9: Code severity | `03-golang/07-code-severity-taxonomy.md` | Extract Code Red / Dangerous |
| U10: Promise/await | `02-typescript/09-promise-await-patterns.md` | Extract, add modern patterns |

### From 04-coding-guidelines-wpon

| Unique Item | Target File | Action |
|-------------|-------------|--------|
| U11: PHP spacing/imports | `04-php/08-spacing-and-imports.md` | Copy with minor formatting |
| U12: ResponseKeyType inventory | `04-php/09-response-key-type-inventory.md` | Copy as reference |
| U13: PHP-Go consistency audit | `04-php/10-php-go-consistency-audit.md` | Copy as reference |

### From 06-enum-spec

| Unique Item | Target File | Action |
|-------------|-------------|--------|
| U15-U17: Remediation plans | `99-archive/` | Keep as historical reference |

---

## Contradiction Resolutions

| # | Decision | Rationale |
|---|----------|-----------|
| C1 | Function max = 15 lines (error lines exempt) | 03's rule is clearer and more practical |
| C2 | File max = 300 lines (Go hard limit 400) | 120 lines (01) is too restrictive |
| C3 | Boolean prefixes: `is/has only` | 03's list is canonical |
| C5 | Go getters: `Field()` not `GetField()` | Merge into 03's Go section explicitly |
| C6 | Newline rules: Use 03's rule IDs + 01's examples | Best of both |
| C7 | Master guideline: 03 v1.2.0 is canonical, 05 v3.0.0 archived | 03 has more content |

---

## Cross-Reference Fixes

| # | File | Current | Fix |
|---|------|---------|-----|
| B1 | `03-coding-guidelines-spec/00-overview.md` L59 | `../01-general-spec/00-overview.md` | Remove or update to correct path (these are in spec root, not under 02-coding-guidelines) |
| B2 | `03-coding-guidelines-spec/00-overview.md` L60 | `../18-error-resolution/00-overview.md` | Same — fix to `../../03-error-manage-spec/` or remove |
| B3 | `03-coding-guidelines-spec/00-overview.md` L61 | `../31-generic-enforce/00-overview.md` | Remove — doesn't exist |
| B4 | `04-coding-guidelines-wpon/07-php-standards/php-spacing-and-imports.md` L192 | `../02-coding-guidelines/code-style.md` | Fix to `../04-coding-guidelines/code-style.md` |

---

## Cross-References

| Reference | Location |
|-----------|----------|
| Phase 2 Audit | `../project/phase2-content-overlap-audit.md` |
| Consolidation Plan | `../planned/coding-guidelines-consolidation-29-plan.md` |
| Naming Compliance | `../project/naming-compliance-issues.md` |
