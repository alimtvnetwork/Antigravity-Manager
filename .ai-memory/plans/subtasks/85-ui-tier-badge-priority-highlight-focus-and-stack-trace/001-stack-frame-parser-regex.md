---
plan: 85-ui-tier-badge-priority-highlight-focus-and-stack-trace
subtask: "001"
title: Fix stack frame parsing for URL-only frames
domain: frontend
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t02--stack-trace
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/48-stack-frame-parser-swallows-url-rca.md
  ambiguity: none
target_files:
  - src/stores/error-store.ts — `parseStackLine`, `StackFrame` (extract to `src/lib/stack-frame-parser.ts`)
  - src/lib/error-report-generator.ts — consumer of parsed frames (read only, confirm no shape change)
status: pending
---

# 001 — Fix stack frame parsing for URL-only frames

## 1. Context
The report shows function `http://tauri.localhost/assets/index-<hash>.j` and file `s`. The pattern `([^\s(]+)?\s*\(?([^:)]+):(\d+):(\d+)\)?` lets the optional function group consume a bare URL and leaves one character for the file.

## 2. Target files and symbols
- src/stores/error-store.ts — `parseStackLine`, `StackFrame` (extract to `src/lib/stack-frame-parser.ts`)
- src/lib/error-report-generator.ts — consumer of parsed frames (read only, confirm no shape change)

## 3. Steps
1. Create `src/lib/stack-frame-parser.ts` exporting `parseStackLine(line: string): StackFrame | null` and the `StackFrame` type moved from `error-store.ts` (re-export from the store so imports keep working).
2. Implement two explicit shapes, tried in order: (a) `at <fn> (<location>)` where `<fn>` may contain spaces and `async `/`new ` prefixes; (b) `at <location>` with no function, giving function `<anonymous>`.
3. `<location>` is parsed from the right: `^(.*):(\d+):(\d+)$` so a URL with `:` in scheme or port stays intact.
4. Keep the return shape identical (function, file, line, column) so the report generator does not change.
5. Remove the old regex from `error-store.ts` and import the new function.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function size and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; each new component or helper lives in its own file.
- Relative paths only in code, scripts, and docs.
- Reuse existing UI conventions (pill/badge toggles, existing dialogs); no new unrelated controls.
- Do not change the default error code `E9001` mapping in this step.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
npm run build
```

## 7. Done When
- [ ] The report example line parses to function `<anonymous>` and the full asset URL as file.
- [ ] `error-store.ts` has no inline stack regex.
- [ ] `npm run build` passes.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
