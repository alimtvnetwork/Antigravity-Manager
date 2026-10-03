# Master Plan: Task 104 - Instance Audit Trail & Live PID Quota Synchronization

## Executive Summary
This plan guides the implementation of:
1. Instance action toolbar "Audit" and "Sync" options in `src/pages/Instances.tsx`.
2. Dedicated `InstanceAuditTrailModal.tsx` showing switch frequency, from/to account switches, and the 4 verified steps (Backup -> Reset -> Restore -> Verify).
3. Backend multi-step tracking instrumentation during account switching (`instance.rs`, `auto_switcher.rs`, `task_history_db.rs`).
4. Live PID detection and active account credential synchronization with Google Gemini quota refreshing on-demand and on a 10-minute periodic scheduler (`scheduler.rs`).
5. Minor SemVer bump to `v4.129.0`, pre-flight gates, and release ceremony.

## User Request (Verbatim)
```text
# High Priority Instruction

Please add here the audit option. Audit trail will show us how the switch actually happened, how many times the switch happened, how the switch happened, from which to which account, all these things I should be able to see with the audit. So I should be able to click on it. I should be able to see at least two, three accounts switch. Okay? And every step, like prompts backup for which projects, how it's going to reset, how it's going to restore the prompts. Okay? And checking after restoring that the prompts were really restored, things like that. And also, I want to have a sync button. So there could be multiple instances active. That's the first thing. Once it's active, in every 10 minutes, it would sync with the PID, whatever the account is currently running. Okay? So based on that, it would also sync the credits and everything else. Do you understand? And there should be a sync button where user can click on sync to see whatever the latest account it is using and whatever the credit that is there. Do you understand? Can you please apply these changes and finally make a minor bump and release?

# Actionable Items Must Follow Non-Negotiable

1. Write a plan and spec first 
2. Implement an audit option to track switch details and account transitions.
3. Ensure the audit trail is clickable and displays at least two to three account switches.
4. Develop a system for prompts backup, reset, and restore verification.
5. Add a sync button for user-initiated synchronization of account and credit details.
6. Implement automatic synchronization every 10 minutes with the current account's PID.
7. Apply changes and perform a minor version bump for release.

Must follow and spawn agent using 

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

learn /learn  if you have to learn something and /plan stuff before working please.

release a minor bump please
```

## Subtasks Breakdown
- **Subtask 01**: Backend Multi-Step Lifecycle Instrumentation & Audit Trail Queries
  - Track Backup, Reset, Restore, and Verify steps.
  - Implement `get_instance_switch_history` and `get_instance_audit_trail` Tauri commands.
- **Subtask 02**: Live PID Detection, Account Discovery & 10-Minute Quota Sync Engine
  - Implement `sync_instance_pid_and_quota`.
  - Add 10-minute periodic sync loop in `scheduler.rs`.
- **Subtask 03**: Frontend Instance Action Toolbar Integration & Audit Trail Modal
  - Add "Audit" and "Sync" buttons in `Instances.tsx`.
  - Create `InstanceAuditTrailModal.tsx` with expandable steppers and switch metrics.
  - Connect store actions in `useInstanceStore.ts`.
- **Subtask 04**: End-to-End Verification, Pre-flight Checks & Minor Release Ceremony
  - Verify local E2E tests, cargo fmt, clippy, frontend build.
  - Run `npm run bump minor` -> v4.129.0.
  - Tag, push, and monitor CI/CD pipeline.
