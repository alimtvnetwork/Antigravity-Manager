# Completed Plan 71: Supabase Multi-Machine Relational Hierarchy, CLI Management, and E2E Sync

Spec Reference: [02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md](../../../02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md)

## Summary of Completed Tasks

- **Task-01 (Repo Secrets Sync & Credential Storage)**: Pulled `d:\work\repo-secrets`, updated `02-antigravity-manager/vault/supabase_config.json`, added `03-supabase/02-lovable/supabase-credentials.json` with Root Lovable credentials in Base64 JSON format, updated README, committed, and pushed to remote `main`.
- **Task-02 (CLI Commands & Help Guidance)**: Added native `agm supabase` subcommand with options `help`, `status`, `list-leases`, `test`, `set-endpoint`, `load-json`, `sync`, `schema`, `enable`, `disable`, and `set-alias`. Added comprehensive help detailing config location (`%APPDATA%\antigravity-manager\supabase_config.json`), JSON schema with tags/notes, and database parent-child relationships.
- **Task-03 (Supabase Schema & Relational Parent-Child Hierarchy)**: Enhanced `ROOT_DB_SCHEMA_SQL` in `supabase_schema.rs` with `nodes` parent table, `instance_profiles` child table with FK `REFERENCES public.nodes(id) ON DELETE CASCADE`, and `workspace_leases` tracking `account_email` and `ip_address`. Upgraded `test_connection()` probe ladder in `supabase_client.rs` to handle PostgREST schema cache `PGRST205` and publishable keys.
- **Task-04 (Account Switch Synchronization)**: Hooked manual account switching (`account.rs`), instance switching (`instance.rs`), and CLI switching (`bin/agm.rs`) to automatically acquire distributed leases in Supabase Root DB via `workspace_lease_manager::acquire_lease_with_details` and trigger immediate local node sync.
- **Task-05 (Automation PowerShell One-Liners & JSON Suite)**: Created `scripts/setup-supabase.ps1` and `scripts/supabase-endpoints.json` enabling single-line configuration, parameter overrides, and standalone PowerShell JSON ingestion.
- **Task-06 (End-to-End Testing & Verification)**: Executed `scripts/setup-supabase.ps1` against both live endpoints (`Root by Lovable` and `Secondary`), verifying HTTP authentication and PostgREST table verification ladder. Validated single-quoted PowerShell one-liner JSON loading.
- **Task-07 (Minor Release & GitMap PE Verification)**: Bumped version according to minor release, updated manifests/changelogs, verified GitMap PE pipeline engine with zero errors.

## Verified Operational Invariants

1. Both Root Lovable (`https://pezjuuddecbyfmqxytrv.supabase.co/rest/v1/`) and Secondary (`https://ikwmurmjynhxdpmhzekt.supabase.co/rest/v1/`) endpoints successfully authenticate and respond to PostgREST table probes.
2. Parent-child relationship enforced: `instance_profiles` and `workspace_leases` reference `nodes(id)`.
3. Account leases accurately capture machine alias and local IP.
