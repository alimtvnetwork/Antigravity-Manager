# Subtask 40.3: Frontend Auto-Discovery, Port Presets, Validation, & Test Email UI

## Scope
Modify `src/components/settings/EmailNotificationSettings.tsx`:
1. Auto-Discovery heuristic:
   - On typing/pasting email, extract domain.
   - Non-gmail: set `smtp_host = mail.<domain>`, `smtp_port = 465`, `imap_host = mail.<domain>`, `imap_port = 993`, `encryption_type = 'SSL'`.
   - Gmail: set `smtp.gmail.com:587` (TLS), `imap.gmail.com:993` (TLS).
2. Port preset pill buttons:
   - IMAP: `993 (SSL/TLS)`, `143 (Plain/STARTTLS)`, `995 (POP3)`.
   - SMTP: `465 (SSL/TLS)`, `587 (TLS)`, `25 (Plain)`.
3. Email address validation:
   - Inline format check and error/helper indicator.
4. Test Connection & Self-Test Email section:
   - "Send Self-Test Email" button with loading state.
   - Live green signal banner/indicator upon success.
   - Error banner if test fails.
5. Modal sizing & footer:
   - Set `maxWidth="max-w-lg"` on `ModalDialog`.
   - Ensure Save Mailbox, Cancel, and Test buttons are clearly accessible.

## Target Files
- `src/components/settings/EmailNotificationSettings.tsx`
