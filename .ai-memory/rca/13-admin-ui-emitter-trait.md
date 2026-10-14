# RCA 13: admin_ui E0599 — missing tauri::Emitter trait import

## Symptom
CI failed on the v4.200.0 release commit (`b62dbb6d`) on all platforms:
```
error[E0599]: no method named `emit` found for reference `&AppHandle`
  in the current scope
```
Both `Check Rust Code` and `Build Tauri App` jobs failed; the Release
workflow failed as a consequence.

## Root cause
In Tauri 2.x, `AppHandle::emit` is provided by the `tauri::Emitter` trait,
which must be in scope. The new `src-tauri/src/proxy/server/admin_ui.rs`
called `handle.emit(...)` without `use tauri::Emitter;`. The existing
`toolchain.rs` precedent (`app.emit("toolchain-progress", ...)`) has the
import; the new file was written without checking.

## How found
`gh run list` showed CI failure on the release commit → downloaded
`--log-failed` → grepped for the red error marker → E0599 at `admin_ui.rs`.

## Fix
Added `use tauri::Emitter;` to `src-tauri/src/proxy/server/admin_ui.rs`.
`cargo fmt` clean. Committed as `5121fbb4`, pushed.

## Verification
- `cargo fmt -- --check` exit 0.
- Full compile gate is CI (gdk-sys unavailable locally).
- CI re-run on `5121fbb4` pending at time of writing.

## Prevention
- When calling trait methods on Tauri types (`emit`, `listen`), always
  verify the trait import (`tauri::Emitter`, `tauri::Listener`) is present
  — the method will not resolve without it, and the error (E0599) doesn't
  name the missing trait.
- New files calling into framework APIs should be checked against an
  existing in-repo precedent for the same API before push.
