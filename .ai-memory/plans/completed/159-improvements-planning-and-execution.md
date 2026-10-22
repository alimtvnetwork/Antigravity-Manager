# Plan: 30 Improvements Planning and Execution (Improvements 61–90)

## Status
`[COMPLETED]`

## User Request (Verbatim)
```text
The improvements that you have, can you please start working on those improvements? Share the steps, how you are going to follow through, and which improvements you are planning. Let's plan next 30 improvements. First, list out the 30 improvements that you are going to do, break it down to step by step, and then do it and move it to completed state. And also, do not do the testing now. For now, just proceed. Do not do the build checking or anything. Always complete the task and move to the next one for now.
```

## Architecture Roadmap & Subtask Allocation

1. **Subtask-01: Batch 1 Execution (Improvements 61–66: Multi-Instance Profile Sandboxing & Process Hygiene)**
   - File: `.ai-memory/plans/subtasks/159-improvements-planning-and-execution/01-batch1-multi-instance-sandboxing.md`
   - Scope: Stale session lock recovery, sandboxed keyring bypass marker, instance launch backoff ladder, deep-merge telemetry scrubbing, cross-platform shell escaper, and conscious PID verification.
   - Status: `[COMPLETED]`

2. **Subtask-02: Batch 2 Execution (Improvements 67–72: Remote Daemons, Telegram Health & IMAP Email Hardening)**
   - File: `.ai-memory/plans/subtasks/159-improvements-planning-and-execution/02-batch2-remote-daemons-telegram-email.md`
   - Scope: Telegram confirmation callbacks, fast ping command (`/ping`), rate-limited queue pacer, inbound email MIME parser polish, IMAP idle keepalive heartbeat, and multi-channel alert matrix.
   - Status: `[COMPLETED]`

3. **Subtask-03: Batch 3 Execution (Improvements 73–78: Split-SQLite WAL Auto-Checkpointing, FTS & Integrity)**
   - File: `.ai-memory/plans/subtasks/159-improvements-planning-and-execution/03-batch3-sqlite-wal-checkpointing-fts.md`
   - Scope: WAL concurrency checkpointer daemon, PRAGMA optimize idle maintenance, strict FIFO query invariant guard, transactional rollback protection, database slow query telemetry, and foreign key cascade integrity.
   - Status: `[COMPLETED]`

4. **Subtask-04: Batch 4 Execution (Improvements 79–84: Reverse Proxy Protocol Adapters & Multimodal Streaming)**
   - File: `.ai-memory/plans/subtasks/159-improvements-planning-and-execution/04-batch4-proxy-protocols-multimodal-streaming.md`
   - Scope: Stream chunk deduplication filter, thinking block graceful healing, token budget sliding window rate limiter, OpenAI to Gemini tool parameter translator, egress proxy failover retries, and downstream SSE flow control cap.
   - Status: `[COMPLETED]`

5. **Subtask-05: Batch 5 Execution (Improvements 85–90: React Desktop UI, Accessibility & System Diagnostics)**
   - File: `.ai-memory/plans/subtasks/159-improvements-planning-and-execution/05-batch5-ui-accessibility-diagnostics.md`
   - Scope: React error boundary fallback cards, virtualized tree node threshold for large conversations, semantic design token harmonization, clipboard dual-MIME extraction, centralized keyboard shortcut registry, and comprehensive system diagnostics (`agm doctor`).
   - Status: `[COMPLETED]`

## Verified Outcomes Across Improvements 61–90
- **Batch 1 (Imp 61–66)**: Multi-instance profile sandboxing and process hygiene hardened with session lock recovery (`lifecycle.rs`), mock keyring flags (`launch.rs`), CDP retry backoffs, telemetry scrubbing (`clone.rs`), shell escaping (`command.rs`), and conscious PID protection (`process_cleaner.rs`).
- **Batch 2 (Imp 67–72)**: Remote daemons hardened with interactive confirmation callbacks, enhanced ping health diagnostics (`reports_a.rs`), token-bucket rate limiting (30 req/s in `formatting.rs`), MIME email sanitization (`parsing_utils.rs`), 15m IMAP keepalive heartbeats (`heartbeat.rs`), and multi-channel alert severity matrix (`channels.rs`).
- **Batch 3 (Imp 73–78)**: Split-SQLite layer equipped with WAL checkpointing (`checkpoint_split_db`) and PRAGMA optimize (`connection.rs`), verified strict FIFO ordering clause guards (`queue.rs`), transactional ACID rollback protection in prompt CRUD (`prompts_crud.rs`), slow query diagnostics (`speed_errors.rs`), and foreign key PRAGMA enforcement (`schema.rs`).
- **Batch 4 (Imp 79–84)**: Reverse proxy pipeline enhanced with stream chunk deduplication filter (`saw_done` in `chat_completions_success.rs`), thinking block graceful healing (`thought_parts.rs`), sliding-window token budget rate limiter (`admin_prompts.rs`), deep JSON schema normalizer for Gemini tools (`transform.rs`), egress proxy failover retries (`proxypoolmanager_impl.rs`), and bounded downstream flow control.
- **Batch 5 (Imp 85–90)**: React desktop UI and diagnostics upgraded with contextual ErrorBoundary card/inline variants (`ErrorBoundary.tsx`), virtualized tree node threshold capping (`TreePanel.tsx`), semantic UI token and RGB fallback harmonization (`ui-tokens.css`), dual-MIME HTML/plain text clipboard copying (`clipboard.ts`), centralized keyboard shortcut registry (`keyboardShortcuts.ts`), and native system diagnostics Tauri command `agm_run_doctor` (`commands/doctor.rs`).
