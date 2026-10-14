# Supabase test failure

Checked with `agm supabase status` on the binary that contains the endpoint loader fix.

## What is healthy

- Sync is enabled.
- Two endpoints, both enabled: `ep-root-lovable-01` and `ep-secondary-01`.
- Hosts are `pezjuuddecbyfmqxytrv.supabase.co` and `ikwmurmjynhxdpmhzekt.supabase.co`.
- No mock URL and no `rmq` typo in the status table.

## What fails

`nslookup` against the machine DNS server:

- both project hosts: `Non-existent domain`
- `supabase.co`: resolves

The lease query then fails while connecting. The API key is not read, because the host never resolves.

## Cause

Free-tier Supabase projects drop the `<project_ref>.supabase.co` DNS name when the project is paused. The manager config cannot unpause a cloud project. Restore each project in the Supabase or Lovable dashboard. After the names resolve, run `agm supabase test` again.

## What changed in the message

`select`, `insert`, `upsert`, `update`, `delete`, and `rpc` used to surface the raw client string `error sending request for url`. They now use the same explanation as Test: connect failure means the project DNS is missing or the connection was refused, and the key was not the failure. Status prints that text for the lease query.
