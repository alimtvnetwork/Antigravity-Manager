# CI/CD RCA 34: Rustfmt Check Failure on Auto-Switcher Unit Tests in Multi-OS Matrix

## 1. Reproduction & Symptoms

In GitHub Actions workflow run `35808218006` (`CI` workflow on branch `main`), the `Check Rust Code (ubuntu-latest)`, `Check Rust Code (windows-2025)`, and `Check Rust Code (macos-latest)` matrix jobs failed at step `Check Rust formatting`:

```text
Run cd src-tauri && cargo fmt -- --check
Diff in src-tauri/src/modules/auto_switcher.rs:1156:
-        let mut acc_a = make_test_account("synth_acc_a", "synth_a@test.local", "gemini-pro", 100, future_time);
+        let mut acc_a = make_test_account(
+            "synth_acc_a",
+            "synth_a@test.local",
+            "gemini-pro",
+            100,
+            future_time,
+        );
Diff in src-tauri/src/modules/auto_switcher.rs:1187:
-        let mut acc_b = make_test_account("synth_acc_b", "synth_b@test.local", "gemini-pro", 100, future_time);
+        let mut acc_b = make_test_account(
+            "synth_acc_b",
+            "synth_b@test.local",
+            "gemini-pro",
+            100,
+            future_time,
+        );
Process completed with exit code 1.
```

## 2. Root Cause Analysis

In `src-tauri/src/modules/auto_switcher.rs`, new unit test function `test_score_candidate_account_weekly_quota_groups_bottleneck` was introduced with single-line invocations of `make_test_account(...)` exceeding line width limits (> 100 characters). Because `cargo fmt -- --check` enforces strict line breaks on argument lists, rustfmt exited with code 1 across all 3 platforms in the matrix.

## 3. Code Fix & Remediation

1. Ran `cargo fmt --all` in `src-tauri`, reformatting `make_test_account` call signatures with standard multi-line vertical arguments.
2. Verified `cargo fmt --all -- --check` exits with code 0 locally.
3. Fixed clippy doc-comment lint in `src-tauri/src/proxy/mappers/claude/utils.rs:42` (`///` converted to standard `//` comment).

## 4. Prevention & Quality Guard

Always run `cargo fmt --all -- --check` across `src-tauri` before committing Rust changes to prevent formatting drift from halting the multi-OS CI matrix.
