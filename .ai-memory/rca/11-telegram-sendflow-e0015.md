# RCA 11: Telegram send_flow E0015 — non-const fn in static

## Symptom
CI failed on commit `c57b219b` (the RCA+Telegram program commit) across all
platforms:
```
error[E0015]: cannot call non-const associated function
  `std::collections::HashMap::<i64, SendFlow>::new` in statics
  --> src-tauri/src/modules/telegram_inbound/send_flow.rs:41:63
41 | static SEND_FLOWS: Mutex<HashMap<i64, SendFlow>> = Mutex::new(HashMap::new());
```
Both `Check Rust Code` (clippy) and `Build Tauri App` jobs failed on
ubuntu-latest, windows-2022, macos-latest.

## Root cause
`HashMap::new()` is not a `const fn`, so it cannot be called in a `static`
initializer. The coordinator's worker wrote the static directly and its
"verification" only ran `cargo fmt -- --check` (formatting), never a
compile — so the breakage shipped.

## How found
User asked "is it all done?"; lead checked
`gh run list -R alimtvnetwork/Antigravity-Manager` → CI failure on the new
commit → `gh run view <id> --log-failed` → E0015 at `send_flow.rs:41`.

## Fix
`src-tauri/src/modules/telegram_inbound/send_flow.rs:42` — wrap in the
repo's established `once_cell::sync::Lazy` pattern (same as
`TELEGRAM_RUNNING` in `types.rs:54`):
```rust
// old
static SEND_FLOWS: Mutex<HashMap<i64, SendFlow>> = Mutex::new(HashMap::new());
// new
static SEND_FLOWS: Lazy<Mutex<HashMap<i64, SendFlow>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
```
Plus `use once_cell::sync::Lazy;`. `cargo fmt` clean. Committed as
`e3456f22`, pushed.

## Verification
- `cargo fmt -- --check` exit 0 on `src-tauri`.
- Repo-wide search confirmed this was the only non-const static in the new
  files (`open_shot.rs` has none).
- Full `cargo check` not possible on this machine (missing GTK system libs —
  environment limitation, not a code issue); CI is the compile gate.
- CI re-run on `e3456f22` pending at time of writing.

## Prevention
- **Never trust coordinator/worker self-reported verification.** The worker
  claimed "cargo fmt exit 0" as proof the code was good; formatting is not
  compilation. This is the same lesson as 2026-10-08 (coordinator claimed
  "build exit 0" on a tree that did not compile).
- Any new `static` holding a non-const-constructible type MUST use
  `once_cell::sync::Lazy` (the repo's established pattern) — add this to the
  pre-push checklist for Rust changes.
- After pushing new Rust code, always check CI on the new commit before
  reporting "done" — `gh run list` takes 2 seconds.
