# Parent Plan: Instance Management and UI/UX Synchronization

## User Request (Verbatim)
```text
# Instance Management and UI/UX Synchronization: high priority instruction, non-negotiable task

I have given you this screenshot where if we go into the list view, we should see the ending part of the exe, and that should be all together in the same line with the data ending part. And also the running status 1632, what does this mean? If it is running, then I think the green symbol is fine. We can just mention 1632. But what is this 1632? Is it the PID? If PID, please confirm. I think that is what it is. Okay. So, the first thing is the synchronization is very wrong. The switching of the accounts is not working at all somehow. So if the PID is not found, does not match, then first of all, it needs to close the PID in order to reload or change or switch account. Okay? So this is not happening at the moment. And also stop the instance should also kill the instance, which is not happening as well. Stop instance button is not working. Okay? I think you need to be careful on this. And also if I go to the switch account option, which actually shows dialogue, which actually contains the four UI UX, which is not in line with the other UI UX. You need to improve that a lot. Is it clear so far? Can you please help me with this?

Also, one more serious issue. If we click on any one of the items, instance and audit trail, we don't see any audit trail. That looks really, really terrible. I think you need to work on it. And at the end, you do the end-to-end testing for auto switcher, stopping the PID with a new instance, actually. You create your new instance. So in this machine, you can run anything, build anything, test anything. Okay? Do not break your own stuff, which instance you are using, try to find that as well. So you need to have this type of CLI commands that would detect that from the CLI that whatever the IDE, if the IDE is running the CLI command, IDE will automatically figure out what is the instance that it is running the command, and it should not break it. Remember to work like that. And always, it should actually create new instance to test and also remove the existing test instances so that it could reduce some space and make sure removing and creating the instance, it should be very efficient. Okay? And also the database should have the updated information. Remember that. Is it understood? At the end, you should do a minor bump and release with everything
```

## Strategic Intent & Systems Reasoning
This plan establishes deterministic process termination, robust pre-switch PID eviction, caller IDE self-preservation, Royal Violet UI/UX alignment, full instance audit trail visibility, and transient test instance sandboxing in Antigravity-Manager.

## Subtask Breakdown

| Task-ID | Subtask Name | Owner | Status | Deliverables |
| :--- | :--- | :--- | :--- | :--- |
| **Task-01** | Spec Authoring & Master Plan Initialization | Lead | `DONE` | Specs in `02-spec/21-app/instance-management-and-ui-ux-synchronization/`, plan and subtasks in `.ai-memory/plans/` |
| **Task-02** | PID Layout & List View Single-Row UI Refactor | Worker 01 | `PENDING` | `InstanceTable.tsx`, `InstanceRowCells.tsx`, single-row layout |
| **Task-03** | Deterministic Process Termination & Stop Instance Fix | Worker 02 | `PENDING` | `close.rs`, `create.rs`, 5-stage termination ladder |
| **Task-04** | Account Switch Synchronization & Pre-Switch Eviction | Worker 02 | `PENDING` | `switch.rs`, lockfile clearing, settle delay |
| **Task-05** | Switch Account Modal Royal Violet UI/UX Modernization | Worker 01 | `PENDING` | `SwitchAccountDialog.tsx`, token compliance, whole-card selection |
| **Task-06** | Instance Audit Trail Logging & UI Modal Visibility | Worker 03 | `PENDING` | `task_history_db`, `InstanceAuditTrailModal.tsx`, dual-tab fallback |
| **Task-07** | Caller Preservation, Transient Test CLI & E2E Testing | Worker 03 | `PENDING` | Ancestor walk, `refuse_if_caller`, `create-test`/`rm-test`, E2E test script |
| **Task-08** | Minor Version Bump & Release Ceremony | Lead | `PENDING` | Version bump, changelogs, README sync, atomic GitMap push |
