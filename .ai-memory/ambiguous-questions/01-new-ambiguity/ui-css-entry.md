# Ambiguity: CSS entry file (`index.css` vs `App.css`)

Status: proceeding by best judgment.

## Question

The task statement lists an "`index.css` base" step, but the repo has no
`src/index.css`. The single CSS entry is `src/App.css` (imported by
`src/main.tsx`, holding the `@tailwind` directives, Ubuntu/Effra fonts,
WebView2 guards, and all L1 vars).

## Decision taken

Implement the base in `src/App.css`; do not create a second entry.
Rationale: splitting `@tailwind base/components/utilities` across two entries
changes cascade order and risks double-injection of preflight.

## If overturned

Create `src/index.css`, move the `@tailwind` directives + base layer there,
import it before `App.css` in `main.tsx`, and re-verify T2/T5 plus a
cold-start render check.
