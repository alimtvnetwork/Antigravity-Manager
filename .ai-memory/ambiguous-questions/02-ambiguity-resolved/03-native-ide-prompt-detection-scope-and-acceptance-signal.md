# For a cloned instance, where do native (non-AGM) running prompts live, and what proves 're-injected'?

Slug: native-ide-prompt-detection-scope-and-acceptance-signal
Status: resolved
Raised: 2026-09-30
Blocking: 87 (subtasks 002, 006)

## Question
For a cloned instance, where do native (non-AGM) running prompts live, and what proves 're-injected'?

## Context
Prompts started natively in the IDE are stored in conversation state under the instance's own home or data directory; AGM-dispatched prompts live in `repo_prompts.db`. Acceptance could be the DB status, a heartbeat log line, or the conversation state.

## Options considered
- Scope: (A) per-instance home/data directories only; (B) per-instance plus global for the default instance.
- Signal: (1) DB `dispatched`; (2) heartbeat `Iteration: N+1` in the workspace log; (3) conversation state RUNNING.

## Impact if guessed wrong
A wrong scope re-injects another instance's prompt or misses the real one; a weak signal reports success when nothing ran.

## Interim provisional default
Scope B (global only for the default instance). Success requires signal (1) and either (2) within a bounded timeout or an explicit typed 'unverified' result; never silent success.

## Resolution

Answered: 2026-09-30
Answer: Go ahead with the proposed approach: per-instance directories only (global only for the default instance); success needs DB `dispatched` plus a heartbeat within a bounded timeout, otherwise a typed unverified result.
Applied solution: Plan 87 steps 002, 005, 006.
