# Subtask 01 — Crash hardening: send-prompt / 3D-prompt (instance section)

> SUPERSEDED 2026-10-13: covered by the completed plan `.ai-memory/plans/supabase-connection-and-ui-ux-enhancement.md` (prompt-tree stdin deadlock fix in `src-tauri/src/modules/repo_db/agy.rs`, shipped v4.195.0). Kept for record only.

## Goal

The window must never die silently when send-prompt or the 3D prompt view is invoked. Failures surface with stack traces via the error module and remain inspectable after restart.

## Steps

1. Locate the send-prompt dispatch path in the instance section (start from the PromptTreeViewModal send/queue handlers and the `send_prompt_now` / `enqueue_prompt` Tauri commands) and locate the "3D prompt" view component via `gitmap aum search`.
2. TS: wrap every `invoke` and every external-source call on these paths in try/catch; route failures to `useErrorStore` (`captureError` for user-facing, `trackWarning` for best-effort) — never a bare `catch {}`.
3. Rust: ensure no panic escapes the command boundary; return `Err` with context (`AppError`), log via `tracing`.
4. Add/verify a frontend error boundary around the instance section so a render failure cannot blank the window.
5. Verify: `npm run build` exit 0, `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`.

## Done when

An induced failure on either path yields a visible traced error in the error module instead of a dead window; pre-flight green.
