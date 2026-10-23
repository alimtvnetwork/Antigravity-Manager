# Subtask 04: Truncation Omission Banner Formatter (`Imp-683`)
Status: COMPLETED
Parent Plan: 211-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Minimalist & Contextual UI:
"Interactive Truncation Callouts: Never render raw `<truncated N bytes>` or `<truncated N lines>` markers as plain text; parse and render them as interactive dark-glass omission banners (`⚡ [Omitted N bytes/lines of context - Click to inspect/expand]`)."

Implemented `src-tauri/src/modules/instance/truncation_omission_banner_formatter.rs` and unit tests in `src-tauri/src/modules/instance/tests_truncation_omission_banner_formatter.rs`.
Registered in `src-tauri/src/modules/instance.rs`.

## Verification Evidence
- Defined `TruncationKind` (`Bytes`, `Lines`) and `OmissionBannerRecord`.
- Implemented `format_bytes_human` (B, KB, MB formatting).
- Implemented `parse_truncation_markers` and `replace_markers_with_banners` generating rich interactive dark-glass banners.
- Added comprehensive unit tests covering human byte formatting, parsing bytes/lines tokens, and string banner substitution.
- Deferred E2E command documented in `02-spec/21-app/211-implementing-next-five-tasks/e2e_master_plan.md`.
