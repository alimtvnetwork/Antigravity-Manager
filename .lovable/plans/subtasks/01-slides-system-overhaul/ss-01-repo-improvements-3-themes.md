# SS-01 Repo-wide improvements (3 themes)

Parent: 01-slides-system-overhaul
Slug: repo-improvements-3-themes
Status: pending
Created: 2026-07-19

Three top-level improvement themes for this repository, each with concrete follow-ups.

## Theme 1 — Guideline single-source-of-truth + enforcement parity

Symptom: rules live in `.cursorrules`, `02-spec/02-coding-guidelines/`, `02-spec/17-consolidated-guidelines/31-…`, `.lovable/coding-guidelines.md`, `eslint.config.js`, `linters/*`. Drift is a chronic issue (already patched once in v5.48-era).

Follow-ups:
- Generate `.cursorrules`, `.lovable/coding-guidelines.md`, and the slides deck content from `02-spec/17-consolidated-guidelines/34-compiled-simple-coding-guidelines.md` via a `scripts/sync-guidelines.mjs` step; fail CI on drift.
- Add ESLint rules for the still-unenforced hard rules: boolean naming prefix (is/has only), no-else-after-return, positive-condition guard, max 2 operands per boolean expression, magic-string as error (not warn).
- Extend `linters/golangci-lint`, `linters/phpcs`, `linters/sonarqube`, `linters/stylecop` with equivalents so cross-language parity matches the unified tier in `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.

## Theme 2 — Slides deck as the canonical teaching surface

Symptom: deck has 16 slides but the compiled simple guideline (v1.4) now covers ~40+ rules (React tuples, method-doc policy, error digest, boolean naming, useEffect discipline, line-gap rules, must-follow block). Deck is behind the spec.

Follow-ups:
- Regenerate the deck from the compiled simple guideline (1 rule → 1 slide, symptom → rule → action).
- Add before/after code diffs for every new rule (React tuples, useEffect readability, method doc policy citing `go.dev/src/go/doc/example.go`, error wrapping, line-gap).
- Ship the deck ZIP as a GitHub Release asset on every version bump (ties to command 01).

## Theme 3 — Release + CI hardening

Symptom: release script exists but slides build, mermaid render, spec-verification, and installer tests run in separate flows; failures don't always block release. Repo also carries residual weight (release-artifacts history, .gitmap snapshots) despite the diet.

Follow-ups:
- Make `scripts/release.mjs` a strict pipeline: sync-guidelines → lint (JS+Go+PHP+C#) → typecheck → vitest → playwright (slides + landing) → mermaid validate+render → spec-verification coverage → build slides + zip → publish + attach `dist.zip`.
- Add pre-push guard already exists — extend it with `slides-app` build smoke to catch deck breakage locally.
- Add a repo-size CI gate (fail if any commit adds >2MB binary outside `slides-app/public/fonts/`).
