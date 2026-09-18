# Subtask 07: React 19 Frontend Settings UI & Two-Way Import/Export

> **Parent Plan:** `.ai-memory/plans/pending/25-email-management-split-security-db-and-remote-control.md`
> **Specification:** `02-spec/21-app/16-email-dispatch-mailbox-remote-management-and-split-security-db.md`
> **Status:** Completed
> **Files:** `src/components/settings/EmailNotificationSettings.tsx`, `src/pages/Settings.tsx`

---

## Objective

Build the frontend UI in `Settings.tsx` for email management without breaking the existing layout:
1. Mailbox account list with default badge, status toggle, test connection, add/edit/delete actions.
2. Account dialog for editing alias, email, password, SMTP host/port, IMAP host/port, encryption type.
3. Notification recipients manager (add email, group name, delete, active toggle).
4. Watcher settings (polling intervals, quota threshold, toggle triggers for quota drop, workspace switch, idle projects).
5. Two-way Import/Export modal supporting JSON, CSV, and Excel (XLSX).
6. Local machine telemetry banner showing machine name and IP.

## Requirements

1. **Clean UI & Responsive Layout:**
   - Follow existing Tailwind CSS styling and card components.
   - Do not break existing tabs or layout in `Settings.tsx`.
2. **Import/Export Experience:**
   - Provide file picker for JSON, CSV, and XLSX.
   - Provide export buttons with immediate file download or clipboard copy.
3. **Error Modal Integration:**
   - When operations fail, feed structured `AppError` into `useErrorModal` for diagnostic copy/paste.
