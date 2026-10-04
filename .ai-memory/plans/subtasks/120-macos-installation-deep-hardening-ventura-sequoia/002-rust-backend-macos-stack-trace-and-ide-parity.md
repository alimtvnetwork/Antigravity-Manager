# Subtask 002 — Rust Backend: macOS Stack Trace & IDE Path Parity

**Plan:** 120 — macOS Installation Deep Hardening
**Status:** pending

## Scope

Files to modify:
- `src-tauri/src/modules/process.rs`
- `src-tauri/src/modules/instance.rs`

---

## 1. `process.rs` — `discover_and_persist_initial_ide_info`

### 1.1 Write Structured JSON Discovery Log

After detecting the IDE path, write a structured JSON log entry to the
platform data-local directory (macOS: `~/Library/Application Support/antigravity/`
via `dirs::data_local_dir()`):

Log file path pattern:
```
{data_local_dir}/antigravity/ide-discovery.log
```

Log entry format (one JSON object per line, newline-delimited):
```json
{
  "timestamp": "<ISO 8601 UTC>",
  "found": true,
  "path": "<absolute path to IDE>",
  "method": "<mdfind|candidate_path|env_var|...>",
  "backtrace": "<RUST_BACKTRACE=1 backtrace string or empty>"
}
```

Implementation notes:
- Use `dirs::data_local_dir()` to resolve the base path; do NOT hardcode `~`.
- Create the parent directory with `std::fs::create_dir_all` before writing.
- Append to the log file (open with `.create(true).append(true)`).
- If writing fails, log a warning via `tracing::warn!` but do not propagate the error.

### 1.2 Add `$HOME/Applications/` to `get_macos_candidate_paths`

In addition to `/Applications/`, the candidate paths function must also check:

```rust
// $HOME/Applications/
if let Some(home) = dirs::home_dir() {
    let user_app = home.join("Applications").join("Antigravity.app");
    candidates.push(user_app);
}
```

Ensure this is inserted before the fallback/unknown-path branch.

---

## 2. `instance.rs` — macOS Instance Launch

### 2.1 Add `$HOME/Applications/*.app/Contents/MacOS/` to Candidate Paths

`get_macos_candidate_paths` (or equivalent function) must also enumerate:

```rust
if let Some(home) = dirs::home_dir() {
    let user_macos_bin = home
        .join("Applications")
        .join("Antigravity.app")
        .join("Contents")
        .join("MacOS");
    // push the expected binary inside Contents/MacOS/
    candidates.push(user_macos_bin.join("Antigravity"));
}
```

### 2.2 Diagnostic Log on Launch Failure

When an instance fails to launch on macOS, write a diagnostic JSON entry
to the same `ide-discovery.log` path:

```json
{
  "timestamp": "<ISO 8601 UTC>",
  "found": false,
  "path": "<attempted path>",
  "method": "launch_attempt",
  "backtrace": "<error debug string>"
}
```

Reuse the same log-append helper introduced in `process.rs` (extract to a
shared private module or inline the logic if the helper is not yet factored out).

### 2.3 Set `RUST_BACKTRACE=1` When Spawning Diagnostics

When spawning any child process for diagnostic or IDE-discovery purposes,
inject the environment variable:

```rust
std::process::Command::new(&binary_path)
    .env("RUST_BACKTRACE", "1")
    // ...rest of args
    .spawn()?;
```

This ensures backtraces are available in logs when the child process fails.

---

## Acceptance Criteria

- [ ] `discover_and_persist_initial_ide_info` in `process.rs` writes JSON to `dirs::data_local_dir()/antigravity/ide-discovery.log`
- [ ] Log entries match the specified JSON schema (timestamp, found, path, method, backtrace)
- [ ] Log file is opened in append mode; directory is created if absent
- [ ] Write failures are logged as `tracing::warn!` and not propagated
- [ ] `get_macos_candidate_paths` in `process.rs` includes `$HOME/Applications/Antigravity.app`
- [ ] `get_macos_candidate_paths` in `instance.rs` includes `$HOME/Applications/Antigravity.app/Contents/MacOS/Antigravity`
- [ ] Launch failure in `instance.rs` appends a diagnostic JSON entry to `ide-discovery.log`
- [ ] Child processes spawned for diagnostics have `RUST_BACKTRACE=1` set
- [ ] No absolute paths (`/home/`, `/root/`, etc.) — use `dirs` crate exclusively
