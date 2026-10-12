# Subtask 05 — Supabase verify + repo secrets

> SUPERSEDED 2026-10-13: covered by the completed plan (Supabase dual-connection + repo-secrets wiring, shipped v4.195.0). Kept for record only. Secret values must never be committed.

## Goal

Both provided Supabase projects verified reachable; credentials wired into GitHub repo secrets; zero secret values in git.

## Steps

1. Read-only connectivity check against both REST endpoints (`/rest/v1/`) with the provided publishable keys (coordinator brief carries the values — never write them to disk, git, logs, or chat).
2. Discover the expected secret/env names from the codebase and workflows (`gitmap aum search` for `SUPABASE`, `supabase_url`, etc.).
3. `gh secret set` the credentials into `alimtvnetwork/Antigravity-Manager` repo secrets. If the code expects a single pair, wire the "Lovable" pair to the canonical names and the "Personal Supa" pair to clearly prefixed names; document the mapping in the commit message.
4. Spot-check `git log -p` / `git status` to prove no secret values entered the tree. Never commit the values.

## Done when

Both endpoints answer; secrets set; tree provably secret-free.
