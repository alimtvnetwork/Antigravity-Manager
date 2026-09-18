# Plan 19: Full Error Management and Global Error Modal Integration

> **Version:** 1.0.0
> **Status:** Completed
> **Completed Date:** 2026-09-16
> **Scope:** Rust Backend (`src-tauri/`), React Frontend (`src/`), Global Error Modal, AI-Sharable Markdown Report

---

## Summary of Accomplishments

1. **Rust Backend (`src-tauri/src/error.rs`)**:
   - Structured `AppError` with standardized diagnostic error codes:
     - `E1001` / `E1002` / `E1003`: Network / Timeout / Rate Limit errors
     - `E2001`: OAuth / Authentication errors
     - `E3001`: SQLite database and transaction errors
     - `E4001`: Filesystem and I/O errors
     - `E5001`: Configuration validation errors
     - `E6001`: Account and session management errors
     - `E8001`: Tauri IPC bridge errors
     - `E9001`: Generic unexpected application exceptions
   - Implemented Universal Response Envelope schema:
     - `StatusBlock`: `is_success`, `is_failed`, `code`, `message`
     - `ErrorsBlock`: `backend_message`, `backend`
     - `AttributesBlock`: `requested_at`, `has_any_errors`
     - Flat fields: `code`, `level`, `message`, `details`, `timestamp`, `backend_stack_trace`
   - Custom `serde::Serialize` implementation on `AppError` serializing into structured envelope JSON rather than a flat string.

2. **Frontend Error State & Interaction Tracking (`src/stores/error-store.ts`)**:
   - Implemented `CapturedError` interface with full diagnostics context.
   - Built in-memory ring buffer capturing the last 10 user interactions (`ClickEvent`) into an arrow-formatted click trail (`Link "Accounts" → Button "Refresh Quotas"`).
   - Built JavaScript stack trace parser (`parseFullStackTrace`) identifying caller functions, source files, and filtering framework internal frames.
   - Zustand store `useErrorStore` managing active error, error queue (`errorQueue`), queue pagination, and open/close modal state.

3. **Compact AI Error Report Generator (`src/lib/error-report-generator.ts`)**:
   - Pure function `generateCompactReport(error: CapturedError): string` outputting structured Markdown formatted for direct copy-pasting into AI chat interfaces (ChatGPT, Claude, Gemini, Antigravity).
   - Generates App metadata, Error Code & Level, Page/Route, User Interaction click-path, Trigger Context, Message, Details, Request info, Backend diagnostics, Frontend stack trace, and Error-code-specific suggested troubleshooting steps (`getSuggestedFixes`).

4. **Global Error Modal Component (`src/components/errors/error-modal.tsx`)**:
   - Modern overlay dialog with dark mode and design token support.
   - Severity badges (Destructive, Warning, Info), Error Code badges, and queue navigation controls (`< Prev` `1 / 3` `Next >`).
   - Four dedicated diagnostic tabs:
     - **Overview**: Primary message, details, trigger context, user click-path trail, suggested fixes.
     - **Backend**: Backend error message, backend stack frames, operation.
     - **Stack Trace**: Structured frame table (Function, File, Line) and raw stack toggle.
     - **Context**: JSON viewer of context/metadata.
   - Instant "Copy Error for AI" primary action button with clipboard write and toast feedback.

5. **Universal Interception & Root Mounting (`src/lib/error-listener.ts`, `src/utils/request.ts`, `src/App.tsx`)**:
   - Global listeners for `window.onerror`, `window.onunhandledrejection`, and `document.onclick`.
   - Wired `src/utils/request.ts` so failed Tauri `invoke` or Web API calls automatically capture error diagnostics into `useErrorStore`.
   - Mounted `<ErrorModal />` in `src/App.tsx` root layout.

---

## File Verification

- `src-tauri/src/error.rs`: Updated and formatted (`cargo fmt` clean).
- `src/stores/error-store.ts`: Created.
- `src/lib/error-report-generator.ts`: Created.
- `src/components/errors/error-modal.tsx`: Created.
- `src/lib/error-listener.ts`: Created.
- `src/utils/request.ts`: Updated.
- `src/App.tsx`: Updated.
