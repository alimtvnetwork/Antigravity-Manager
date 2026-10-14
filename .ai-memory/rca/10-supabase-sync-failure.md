# RCA 10 — Supabase health/diagnostics show a failure

**Status:** BLOCKED — fix requires user action (cloud project paused); no in-repo code change needed.
**Date:** 2026-10-15
**Task:** root-cause-analysis-and-telegram-staff-improvement / Task-08

## Symptom

Supabase health/diagnostics in the app report a failure:

- Settings → Supabase page: per-endpoint "Test" button shows a failure and the endpoint badge flips to the red **Unreachable** label (`src/components/settings/SupabaseSync/EndpointsSection.tsx`, the `result && !result.isSuccess` branch).
- Same failure from the CLI: `agm supabase test` prints `FAIL` with the same message.
- The message shown is the transport-error text from the client: "Cannot connect to `https://<project_ref>.supabase.co`. DNS for this project ref does not exist (NXDOMAIN) ... The API key was not the thing that failed. Unpause the project in the Supabase or Lovable dashboard."

## Root cause (named)

**Cloud-side paused free-tier Supabase projects — connection layer (DNS NXDOMAIN), not auth, not schema, not client config.**

The two configured endpoints are:

- root: `https://pezjuuddecbyfmqxytrv.supabase.co` (Lovable-hosted project)
- secondary: `https://ikwmurmjynhxdpmhzekt.supabase.co`

Free-tier Supabase projects pause after ~7 days of inactivity, and Supabase **unpublishes `<project_ref>.supabase.co` from DNS while paused**. The app's `test_connection()` probe ladder fails at the very first network request (Probe 1, `GET /rest/v1/`) because the hostname does not resolve. The client classifies this as `e.is_connect()` → NXDOMAIN → tells the user to unpause the project. The API key, RLS, schema, and the configured URLs are all fine; the failure happens before any of those layers are reached.

## How found

1. **Mapped the diagnostics surface** (GitMap search only): UI `handleTestEndpoint` in `src/components/settings/SupabaseSync/useSupabaseSyncHandlers.ts` → `supabaseService.testEndpoint` (`src/services/supabaseService.ts:147`) → Tauri `test_supabase_endpoint` (`src-tauri/src/commands/supabase.rs:49`) → `SupabaseClient::test_connection()` (`src-tauri/src/modules/supabase_client/client.rs:126`) — a three-probe ladder: (1) `GET {base}/rest/v1/`, (2) table probes `GET /rest/v1/{nodes,command_queue}?limit=0`, (3) `GET /auth/v1/health`. Transport failures are classified in `explain_transport_error` (`client.rs:87-111`).
2. **Reviewed the ladder for in-repo defects that could fail on a healthy project**: none found. Probe 1 correctly falls through on the publishable-key 404 ("Secret API key required" OpenAPI block) to the table probes; Probe 2 returns success on PGRST204/205-style "schema cache / does not exist" bodies ("run Schema Migration to create tables"); 401/403 returns an explicit auth-failure message. `normalize_supabase_url` (`src-tauri/src/modules/supabase_client/types.rs:75-97`) rewrites the known user typo `pezjuuddecbyrmqxytrv` → `pezjuuddecbyfmqxytrv`, which is a no-op on the correct URL and matches the documented real project ref. Seed/discover logic purges mock/placeholder endpoints and dedupes by normalized URL (shipped v4.198.0). No silent-error violations on this path (worker propagates with `?`; UI `testEndpoint` never rejects — it returns `is_success: false` on IPC failure).
3. **Attempted live reproduction** from this sandbox: direct HTTPS probe to `https://pezjuuddecbyfmqxytrv.supabase.co/rest/v1/` timed out (curl exit 28 after 20 s) — outbound egress is blocked/intercepted here, so a live repro was impossible from this machine.
4. **Used the recorded evidence from the user's own machine** (2026-10-14 E2E run, `.ai-memory/memory/learned/27-e2e-instance-autoswitch-prompt-playbook.md` §3.7, §8 case D3, result block): local config holds exactly the two real endpoints, and **both hosts returned DNS NXDOMAIN while `supabase.co` itself resolved** — recorded as `DNS: CLOUD PAUSED (NXDOMAIN)`. Playbook rule 8 codifies this: "Supabase NXDOMAIN is a paused cloud project, not a reason to invent a new project." Same conclusion in `.ai-memory/issues/63-e2e-prompt-tree-supabase-email-and-autoswitch-rca.md` item 5.

