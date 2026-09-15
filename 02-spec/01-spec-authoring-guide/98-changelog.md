# Spec Authoring Guide — Changelog

**Version:** 3.2.0
**Last Updated:** 2026-04-16

---

## [2026-09-10] v2.2.0 — Application Specification Blind-AI Audit v1 generated in `02-spec/25-app-spec-audit/01-audit-2026-09-10-v1.md` (Score: 89/100, Band B)

## [2026-09-10] v2.1.0 — Application Specification Blind-AI Audit v2 remediated (100% closed, archived to `.lovable/plans/completed/13-spec-remediation-completed.md`)

## [2026-09-10] v2.0.0 — Application Specification Blind-AI Audit v2 generated in `02-spec/25-app-spec-audit/02-audit-2026-09-10-v2.md`

## [2026-09-10] v1.2.0 — Application Specification Blind-AI Audit v1 generated in `02-spec/25-app-spec-audit/01-audit-2026-09-10-v1.md`

## [2026-09-10] v1.1.0 — Application Specification Blind-AI Audit remediated (100% closed, archived to `.lovable/plans/completed/12-spec-remediation-completed.md`)

## [2026-03-30] v2.0.0 Compliance Rollout

**Scope:** Project-wide sub-folder `01-index.md` upgrade
**Total files upgraded:** 139 sub-folder overviews + 2 new files created
**Health impact:** 100% compliance with Spec Authoring Guide v2.0.0

### Summary

Upgraded all `01-index.md` files across the entire spec tree to include the four mandatory sections introduced by the Spec Authoring Guide v2.0.0:

1. **AI Confidence** — metadata field (High for all modules)
2. **Ambiguity** — metadata field (None for all modules)
3. **Keywords** — searchable tags derived from module context
4. **Scoring** — standardized compliance table

### Phases

| Phase | Scope | Files |
|-------|-------|-------|
| 1 | Module-level overviews (01–36, 99) | 31 |
| 2 | Module 01 sub-folders | 12 |
| 3 | Module 02 sub-folders | 60 |
| 4 | Module 03 sub-folders | 5 |
| 5 | Modules 04–36, 99, validation-reports | 62 |

### New Files Created

| File | Module |
|------|--------|
| `02-spec/02-coding-guidelines/05-rust/97-acceptance-criteria.md` | Rust Coding Standards |
| `02-spec/02-coding-guidelines/05-rust/99-consistency-report.md` | Rust Coding Standards |

### Method

- Phases 1–2: Manual per-file upgrades
- Phases 3–5: Automated Python script with keyword derivation and version bumping
- Post-processing: Keyword cleanup pass to remove path artifacts

### Verification

- Final scan confirmed 0 non-compliant `01-index.md` files (excluding root dashboard)
- Parent consistency reports updated where applicable

---

## Cross-References

- [Spec Authoring Guide Overview](./01-index.md)
- [Acceptance Criteria](./97-acceptance-criteria.md)
- [Consistency Report](./99-consistency-report.md)
