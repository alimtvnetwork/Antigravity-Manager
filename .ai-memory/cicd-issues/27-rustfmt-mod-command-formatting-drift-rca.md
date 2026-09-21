# RCA-27: Rust Formatting Drift in Commands Module and CI Runner Gate Gap

## 1. Symptom

In GitHub Actions CI pipeline run [#35585673860](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35585673860), step `Check Rust formatting` failed across all runner operating systems (`macos-latest`, `windows-2025`, `ubuntu-latest`):

```text
Diff in src-tauri/src/commands/mod.rs:221:
             let has_token = token_trimmed.len() > 0;
             if has_token {
                 if let Ok(accounts) = modules::list_accounts() {
-                    if let Some(matching) = accounts.into_iter().find(|a| a.token.refresh_token == token_trimmed) {
+                    if let Some(matching) = accounts
+                        .into_iter()
+                        .find(|a| a.token.refresh_token == token_trimmed)
+                    {
                         modules::logger::log_info(&format!(
                             "   Auto-bound current account from editor DB: {}",
                             matching.email
Diff in src-tauri/src/commands/mod.rs:228:
                         ));
-                        let _ = modules::account::set_current_account_id_with_target(&matching.id, current_target);
+                        let _ = modules::account::set_current_account_id_with_target(
+                            &matching.id,
+                            current_target,
+                        );
                         return Ok(Some(matching));
                     }
                 }
Error: Process completed with exit code 1.
```

## 2. Root Cause

1. **Unformatted Long Lines in `src-tauri/src/commands/mod.rs`:** In the previous release turn, the editor DB auto-binding logic was added to `get_current_account`. Two lines exceeded the standard Rust line limit and chaining conventions (`accounts.into_iter().find(...)` and `set_current_account_id_with_target(...)`).
2. **Local Runner Quality Gate Gap:** While `.github/workflows/ci.yml` strictly runs `cargo fmt -- --check`, `03-ai-scripts/02-shared-engine.py` had not registered a Rust formatting check or TypeScript typecheck in `CI_JOBS_MATRIX`. Consequently, running `03-ai-scripts/06-cicd-local-runner.py` passed all 36 existing gates without verifying Rust format compliance.

## 3. Resolution

1. **Rust Formatting Applied:** Formatted `src-tauri/src/commands/mod.rs` using `cargo fmt` to break method chains and function arguments across multi-line blocks conforming to Rustfmt rules. Verified with `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` exiting with code 0.
2. **Registered in Local CI/CD Matrix:** Added `"Rust Format Check"` and cross-platform `"TypeScript Check"` into `CI_JOBS_MATRIX` in `03-ai-scripts/02-shared-engine.py`:
   - `"Rust Format Check": ["cargo", "fmt", "--manifest-path", "src-tauri/Cargo.toml", "--", "--check"]`
   - `"TypeScript Check": ["node", "node_modules/typescript/bin/tsc", "--noEmit"]`
3. **Verified Local Green Gates:** Ran `python 03-ai-scripts/06-cicd-local-runner.py` verifying all 38 gates passed in 8.44s.

## 4. Prevention & Learnings

- **Mandatory Pre-Commit Rustfmt:** Any modification touching `.rs` files must be formatted with `cargo fmt` prior to git commit.
- **Local Runner Parity with CI Workflows:** All mandatory CI steps from `.github/workflows/ci.yml` must have exact 1:1 job counterparts registered in `CI_JOBS_MATRIX` inside `03-ai-scripts/02-shared-engine.py`.
- **Cross-Platform Node Invocations:** Use `["node", "node_modules/typescript/bin/tsc", "--noEmit"]` rather than bare `npx` so scripts execute seamlessly on Windows without shell interpreter discrepancies.
