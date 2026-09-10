# SS-02 Slides system improvement backlog (70 tasks)

Parent: 01-slides-system-overhaul
Slug: slides-70-task-backlog
Status: in-progress (13/70 shipped as of v5.56.0)

## Completed (reconciled 2026-07-19, v5.56.0)

Shipped in v5.50.0 - v5.55.0 across prior sessions:

- Task 8 (document.title sync) - landed with URL-driven nav.
- Task 15 (semantic classes audit in `.slide-content`) - completed with PrincipleCard rewrite.
- Task 45 (frontend global error boundary) - `ErrorBoundary` in slides-app.
- Task 63 (slides build wired into `scripts/release.mjs`) - v5.52.0.
- Task 64 (attach `slides-deck.zip` to GitHub Release) - v5.52.0.
- Task 65 (README link to slides deck asset) - v5.53.0.
- Task 66 (`scripts/sync-guidelines.mjs`) - v5.50.0.
- Task 68 (smoke test rule-id coverage) - v5.49.0 + extended.
- Task 69 (Playwright visual regression baseline) - `tests/visual.spec.ts` + `slides-visual.yml`.
- Task 70 (axe-core a11y per slide) - v5.51.0 + `a11y.spec.ts`.
- Extra: TOC slide with deep links and progress dots.
- Extra: per-block review checklists (`ReviewCheckbox`, `lib/progress.ts`).
- Extra: SRA structural validator (`scripts/validate-slides-sra.mjs`) in CI.

Remaining: 57 tasks (infra 1-7, 9-14; content 16-62; build 67).
Created: 2026-07-19

Ordered backlog to evolve `slides-app/` into the canonical teaching surface for the compiled simple coding guideline (`02-spec/17-consolidated-guidelines/34-compiled-simple-coding-guidelines.md` v1.4.0). Every slide follows the Symptom → Rule → Action pattern already used by `ActionPanel`.

Legend: [INFRA] deck framework, [CONTENT] new slide, [A11Y] accessibility, [BUILD] build/release, [TEST] verification.

## A. Deck infrastructure (tasks 1-15)

1. [INFRA] Extract `deck.ts` entries into `slides-app/src/deck/registry.ts` grouped by section (Principles, Naming, Control-flow, Errors, React, Workflow, Closing).
2. [INFRA] Add a `SlideSection` type and render section dividers between groups in grid view.
3. [INFRA] Add `SymptomRuleAction` compound component (S/R/A three-column layout) so every rule slide has the same shape.
4. [INFRA] Add `RuleBadge` component (Hard / Warn / Style) driven by a rule-severity enum.
5. [INFRA] Add `LanguageTabs` component so a single rule can show Go / TS / PHP / Rust snippets without duplicating slides. ✅ v5.64.0
6. [INFRA] Add `CalloutQuote` for the "comments lie, code does not" style aphorisms. ✅ v5.63.0
7. [INFRA] Add `RuleAnchor` slug per slide (`/slides/naming-camelcase`) for deep-linking from the docs site.
8. [INFRA] Sync deck title to `document.title` on slide change (per slides-app spec §9).
9. [INFRA] Add `?print` route rendering all slides stacked for PDF handout export.
10. [INFRA] Add `?step=N` sub-step URL coordinate for staged reveals (bullets, diffs). ✅ v5.65.0 (hash form `#/id/<slide-id>/<step>`)
11. [INFRA] Presenter view: show notes + next-slide preview + rule id.
12. [INFRA] Grid view: filter by section + severity.
13. [INFRA] Add search-in-deck (Cmd+K) matching slide title, rule id, and tags. ✅ v5.67.0 (added `tags?: readonly string[]` to `SlideEntry`, extended palette haystack + tag chip rendering in `App.tsx`)
14. [INFRA] Dark-mode audit: ensure all diff/code contrast meets WCAG AA. ✅ v5.66.0 (`slides-app/scripts/contrast-audit.mjs`; caught `--accent` 2.47:1 on light bg, darkened to 30% L for 4.02:1)
15. [INFRA] Replace any remaining Tailwind `text-*` inside `.slide-content` with semantic classes per slides-app skill.

## B. Content: Must-Follow + core principles (tasks 16-22)

16. [CONTENT] "Must Follow" opener slide (from §Must Follow of file 31) with 5 non-negotiables. ✅ v5.68.0 (`slides-app/src/slides/13-must-follow.tsx`, ruleId `MUST-001`)
17. [CONTENT] Root-cause-first workflow slide (one-sentence RCA before fix). ✅ v5.69.0 (`slides-app/src/slides/14-root-cause-first.tsx`, ruleId `MUST-002`)
18. [CONTENT] Spec-first vs code-first (before/after: PR touching only code vs PR with spec+code). ✅ v5.70.0 (`slides-app/src/slides/15-spec-first-vs-code-first.tsx`, ruleId `MUST-003`)
19. [CONTENT] "Comments lie, code does not" slide with Go `path.Clean` doc example and link to https://go.dev/src/go/doc/example.go. ✅ v5.71.0 (`slides-app/src/slides/16-comments-lie-code-does-not.tsx`, ruleId `MUST-004`)
20. [CONTENT] Method-documentation decision tree (simple method → no docs; complex + irreducible → doc). ✅ v5.72.0 (`slides-app/src/slides/17-method-doc-decision-tree.tsx`, ruleId `MUST-005`)
21. [CONTENT] Version-bump myth-buster slide (no longer required per v1.2 update). ✅ v5.73.0 (`slides-app/src/slides/18-version-bump-myth-buster.tsx`, ruleId `MUST-006`)
22. [CONTENT] Trust-boundary teaser (Main > Worker > Backup > Git) linking to spec/19 chs. 26/27. — DONE v5.74.0 (`slides-app/src/slides/19-trust-boundaries-teaser.tsx`, `MUST-007`).