## Fix

**No in-repo code fix.** The code already diagnoses this exact condition and prints the correct remediation. Changing code (e.g. retry logic or a different endpoint) cannot fix a paused cloud project, and creating a replacement project is explicitly forbidden by the standing playbook rule.

**User action required** (exact steps):

1. Open the Supabase dashboard (supabase.com → Dashboard) and select the project whose ref is `pezjuuddecbyfmqxytrv` (the Lovable-linked root project). If it was created via Lovable, open the Lovable project dashboard instead. Click **Restore / Unpause** (free-tier paused projects show a "Paused" banner with a restore button).
2. Repeat for the secondary project `ikwmurmjynhxdpmhzekt`.
3. Wait 2–5 minutes for Supabase to republish the DNS names.
4. In AGM: Settings → Supabase → press **Test** on each endpoint. Expect green "Connected successfully (…)" and the badge flips from "Unreachable" back to "Active". Or run `agm supabase test` and expect `PASS (HTTP 200)`.

If the local config on the user's machine still shows mock URLs (`mock-root.supabase.co` / `mock-secondary.supabase.co`) or the old `pezjuuddecbyrmqxytrv` spelling, press **Auto-Discover from Repo Secrets** in the Supabase page (or `agm supabase status` to inspect) before testing — do NOT create a new Supabase project to "fix" the NXDOMAIN.

## Verification

- Could not verify live from this sandbox (egress blocked). Verification is the user's steps 4 above: after unpausing, the app's own diagnostics (`test_connection()` probe ladder via the UI Test button or `agm supabase test`) are the check — expected result: `is_success: true`, message starting "Connected successfully", badge "Active".
- Verified at code level that for an unpaused project with a valid key and migrated tables, the ladder returns success at Probe 1 or Probe 2 (reviewed `client.rs:126-290`); no code path can report success on a still-paused project.

## Prevention

- The app already surfaces the exact remediation in the failure message itself ("Unpause the project in the Supabase or Lovable dashboard") — no UI copy change needed.
- Recurrence note: free-tier projects re-pause after ~7 days of inactivity. If fleet sync must stay up unattended, the durable fix is moving the two projects to a paid Supabase tier (or a scheduled wake-up, e.g. periodic heartbeat keeps it warm — note: heartbeats only help while the project is already awake; they do not unpause it).
- Standing rule, do not regress: **never create a new Supabase project or write a new project ref to fix NXDOMAIN** (playbook rule 8). NXDOMAIN + `supabase.co` resolving = paused project, full stop.

## Files referenced (no files changed)

- `src-tauri/src/modules/supabase_client/client.rs` — `test_connection` (line 126), `explain_transport_error` (lines 87-111)
- `src-tauri/src/modules/supabase_client/types.rs` — `normalize_supabase_url` (lines 75-97)
- `src-tauri/src/commands/supabase.rs` — `test_supabase_endpoint` (line 49)
- `src-tauri/src/modules/cli/supabase.rs` — `agm supabase test` subcommand
- `src/components/settings/SupabaseSync/useSupabaseSyncHandlers.ts` — `handleTestEndpoint`
- `src/components/settings/SupabaseSync/EndpointsSection.tsx` — Unreachable badge
- `.ai-memory/memory/learned/27-e2e-instance-autoswitch-prompt-playbook.md` — prior evidence (CLOUD PAUSED)
