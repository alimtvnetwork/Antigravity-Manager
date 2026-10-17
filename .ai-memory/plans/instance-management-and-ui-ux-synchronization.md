# Plan: instance-management-and-ui-ux-synchronization

Status: COMPLETED
Branch: main
Release: v4.217.0
Completed At: 2026-10-17

## User Request (Verbatim)

# Instance Management and UI/UX Synchronization: high priority instruction, non-negotiable task

Also, one more serious issue. If we click on any one of the items, instance and audit trail, we don't see any audit trail. That looks really, really terrible. I think you need to work on it. And at the end, you do the end-to-end testing for auto switcher, stopping the PID with a new instance, actually. You create your new instance. So in this machine, you can run anything, build anything, test anything. Okay? Do not break your own stuff, which instance you are using, try to find that as well. So you need to have this type of CLI commands that would detect that from the CLI that whatever the IDE, if the IDE is running the CLI command, IDE will automatically figure out what is the instance that it is running the command, and it should not break it. Remember to work like that. And always, it should actually create new instance to test and also remove the existing test instances so that it could reduce some space and make sure removing and creating the instance, it should be very efficient. Okay? And also the database should have the updated information. Remember that. Is it understood? At the end, you should do a minor bump and release with everything

## slug: instance-management-and-ui-ux-synchronization

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/instance-management-and-ui-ux-synchronization/ and enqueue plan task in .ai-memory/plans/instance-management-and-ui-ux-synchronization.md (subtasks in .ai-memory/plans/subtasks/instance-management-and-ui-ux-synchronization/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well

9. Improve UI/UX consistency in the switch account dialog.
10. Implement audit trail visibility for instances.
11. Conduct end-to-end testing for auto switcher and PID management.
12. Develop CLI commands to detect and manage instances without breaking existing setups.
13. Ensure efficient creation and removal of test instances.
14. Update the database with current information.
15. Perform a minor version bump and release upon completion.

## Media & Visual Assets

![Prompt Tree & Export Menu](assets/screenshots/instance-management-and-ui-ux-synchronization-01.png)
![IDE Conversation Execution](assets/screenshots/instance-management-and-ui-ux-synchronization-02.png)

## Root Cause Analysis & Problem Classification

1. **Audit Trail Missing Visibility**:
   - When clicking on an instance or viewing the instance audit trail, records appear empty.
   - Root causes: The frontend `InstanceAuditTrailModal.tsx` or `AuditPanel.tsx` queries `get_task_history` or `get_instance_audit_trail` with specific filters (`instance_id` or action codes) that either fail to match the composite ID format (`default`, `default-copy-NNNN`) or the backend task history split database stores entries under actions (`SWITCH_ACCOUNT`, `BACKUP_PROMPTS`) where the instance field is unpopulated or mismatched.
   - Fix: Unify query key mapping, fallback to full recent history for the selected instance, ensure payload JSON is parsed cleanly, and verify IPC returns non-empty result sets.

2. **Switch Account Dialog UI/UX Consistency**:
   - Layout, button heights, hover states, and quota displays in `SwitchAccountDialog.tsx` must align with the dark-glass pill styling system.
   - Quota sorting: Highest available quota (4h or weekly headroom) must appear first.

3. **Active Caller Instance Detection in CLI**:
   - When running CLI commands like `agm switch`, `agm instances rm`, or auto-switch from within an IDE terminal, the CLI must automatically detect the parent IDE process tree (checking parent PIDs and known instance directory matches).
   - Once identified, the active caller instance is marked as protected (`is_caller_protected = true`) and cannot be terminated or mutated by the test/switch commands.

4. **Efficient Test Instance Lifecycle**:
   - New commands: `agm instances create-test` and `agm instances rm-test` (or `--test` flags) that provision data-only lightweight sandbox instances, test auto-switching and process termination safely, and purge all temporary files and database entries cleanly upon exit.

5. **End-to-End Verification**:
   - Run automated verification testing the complete auto-switcher lifecycle with a dedicated test instance, proving the target PID is stopped and switched while the caller IDE remains completely uninterrupted.

## Subtask Breakdown

- `01-audit-trail-visibility.md`: Fix instance audit trail loading, query handling, and UI table display in `InstanceAuditTrailModal` and `AuditPanel`.
- `02-switch-dialog-ui-ux.md`: Modernize `SwitchAccountDialog` with consistent styling, quota sorting, and hover effects.
- `03-cli-caller-detection-and-safety.md`: Implement CLI caller instance detection (`resolve_caller_instance`) to protect running IDE instances from suicide kills.
- `04-test-instance-lifecycle-and-e2e.md`: Implement efficient test instance creation/removal (`create-test`, `rm-test`) and author automated E2E script.
- `05-database-sync-and-release.md`: Audit SQLite database consistency, pre-flight checks, minor bump, and release notes.
