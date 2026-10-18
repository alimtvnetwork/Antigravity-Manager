# Specification: UI/UX Switch Account Modal & Instance Audit Trail

## 1. Switch Account Modal UI/UX Redesign
- **Aesthetic Requirement:** Align with Antigravity-Manager dark-glass aesthetic (`bg-[#120d1d]/90`, `border-[#3a2258]`, `backdrop-blur-xl`, rounded pill capsules).
- **Structure:**
  - Header: Segmented pill with active instance name, current account email, and close button.
  - Body: 4-card / multi-card selection view with high-contrast active state (`border-violet-500`, glow ring, emerald badge for high quota).
  - Search / Filter bar: Quick filter by email or quota tier.
  - Footer: Clear confirmation action with non-blocking feedback and spinner.

## 2. Instance Audit Trail Visibility
- **Database Backend:** Query `task_history_db` and SQLite audit tables (`history-*.db` and `task_index.db`) for events tagged with `instance_id`.
- **Event Classification:**
  - `SWITCH_ACCOUNT`: timestamp, old_account, new_account, trigger (manual/auto).
  - `START_INSTANCE`: timestamp, pid, executable, data_dir.
  - `STOP_INSTANCE`: timestamp, previous_pid, termination_status.
  - `ERROR`: timestamp, code, error_message.
- **UI Rendering in `InstanceAuditModal.tsx`:**
  - Render a vertical timeline with icons matching event classification.
  - Provide human-readable relative time (e.g. `2 minutes ago`) with exact timestamp tooltip.
  - If no events exist, display a clean empty state with a "Record First Action" prompt rather than an unstyled blank pane.
