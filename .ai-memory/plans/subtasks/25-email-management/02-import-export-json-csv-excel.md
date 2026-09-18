# Subtask 02: Import/Export Engine for JSON, CSV, and Excel (XLSX)

> **Parent Plan:** `.ai-memory/plans/pending/25-email-management-split-security-db-and-remote-control.md`
> **Specification:** `02-spec/21-app/16-email-dispatch-mailbox-remote-management-and-split-security-db.md`
> **Status:** Completed
> **Files:** `src-tauri/src/modules/email_io.rs`

---

## Objective

Build a two-way import and export engine for email accounts and notification recipients supporting:
1. JSON export/import.
2. CSV export/import (clean headers, quoting, comma delimitation).
3. Excel (XLSX) or XML-based multi-table spreadsheet representation.
4. SQLite vault backup / restore capabilities.

## Requirements

1. **Formats:**
   - **JSON:** Complete payload including accounts and recipient groups.
   - **CSV:** Accounts (`alias,email,smtp_host,smtp_port,imap_host,imap_port,encryption_type,is_default,is_active`) and recipients (`email,group_name,is_active`).
   - **Excel:** Clean tabular format with fallback or XML Spreadsheet 2003 / ZIP XLSX representation.
2. **Safety & Validation:**
   - Strict validation before insert/merge to prevent partial corruption.
   - Idempotent upsert by email address.
   - Masked password exports to prevent plaintext leaks.
