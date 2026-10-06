# 02 - Should `is_instance_running` trust a matching saved PID without a process scan?

Slug: is-instance-running-saved-pid-fast-path
Status: open
Raised: 2026-10-06
Blocking: any further change to `src-tauri/src/modules/instance.rs` `is_instance_running` or the PID cache cadence

## Original request (verbatim intent, 2026-10-02 session)

> Trust the PID saved at launch when its path matches. Run a PID cache refresh every 10 minutes by default, with a minimum of 3 minutes.

## Context

- v4.124.0 and `366d84e5` implemented: saved PID identity match returns true with no scan; scan only on a miss, then save the real PID.
- `a0c5cc18` and `3581207d` (v4.155.0, by a later session) changed the matching branch to call `find_pids_for_data_dir(data_dir, is_default)` and require `pids.contains(&pid)`. Current code: `src-tauri/src/modules/instance.rs:2584`.
- Those commits were part of cross-instance running detection fixes (plans 120, 123, 135), so the extra scan may be a deliberate guard against PID reuse by another instance.

## Ambiguity

The current code scans the process table on every call, even when the saved PID matches. That conflicts with the user's "trust the saved PID" directive but may fix a real cross-instance false positive.

## Options

### Option A - Keep the scan on every call

- Pros: Guards against a recycled PID that now belongs to another instance.
- Cons: Restores the CPU cost the user asked to remove; the 10 minute refresh becomes moot for this path.

### Option B - Restore the trust-only fast path

- Pros: Matches the user directive; cheapest.
- Cons: A reused PID could report the wrong instance as running until the next refresh.

### Option C - Trust the saved PID only if its command line contains this instance's marker

- Pros: `process_identity_matches` / `should_spare_pid` markers (`/instances/{id}/`, `antigravity-{id}`, data_dir) check one process, so no full scan and no cross-instance false positive.
- Cons: Default instance has no marker in its args on some platforms.

## Recommendation (not applied)

Option C, falling back to the scan only for the default instance. Needs the user's decision before any code change.
