# Account Switch Verification & Audit Specification

## 1. Scope & Standards

This specification establishes the verification criteria, diagnostic checks, and audit trails required for validating manual and automatic account switching across Antigravity tools.

---

## 2. Verification Checklist & Acceptance Gates

| Gate # | Phase | Verification Rule | Tool / Source | Pass Condition |
| :--- | :--- | :--- | :--- | :--- |
| **G1** | Sandbox Provisioning | Isolated instance creation | `agm instances test-create <name> --json` | Valid JSON, instance prefixed with `test-`, isolated AppData directory |
| **G2** | Prompt Seeding | Multimodal prompt registration | `active_prompts` & `backup_prompts` | Prompt text present, `has_images = 1`, `image_payload` populated |
| **G3** | Pre-Switch Snapshot | Backup creation | `backup-prompts.db` & resume task file | Row created in `prompt_backups` with `is_restored = 0`, file exists in workspace |
| **G4** | Switch Execution | Account switch command | `agm switch` & `gitmap switch` | Exit code 0, target credentials bound, caller IDE untouched |
| **G5** | Settling Period | 10-second stability check | Process monitoring / timer | Process alive after $\ge 10$ seconds, port/channel active |
| **G6** | Prompt Re-injection | Prompt restored post-switch | `active_prompts` & `.antigravity_resume_task.<inst>.json` | Status transition to `dispatched`, image payload verified |
| **G7** | Audit Log Audit | Switch logged in task history | `task_index.db:hot_tasks_cache` | `action = 3 (SwitchAccount)`, accurate timestamp, metadata recorded |
| **G8** | Error Diagnostics | SQLite check & error log | `agm errors list` & `agm errors fix` | Zero unhandled critical runtime errors, 8/8 diagnostic checks pass |
| **G9** | Clean Teardown | Zero leftover test files | `agm instances test-clean <name> --json` | All `test-*` processes and folders purged, developer IDE healthy |

---

## 3. Audit Trail Schema: `task_index.db:hot_tasks_cache`

```sql
SELECT
    task_id,
    action,           -- 3 = SwitchAccount
    account_email,
    instance_id,
    details,
    created_at
FROM hot_tasks_cache
ORDER BY created_at DESC
LIMIT 10;
```

### Action Code Mapping (`AuditAction`)
- `1`: `CreateInstance`
- `2`: `DeleteInstance`
- `3`: `SwitchAccount`
- `4`: `AutoSwitchTriggered`
- `5`: `PromptBackup`
- `6`: `PromptRestore`

---

## 4. Diagnostics & Error Telemetry Verification

The diagnostics system validates the health of the SQLite speed databases and daemon configurations:
```powershell
# List active errors and telemetry
cargo run --bin agm -- errors list --json

# Execute self-healing repairs
cargo run --bin agm -- errors fix
```

The repair ladder must report:
1. `Storage config integrity`: OK
2. `Database connectivity`: OK
3. `Table schemas`: OK
4. `WAL mode enabled`: OK
5. `Orphan task cleanup`: OK
6. `Index sanity`: OK
7. `Foreign key integrity`: OK
8. `PRAGMA quick_check(1)`: OK
Result: `8/8 PASSED`.
