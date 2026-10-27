# Subtask 04: Email Template Builder, Remote Playground & CLI Parity

**Subtask ID:** `232-04`  
**Parent Task:** `232-warm-up-ui-and-emailing-ui-improvements`  
**Status:** `PENDING`  
**Improvements Covered:** Improvements 61–80, 91–100 (Clusters 7, 8, 10)  
**Target Specs:**  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/01-architecture-spec.md`  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/03-emailing-ui-improvements-spec.md`  

---

## Technical Scope & Deliverables
1. **Visual Email Template Designer (Improvement 61)**: Split-pane WYSIWYG editor with drag-and-drop dynamic tokens (`{{node_alias}}`, `{{credits_remaining}}`).
2. **Dual-Theme Light/Dark Simulator (Improvement 62)**: Side-by-side rendering in Outlook Light Mode and Gmail Dark Mode.
3. **Pluggable Brand Theme Styler (Improvement 63)**: Gradient headers, accent color pickers, and font selectors.
4. **HTML Email Size Guard (<102KB) (Improvement 65)**: Byte counter warning against Gmail clipping.
5. **Interactive Omission Banners (Improvement 66)**: Transform massive CLI dumps (>50 lines) into expandable dark-glass callouts.
6. **Inbound Remote Command Playground (Improvement 71)**: Sandboxed dry-run parser simulating email command routing without sending emails.
7. **SPF / DKIM Header Security Shield (Improvement 72)**: Cryptographic header validation rejecting forged sender commands.
8. **HMAC-SHA256 Command Token & Secret PIN (Improvement 73)**: Pre-shared token or 6-digit TOTP verification.
9. **Granular Per-Command ACL Matrix (Improvement 75)**: Checkbox matrix controlling remote command permissions.
10. **Caller Instance Self-Preservation Guard (Improvement 80)**: Prevent remote actions from stopping caller's active IDE.
11. **Email CLI Command Tree (Improvements 91–100)**:
    - `agm email doctor` (5-stage ladder CLI)
    - `agm email watch --tail` (Interactive terminal HUD)
    - `agm email test-inbound` (Headless command parser simulator)
    - `agm email audit list` (SQLite audit querying with `--json`)
    - `agm email templates ls` (ANSI template previewer)
    - `agm email send-custom` (Direct notification dispatcher)
    - GitMap macro integration & zero-storage action purge triggers.

---

## File Boundaries
- `Exclusive Files`:
  - `src/components/email/TemplateDesigner/TemplateDesignerModal.tsx`
  - `src/components/email/CommandPlayground/CommandPlaygroundModal.tsx`
  - `src-tauri/src/bin/agm/email_cmds.rs`
  - `src-tauri/src/modules/email_inbound/security.rs`
- `Read-Only Context Files`:
  - `src-tauri/src/modules/email_sender/templates_a.rs`
  - `src-tauri/src/modules/email_inbound/dispatcher.rs`
  - `src-tauri/src/modules/email_vault_db/`

---

## Acceptance Criteria
- [ ] Template designer renders live HTML with dual-theme toggle.
- [ ] Command playground validates and simulates inbound email commands without network traffic.
- [ ] SPF/DKIM verification shield intercepts unauthenticated commands.
- [ ] `agm email doctor` and `agm email test-inbound` execute with exit 0 and valid `--json` outputs.
