# Ambiguity & Decision Log — No-Questions Mode

This folder captures questions / ambiguities that arose during the
40-task **No-Questions Mode** run. Instead of pausing the work loop
to ask the user, the AI records each point of confusion here with
full context, the options considered, the inference made, and the
rationale — so the user can review every assumption in one pass at
the end.

## Conventions

- One file per task that produced an ambiguity. **Not every task
  generates an entry** — only those where a real interpretive choice
  was made.
- Filename: `xx-brief-title.md` where `xx` is a zero-padded sequence
  starting at `01` and incrementing monotonically across the run
  (independent of how many tasks have been processed).
- Each entry MUST include: original user request (verbatim), the
  ambiguity, every option considered with pros/cons, the chosen
  option, and the rationale.

## Entry template

```markdown

# xx — <brief title>

## Original request

> <verbatim user message that triggered the task>

## Context

<files touched, prior state, relevant constraints>

## Ambiguity

<the specific point of confusion in 1–3 sentences>

## Options considered

### Option A — <name>

- **Pros:** …
- **Cons:** …

### Option B — <name>

- **Pros:** …
- **Cons:** …

### Option C — <name>

- **Pros:** …
- **Cons:** …

## Chosen — <Option X>

**Recommendation rationale:** <why this option won>

## Reversibility

<how hard would it be to switch to a different option later>
```

## Index

| # | Title | Task summary | Status |
|---|---|---|---|
| [01](./01-ignored-deleted-reason-scope.md) | `ignored-deleted` reason enrichment scope | Diversify reason text per provenance vs. also emit rename OLD-side rows. Chose **Option A** (text only — preserves row contract). | Pending review |

---

The user will review entries at the end of the 40-task run and either
confirm the inferred choice or request a follow-up adjustment.
