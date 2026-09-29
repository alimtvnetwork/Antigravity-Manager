# 72. Local-Only On-Demand End-to-End Test Suite for Instance Switching & Prompt Recovery

## 1. Verbatim User Request

```text
Have you end-to-end tested all these functionalities that I have just described regarding the instance switch, instance fast-forward or account change, and restore the running prompts and make sure the account is switched by taking the screenshot from the settings? All these things, can you please confirm that you have done end-to-end testing here in this machine running all this, and things are working fine? You can confirm it. So basically, we're going to write some end-to-end tests, but this would be local only for a specific reason. It's not like this test will run in the future in the local, not in the CI/CD, but only if I request it to. Can you please configure it like this and confirm that it is done like this?
```

## 2. Architectural Design & Zero-CI Quarantine Standard

### 2.1 The Problem & Isolation Guarantee
End-to-end tests for instance switching involve:
- Electron process tree spawning and graceful termination.
- Reading and writing local SQLite databases (`state.vscdb`, `repo_db`, `backup_prompts.db`).
- Modifying instance file systems and creating screenshot telemetry artifacts.

Running these tests during standard CI/CD runs (GitHub Actions) or during routine developer local test runs (`cargo test ./...`, `06-cicd-local-runner.py`) is strictly banned because:
1. CI/CD runners lack display servers, GPU contexts, or actual Antigravity Electron binaries.
2. Spawning local processes on shared systems can introduce flakiness or resource contention.
3. The user explicitly commanded: *"this would be local only for a specific reason. It's not like this test will run in the future in the local, not in the CI/CD, but only if I request it to."*

### 2.2 Dual Skip-by-Default Isolation Guard
To ensure the test is never run inadvertently, it implements a dual guard:
1. **Harness Level**: Rust attribute `#[ignore = "local_only_e2e"]`. Cargo automatically ignores all tests marked with `#[ignore]` in standard test runs (`cargo test`, `cargo test --lib instance`).
2. **Runtime Level**: Environment variable check `if std::env::var("RUN_TEMP_E2E").as_deref() != Ok("1") { return; }`. Even if someone passes `-- --ignored`, the test immediately skips execution unless `RUN_TEMP_E2E=1` is explicitly provided.

### 2.3 On-Demand Execution Command Protocol
The test can ONLY be triggered when explicitly requested via:
```powershell
$env:RUN_TEMP_E2E="1"; cargo test --lib instance test_local_e2e -- --ignored --nocapture
```
Or via the CLI:
```powershell
agm test-instance-flow
```

## 3. Acceptance Criteria (AC)

- **AC-01**: Host IDE PID `8116` / child `5004` must be identified and protected before any test action.
- **AC-02**: The test must be marked `#[ignore = "local_only_e2e"]` and verify `RUN_TEMP_E2E == "1"`.
- **AC-03**: Default `cargo test --lib instance` must ignore the test (0 run, 1 ignored).
- **AC-04**: On-demand invocation must test complete workflow:
  - Isolated instance creation & cloning.
  - Project binding & prompt seeding (1 active running, 2 queued).
  - Folder-to-PID lookup.
  - Prompt snapshot & backup.
  - Account rotation in `state.vscdb`.
  - Prompt restoration & .antigravity_resume_task.json writing.
  - Verification query.
  - Safe cleanup & teardown.
- **AC-05**: All pre-flight CI/CD checks (`cargo fmt`, `cargo clippy`, `npm run build`) must pass 100% green.
