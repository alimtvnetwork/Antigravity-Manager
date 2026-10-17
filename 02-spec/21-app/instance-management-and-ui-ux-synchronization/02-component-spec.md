# Component Specification: Instance Management & UI/UX Synchronization

## 1. Instance Audit Trail Modal (`InstanceAuditTrailModal.tsx`)

### Problem
Clicking an instance row or card to open the audit trail previously showed "No Account Switches Recorded" if the instance had not undergone an explicit account rotation, even if prompt backups, restores, PID checks, or scheduler events occurred. Furthermore, account switches recorded with `instance_id = "antigravity"` or `None` were not mapped to `"default"`.

### Solution
1. **Multi-Source Fetching**:
   - `InstanceAuditTrailModal` calls both `getInstanceSwitchHistory(instance.id)` and `getInstanceAuditTrail(instance.id)`.
   - Adds a dual-pill mode selector: `Switch Cycles (N)` and `All Operations (N)`.
   - If `recent_switches` is 0 but general audit events exist, the modal automatically falls back to displaying `All Operations` rather than a blank empty card.
2. **Action Item Cards**:
   - Renders task cards with timestamp, action badge, status indicator, duration, and collapsible parameters / details.
3. **Identity Header**:
   - Displays instance name, sequence number, instance ID, and last active timestamp.

## 2. Switch Account Dialog (`SwitchAccountDialog.tsx`)

### Design & Visual Polish
1. **Dark-Glass Capsule Styling**:
   - Modal background: `backdrop-blur-md bg-white/95 dark:bg-[#120d1d]/95` with subtle border `border-slate-200 dark:border-[#40285e]`.
   - Distinct Close button with 'X' icon in the header in addition to footer button.
2. **Quota Representation & High Contrast**:
   - Score badges: 4h quota, Weekly quota, Total score.
   - Progress bar: Emerald (>=50%), Amber (>=20%), Rose (<20%) with neon glow on high quotas.
3. **Selection UX**:
   - Active bound account has distinct teal border and checkmark badge.
   - Rows feature smooth hover transitions (`hover:border-teal-400/60 hover:bg-slate-100/80 dark:hover:bg-[#1f1635]`).
   - Disables all buttons during an in-flight switch with animated `RotateCw` spinner.

## 3. Caller Instance Detection & Self-Preservation (`instance/close.rs`, `bin/agm/instance_cmds.rs`)

### Algorithm: Exact Caller Matching
1. Traverse process ancestor tree up to 24 levels from `std::process::id()`.
2. For each ancestor process:
   - Check executable path: if it matches `Antigravity.exe` or `antigravity`:
     - Inspect process command line arguments (`proc.cmd()`).
     - Look for `--user-data-dir` or `-user-data-dir`.
     - If `--user-data-dir <path>` matches an instance in `registry.instances`, return `inst.id`.
     - If no `--user-data-dir` is present, but the process is `Antigravity.exe`, return `"default"`.
   - Check PID against `inst.pid` in registry.
3. Safety Guard (`refuse_if_caller_instance`):
   - In CLI commands `instances stop`, `instances rm`, `instances switch`, `switch`, refuse execution if target matches detected caller ID.
   - CLI flags: `--force` or `--test` bypass when explicitly requested.

## 4. Test Instance Lifecycle Commands (`bin/agm/instance_cmds.rs`, `modules/instance/`)

### Commands
- `agm instances create-test [--name <name>] [--json]`:
  Creates a transient `--data-only` instance named `test-<short-id>`, registers it in `registry.json`, and records creation in `task-history`.
- `agm instances rm-test [<id>] [--all] [--json]`:
  Kills any running test process, purges directory, unregisters from `registry.json`, and cleans database references.
