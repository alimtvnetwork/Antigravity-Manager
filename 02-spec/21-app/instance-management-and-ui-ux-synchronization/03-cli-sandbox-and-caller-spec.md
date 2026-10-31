# Specification: Caller IDE Self-Preservation & Transient Test Sandbox CLI

## 1. Caller IDE Self-Preservation Invariant
- **Core Principle:** An automated test, CLI command, or background daemon MUST NEVER terminate the developer's working IDE session (`detect_caller_instance_id()`).
- **Detection Algorithm:**
  1. Inspect environment variable `AGM_INSTANCE_ID`.
  2. Inspect parent process PID hierarchy and compare against registered instance PIDs in SQLite.
  3. Inspect current working directory path against instance workspace storages.
- **Enforcement:**
  - If a command targets the detected caller instance ID (e.g. `agm instances stop <caller_id>`), the command MUST fail-fast with exit code 2 and message:
    `Refusing to stop active caller IDE instance '<caller_id>' without --force flag.`

## 2. Transient Test Sandboxing CLI
- **Commands:**
  - `agm instances test-create [--name <name>]`:
    - Rapidly clones or constructs a lightweight mock instance in `<temp_dir>/agm-test-instances/<name>`.
    - Seeds mock `storage.json` and minimal config.
    - Registers the instance in SQLite tagged as `is_test: true`.
    - Returns JSON payload with test instance metadata.
  - `agm instances test-clean [--all]`:
    - Identifies all instances tagged with `is_test: true` or matching test prefixes (`test-*`, `agm-test-*`).
    - Kills any running test PIDs.
    - Removes filesystem directory cleanly.
    - Purges database records.
- **Zero-Storage Remnant Policy:** Test executions must leave 0.0 MB remnants on disk after test suite completion.