## C. Content: Naming, style, structure (tasks 23-32)

22a. [CONTENT] Backup-tier freeze rationale (why `MainWorker.Backup.*` seed keys stay defaults-only until Phase 12). — DONE v5.75.0 (`slides-app/src/slides/20-backup-tier-freeze.tsx`, `MUST-008`). Inserted between task 22 and 23 during execution.
23. [CONTENT] PascalCase entities / camelCase fields / `{Table}Id` PK naming. ✅ v5.76.0 (`slides-app/src/slides/21-db-schema-naming.tsx`, ruleId `NAM-002`)
24. [CONTENT] Zero-underscore policy + full-uppercase acronyms. ✅ v5.77.0 (`slides-app/src/slides/22-zero-underscore-acronyms.tsx`, ruleId `NAM-003`)
25. [CONTENT] Boolean naming: is/has only + positive framing (before/after). ✅ v5.78.0 (`slides-app/src/slides/23-boolean-naming.tsx`, ruleId `BOOL-002`)
26. [CONTENT] No boolean parameters (refactor to two named methods). ✅ v5.79.0 (`slides-app/src/slides/24-no-boolean-parameters.tsx`, ruleId `BOOL-003`)
27. [CONTENT] Enum standards (cross-language PascalCase, strict parse). ✅ v5.80.0 (`slides-app/src/slides/25-enum-standards.tsx`, ruleId `ENUM-001`)
28. [CONTENT] Line-gap discipline: blank line before return/throw, after closing brace, import grouping.
29. [CONTENT] File size tier (<300 LOC, <100 for .tsx) with real repo example.
30. [CONTENT] Function size tier (8 warn / 15 hard) with waiver syntax demo.
31. [CONTENT] Max 3 params + cognitive complexity ≤10.
32. [CONTENT] Named types over tuples (React `useUser`, `ProfileCardProps` example from v1.4).

## D. Content: Control flow (tasks 33-40)

33. [CONTENT] Positive-condition rule + guard-clause refactor.
34. [CONTENT] No-nested-if pyramid (before/after with early returns).
35. [CONTENT] Two-operand max per boolean expression (extract intent to named helper).
36. [CONTENT] No else-after-return (`no-lonely-if`).
37. [CONTENT] useEffect: no negative conditions, extract guards to booleans.
38. [CONTENT] Minimize useEffect count (derive don't sync).
39. [CONTENT] Mandatory cleanup in effects (subscription, timers).
40. [CONTENT] Avoid for/forEach in render; prefer map/filter/reduce.

## E. Content: Error management (tasks 41-48)

41. [CONTENT] Never swallow errors (before: bare catch; after: apperror.Wrap).
42. [CONTENT] Registered error codes only (link to MWS catalogue).
43. [CONTENT] Explicit file/path in every error log.
44. [CONTENT] Retry policy vs surface-and-fail decision matrix.
45. [CONTENT] Frontend global error boundary + toast policy.
46. [CONTENT] Structured logging one-liner discipline (from slide 05, refreshed).
47. [CONTENT] Magic-string elimination (constants package) promoted to error.
48. [CONTENT] `any` prohibition + switch exhaustiveness (TS §2.1).

## F. Content: React, TS, cross-language (tasks 49-56)

49. [CONTENT] Named types for hook returns + props.
50. [CONTENT] Composition over prop drilling.
51. [CONTENT] Memoization only after profiling (guardrail).
52. [CONTENT] Accessibility floor: semantic HTML, alt text, keyboard focus.
53. [CONTENT] Go one-liner slide (idiomatic error return, no panics).
54. [CONTENT] Rust one-liner slide (snake_case identifiers exception).
55. [CONTENT] PHP + PowerShell one-liner slide (PascalCase Verb-Noun, kebab-case files).
56. [CONTENT] C# one-liner slide (StyleCop ruleset link).

## G. Workflow, testing, security (tasks 57-62)

57. [CONTENT] Testing minimums: happy + edge + error path.
58. [CONTENT] Security defaults: no secret logging, validate at boundaries.
59. [CONTENT] Caching policy (explicit TTL, deterministic keys, invalidate on mutation).
60. [CONTENT] Decision tree slide (catches 80% of common violations).
61. [CONTENT] Common-violation clinic (5 real repo diffs, anonymized).
62. [CONTENT] Closing slide refresh with QR to 02-spec/17/31.

## H. Build, release, distribution (tasks 63-70)

63. [BUILD] Wire `slides-app` build into `scripts/release.mjs` (build + zip + verify offline contract).
64. [BUILD] Attach `slides-app/dist.zip` to every GitHub Release (per command 01).
65. [BUILD] Add "Slides deck" link to root README release section pointing to latest release asset.
66. [BUILD] Add `scripts/sync-guidelines.mjs` that regenerates deck rule cards from file 31 frontmatter.
67. [BUILD] Fail CI if deck slide count is less than rule count in file 31.
68. [TEST] Extend `slides-app/tests/smoke.spec.ts` to assert every rule id from file 31 appears in the deck.
69. [TEST] Add visual-regression baseline (Playwright screenshots per slide).
70. [A11Y] Add axe-core Playwright check per slide; fail CI on serious/critical violations.
