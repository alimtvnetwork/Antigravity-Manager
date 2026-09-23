# Plan 66: AGM Native Terminal CLI Expanded Commands Suite

Spec Reference: [02-spec/21-app/20-agm-cli-expanded-commands.md](../../../02-spec/21-app/20-agm-cli-expanded-commands.md)
Status: COMPLETED
Slug: 66-agm-cli-expanded-commands

## 1. Architectural Context & Intent

Extends the native `agm` terminal command-line tool with high-utility commands mirroring the developer experience of GitMap:
- `agm doctor` / `agm check`: Multi-point health check across SQLite DBs (`accounts.json`, `email_vault.db`, `repo_prompts.db`, `security.db`, `thinking_store.db`), proxy gateway port 8045, sandbox instances, and PATH registration.
- `agm accounts` / `agm acc`: Inspection of registered accounts, tiers, and weekly quotas with `--active` and `--json` support.
- `agm switch <email>`: Direct terminal account switching with device profile synchronization.
- `agm prompts` / `agm prompts ls`: Listing active prompt tasks from `repo_prompts.db` and template categories from `01-prompts/`.
- `agm proxy` [status | test]: Local proxy engine state inspection and loopback latency probe.
- `agm sync`: Synchronize accounts, instance profiles, and DB schemas.
- `agm pull`: Synchronize repository branch with remote origin (`git pull origin main`).
- `agm clean` / `agm purge`: Safe cleanup of temporary build artifacts and test folders strictly protecting database vaults.
- `agm logs`: Terminal viewing of recent application logs with `--tail <N>` and `--filter <text>`.

Also bound all expanded commands into the inbound email remote control engine (`email_inbound.rs`) with 2-phase receipts and 10-second debounce rate-limiting.

## 2. Deliverables & Subtask Summary

| Traceable ID | Scope & Deliverable | Status |
|---|---|---|
| `Task-01` | Canonical Spec 20 & Spec Index Registration | COMPLETED |
| `Task-02` | Implement Expanded Commands in `src-tauri/src/bin/agm.rs` | COMPLETED |
| `Task-03` | Bind Inbound Email Remote Control to New Commands in `email_inbound.rs` | COMPLETED |
| `Task-04` | Expand Local-Only E2E Test Suite (`40-test-email-permutations-e2e.py`) | COMPLETED |
| `Task-05` | Version Bump (`v4.63.0`), Changelog, Plan Consolidation & Release Ceremony | COMPLETED |

## 3. Verification & Quality Gates

- `cargo check --manifest-path src-tauri/Cargo.toml --bin agm`: Verified 0 errors.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib modules::email_inbound::tests`: Verified 9/9 passed 100%.
- `python 03-ai-scripts/40-test-email-permutations-e2e.py`: Verified 30/30 permutations + all CLI commands passed 100%.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: Verified formatted.
- Full atomic git commit pushed to `origin main` and tagged release `v4.63.0` triggered.
