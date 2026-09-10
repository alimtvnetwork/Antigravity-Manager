# Avoid: Per-Task Folders Under `.lovable/`

**Status:** 🚫 Blocked — single-file convention required
**Recorded:** 2026-04-19

---

## Rule

The `.lovable/` directory MUST follow the single-file convention defined in `prompts/03-write-prompt.md`:

| Concern | Single file |
|---------|------------|
| Roadmap | `.lovable/29-plan.md` (with `## Completed` section) |
| Suggestions | `.lovable/suggestions.md` (with `## Implemented Suggestions` section) |
| Hard prohibitions | `.lovable/strictly-avoid.md` |

Never create per-task folders such as `.lovable/completed-tasks/`, `.lovable/pending-tasks/`, `.lovable/suggestions/`, or `.lovable/strictly-avoid/`. They fragment knowledge and make the next AI session miss context.

---

## What Was Removed (2026-04-19)

During the session restructure these directories were collapsed back into the canonical single files:

- `.lovable/completed-tasks/` — content already represented in `.lovable/29-plan.md` `## Completed Plans` section.
- `.lovable/pending-tasks/` — sole file (`gap-analysis-p1-p2.md`) was already complete; tracked under `## Completed Plans`.
- `.lovable/suggestions/` — five detail files merged into `.lovable/suggestions.md` Active Suggestions block.
- `.lovable/strictly-avoid/` — three rule files merged into `.lovable/strictly-avoid.md`.

---

## Per-Issue Folders Are Different

Pending and solved issues legitimately need one file per issue:

- `.lovable/pending-issues/NN-short-description.md`
- `.lovable/solved-issues/NN-short-description.md`

This is allowed because each issue carries a distinct lifecycle (root cause, attempts, solution, learning). Plans/suggestions do not.

---

## Memory Topic Folders Are Also Different

`.lovable/memory/` uses topic-grouped folders (`workflow/`, `decisions/`, `sessions/`, `project/`, `features/`, `issues/`, `avoid/`, etc.). Each topic folder MUST be listed in `index.md`.

---

*Avoid note — v1.0.0 — 2026-04-19*
