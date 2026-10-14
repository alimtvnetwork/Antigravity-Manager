# RCA-02 — CI Clippy `error[E0061]`: `send_telegram_message_with_markup` grew a 5th parameter; 3 call sites missed

Repo: Antigravity-Manager (`src-tauri/`) · CI workflow step: Clippy inside `Check Rust Code` · Failing commit: `d71ae907` · Fix commit: `705bc024`

## Symptom

The CI workflow `Check Rust Code` failed at the Clippy step on commit
`d71ae907` with the Rust compiler error:

```
error[E0061]: this function takes 5 arguments but 4 arguments were supplied
  --> src-tauri/src/modules/telegram_inbound/daemon.rs:311:33
   |
311 | / ...send_telegram_message_with_markup(
312 | |     &clean_token,
313 | |     chat_id,
314 | |     &reply,
315 | |     markup_opt,
316 | | )
   | |__^ an argument of type `bool` is missing
   |
note: function defined here
  --> src-tauri/src/modules/telegram_inbound/formatting.rs:235:1
```

The same `E0061` error was reported three times, at three different call
sites in the same file (`src-tauri/src/modules/telegram_inbound/daemon.rs`,
then-lines 311, 459, and 544). Because a compile error is fatal, the entire
`Check Rust Code` job failed — nothing else could be checked.

What `E0061` means for a zero-context reader: the Rust compiler counts the
arguments at a call site and compares them with the function's declared
parameters. `send_telegram_message_with_markup` was declared with **5**
parameters, but three call sites still passed **4**. Rust has no default
arguments, so the missing argument is a hard compile error, not a warning.

## Root cause

**One line:** a new 5th parameter (`disable_preview: bool`) was added to
`send_telegram_message_with_markup`, but three of its call sites were not
updated — the author's find-and-replace only caught the first one.

Step by step:

1. The function `send_telegram_message_with_markup` lives in
   `src-tauri/src/modules/telegram_inbound/formatting.rs:235`. Its job: send
   a text message to a Telegram chat, with optional reply markup (e.g. an
   inline keyboard) and **optional link-preview suppression** (for compact
   status notifications).
2. A change added the 5th parameter `disable_preview: bool` to the signature:

   ```rust
   pub async fn send_telegram_message_with_markup(
       bot_token: &str,
       chat_id: i64,
       text: &str,
       reply_markup: Option<serde_json::Value>,
       disable_preview: bool,   // <-- NEW
   ) -> Result<(), AppError>
   ```

3. The author updated the call sites they found — but their search only caught
   the *first* call site. Three call sites in
   `src-tauri/src/modules/telegram_inbound/daemon.rs` (then-lines 311, 459,
   544) still passed only 4 arguments, e.g.:

   ```rust
   send_telegram_message_with_markup(
       &clean_token,
       chat_id,
       &reply,
       markup_opt,   // <-- missing the 5th argument `false`
   )
   ```

4. The incomplete update was pushed as `d71ae907`. The author's mental model
   was "CI will catch it if I missed one" — and CI did catch it, at the cost
   of a red pipeline, a blocked queue, and a second fix commit.

Why "the author's search only caught the first" is plausible: the three missed
sites are deeply nested inside `record_ignored(...)` wrappers and different
indentation levels in `daemon.rs` (callback-query handling, command reply,
status reply), so a quick eyeball scan or an editor "find next" stopped too
early. A repo-wide textual search would have listed all of them at once.

## How it was found (exact commands)

All commands run from the repo root (`~/workspace/repos/Antigravity-Manager`).
`gitmap pe` is GitMap's read-only diagnostic helper; `gh run view` reads CI
logs from GitHub (read-only).

