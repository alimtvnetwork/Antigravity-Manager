---
name: no-questions-mode
description: Active 40-task no-questions run. Never call ask_questions; log ambiguities to .lovable/question-and-ambiguity/xx-title.md with options + rationale.
type: preference
---

# No-Questions Mode (active)

For the 40-task batch starting 2026-04-26: **never** invoke
`questions--ask_questions`. When ambiguity arises:

1. Pick the best-suited inference and proceed.
2. Log the decision at
   `.lovable/question-and-ambiguity/xx-brief-title.md` (xx =
   zero-padded sequence, monotonic across the run, starts at 01).
3. Each entry uses the template in
   `.lovable/question-and-ambiguity/01-index.md` and MUST include:
   verbatim original request, context, the ambiguity, every option
   considered with pros/cons, chosen option (with recommendation
   rationale), and a reversibility note.
4. Update the **Index** table in that README — one row per new
   entry — so the user can scan all decisions at the end.
5. Skip the log when a task has no real interpretive choice. Trivial
   / unambiguous tasks just get done.

**How to apply:** Default to action. If a task is genuinely
unactionable without input, still pick the most defensible
interpretation, log it, and ship.

**Resume trigger:** Stop this mode only when the user says
"ask questions if any understanding issues" (or an equivalent
explicit signal). Until then, no questions — even if the request is
broad or design-open.
