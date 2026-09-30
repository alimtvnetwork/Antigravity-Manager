# Subtask 01: Instance Switching Stickiness & Modal Dismissal

Traceability ID: Task-01, Task-02, Task-03
Spec Reference: [02-spec/21-app/26-instance-switching-ui-telegram-email-fix.md](../../../02-spec/21-app/26-instance-switching-ui-telegram-email-fix.md)
Target Files: src/components/navbar/InstanceSelector.tsx, src/components/modals/agy-clean-modal.tsx, src-tauri/src/modules/instance.rs, src-tauri/src/modules/db.rs
Action:
1. In `InstanceSelector.tsx`, fix instance switching to ensure selecting/activating an instance commits the new `active_instance_id` without reverting to `worker-alpha`. Add an explicit close button (`X`) to the header of the instance popover/modal. Close any open cleaner modal when instance modal opens.
2. In `agy-clean-modal.tsx`, add an explicit close button (`X`) in the top-right header, handle click-outside backdrop dismiss and `Escape` key.
3. In `InstanceSelector.tsx`, redesign active/selected row styling from dark-on-dark to high contrast (e.g. bright/white background with dark bold text in dark mode or vibrant high-contrast badge/border), and add `scrollIntoView` for the active/focused item upon mount or change.
4. In `instance.rs` and `db.rs`, verify `switch_instance` and `activate_instance` save the new target instance ID immediately to `instances.json` and split SQLite before launching the IDE process, eliminating any race condition that causes fallback to default.

Acceptance Criteria:
- Selecting an instance in the UI does not revert back to worker-alpha.
- Both Instance popover and Conversation Cleaner modal have visible `X` close buttons and dismiss cleanly.
- Active instance row in the selector has high contrast and scrolls into view when opened.
Targeted Verification: git diff --stat src/components/navbar/InstanceSelector.tsx src/components/modals/agy-clean-modal.tsx
