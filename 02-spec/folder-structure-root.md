# Folder Structure Root — Redirect

**Updated:** 2026-04-16

---

## Single Source of Truth

The complete folder structure specification — including repository organization, `.ai-memory/` AI metadata layers, numbering policy, required folders, rules, and validation checklists — is maintained in:

> **📄 [Canonical Folder Structure Specification](../.ai-memory/folder-structure.md)**
> **📄 [`02-spec/01-spec-authoring-guide/02-folder-structure.md`](./01-spec-authoring-guide/02-folder-structure.md)** (Spec Hierarchy Guide)

This file previously contained a full copy of the folder structure rules. To eliminate duplication and maintain a single source of truth, all content has been consolidated into the spec authoring guide.

---

## Root Numbered Folder Architecture

| Root Folder | Purpose |
|---|---|
| `01-prompts/` | Prompt Architect prompt library (workflows, coding standards, instructions) |
| `02-spec/` | Master engineering specifications (01–20 core standards, 21+ application specs) |
| `03-ai-scripts/` | Reusable deterministic Python automation and quality gate scripts |
| `.ai-memory/` | Agent memory, master plans, subtasks, and rule sets |

## Spec Numbering Quick Reference

| Range | Purpose |
|-------|---------|
| 01–20 | Core fundamentals (principles, standards, integrations, research) |
| 21+ | App-specific content (`21-app/` features, `22-app-issues/`, `23-app-db/`, `24-app-ui/`) |

For the full specification, required folder list, AI instructions, and validation checklist, see the canonical source above.

---

```
IMPORTANT — AI INSTRUCTION:
- The canonical folder structure spec is 01-spec-authoring-guide/02-folder-structure.md
- This file is a redirect — do NOT duplicate folder structure rules here.
- Read the canonical source for all structural decisions.
```
