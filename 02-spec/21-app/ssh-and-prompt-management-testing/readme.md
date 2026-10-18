# SSH and Prompt Management Testing Specification

## 1. Overview & Executive Summary

This specification defines the architectural, privacy, ergonomic, and telemetry standards for the **SSH and Prompt Management Testing** initiative:
1. **SSH Fleet Telemetry & Credential Hygiene**: Eliminates the display of administrative usernames (`Administrator`, `root`) and passwords in the SSH fleet cards within the Instance tab. Displays essential operational identifiers: node IP address, node alias, active instance count, and associated account emails.
2. **Collapsible Prompt Tree & Automated Prompts Filter**: Introduces a collapsible/expandable sidebar container for the prompt tree, defaulting to collapsed/hidden state. Automates filtering to hide system messages, heartbeats, and telemetry turns by default, with one-click toggle controls and CLI visibility options.
3. **Multimodal Prompt Image Clipboard (IDE & OneNote Integration)**: Perfects image copying from prompt turns, allowing images to be copied as single text (Markdown image tags, Base64 data URIs) and rich clipboard HTML/DIB payloads that paste seamlessly into external IDEs (VS Code, Cursor) and OneNote.
4. **Prompt Backup & Restore Verification**: Establishes rigorous logging and verification confirming that exact prompts (with embedded multimodal image payloads and sequence tags) are backed up to SQLite databases upon account/instance switching and restored back intact.
5. **Standalone Telegram Hub**: Promotes Telegram bot and inbound daemon management from the settings modal to a standalone navigation item in the top header dropdown (adjacent to the instance selector). Provides an interactive connection setup guide and native CLI commands.
6. **Speed Error Database (`errors.db`) & AI Diagnostics**: Implements a dedicated SQLite speed database (`errors.db`) logging all application errors with millisecond timestamps, severity, and stack traces. Exposes terminal CLI commands (`agm errors list`, `agm errors fix`) and provides an AI skill guide (`agm-ai-diagnostics-and-logs`) for self-healing diagnostics.

## 2. Specification Inventory

- [01-architecture-spec.md](01-architecture-spec.md): Architectural design, data schemas, API contracts, and security boundaries.
