# Subtask 04: Email Export Polish, PS Prefix & Prompt Injection Fix
Traceability ID: Task-05, Task-06, Task-10
Spec Reference: [02-spec/21-app/23-comprehensive-ui-quota-installer-and-settings-restoration.md](../../../02-spec/21-app/23-comprehensive-ui-quota-installer-and-settings-restoration.md)
Target Files: src/components/settings/EmailNotificationSettings.tsx, src/components/settings/MailboxExportModal.tsx, src-tauri/src/modules/email_inbound.rs, src-tauri/src/modules/cli.rs
Action:
- Add tooltip indicator for Export/Import buttons in Email section.
- Modernize Export modal, update footer to "Antigravity Manager Tools".
- Strip `powershell:` and `ps:` prefix before passing command string to PowerShell.
- Default to PowerShell on Windows if no prefix is given.
- Intercept `Project: ...` prompt injection templates so they format or route cleanly instead of erroring with "Project: The term 'Project:' is not recognized".
- Standardize all CLI mentions to `gitmap` lowercase.
Acceptance Criteria:
- `ps: Get-Process` and `Get-Process` both execute cleanly in PowerShell.
- Prompt injection does not throw PowerShell command not found.
Status: Completed
Targeted Verification: Verified prefix stripping for powershell:, ps:, ps , bash:, sh:, cmd: in test_execute_cli_command and execute_safe_cli_command, added AI prompt simulation interceptor, updated export modal footer and tooltips.