```bash
export PATH="$HOME/.local/bin:$PATH"
cd ~/workspace/repos/Antigravity-Manager

# 1. Surface the failing tree / pipeline state (read-only diagnostics)
gitmap pe

# 2. Pull the exact failing CI log (read-only). The --log-failed output listed
#    all three E0061 sites with file:line.
gh run view --job <failing-job-id> --log-failed
#    -> error[E0061] at daemon.rs:311, daemon.rs:459, daemon.rs:544

# 3. Enumerate EVERY call site of the function repo-wide (the step the author skipped).
#    House rule: codebase search ONLY via gitmap aum search — never grep/rg.
gitmap aum search "send_telegram_message_with_markup" src-tauri/src -e .rs
```

Step 3 is the key diagnostic AND the preventive step in one: it returns every
hit with file:line — the definition in `formatting.rs:235`, the three
`daemon.rs` sites, plus callers in
`src-tauri/src/modules/notification_hub/telegram_alerts.rs:232` and
`src-tauri/src/modules/notification_hub/telemetry.rs:180`. Counting the hits
and checking each one's argument count is what proves the fix is complete.

## Fix

Behavior-preserving fix: pass `false` (link previews stay enabled, i.e. the
old default behavior) as the new 5th argument at each of the three missed
sites.

File `src-tauri/src/modules/telegram_inbound/daemon.rs` — three sites,
identical pattern. **Before** (then-lines 311, 459, 544):

```rust
send_telegram_message_with_markup(
    &clean_token,
    chat_id,
    &reply,
    markup_opt,
)
```

**After:**

```rust
send_telegram_message_with_markup(
    &clean_token,
    chat_id,
    &reply,
    markup_opt,
    false,          // disable_preview = false → behavior unchanged
)
```

Then the author verified completeness with the repo-wide search from "How it
was found" step 3 — every caller now passes 5 arguments — and ran `cargo fmt`
to confirm formatting was clean. Committed as `705bc024`.

Why `false` is safe: `disable_preview: false` means "do not suppress link
previews", which is exactly what the function did before the parameter
existed. The fix changes zero runtime behavior; it only satisfies the new
signature.

## Verification evidence

1. After the fix, the same repo-wide search shows all call sites passing 5
   arguments. Verified again today (2026-10-15):
   `gitmap aum search "send_telegram_message_with_markup" src-tauri/src -e .rs`
   → 12 hits; every *call* site (daemon.rs:27, 35, 47, 276, 481;
   telegram_alerts.rs:232; telemetry.rs:180; formatting.rs:337, 346) ends with
   `false`/`true` as the 5th argument, matching the 5-parameter definition at
   formatting.rs:235. Zero 4-argument callers remain.
2. Spot-read of the fixed sites today shows the trailing `false,` in place
   (e.g. daemon.rs around lines 276–284 and 481–489, via
   `gitmap cat src-tauri/src/modules/telegram_inbound/daemon.rs`).
3. `cargo fmt` clean after the fix (reported at fix time).
4. CI `Check Rust Code` (Clippy) went green on `705bc024` — E0061 is a
   compile error, so a green Clippy run *proves* every call site now matches
   the signature. The compiler is the final authority here.

## Prevention — checklist item

**After ANY function-signature change, before pushing, do this exact
sequence** (from the repo root):

```bash
export PATH="$HOME/.local/bin:$PATH"
cd ~/workspace/repos/Antigravity-Manager

# 1. List every call site of the changed function (gitmap only — never grep)
gitmap aum search "<function_name>" src-tauri/src -e .rs

# 2. Count the call-site hits. Open EACH hit and confirm the argument count
#    matches the new signature. Do not stop at the first one — the failure
#    mode of this incident is precisely "stopped after the first hit".

# 3. Compile-gate locally (Clippy compiles the code; no separate cargo check needed)
cd src-tauri && cargo clippy --all-targets --all-features

# 4. Formatting gate
cargo fmt -- --check
```

One-line rule for the dummy AI: **"CI will catch it" is not a strategy —
it is a confession you skipped step 1.** A repo-wide `gitmap aum search`
takes under a second and would have listed all three sites on one screen.
Search, count, verify every hit, then push.
