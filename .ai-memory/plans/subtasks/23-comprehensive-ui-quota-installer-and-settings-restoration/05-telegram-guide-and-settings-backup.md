# Subtask 05: Telegram Guide & Settings Unified Backup/Restore
Traceability ID: Task-07, Task-08, Task-09
Spec Reference: [02-spec/21-app/23-comprehensive-ui-quota-installer-and-settings-restoration.md](../../../02-spec/21-app/23-comprehensive-ui-quota-installer-and-settings-restoration.md)
Target Files: src/components/settings/TelegramSettings.tsx, src/components/settings/AutoSwitcherSettings.tsx, src/components/settings/SupabaseSyncSettings.tsx, src/pages/Settings.tsx, src/components/navbar/Navbar.tsx
Action:
- Align Telegram Bot Token and Allowed Chat ID input styling.
- Add "Create Bot Guide" / "Bot Setup Wizard" button with interactive instructions.
- Add Import/Export dropdown to Auto-Switcher card.
- Add full system Backup & Restore button in Advanced / Supabase section.
- Modernize Debug console trigger icon and link.
Acceptance Criteria:
- Telegram inputs aligned with helper modal.
- Switcher and system backup/restore options accessible.
Status: Completed
Targeted Verification: Aligned Telegram form fields, added Create Bot Guide modal, added Actions dropdown for AutoSwitcher (export/import/reset), added Backup & Restore buttons in Settings toolbar and SupabaseSyncSettings, added Debug console toggle to NavSettings.
