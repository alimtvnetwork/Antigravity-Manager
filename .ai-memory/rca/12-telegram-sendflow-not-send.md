# RCA 12: Telegram send_flow — future not Send (MutexGuard across await)

## Symptom
CI failed on commit `e7a2dc2a` (the E0015 fix commit) on all platforms:
```
error: future cannot be sent between threads safely
  --> src/modules/telegram_inbound/send_flow.rs:592:84
help: within `{async block@src/modules/telegram_inbound/daemon.rs:427:33}`,
  the trait `std::marker::Send` is not implemented for
  `std::sync::MutexGuard<'_, HashMap<i64, SendFlow>>`
```

## Root cause
In `consume_send_prompt`, the `SEND_FLOWS` `std::sync::MutexGuard` was
acquired at the top of the async fn and — despite an explicit `drop(flows)`
before the `.await` — the borrow checker still considered the guard live
across `dispatch_send_prompt(...).await`. `std::sync::MutexGuard` is not
`Send`, so any future holding it cannot be spawned on the multi-threaded
Tauri async runtime. The `drop()` was insufficient because the `flow` borrow
derived from the guard kept the guard's lifetime extended into the await
region.

## How found
`gh run list` showed CI failure on the fix commit → downloaded
`--log-failed` → grepped for the red `error` marker → traced to
`send_flow.rs:592` + `daemon.rs:427`.

## Fix
`src-tauri/src/modules/telegram_inbound/send_flow.rs` — restructured
`consume_send_prompt` so the lock lives in a scoped block that returns a
plain-data `Next` enum (`LockedReply` or `Dispatch { machine, instance,
conversation }`); the `.await` happens AFTER the block, with the guard
provably dropped. No `MutexGuard` crosses an await boundary anywhere in the
file (verified: all other `.lock()` sites are in sync fns).
Committed as `283efa42`, pushed. `cargo fmt` clean.

## Verification
- `cargo fmt -- --check` exit 0.
- Repo-wide check: no other `SEND_FLOWS.lock()` site sits in an async fn.
- Full `cargo check` impossible here (missing GTK system libs); CI is the
  compile gate — CI re-run on `283efa42` pending at time of writing.

## Prevention
- **Never hold a `std::sync::Mutex` guard across `.await`.** In async code,
  either scope the lock in a block and extract owned data, or use
  `tokio::sync::Mutex` (whose guard IS Send). Prefer the scoped-block
  pattern for short critical sections.
- When fixing one compile error, re-check CI on the fix commit before
  declaring done — cascading errors are common (this was error #2 of 2 in
  the same file).
