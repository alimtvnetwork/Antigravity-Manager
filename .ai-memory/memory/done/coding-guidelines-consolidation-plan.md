# Plan: Consolidate Coding Guidelines into AI-Optimized Master Guideline

**Updated:** 2026-03-31
**Version:** 1.0.0
**Status:** Planned

---

## Overview

The `02-spec/02-coding-guidelines/` folder contains **5 overlapping source folders** plus an enum spec, each representing different iterations/formats of the same coding standards. The goal is to consolidate these into a single, authoritative, AI-optimized coding guideline that prevents hallucination and gives AI agents everything needed to write best-practice code.

---

## Current State Analysis

### Source Folders (5 + enum)

| # | Folder | Format | Content Summary |
|---|--------|--------|-----------------|
| 01 | `01-pre-code-review-guides/` | Raw review notes | 3 files: Conditionals, FE (promise/await), Golang review guides. Rich practical examples but informal, has naming violations |
| 03 | `03-coding-guidelines-spec/` | Formal spec (canonical) | 5 sub-categories (cross-language, TS, Go, PHP, Rust) with 60+ files. Well-structured, has overviews, acceptance criteria, consistency reports |
| 04 | `04-coding-guidelines-wpon/` | WPOnboard format | 5 sub-folders with mixed naming. Duplicate content with 03, some unique PHP patterns |
| 05 | `05-coding-guidelines-workflowy/` | WorkFlowy export | 12 files + TS enums sub-folder. Master guidelines document is comprehensive with practical rules |
| 06 | `06-enum-spec/` | Enum specification | Detailed Go enum patterns, migration checklists, remediation plans |

### Key Observations

1. **`03-coding-guidelines-spec/`** is the most complete and well-structured — should be the base
2. **`05-coding-guidelines-workflowy/`** has the best practical master guidelines (TypeScript-focused)
3. **`01-pre-code-review-guides/`** has unique practical examples (lazy evaluation, regex, newline styling, Go review)
4. **`04-coding-guidelines-wpon/`** largely duplicates 03 but has some unique PHP content
5. **`06-enum-spec/`** has detailed Go enum patterns not fully captured in 03

---

## Compliance Issues Found

### Naming Convention Violations

| Issue | Location | Fix |
|-------|----------|-----|
| PascalCase + spaces in filenames | `01-pre-code-review-guides/FE Guidelines.md` | Rename to `02-fe-guidelines.md` |
| PascalCase + spaces in filenames | `01-pre-code-review-guides/Conditional Guidelines.md` | Rename to `01-conditional-guidelines.md` |
| PascalCase + spaces in filenames | `01-pre-code-review-guides/Golang Code Review Guides.md` | Rename to `03-golang-code-review-guides.md` |
| Duplicate numeric prefix `05-` | `04-coding-guidelines-wpon/05-golang-standards/` and `05-typescript-standards/` | Rename one |
| Missing `00-overview.md` | `01-pre-code-review-guides/`, `04-coding-guidelines-wpon/`, `05-coding-guidelines-workflowy/` | Create overviews |
| Missing `99-consistency-report.md` | `01-pre-code-review-guides/`, `04-coding-guidelines-wpon/`, `05-coding-guidelines-workflowy/` | Create reports |
| Missing numeric prefix on folder `02` | `02-coding-guidelines/` has no parent `00-overview.md` at root level | Create it |

### Cross-Reference Path Issues

| Issue | Location |
|-------|----------|
| References to `../01-general-spec/`, `../18-error-resolution/`, `../31-generic-enforce/` — these don't exist in current `spec/` tree | `03-coding-guidelines-spec/00-overview.md` |
| References use `02-spec/02-coding-guidelines/` paths but folder is now `02-spec/02-coding-guidelines/` | Various |

---

## Atomic Task Plan

### Phase 1: Fix Naming & Compliance Issues

