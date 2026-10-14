# RCA-01 — CI `Check Rust Code` failed: `cargo fmt` wanted `run_caught_prompt_task` wrapped

Repo: Antigravity-Manager (`src-tauri/`) · CI workflow step: `Check Rust formatting` · Failing commit: `e04cc7b` · Fix commit: `9b968fb5`

## Symptom

The CI workflow `Check Rust Code` failed at the step `Check Rust formatting`
(exit 1) on **all three OS runners** (the job runs a matrix, e.g.
ubuntu-latest / windows-latest / macos-latest). A formatting-only failure on
every runner at once is the signature of `cargo fmt -- --check` rejecting the
pushed code — the formatting rules do not differ between OSes, so all runners
agree and all runners fail.

The step itself runs this command:

```
cd src-tauri && cargo fmt -- --check
```

`--check` means "do not modify anything; exit non-zero if anything *would*
be reformatted". Exit 1 = at least one file is not formatted the way
`rustfmt` wants. The CI log shows a diff for one file:

```
Diff in src-tauri/src/commands/instance.rs at line 313:
-fn run_caught_prompt_task<F>(label: &str, task: F) -> Result<crate::modules::repo_db::ActivePrompt, String>
+fn run_caught_prompt_task<F>(
+    label: &str,
+    task: F,
+) -> Result<crate::modules::repo_db::ActivePrompt, String>
```

(In other words: CI printed the exact reformatting it expected. The old
single-line `fn` signature exceeded `rustfmt`'s 100-column `max_width`, so
`rustfmt` wants each parameter on its own line.)

## Root cause

**One line:** a hand-written `fn` signature longer than 100 columns was
committed without running `cargo fmt`.

Explanation, step by step, for zero-context readers:

1. `rustfmt` is Rust's official code formatter; `cargo fmt` runs it on the
   project. It enforces a maximum line width of 100 columns (`max_width = 100`,
   the default).
2. The function `run_caught_prompt_task` in
   `src-tauri/src/commands/instance.rs:313` was written as a single long line:

   ```rust
   fn run_caught_prompt_task<F>(label: &str, task: F) -> Result<crate::modules::repo_db::ActivePrompt, String>
   ```

   That line is 113 characters — 13 over the limit.
3. `rustfmt`'s rule for an over-long function signature: put the function name
   and opening paren on the first line, one parameter per line, and the closing
   paren plus return type on the last line.
4. Because the author's editor (or habit) did not auto-format on save, the
   over-long line was committed as-is in `e04cc7b`, and CI — which runs
   `cargo fmt -- --check` — rejected it.

The bug class is "formatting-only": the code compiled and ran perfectly. The
failure is purely mechanical, which is exactly why it is cheap to prevent and
expensive to ignore (it blocks the whole CI pipeline on every runner).

## How it was found (exact commands)

All commands below are run from the repo root
(`~/workspace/repos/Antigravity-Manager`). `gitmap pe` is GitMap's "pull /
examine" helper that surfaces the failing tree state; it is read-only
diagnostics here.

```bash
export PATH="$HOME/.local/bin:$PATH"
cd ~/workspace/repos/Antigravity-Manager

# 1. See the failing state (read-only diagnostics)
gitmap pe

# 2. Reproduce the CI failure locally — this is the exact command CI runs
cd src-tauri && cargo fmt -- --check
#    -> exit 1, and prints the diff quoted under "Symptom"

# 3. Read the offending line (no grep allowed in this house; gitmap search/cat only)
gitmap cat src-tauri/src/commands/instance.rs | sed -n '313p'
```

Step 2 is the whole diagnosis: the `--check` diff names the file and line.
No deeper investigation was needed — when CI fails on a *formatting* step, the
local `cargo fmt -- --check` output *is* the diagnosis.

## Fix

The fix is mechanical: let `rustfmt` do the wrapping, then commit only the
one file.

```bash
export PATH="$HOME/.local/bin:$PATH"
cd ~/workspace/repos/Antigravity-Manager/src-tauri
cargo fmt            # rewrites instance.rs in place
```

File `src-tauri/src/commands/instance.rs`, around line 313:

**Before** (one line, 113 cols):

```rust
fn run_caught_prompt_task<F>(label: &str, task: F) -> Result<crate::modules::repo_db::ActivePrompt, String>
```

**After** (`cargo fmt` output, verified on disk today):

```rust
fn run_caught_prompt_task<F>(
    label: &str,
    task: F,
) -> Result<crate::modules::repo_db::ActivePrompt, String>
where
    F: FnOnce() -> Result<crate::modules::repo_db::ActivePrompt, String> + Send,
{
```

The function is generic over `F`, takes a `label` and a closure `task`, and
returns the task's `Result` (an `ActivePrompt` DB row) — `run_caught_prompt_task`
executes the closure inside `std::panic::catch_unwind` so a panicking prompt
task becomes a logged `Err(String)` instead of crashing the backend.

Commit facts: exactly one file changed —
`src-tauri/src/commands/instance.rs`, **4 insertions, 1 deletion** (the one
long line replaced by the four-line wrapped signature). Pushed as commit
`9b968fb5`.

## Verification evidence

1. Re-ran the exact CI command locally after the fix:
   `cd src-tauri && cargo fmt -- --check` → **exit 0, no diff printed**.
2. Re-ran CI: the `Check Rust Code` job went green on all runners after
   `9b968fb5` (a formatting diff cannot regress at runtime, and Clippy/build
   steps passed on the same commit — the code was unchanged semantically).
3. Today (2026-10-15) the file on disk still shows the wrapped signature at
   `src-tauri/src/commands/instance.rs:313` (verified via
   `gitmap cat src-tauri/src/commands/instance.rs | sed -n '300,325p'`),
   confirming the fix is the current canonical state.

## Prevention — checklist item

**Pre-push pre-flight (repo AGENTS.md, "Formatting & CI Discipline"):**

Before every `git push` that touches Rust code, run from the repo root:

```bash
export PATH="$HOME/.local/bin:$PATH"
cd ~/workspace/repos/Antigravity-Manager/src-tauri && cargo fmt -- --check
```

- If it exits 0 → push.
- If it exits 1 → run `cargo fmt` (no `--check`), re-verify with `--check`,
  then push.

One-line rule for the dummy AI: **never push Rust code you have not run
`cargo fmt -- --check` on.** A formatting failure is never "CI being picky" —
it is a deterministic, local, 2-second check you skipped. Better yet, enable
format-on-save in your editor so the check can never fail in the first place.
