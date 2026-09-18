# 4-Part Root Cause Analysis: Rustfmt Discrepancies in Instance, Repo DB, and Proxy Server Handlers

> **Version:** 1.0.0
> **Date:** 2026-09-18
> **Failed Runs:** [#35334190423](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35334190423) (CI)
> **Trigger Commit:** `f6ca2060`
> **Fixed In:** `HEAD`

---

## 4-Part Root Cause Analysis (RCA)

### 1. Symptom (Why it happened)
In GitHub Actions CI workflow run [#35334190423](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35334190423), all three matrix runner platforms (`windows-2025`, `ubuntu-latest`, and `macos-latest`) for the job `Check Rust Code` failed at the `Check Rust formatting` step executing `cargo fmt --all -- --check`:

```text
Diff in src-tauri/src/modules/instance.rs:623:
                 for pid in &pids {
                     if system.process(sysinfo::Pid::from_u32(*pid)).is_some() {
-                        let _ = Command::new("kill")
-                            .args(["-9", &pid.to_string()])
-                            .output();
+                        let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
                     }
                 }
...
Diff in src-tauri/src/modules/repo_db.rs:51:
-    let conn = Connection::open(&path).map_err(|e| format!("Failed to open repo database: {}", e))?;
+    let conn =
+        Connection::open(&path).map_err(|e| format!("Failed to open repo database: {}", e))?;
...
Diff in src-tauri/src/proxy/server.rs:1461:
-                    error: "Another switch or rotation operation is already in progress".to_string(),
+                    error: "Another switch or rotation operation is already in progress"
+                        .to_string(),
...
-async fn admin_list_repo_projects() -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
+async fn admin_list_repo_projects() -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)>
+{
...
-async fn admin_list_repo_prompts() -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
+async fn admin_list_repo_prompts() -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)>
+{
```

The failure blocked the CI validation pipeline across Linux, macOS, and Windows.

### 2. How it happened & Root Cause
#### How it happened:
In commit `f6ca2060` (and `0c344085`), new functionality was implemented for auto-switching 15% quota thresholds, split repository DB prompt dispatching, Ubuntu instance process termination, and account rotation API endpoints.
Because local development occurred on Windows without a locally installed `cargo` toolchain, `cargo fmt` was not executed before pushing to GitHub. As a result, standard Rust code layout conventions were slightly divergent from `rustfmt` formatting defaults.

#### Root Cause:
1. **`src-tauri/src/modules/instance.rs`:**
   - In `close_instance()`, `Command::new("kill").args(...).output()` was authored across three lines rather than collapsed to a single line under 100 characters.
   - In unit tests `test_instance_config_serialization` and `test_ubuntu_instance_switching_end_to_end_flow`, assertion arguments and `conn.execute()` expressions exceeded the 100-character line length limit.
2. **`src-tauri/src/modules/repo_db.rs`:**
   - Chained calls on `Connection::open(&path)` and `decode_uri_to_path()` (`.replace("%20", " ").replace(...)`) exceeded 100 characters on a single line.
   - In `detect_running_projects()`, `find_pids_for_data_dir()`, `serde_json::from_str()`, and `workspace_storage_path` assignments required multi-line line wrapping matching `rustfmt` indentation heuristics.
3. **`src-tauri/src/proxy/server.rs`:**
   - In `admin_rotate_account()`, the 101-character line `error: "Another switch or rotation operation is already in progress".to_string()` exceeded the line boundary by 1 character.
   - In `admin_list_repo_projects()` and `admin_list_repo_prompts()`, the return type signatures `Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)>` brought the signature line length to 101 characters, requiring `rustfmt` to place the function body opening brace `{` on a newline.

### 3. Code Fix & Resolution
The formatting in all three affected Rust source files was updated to conform precisely to `rustfmt` standards:

1. **`src-tauri/src/modules/instance.rs`:**
   - Collapsed `Command::new("kill").args(["-9", &pid.to_string()]).output();` to a single line.
   - Split long `assert_eq!`, `conn.execute()`, and `conn.query_row()` expressions across multiple lines with proper indentation.
2. **`src-tauri/src/modules/repo_db.rs`:**
   - Wrapped `Connection::open(&path)` assignment across two lines.
   - Wrapped `.replace("%20", " ").replace("%3A", ":").replace("%3a", ":")` onto separate lines.
   - Wrapped `find_pids_for_data_dir`, `serde_json::from_str`, and `workspace_storage_path` assignments.
3. **`src-tauri/src/proxy/server.rs`:**
   - Wrapped `error: "Another switch or rotation operation is already in progress"` and `.to_string(),`.
   - Placed `{` on a newline for `admin_list_repo_projects` and `admin_list_repo_prompts`.

### 4. Verification & Prevention
1. **Verification:**
   - Inspected `git diff` against the exact output in CI run `#35334190423`. All formatting changes match AST output 1:1.
   - Normalized newlines using `03-ai-scripts/04-newline-fixer.py`.
   - Verified that zero absolute paths exist in documentation and markdown files.
2. **Prevention Rules:**
   - When authoring Axum handlers returning `Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)>`, if the line exceeds 100 characters, place the function body opening brace `{` on its own newline.
   - Maintain a strict 100-character line length limit for all Rust source files.
   - String literals combined with method calls (`"string".to_string()`) should be wrapped when approaching column 95 to prevent 1-2 character overflows.
