# Task-08: Supabase investigation + RCA

## Status
DONE

## Objective
Reproduce the reported Supabase failure, find its root cause, document it
as an RCA in `.ai-memory/rca/`, and fix it if the fix is in-repo code.

## Steps
1. Discover the Supabase health/diagnostics surface via GitMap search
   (`src-tauri/src/modules/supabase_sync/`, diagnostics commands, UI
   indicators). Read how the app currently tests the connection.
2. Reproduce: run the same checks the app runs (read-only; never write
   real credentials anywhere — use the existing `repo-secrets` flow if
   credentials are needed, and never log them).
3. Root-cause the failure: connection (DNS/TLS/endpoint), auth (key
   validity, RLS), schema (missing tables), or client config (wrong URL,
   stale cached endpoint).
4. Write the RCA file (symptom → root cause → how found → fix →
   verification → prevention).
5. If the fix is in-repo code (retry logic, endpoint handling, error
   surfacing), implement it following the "No Silent Errors" rule and
   `cargo fmt` cleanliness. If it needs user action (e.g. rotate a key),
   write the exact user-facing steps instead and mark the subtask
   `BLOCKED: needs user <specific action>`.

## Acceptance
- [ ] RCA written with a named root cause.
- [ ] Either fixed in-repo with verification, or blocked with exact
      user steps.

## Constraints
- GitMap search only. Relative paths only. NEVER write secret values to
  disk, git, logs, or chat. No workflow changes.
