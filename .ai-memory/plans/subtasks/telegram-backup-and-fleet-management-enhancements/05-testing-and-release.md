# Subtask 05: E2E Testing, Pre-flight Quality Gates & Release Ceremony

**Subtask ID**: `sub-05-release`  
**Parent Plan**: `.ai-memory/plans/telegram-backup-and-fleet-management-enhancements.md`  
**Status**: PENDING  

## Objectives
1. Run all unit test suites (`telegram_inbound`, `repo_db`).
2. Run transient sandbox test (`agm instances test-create` / `test-clean`).
3. Run pre-flight quality gates: `cargo fmt`, `cargo clippy`, `npm run build`.
4. Minor version bump to `v4.243.0`, update changelogs and READMEs with `@aukgit` attribution only.
5. Push commit and tag, verify CI/CD telemetry via `gitmap pe -t`.
