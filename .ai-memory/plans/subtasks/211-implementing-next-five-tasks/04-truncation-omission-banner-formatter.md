# Subtask 04: Truncation Omission Banner Formatter (`Imp-683`)
Status: IN_PROGRESS
Parent Plan: 211-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Minimalist & Contextual UI:
"Interactive Truncation Callouts: Never render raw `<truncated N bytes>` or `<truncated N lines>` markers as plain text; parse and render them as interactive dark-glass omission banners (`⚡ [Omitted N bytes/lines of context - Click to inspect/expand]`)."

Implement `src-tauri/src/modules/instance/truncation_omission_banner_formatter.rs` and unit tests in `src-tauri/src/modules/instance/tests_truncation_omission_banner_formatter.rs`.

## Requirements
- Parse raw truncation tokens: `<truncated N bytes>` and `<truncated N lines>`.
- Format into rich interactive omission banners: `⚡ [Omitted {formatted_count} of context - Click to inspect/expand]`.
- Support human-friendly byte formatting (B, KB, MB).
- Produce structured records with byte offset, marker type, human label, and interactive attributes.
- Comprehensive unit tests covering byte/line parsing, boundary checks, and HTML/JSX banner generation.
