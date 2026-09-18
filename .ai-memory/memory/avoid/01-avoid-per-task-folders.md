# Avoid: Per-Task Folders Under `.ai-memory/`

**Status:** 🚫 Blocked — single-file convention required
**Recorded:** 2026-04-19

---

## Rule

The `.ai-memory/` directory MUST follow the single-file convention defined in `prompts/03-write-prompt.md`:

| Concern | Single file |
|---------|------------|
| Roadmap | `.ai-memory/29-plan.md` (with `## Completed` section) |
| Suggestions | `.ai-memory/suggestions.md` (with `## Implemented Suggestions` section) |
| Hard prohibitions | `.ai-memory/strictly-avoid.md` |

Never create per-task folders such as `.ai-memory/completed-tasks/`, `.ai-memory/pending-tasks/`, `.ai-memory/suggestions/`, or `.ai-memory/strictly-avoid/`. They fragment knowledge and make the next AI session miss context.

---

## What Was Removed (2026-04-19)

During the session restructure these directories were collapsed back into the canonical single files:

- `.ai-memory/completed-tasks/` — content already represented in `.ai-memory/29-plan.md` `## Completed Plans` section.
- `.ai-memory/pending-tasks/` — sole file (`gap-analysis-p1-p2.md`) was already complete; tracked under `## Completed Plans`.
- `.ai-memory/suggestions/` — five detail files merged into `.ai-memory/suggestions.md` Active Suggestions block.
- `.ai-memory/strictly-avoid/` — three rule files merged into `.ai-memory/strictly-avoid.md`.

---

## Per-Issue Folders Are Different

Pending and solved issues legitimately need one file per issue:

- `.ai-memory/pending-issues/NN-short-description.md`
- `.ai-memory/solved-issues/NN-short-description.md`

This is allowed because each issue carries a distinct lifecycle (root cause, attempts, solution, learning). Plans/suggestions do not.

---

## Memory Topic Folders Are Also Different

`.ai-memory/memory/` uses topic-grouped folders (`workflow/`, `decisions/`, `sessions/`, `project/`, `features/`, `issues/`, `avoid/`, etc.). Each topic folder MUST be listed in `index.md`.

---

*Avoid note — v1.0.0 — 2026-04-19*
