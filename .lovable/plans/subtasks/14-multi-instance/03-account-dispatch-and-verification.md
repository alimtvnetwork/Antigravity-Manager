# Subtask 03: Account Row Instance Dispatch & Cross-Platform Verification

> **Subtask Path:** `.lovable/plans/subtasks/14-multi-instance/03-account-dispatch-and-verification.md`
> **Parent Plan:** [.lovable/plans/pending/14-multi-instance-orchestration-and-ubuntu-parallelism.md](../../pending/14-multi-instance-orchestration-and-ubuntu-parallelism.md)
> **Status:** Pending Review

---

## 1. Objectives

1. Update `src/components/accounts/AccountRow.tsx`:
   - Enhance the switch button (`⇄`) action:
     - Primary click: switches/opens this account in the currently selected active instance.
     - Secondary menu (or hover/split dropdown): "Open in Specific Instance..." or "Open in New Cloned Instance".
2. Bind account switching to `launch_in_instance` IPC command rather than the legacy single-instance `switch_account`.
3. Verify cross-platform stability on Windows and Ubuntu/Linux:
   - Ensure two instances can run simultaneously with different email addresses.
   - Verify that closing one instance on Ubuntu does not kill other instances.
   - Verify `--password-store="basic"` prevents credential collision on GNOME Keyring.

---

## 2. Acceptance Criteria

- [ ] AccountRow switch button dispatches to the selected instance without killing existing windows.
- [ ] Multiple Antigravity windows can be open simultaneously on both Windows and Ubuntu.
- [ ] No regression in single-instance mode (Default instance continues to function normally).
