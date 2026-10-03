# Specification 104: Instance Audit Trail & Live PID Quota Synchronization

## 1. Overview & Goal

This specification defines the architecture, database schema, IPC contracts, and user interface for:
1. **Instance Action Toolbar Audit Option**: A dedicated "Audit" button in each instance card action capsule (alongside `Stop`, `Switch`, `FF`, `Clone`), opening a comprehensive, interactive **Instance Audit Trail Modal**.
2. **Switch History & Multi-Step Lifecycle Tracking**: Tracking how many times the switch happened, from which account to which account (prominently showing at least the last 2–3 switches), with an expandable 4-step lifecycle:
   - **Step 1: Prompts Backup**: Projects captured, conversation IDs, enqueued prompt counts, backup target path.
   - **Step 2: Account Reset**: Process termination (PID), state database clearing, new account authentication swap.
   - **Step 3: Prompts Restore**: Injection into destination workspace / `.antigravity_resume_task.json`.
   - **Step 4: Post-Restore Verification**: Explicit verification confirming prompt files and active session restoration succeeded.
3. **On-Demand & 10-Minute Periodic PID Quota Synchronization**:
   - A dedicated **Sync** button in the instance card action capsule and top toolbar.
   - On click, it detects the live running PID, inspects the active account authenticated in the instance's profile storage (`data_dir` / keychain / auth state), updates `bound_account_id` and `bound_email`, and queries Google Gemini API to synchronize credits and quota.
   - A 10-minute periodic background synchronization in the scheduler for all active instances.
4. **Minor Release Ceremony**: Minor version bump (`v4.129.0`), changelog synchronization, and GitHub release deployment.

## 2. User Request (Verbatim)

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

## 3. UI Visual Design & Layout Placement

Reference Screenshot: `assets/screenshots/104-instance-audit-trail-and-pid-quota-sync-01.png`

```
+-----------------------------------------------------------------------------------------+
| #4  v3-3334   Set Default                                               Active Target  |
|                                                                                         |
| Account:  mostafiz.hera.bootcamp.riseup@gmail.com                               [PRO]   |
| Gemini 3.1 Pro (High)                                                [4h 43m] [ 92% ]   |
| [=============================================================================]         |
| Status: Running (PID: 15720)                                      Profile ID: v3-3334   |
| Path: C:\Users\Administrator\.antigravity_tools\instances\v3-3334\data                  |
| Exec: C:\Users\Administrator\AppData\Local\Programs\antigravity\Antigravity-v3-3334.exe |
|                                                                                         |
| [ Stop | Switch | FF | Clone | Audit | Sync | Gear | Trash | Rotate ]                  |
+-----------------------------------------------------------------------------------------+
```

### Action Pill Capsule Integration
In `src/pages/Instances.tsx`, update the bottom action button row to include:
- `Audit`: Opens `<InstanceAuditTrailModal instance={inst} isOpen={...} onClose={...} />`.
- `Sync`: Calls `syncInstancePidAndQuota(inst.id)`, showing an animated spin on the icon while active.

## 4. Multi-Step Lifecycle Architecture

### Data Contract: `InstanceSwitchRecord`
```typescript
export interface SwitchStepLog {
  step_number: number;
  step_name: 'backup_prompts' | 'reset_process' | 'restore_prompts' | 'verify_restored';
  title: string;
  status: 'passed' | 'failed' | 'skipped';
  details: string;
  projects?: string[];
  prompt_count?: number;
  target_pid?: number;
  verification_passed?: boolean;
  timestamp: string;
}

export interface InstanceSwitchRecord {
  id: string;
  instance_id: string;
  instance_name: string;
  from_account_email: string;
  to_account_email: string;
  from_account_id?: string;
  to_account_id?: string;
  switch_reason: string;
  total_switch_count_for_instance: number;
  duration_ms: number;
  steps: SwitchStepLog[];
  created_at: string;
}
```

### 4 Discrete Execution Phases
1. **Phase 1: Backup Prompts**
   - Identify active workspace projects for `instance_id`.
   - Snapshot enqueued and running conversations via `requeue_running_conversations_for_instance`.
   - Store `.antigravity_resume_task.json` and persist backup snapshot.
   - Record projects list, prompt count, and backup file location.
2. **Phase 2: Reset Process**
   - Kill lingering instance process PID cleanly.
   - Purge lockfiles (`lockfile`, `code.lock`, `DevToolsActivePort`).
   - Swap target account authentication credentials into `data_dir` or Antigravity state.
3. **Phase 3: Restore Prompts**
   - Stage `.antigravity_resume_task.json` with `auto_boot: true`.
   - Launch instance via `launch_instance_inner`.
   - Wait for prompt channel stabilization (6.0s delay).
   - Restore conversations into prompt queue.
4. **Phase 4: Post-Restore Verification**
   - Scan target instance's workspace/state for resume task consumption.
   - Confirm active session received prompts and is progressing.
   - Record verification outcome: `verification_passed: true`.

## 5. Live PID Detection & Quota Synchronization

### Backend IPC Command: `sync_instance_pid_and_quota`
1. For target `instance_id`:
   - Enumerate running OS processes matching `Antigravity` and inspect `--user-data-dir` argument to find exact PID.
   - Read active account from instance `data_dir` (e.g. `state.vscdb`, credentials file, or token storage).
   - Resolve account in `accounts.json` matching the active email/id.
   - If account changed or differs from `instance.bound_account_id`, update `instance.bound_account_id` and `instance.bound_email`.
   - Fetch real-time quota and credits from Google Gemini API via `fetch_account_quota`.
   - Update instance state in `instances.json` and persist to SQLite index cache.
   - Return updated `Instance` object.

### 10-Minute Periodic Background Sync
In `src-tauri/src/modules/scheduler.rs`:
- Loop every 600 seconds (10 minutes).
- Iterate through all registered instances with status `running` or `active`.
- Verify live PID is healthy.
- Sync active account credentials and refresh Gemini quotas.
- Emit `instances:updated` event to frontend so UI dynamically reflects updated credits.

## 6. Verification Criteria
1. Clicking "Audit" on any instance opens the modal and displays total switch count and at least 2–3 switches.
2. Each switch entry expands into 4 verified steps: Prompts Backup, Account Reset, Prompts Restore, Post-Restore Verification.
3. Clicking "Sync" immediately inspects running PID, detects active account, refreshes credits, and updates UI state.
4. Background 10-minute scheduler logs periodic synchronization runs without crashing or leaking memory.
5. All pre-flight gates pass: `cargo fmt`, `cargo clippy`, `npm run build`, and `e2e-instance-commands-test.ps1`.
6. Minor version bump to `v4.129.0` with clean changelog and git release.
