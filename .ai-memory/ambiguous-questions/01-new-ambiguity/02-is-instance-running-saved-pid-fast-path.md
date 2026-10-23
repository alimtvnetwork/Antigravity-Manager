# 02 - Should `is_instance_running` trust a matching saved PID without a process scan?

Slug: is-instance-running-saved-pid-fast-path
Status: resolved 2026-10-23
Blocking: any further change to `src-tauri/src/modules/instance.rs` `is_instance_running` or the PID cache cadence

## Resolution (user decision, 2026-10-23)

> Trust but always verify before execution. If mismatch full rescan

Audit of the current implementation (`src-tauri/src/modules/instance/pid_util.rs`, `is_instance_running`) against that rule — the code already conforms, no change needed:

- Trust: the saved PID (config or stored) is the first candidate.
- Always verify before the verdict: the saved PID must pass `saved_pid_matches` (that one process is alive and is Antigravity) AND `live_pid_is_instance_window` (that one process is this instance's window — clone-executable marker, `/instances/{id}` argument, or data-dir match; for the default instance, a plain stock main window that is not a copy). The Windows fallback for processes whose arguments cannot be read still verifies identity and refuses PIDs protected by another instance (`should_spare_pid`).
- Mismatch → full rescan: any verification failure falls through to `first_live_window_pid` (full data-dir scan plus the cached process list), and the PID found by the scan is recorded as the new saved PID.
- No blind-trust path exists elsewhere: every other `saved_pid_matches` caller pairs it with a liveness check or the same window verification.

Note: this file's premise (a full scan on every call at the old `instance.rs:2584`) predates the `pid_util.rs` refactor; the scan-on-every-call behavior it describes no longer exists.

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
