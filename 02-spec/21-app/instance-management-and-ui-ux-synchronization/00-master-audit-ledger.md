# Master Audit Ledger: Instance Management and UI/UX Synchronization

**Slug**: `instance-management-and-ui-ux-synchronization`  
**Created**: 2026-10-16  
**Status**: IN_PROGRESS  

---

## 1. User Request (Verbatim)

```text
# Instance Management and UI/UX Synchronization: high priority instruction, non-negotiable task

I have given you this screenshot where if we go into the list view, we should see the ending part of the exe, and that should be all together in the same line with the data ending part. And also the running status 1632, what does this mean? If it is running, then I think the green symbol is fine. We can just mention 1632. But what is this 1632? Is it the PID? If PID, please confirm. I think that is what it is. Okay. So, the first thing is the synchronization is very wrong. The switching of the accounts is not working at all somehow. So if the PID is not found, does not match, then first of all, it needs to close the PID in order to reload or change or switch account. Okay? So this is not happening at the moment. And also stop the instance should also kill the instance, which is not happening as well. Stop instance button is not working. Okay? I think you need to be careful on this. And also if I go to the switch account option, which actually shows dialogue, which actually contains the four UI UX, which is not in line with the other UI UX. You need to improve that a lot. Is it clear so far? Can you please help me with this?

Also, one more serious issue. If we click on any one of the items, instance and audit trail, we don't see any audit trail. That looks really, really terrible. I think you need to work on it. And at the end, you do the end-to-end testing for auto switcher, stopping the PID with a new instance, actually. You create your new instance. So in this machine, you can run anything, build anything, test anything. Okay? Do not break your own stuff, which instance you are using, try to find that as well. So you need to have this type of CLI commands that would detect that from the CLI that whatever the IDE, if the IDE is running the CLI command, IDE will automatically figure out what is the instance that it is running the command, and it should not break it. Remember to work like that. And always, it should actually create new instance to test and also remove the existing test instances so that it could reduce some space and make sure removing and creating the instance, it should be very efficient. Okay? And also the database should have the updated information. Remember that. Is it understood? At the end, you should do a minor bump and release with everything

You need to fix test with timeout please ,cannot run test for hours and hours, this stupidty
```

---

## 2. Screenshot & Evidence References

- `assets/screenshots/instance-management-and-ui-ux-synchronization-01.png`: List view showing Status & PID `Running - 1632`, `FILE / DATA PATH`, and `EXECUTABLE PATH`.
- `assets/screenshots/instance-management-and-ui-ux-synchronization-02.png`: Multi-panel image showing stop instance button hover failure, switch account modal styling discrepancy, and empty audit trail modal.
- `assets/screenshots/instance-management-and-ui-ux-synchronization-03-test-hang.png`: PowerShell test hanging indefinitely at `Closing Instance A based on folder path ... and PID (2792)...`.

---

## 3. Discrete Deliverables & Action Items

- **Task-01: PID Clarification & List View Path Alignment**
  - Clarify `1632` is the OS Process ID (`PID`). Label clearly as `Running · PID 1632` with tooltip.
  - In `InstanceTable.tsx` / `InstanceTableRow.tsx`, align the ending portion of the executable file name (`Antigravity-default-copy-5597.exe`) and data path (`.../data`) cleanly in the same line without awkward breaks.
- **Task-02: Stop Instance Process Killing & Switch Account PID Teardown**
  - Fix `close_instance` in `src-tauri/src/modules/instance/close.rs`: ensure process termination does not falsely exit early when `pids.is_empty()` due to strict data_dir filters. If `config.pid` or cached PID is alive, kill it reliably via `taskkill /F /T /PID <pid>`.
  - Ensure account switching always tears down the active PID before injecting credentials and relaunching.
- **Task-03: Switch Account Dialog UI/UX Modernization**
  - Redesign `SwitchAccountModal.tsx` / `AccountSwitchDialog.tsx` to strictly align with the Royal Violet dark-glass design system (`rounded-2xl`, glass backdrop, high-contrast typography, emerald quota accents, borderless actions).
- **Task-04: Instance Switch Audit Trail Persistence & Visibility**
  - Fix `InstanceAuditModal.tsx` and `task_history_db`: ensure account switch events and rotation steps are persisted into `task_history.db` and properly retrieved by `instance_id`.
- **Task-05: Test Suite Timeout Hardening (`scripts/test-instance-e2e.ps1`)**
  - Implement `Invoke-AgmWithTimeout` with strict 30-second bounded execution on all CLI and helper scripts to eliminate hangs.
- **Task-06: Self-Instance Detection CLI & Safe Test Sandboxing**
  - Implement `agm instances which-self` / `detect-caller-instance` detecting the parent process tree so tests never kill the operator's active IDE instance.
  - Ensure fast creation and cleanup of test sandboxes.
- **Task-07: Version Bump & Release Ceremony**
  - Bump minor version to `4.209.0`, update changelogs/READMEs strictly attributing `@aukgit`, commit via `gitmap cpf`, tag `v4.209.0`, and push.
