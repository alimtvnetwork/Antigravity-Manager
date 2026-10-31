# Master Plan 234: Telegram Feature Enhancement

## Status: COMPLETED
**Slug**: `telegram-feature-enhancement`
**Spec Reference**: `02-spec/21-app/234-telegram-feature-enhancement/`
**Release**: `v4.275.0`

## Executive Summary
Remote Telegram status is now decision-complete. `/status` and `/observe` fan out one compact HTML card per registered instance with running/queued prompts and projects, a 100-word preview, Copy (256-char Bot API cap), Full prompt document, Rerun, Done?, Released?, Auto, Fast-Forward, Screenshot, and Refresh. Instances CSV is a real download (`application/octet-stream`, timestamped filename, prompt/project columns). Desktop hover shade animates color only. Isolated switch+restore E2E on `agm instances test-create` restored the same prompt id after account switch; Default and v2-3127 were never stopped.

## Modular Subtasks
1. [x] **Subtask 01**: Per-instance status cards, copy/rerun/enqueue/FF
   - File: `.ai-memory/plans/subtasks/234-telegram-feature-enhancement/01-instance-status-cards.md`
2. [x] **Subtask 02**: CSV download, screenshot mosaic, copy_text 256, CSS3 hover shade
   - File: `.ai-memory/plans/subtasks/234-telegram-feature-enhancement/02-csv-screenshot-copy-hover.md`
3. [x] **Subtask 03**: Isolated switch+restore E2E, minor bump `v4.275.0`
   - File: `.ai-memory/plans/subtasks/234-telegram-feature-enhancement/03-e2e-and-minor-release.md`

## Invariants
- Caller IDE Default and v2-3127 never stopped.
- FIFO restore `ORDER BY created_at ASC, id ASC`.
- Sole attribution to `@aukgit` (`(Thanks to @aukgit)`).
- Strict relative Git paths across all docs and commits.
- Local full-lib `cargo test` / `cargo check --bin agm` skipped on this 12 GB host (rustc OOM); CI compiles.
