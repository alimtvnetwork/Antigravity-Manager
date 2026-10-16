# Parent Plan: Instance Management and UI/UX Synchronization

**Slug**: `instance-management-and-ui-ux-synchronization`  
**Ledger Reference**: `02-spec/21-app/instance-management-and-ui-ux-synchronization/00-master-audit-ledger.md`  
**Execution Protocol**: `execute-parent-task-with-n-steps-v6` ($N=300, A=2, H=2, C=30$)

---

## Subtask Inventory

1. **Subtask 01 (`.ai-memory/plans/subtasks/instance-management-and-ui-ux-synchronization/01-pid-display-and-path-alignment.md`)**:
   - Confirm and label `PID` in Status badge (`Running · PID 1632`).
   - Align ending part of executable name and data path in `InstanceTableRow.tsx`.
2. **Subtask 02 (`.ai-memory/plans/subtasks/instance-management-and-ui-ux-synchronization/02-stop-instance-and-switch-kill-fix.md`)**:
   - Fix `close_instance` in `close.rs` to reliably terminate running PIDs on Windows (`taskkill /F /T /PID`).
   - Fix account switch pre-teardown to ensure the old PID is killed before credential rewrite.
3. **Subtask 03 (`.ai-memory/plans/subtasks/instance-management-and-ui-ux-synchronization/03-switch-account-dialog-redesign.md`)**:
   - Overhaul `SwitchAccountModal.tsx` into Royal Violet dark-glass theme with modern card layouts and emerald badges.
4. **Subtask 04 (`.ai-memory/plans/subtasks/instance-management-and-ui-ux-synchronization/04-instance-audit-trail-visibility.md`)**:
   - Ensure switch audit records are persisted in `task_history_db` and queryable via `get_instance_switch_history` IPC.
5. **Subtask 05 (`.ai-memory/plans/subtasks/instance-management-and-ui-ux-synchronization/05-test-timeout-and-self-instance-protection.md`)**:
   - Add strict 30s timeouts to all steps in `scripts/test-instance-e2e.ps1`.
   - Add caller self-instance detection (`agm instances current` / `which-self`) to protect active developer IDE.
6. **Subtask 06 (`.ai-memory/plans/subtasks/instance-management-and-ui-ux-synchronization/06-e2e-verification-bump-and-release.md`)**:
   - Run end-to-end multi-instance tests with timeouts.
   - Minor bump from the current version, update changelogs/READMEs (`@aukgit`), commit, and push the tag.
