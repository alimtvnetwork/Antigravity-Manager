---
name: Skip Stub Spec Folders from Audits
description: Four stub spec folders permanently excluded from AI-readiness audits, remediation, and acceptance-criteria work
type: constraint
---

# Skip Stub Spec Folders

The following four spec folders are **intentional stubs** and must be **excluded** from all future spec-audit, scoring, and AI-readiness remediation work:

| Folder | Lines | Reason |
|--------|-------|--------|
| `02-spec/21-app/` | 33 | Placeholder, no app-specific specs yet |
| `02-spec/22-app-issues/` | 33 | Placeholder, no issues documented yet |
| `02-spec/23-app-db/` | 51 | Placeholder, no schema/acceptance criteria |
| `02-spec/24-app-ui-design-system/` | 50 | Placeholder, no UI specs yet |

**Why:** These are reserved namespaces for future app-specific content. They score <20 in audits but that is **expected and acceptable** — they are scaffolding, not real specs.

**How to apply:**
- Exclude from `spec-audit-*.{md,csv,json}` corpus calculations.
- Do **not** write `97-acceptance-criteria.md` or `99-consistency-report.md` for them.
- Do **not** propose demoting them to `_drafts/` — they stay in place as numbered placeholders.
- When computing audit averages, exclude these 4 folders → effective corpus = 18 folders.