| Task # | Task | Priority |
|--------|------|----------|
| T-001 | Rename files in `01-pre-code-review-guides/` to kebab-case with numeric prefixes | High |
| T-002 | Create `00-overview.md` for `01-pre-code-review-guides/` | High |
| T-003 | Fix duplicate prefix `05-` in `04-coding-guidelines-wpon/` (rename typescript to `04-typescript-standards/` or golang to `06-golang-standards/`) | High |
| T-004 | Create `00-overview.md` for `04-coding-guidelines-wpon/` | High |
| T-005 | Create `00-overview.md` for `05-coding-guidelines-workflowy/` (currently has `00-master-coding-guidelines.md` which serves as overview but doesn't follow template) | Medium |
| T-006 | Create `99-consistency-report.md` for folders missing it | Medium |
| T-007 | Create root `00-overview.md` for `02-spec/02-coding-guidelines/` if missing | High |

### Phase 2: Audit Content Overlap & Unique Content

| Task # | Task | Priority |
|--------|------|----------|
| T-008 | Map all unique rules from `01-pre-code-review-guides/` not in `03-coding-guidelines-spec/` (lazy evaluation, regex usage, newline styling, nesting avoidance, code mutation) | High |
| T-009 | Map all unique rules from `04-coding-guidelines-wpon/` not in `03-coding-guidelines-spec/` (PHP spacing/imports, response-key-type-inventory) | Medium |
| T-010 | Map all unique rules from `05-coding-guidelines-workflowy/` not in `03-coding-guidelines-spec/` (master guideline enforcement checklist, type-safety remediation plan) | High |
| T-011 | Map unique enum patterns from `06-enum-spec/` not in `02-coding-guidelines/03-golang/01-enum-specification/` | Medium |
| T-012 | Identify contradictions between sources (e.g., different line limits, naming rules) | High |

### Phase 3: Design the Consolidated AI Guideline Structure

| Task # | Task | Priority |
|--------|------|----------|
| T-013 | Design the new consolidated folder structure under `03-coding-guidelines-spec/` | High |
| T-014 | Define which content goes into cross-language vs language-specific files | High |
| T-015 | Design anti-hallucination section: explicit "DO NOT" rules with examples | High |
| T-016 | Design "AI Quick Reference" section — a condensed checklist for AI to validate code against | High |
| T-017 | Design practical examples section — before/after code for each major rule | Medium |

### Phase 4: Merge Content into Consolidated Structure

| Task # | Task | Priority |
|--------|------|----------|
| T-018 | Merge unique content from `01-pre-code-review-guides/` into cross-language guidelines | High |
| T-019 | Merge unique content from `04-coding-guidelines-wpon/` into language-specific files | Medium |
| T-020 | Merge unique content from `05-coding-guidelines-workflowy/` master guidelines into consolidated master | High |
| T-021 | Merge unique enum patterns from `06-enum-spec/` into Go enum specification | Medium |
| T-022 | Write the AI anti-hallucination section with explicit forbidden patterns | High |
| T-023 | Write the AI Quick Reference checklist | High |

### Phase 5: Archive Old Sources & Update References

| Task # | Task | Priority |
|--------|------|----------|
| T-024 | Move `01-pre-code-review-guides/` content to archive (keep originals as reference) | Medium |
| T-025 | Move `04-coding-guidelines-wpon/` content to archive | Medium |
| T-026 | Move `05-coding-guidelines-workflowy/` content to archive | Medium |
| T-027 | Update all cross-references across the spec tree | High |
| T-028 | Update `02-spec/02-coding-guidelines/` root `00-overview.md` to reflect new structure | High |
| T-029 | Run consistency report validation on entire `02-coding-guidelines/` tree | High |

### Phase 6: Quality Assurance

| Task # | Task | Priority |
|--------|------|----------|
| T-030 | Verify zero broken cross-references | High |
| T-031 | Verify all files pass kebab-case + numeric prefix rules | High |
| T-032 | Verify AI Confidence and Ambiguity scores on all overviews | Medium |
| T-033 | Generate final consistency reports for all sub-folders | High |
| T-034 | Review consolidated guideline for completeness — no rules lost from any source | High |

---

## Success Criteria

1. **Single source of truth** — One consolidated `03-coding-guidelines-spec/` with all rules from all 5 sources
2. **Zero content loss** — Every unique rule from every source is preserved
3. **AI-optimized** — Anti-hallucination section, explicit DO/DON'T examples, quick reference checklist
4. **Fully compliant** — 100/100 health score, zero broken links, all naming conventions followed
5. **Archived originals** — Old sources preserved in archive for historical reference

---

## Cross-References

| Reference | Location |
|-----------|----------|
| Spec Authoring Guide | `../../02-spec/01-spec-authoring-guide/00-overview.md` |
| Current Coding Guidelines | `../../02-spec/02-coding-guidelines/00-overview.md` |
| Naming Compliance Issues | `../project/naming-compliance-issues.md` |
