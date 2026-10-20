# Subtask 06: E2E Testing, Telegram Test & Release

**Subtask ID**: `06-e2e-testing-and-release`  
**Parent Plan**: `.ai-memory/plans/button-functionality-and-testing-instructions.md`  
**Status**: QUEUED  

## Intent
Execute comprehensive testing: create an isolated test instance (`agm instances test-create`), verify button actions, test Telegram prompt dispatching, run local pre-flight checks, perform minor version bump, and verify CI/CD via `gitmap b -t`.

## Files
- `scripts/test-instance-e2e.ps1`
- `src-tauri/src/modules/telegram_inbound/mod.rs`
- `package.json`, `src-tauri/Cargo.toml`
