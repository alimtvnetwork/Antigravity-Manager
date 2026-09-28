# Consolidated Plan 82: Status Telemetry UI & Email Header Overhaul

- **Spec Reference:** [02-spec/21-app/71-status-telemetry-ui-and-email-header-overhaul.md](../../../02-spec/21-app/71-status-telemetry-ui-and-email-header-overhaul.md)
- **Status:** `completed`
- **Execution Loops:** 1 Continuous Single-Pass Multi-Stage Loop

## Origin & Ingestion
Initiated from user feedback on Telegram `/status` telemetry UX degradation, unreadable blue code blocks, obscure `Node-XXXXXX` machine naming, broken truncation line breaks (`... [truncated]`), redundant running badges, missing auto-switch threshold percentage, missing prompt duration, and requirement for `AGM vX.Y.Z` email subjects without pipes.

## Delivered Architectural Changes

### 1. Header & Node Telemetry Refinement
- Telegram `/status` (observe) report header rebranded to `🔭 AGM v<version> Observation & Telemetry Report`.
- Generic `Node-XXXXXX` replaced with real Windows Machine Name (`COMPUTERNAME` / hostname) and configured operator Alias.
- Removed noisy `Backup Batches: ...` line.
- Added auto-switch threshold percentage to the Quota / Tier bullet (`switch threshold: 15%`).

### 2. Workspaces & Prompts Visualization
- Workspaces separated into clean `🟢 Running` (with 🟢 symbol and prompt snippets) and `⚪ Idle Workspaces` with vertical spacing.
- Abbreviated project names (`Antigravity-Manager` -> `AGM`).
- Eliminated Telegram `<code>` monospace wrapping on prompt bodies that rendered as unreadable dark-blue-on-dark-blue on mobile/desktop clients; now rendered in clean indented white text.
- Replaced `Recent Prompts Queue` and `[dispatched]` with `⚡ Running Prompts:` and calculated elapsed duration (`(running Xm Ys)`).
- Replaced `clean_for_telegram_html`'s `\n... [truncated]` line break artifact with clean inline ellipsis `...`.

### 3. Interactive Prompt Expansion Command
- Added `/expand <id>` (and `/expand`) command handler to display full untruncated prompt instructions.
- Added footer guidance to `/status`: `💡 Send /expand <id> to view full prompt text, or /active for live table.`

### 4. Email Telemetry Subject Standardization
- Updated `format_subject_with_telemetry` and all notification templates across `src-tauri/src/modules/email_sender.rs`, `email_inbound.rs`, `agm.rs`, and `notification_hub.rs` to format telemetry prefix as `[AGM vX.Y.Z | <machine> | <ip>]` without `Antigravity` and without pipe between `AGM` and version number.
- Updated all associated unit tests to validate the new `[AGM v...]` standard.
