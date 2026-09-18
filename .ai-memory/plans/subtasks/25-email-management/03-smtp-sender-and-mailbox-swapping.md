# Subtask 03: Outbound SMTP Sender & Failover Mailbox Swapping

> **Parent Plan:** `.ai-memory/plans/pending/25-email-management-split-security-db-and-remote-control.md`
> **Specification:** `02-spec/21-app/16-email-dispatch-mailbox-remote-management-and-split-security-db.md`
> **Status:** Completed
> **Files:** `src-tauri/src/modules/email_sender.rs`

---

## Objective

Implement outbound email delivery with automatic mailbox failover pool swapping, anti-spam MIME formatting, and rich HTML templates.

## Requirements

1. **Mailer Delivery:**
   - Connect to SMTP server using standard encryption (`TLS`, `STARTTLS`, `SSL`).
   - Format multipart MIME emails (plain text fallback + clean styled HTML).
   - Encode headers (RFC 2047 / utf-8) to avoid spam scoring.
2. **Failover Swapping Pool:**
   - Attempt delivery first using the designated `is_default` mailbox.
   - If delivery encounters an SMTP transport error, rate limit, or authentication issue, log failure and seamlessly retry with the next active mailbox in `email_accounts`.
   - Cycle through all active mailboxes before returning an error.
3. **Template Engine:**
   - HTML notification templates for:
     - Quota drop alert (<15% credit remaining).
     - Workspace switch notice.
     - Idle workspace notice (running projects without active prompts).
     - Remote command execution results.
     - Help cheat sheet.
