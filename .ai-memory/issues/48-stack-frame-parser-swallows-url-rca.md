# Issue 48: Stack frame parser splits a URL mid-path

Status: open (planned, not fixed)
Raised: 2026-09-30
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Plan: [85](../plans/pending/85-ui-tier-badge-priority-highlight-focus-and-stack-trace.md)

## 1. Symptom
The diagnostic report shows function `http://tauri.localhost/assets/index-<hash>.j` and file `s`.

## 2. Trigger
A stack line with no function name, only `at <url>:line:col`.

## 3. Root cause
In `src/stores/error-store.ts::parseStackLine` the optional function group `([^\s(]+)?` greedily matches the whole URL, and the file group `([^:)]+)` then captures a single character before `:line:col`.

## 4. Why it escaped
Error reports were only ever read by eye; no test fed a URL-only frame.

## 5. Fix (planned)
Parse with two explicit shapes (named frame with parentheses; bare location) in a pure helper with its own file and test. See plan 85 subtasks 001-002.

## 6. Prevention
Pure parser in its own module with table-driven tests including URL-only, `async`, `eval`, and Windows paths.

## 7. Regression check
Frontend unit test for `parseStackLine` (runner decision: ambiguity 04).
