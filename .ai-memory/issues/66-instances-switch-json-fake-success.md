# Issue 66: `agm instances switch --json` reports switched without switching

Status: FIXED in v4.278.0 (`68622ceef`)
Raised: 2026-10-29
Related: `.ai-memory/memory/learned/31-prompt-tree-copy-skill-refs-telegram-e2e-and-json-switch.md`

## 1. Symptom

`agm instances switch <sandbox> <email> --json` printed `"success": true` and `"status": "switched"`. `agm instances ls --json` still showed the previous `bound_email`.

Live: sandbox started on `rokixshohag1@gmail.com`. JSON switch to `david.copper.riseup@gmail.com` claimed switched. List still showed `rokixshohag1@gmail.com`.

The non-JSON command on the same ids then actually switched, backed up prompt `queued-1793282428503-0000-2837d0`, restored FIFO, and dispatched.

## 2. Trigger

Any `agm instances switch` (or `agm switch`) invoked with `--json` / `-j` when the local REST daemon path did not return `data`.

## 3. Root Cause

In `src-tauri/src/bin/agm/instance_cmds.rs` the JSON branch:

1. Optionally POSTed `/instances/{id}/switch` if a daemon was running.
2. On miss, built `{ instance_id, account, status: "switched" }` and `print_and_exit`.
3. Never reached `instance_dispatch::cmd_instances`, which is the path that calls `switch_account_to_instance`.

That was a fake success. House rule: a failed or skipped op must not report success.

## 4. Fix (shipped v4.278.0)

JSON switch forwards to `route_instances_switch` / `switch_account_to_instance`, then reads `bound_email` from the registry. If the email did not change, `success: false` and exit 1. Isolated E2E: `agm instances switch <sandbox> definitely-not-an-account@example.invalid --json` returned `success: false`.

## 5. E2E rule

Prove a switch with `agm instances ls` `bound_email` (and PID if launched). Never treat `--json` `"status": "switched"` as evidence.
