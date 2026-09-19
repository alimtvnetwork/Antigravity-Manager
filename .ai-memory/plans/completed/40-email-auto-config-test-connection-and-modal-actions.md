# Plan 40: Email Auto-Configuration, DNS Socket Resolution, Self-Test Mail, and Modal Action Visibility

## 1. Problem Statement & User Requirements
The user identified critical bugs and missing features in the Mailbox configuration modal:
1. **Custom Domain Server Auto-Discovery / Guessing:**
   When entering an email address (e.g. `ai-agm-tool-v2@hire-seoexperts.com`), if it is not Gmail, automatically guess incoming and outgoing servers as `mail.<domain>`, defaulting IMAP port to `993` and SMTP port to `465` (SSL/TLS), with quick suggestion presets (POP3 995, SMTP 587, IMAP 143).
2. **Modal Action Buttons Clipped / Truncated:**
   In `ModalDialog.tsx`, `.modal-box` had `overflow-hidden` and fixed `max-w-sm` with unconstrained vertical content, clipping off `Save Account`, `Test Connection`, and `Cancel` buttons outside the viewport.
3. **Interactive Test Connection & Self-Test Email:**
   A test section in the modal that connects to SMTP/IMAP, sends a test email to itself (`from == to`), and displays a clear green signal on success.
4. **Hostname DNS Resolution Bug in Backend:**
   In `email_sender.rs` and `email_inbound.rs`, hostnames like `mail.hire-seoexperts.com` were parsed via `SocketAddr::from_str` (`addr.parse()`), which fails on all domain names because it only accepts numeric IP addresses. Fixed using `ToSocketAddrs` resolution.
5. **Machine-Bound RSA Password Vault:**
   Ensured passwords continue to be secured in `email_passwords.db` using machine-bound salted SSH RSA identity encryption.

## 2. Implemented Architecture & Deliverables
1. `src/components/common/ModalDialog.tsx`:
   - Added `maxWidth?: string` prop (default `'max-w-sm'`).
   - Added `max-h-[90vh] flex flex-col` to `.modal-box`.
   - Wrapped modal body in a flex-1 scrollable container (`flex-1 min-h-0 overflow-y-auto`) so the modal never pushes buttons outside the viewport.
   - Pinned footer action buttons to the bottom with `shrink-0 pt-3 mt-auto border-t border-gray-100 dark:border-base-200`.
2. `src-tauri/src/modules/email_sender.rs`:
   - Fixed DNS resolution using `std::net::ToSocketAddrs` for hostname and IP resolution.
   - Implemented `send_via_account_credentials(account: &EmailAccount, password: &str, subject: &str, html_body: &str, recipients: &[String])`.
   - Updated `send_via_account` to delegate to `send_via_account_credentials`.
   - Added `render_self_test_email(email: &str, machine_name: &str, machine_ip: &str) -> (String, String)` for HTML self-test receipts.
3. `src-tauri/src/modules/email_inbound.rs`:
   - Fixed DNS resolution using `std::net::ToSocketAddrs` in `poll_unread_messages`.
4. `src-tauri/src/commands/email.rs`:
   - Added `test_direct_email_connection(account: EmailAccountInput) -> AppResult<String>`.
5. `src-tauri/src/lib.rs`:
   - Registered `commands::test_direct_email_connection` in `generate_handler!`.
6. `src/services/emailService.ts`:
   - Exported `testDirectEmailConnection(account: EmailAccountInput): Promise<string>`.
7. `src/components/settings/EmailNotificationSettings.tsx`:
   - Added `handleEmailChange` auto-discovery heuristics: parses domain, auto-fills `mail.<domain>`, port 465 (SMTP), port 993 (IMAP) for non-Gmail, or `smtp.gmail.com:587` and `imap.gmail.com:993` for Gmail.
   - Added real-time email address validation with format feedback.
   - Added port preset buttons for SMTP (465 SSL, 587 TLS, 25 Plain) and IMAP (993 SSL, 143 STARTTLS, 995 POP3).
   - Added dedicated Test Connection card with "Send Test Email" action, live spinner, green signal badge with animated pulse on success, and alert banner on failure.
   - Upgraded modal dialog with `maxWidth="max-w-lg"`, `confirmText="Save Mailbox"` / `"Update Mailbox"`, and `cancelText="Cancel"`.

## 3. Verification & Quality Gates
- Ran `npx tsc --noEmit`: 0 TypeScript errors.
- Verified strict boolean principles (implicit positive checks, no `== true`, no mixed polarity).
- Verified strict relative git paths across documentation and plans.
