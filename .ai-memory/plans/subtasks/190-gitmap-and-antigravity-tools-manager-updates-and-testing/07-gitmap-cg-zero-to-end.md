# Subtask 07 — gitmap CG execute zero-to-end

Repo: gitmap-v28. Spec: 02-gitmap-cg-execute-spec.md steps 2-4 and 6-9.
Status: PENDING. Owner: Worker waves 3-5 (largest subtask; split per wave).

## Goal
Every CG execute prompt in 01-prompts/15-cg-execute/ is either applied repo-wide or explicitly closed with a linter-verified note.

## Waves (in spec order)
- W3a: prompt 24 (isolate destructive OS/heavy unit tests), prompts 12 (relative paths), 13 (CLI commands & help).
- W3b: prompt 14 (function signatures) → 18 (argument reduction) → 22 (file-size & function reduction; first targets: cli/cmd/root.go dispatch 116 lines / Run 99, cli/store/agent_store.go InitTaskDB 86).
- W4a: prompt 29 (DRY/library extraction) → 25 (nuclear package modularization) — conservative: extract only proven duplication; modularization limited to the prompt's own criteria.
- W4b: efficiency — 21 (lazy regex), 23 (string ops), 37 (EqualFoldAny), 38 (map dispatch), 36 (variadic), 34+39 (pointer/value semantics).
- W5: surfaces — 15 (TypeScript), 17 (terminal UI), 16 remainder (enums/traits), 26/27/28 (Python trio in 03-ai-scripts/), 33 (branch immutability), 30 (clean build & caches), 35 (clean artifacts); meta — 31, 32, then prompt 01 as final sweep.

## Method per prompt
1. Read the prompt file in 01-prompts/15-cg-execute/ in full.
2. Enumerate violations repo-wide via gitmap aum search (never rg/grep).
3. Fix in the prompt's spirit; keep functions 8-15 lines, positive booleans, AppError returns.
4. Record per-prompt closure: hits before → after, files touched.

## Rules
- NO builds, NO test runs at any point in this subtask (user instruction — absolute).
- If a prompt's full application would require an architectural decision the prompt does not settle, apply the unambiguous majority and log the remainder as a follow-up in the parent plan — do not invent architecture.
- GitMap-only search. Relative paths only. Lowercase filenames.
