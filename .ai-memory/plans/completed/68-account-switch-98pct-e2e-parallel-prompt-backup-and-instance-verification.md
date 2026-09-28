# Plan 68: Account Switch 98% Simulation E2E, Parallel Prompt Backup/Restore, Multi-VM Collision Shielding & Sandbox Lifecycle Verification

Spec Reference: [02-spec/21-app/68-account-switch-98pct-e2e-parallel-prompt-backup-and-instance-verification.md](../../../02-spec/21-app/68-account-switch-98pct-e2e-parallel-prompt-backup-and-instance-verification.md)

## Summary
Orchestrate and execute end-to-end verification of account auto-switching at a 98% simulation threshold with live pre-activation refresh validation, cross-VM collision avoidance using multi-channel leases (Supabase and Inbound Email), high-speed parallel backup of running prompts into SQLite with image/metadata fidelity, Fast-Forward UI button delegation, post-switch prompt re-injection and liveness verification with human-readable Telegram & Email alerts, sandbox instance lifecycle verification, CLI help polish, and reversion to the production 15% threshold before final minor release.

## Deliverables Mapping
- **Task-01**: Author canonical specification 68 and register in spec indices.
- **Task-02**: Account switch algorithm 98% simulation threshold, pre-switch refresh verification, and multi-channel collision check (Supabase + Email).
- **Task-03**: Parallel running prompts backup and restore to SQLite engine (`agm prompts backup/restore`).
- **Task-04**: Post-switch re-injection and process liveness verification with Telegram and Email broadcast.
- **Task-05**: Multi-instance mode lifecycle and sandbox verification (create, test, cleanup).
- **Task-06**: Help text and usage examples polish across all AGM CLI commands.
- **Task-07**: Default threshold reversion to 15%, minor release ceremony bump, and GitMap PE verification.
