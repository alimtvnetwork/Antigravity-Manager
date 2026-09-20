# Subtask: UI Email Accounts Table Hover Contrast

**Target Files:**
- `src/components/settings/EmailNotificationSettings.tsx`
- `src/pages/Email.tsx`

**Action:**
1. In `src/components/settings/EmailNotificationSettings.tsx`:
   - Replace line 634 `<tr className="hover:bg-gray-50/50 dark:hover:bg-base-200/50 transition-colors">` with:
     `<tr className="hover:bg-gray-100/70 dark:hover:bg-slate-800/80 transition-colors group">`.
   - Update text classes across table cells so they brighten to crisp white on hover:
     - `acc.alias`: `text-gray-900 dark:text-slate-100 group-hover:text-white font-semibold`.
     - `acc.email`: `text-gray-500 dark:text-slate-400 group-hover:text-slate-200`.
     - `acc.smtp_host` & `imap_host`: `text-gray-700 dark:text-slate-300 group-hover:text-slate-100 font-mono`.
   - Polish `Test SMTP` and `Test IMAP` action buttons with proper dark mode borders and hover feedback.

**Constraints:**
- Eliminate washed-out gray bars on hover in dark mode.
- Maintain high contrast ratio (>4.5:1) in both light and dark themes.
