# Root Cause Analysis: Linux Tauri Binary Linker Error Undefined OpenSSL Symbols

## 1. Symptom

In GitHub Actions CI (run `35484234019`) and Release (run `35484238283`) on Ubuntu runners (`ubuntu-latest`, `ubuntu-24.04-arm`, `ubuntu-22.04`):
All non-Linux and check-only jobs passed (macOS and Windows builds succeeded; `cargo check`, `cargo fmt`, `clippy`, and `cargo test` all passed).
However, during the Tauri binary link step (`npm run tauri build -- --debug --no-bundle` or `npm run tauri build`), the Linux linker aborted with:
```text
rust-lld: error: undefined symbol: SSL_CTX_ctrl
rust-lld: error: undefined symbol: SSL_ctrl
rust-lld: error: undefined symbol: ERR_get_error_all
rust-lld: error: undefined symbol: SSL_read_ex
rust-lld: error: undefined symbol: SSL_write_ex
collect2: error: ld returned 1 exit status
error: could not compile `agm-alim` (bin "agm-alim") due to 1 previous error
```

## 2. Root Cause

1. In `src-tauri/Cargo.toml`, `native-tls = "0.2"` was added to support direct TLS/SMTPS connections in `email_sender.rs` and `email_inbound.rs`.
2. On Windows (`Schannel`) and macOS (`Security.framework`), `native-tls` uses native OS crypto stacks without OpenSSL.
3. On Linux, `native-tls` defaults to dynamic OpenSSL linking via `openssl-sys`.
4. In Tauri v2 projects where `agm-alim` is an executable binary linking against `antigravity_tools_lib` (`rlib`), GCC/rust-lld links GTK3 and WebKitGTK with `--as-needed`. Because `agm-alim` does not directly declare C linkage for OpenSSL and `antigravity_tools_lib` is a static rlib, the dynamic `libssl.so` and `libcrypto.so` symbols are not linked into the final binary artifact, causing unresolved symbol errors (`SSL_CTX_ctrl`, `ERR_get_error_all`, `SSL_read_ex`).

## 3. Resolution

1. **Enable Vendored OpenSSL Feature in `src-tauri/Cargo.toml`**:
   Updated the dependency to:
   ```toml
   native-tls = { version = "0.2", features = ["vendored"] }
   ```
   Similar to `rusqlite = { features = ["bundled"] }`, enabling `vendored` activates `openssl-src`, which statically compiles OpenSSL 3.x from source on Linux and embeds static archives (`libssl.a`, `libcrypto.a`) directly into the Rust rlib.
2. **Lockfile Synchronization**:
   Regenerated `src-tauri/Cargo.lock`, embedding `openssl-src 300.6.1+3.6.3` into the dependency tree.
3. **Quality Gate Verification**:
   - `run.ps1 -Check` (TypeScript and Rust formatting): Passed.
   - `python 03-ai-scripts/06-cicd-local-runner.py`: Passed all 36 quality gates in 9.59s.

## 4. Prevention & Learnings

1. For cross-platform desktop applications (Tauri/Electron), dynamic C-library dependencies on Linux must always be statically vendored (e.g. `rusqlite` with `bundled`, `native-tls` with `vendored`, or pure Rust alternatives like `rustls`).
2. When introducing networking or encryption crates, verify Linux binary linking behavior in addition to compilation typechecks.
