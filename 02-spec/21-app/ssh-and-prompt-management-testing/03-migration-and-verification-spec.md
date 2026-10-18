# Migration, Schema Verification & Testing Protocol

> **Specification Reference:** `02-spec/21-app/ssh-and-prompt-management-testing/03-migration-and-verification-spec.md`  
> **Parent Initiative:** `ssh-and-prompt-management-testing`  
> **Author:** `@aukgit`  
> **Status:** `[DRAFT / APPROVED]`

---

## 1. Overview & Architectural Scope

This specification defines the migration protocols, database schema verification standards, backward compatibility invariants, and verification matrix for the **SSH and Prompt Management Testing** feature suite across Antigravity-Manager (AGM).

The scope encompasses 6 interconnected functional pillars:
1. **SSH Fleet Telemetry & Credential Redaction**: Complete removal of administrative usernames (`Administrator`, `root`) and passwords from UI fleet cards; clear display of operational metadata (Node IP, Node Alias, running instance count, account emails).
2. **Collapsible Prompt Tree & Automated Prompts Filtering**: Default-hidden prompt tree container, default-filtered automated prompt entries (system messages, goal heartbeats, tool calls), and interactive unhide toggles with CLI parity.
3. **Multimodal Prompt Image Clipboard**: Dual-format image copying supporting clean Markdown Base64 data URIs for IDEs and rich HTML clipboard payloads with embedded images for Microsoft OneNote.
4. **Prompt Backup & Restore Verification**: Guaranteed snapshotting of in-flight prompts with full multimodal image payloads into SQLite databases upon switching/rotation, verified via structured audit traces (`[PROMPT_BACKUP]`, `[PROMPT_RESTORE]`).
5. **Standalone Telegram Hub & Interactive Guide**: Extraction of Telegram inbound daemon settings from the settings modal into a standalone navigation route in the navbar dropdown, accompanied by an interactive BotFather setup guide modal and dedicated CLI commands (`agm telegram ...`).
6. **Speed Error Database (`errors.db`) & AI Diagnostics**: Dedicated SQLite WAL speed database logging all system failures, integration with the Antigravity Tools log viewer, self-healing CLI tools (`agm errors list`, `agm errors fix`, `agm failed-commands`), and AI skill documentation.

---

## 2. Database Schema Verification & Migrations

AGM adheres strictly to the decoupled **Split SQLite Database Architecture**. Operational data is partitioned to ensure concurrent multi-process access, zero lock contention, and zero corruption during unexpected process restarts.

### 2.1 Speed Error Database (`errors.db`)

The error database is located at `~/.antigravity_tools/errors.db` (or `data/errors.db` in local portable mode). It operates under Write-Ahead Logging (`WAL`) mode with `NORMAL` synchronous pragma.

#### Table Definition: `errors`
```sql
CREATE TABLE IF NOT EXISTS errors (
    id TEXT PRIMARY KEY,
    error_code TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    stack_trace TEXT,
    source TEXT,
    trigger_action TEXT,
    metadata_json TEXT,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_errors_created_at ON errors(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_errors_code ON errors(error_code);
CREATE INDEX IF NOT EXISTS idx_errors_severity ON errors(severity);
```

#### Verification Rules:
1. **WAL & Pragma Verification**: At connection initialization (`src-tauri/src/modules/error_db/db.rs`), verify that `PRAGMA journal_mode = WAL;`, `PRAGMA synchronous = NORMAL;`, `PRAGMA busy_timeout = 5000;`, and `PRAGMA foreign_keys = ON;` execute successfully.
2. **Deterministic Time Precision**: `created_at` MUST store millisecond Unix epoch integers (`chrono::Utc::now().timestamp_millis()`) to guarantee sub-millisecond sorting resolution during burst error events.
3. **Error Code Normalization**: `error_code` must match standard AGM diagnostic taxonomy (`E1001`–`E9001`) with automatic fallback to `E1001` for untyped exceptions.

---

### 2.2 Split Repo & Prompt Database (`repo_prompts.db`)

The repository and prompt tracking database is located at `~/.antigravity_tools/repo_prompts.db`.

#### Table: `active_prompts` (Multimodal Payload Extension)
```sql
CREATE TABLE IF NOT EXISTS active_prompts (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    repo_path TEXT NOT NULL,
    prompt_content TEXT NOT NULL,
    model TEXT,
    session_id TEXT,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    image_payload TEXT,
    FOREIGN KEY(project_id) REFERENCES running_projects(id)
);

CREATE INDEX IF NOT EXISTS idx_active_prompts_instance_status 
ON active_prompts(instance_id, status);
```

#### Migration Step: `image_payload` Column Backfill
When opening `repo_prompts.db`, execute the following idempotent migration:
```rust
// Idempotent schema migration: add image_payload column if missing
crate::error::record_ignored(
    conn.execute("ALTER TABLE active_prompts ADD COLUMN image_payload TEXT", []),
    "add image_payload column migration",
);
```

#### Table: `failed_commands` (Self-Healing Command Telemetry)
```sql
CREATE TABLE IF NOT EXISTS failed_commands (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command TEXT NOT NULL DEFAULT '',
    full_args TEXT NOT NULL DEFAULT '',
    domain TEXT NOT NULL DEFAULT 'root',
    error_code TEXT NOT NULL DEFAULT 'E1001',
    message TEXT NOT NULL DEFAULT '',
    suggestions TEXT NOT NULL DEFAULT '',
    hit_count INTEGER NOT NULL DEFAULT 1,
    working_dir TEXT NOT NULL DEFAULT '',
    agm_version TEXT NOT NULL DEFAULT '',
    is_resolved INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_failed_commands_cmd ON failed_commands(command, domain);
CREATE INDEX IF NOT EXISTS idx_failed_commands_hits ON failed_commands(hit_count DESC);
```

#### Verification Rules:
1. **FIFO Prompt Invariant**: Any query restoring or dispatching prompts from `active_prompts` MUST use:
   ```sql
   SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, image_payload
   FROM active_prompts
   WHERE instance_id = ?1 AND status IN ('backed_up', 'queued', 'pending')
   ORDER BY created_at ASC, id ASC
   LIMIT 1;
   ```
   *Strict Invariant*: Inverted LIFO ordering (`ORDER BY updated_at DESC`) is strictly prohibited.
2. **Cross-Instance Re-attribution**: Upon rotating between instances (`target.instance_id != current_instance_id`), in-flight prompts in `active_prompts` must be re-attributed to `target.instance_id` to prevent prompt amnesia.

---

## 3. Backward Compatibility Invariants

All implementation changes must adhere to the core architectural invariants of Antigravity-Manager:

### 3.1 Strict Relative Git Paths
- **Rule**: All file paths stored in database envelopes, export snapshots, and markdown documentation must be relative to the repository root.
- **Prohibition**: Absolute filesystem paths (`C:\...`, `/home/...`) and URI schemes (`file:///...`) are strictly banned from persistent schemas and UI display strings.

### 3.2 Positive Boolean Field Naming
- **Rule**: All state flags, options, and model fields must use positive boolean prefixes (`is`, `has`, `should`, `can`).
- **Examples**:
  - `is_automated_hidden: boolean` (NOT `hide_automated` or `show_all`)
  - `is_bot_running: boolean` (NOT `bot_stopped`)
  - `is_online: boolean` (NOT `is_offline`)
  - `is_resolved: number` (0 or 1 in SQLite; NOT `unresolved`)
- **Conditionals**: Avoid double negatives (`!isNotRunning`). Use explicit positive conditions (`if (isBotRunning)`).

### 3.3 Zero Silent Errors
- **Rust Backend**:
  - All fallible operations must propagate via `AppError` or log via `tracing`.
  - For genuinely best-effort cleanup or notifications, wrap with `crate::error::record_ignored(result, "action_name")` and attach an explicit `// Justification:` comment explaining safety.
- **Frontend / React**:
  - Unhandled exceptions and IPC failures must flow into `src/stores/error-store.ts`.
  - Use `useErrorStore.getState().captureError(err, meta)` for critical user-facing failures (launches modal).
  - Use `useErrorStore.getState().trackWarning(err, meta)` for benign/best-effort failures (records into error history without interrupting the user). Bare `catch {}` is strictly forbidden.

### 3.4 Process Spawning & Job Breakaway
- Child process spawning on Windows must never unconditionally use `CREATE_BREAKAWAY_FROM_JOB` (`0x01000000`) without a fallback ladder (`spawn_with_job_breakaway_fallback`). If running inside a restricted job object, `CreateProcessW` must gracefully fall back to standard flags.

---

## 4. Verification Matrix for Core Requirements

| Req # | Requirement Area | Target Components | Expected Behavior | Verification Procedure |
|---|---|---|---|---|
| **REQ-01** | SSH Fleet Telemetry & Credential Redaction | `src/components/instances/fleet/FleetMachineCard.tsx`<br>`src/components/instances/fleet/FleetMachinesSection.tsx` | - Administrative usernames (`Administrator`, `root`, etc.) and passwords are never rendered.<br>- Node IP, alias, running instance count, and bound account emails are prominently displayed.<br>- Offline nodes show clear status indicator. | 1. Mount fleet machine section with mock remote nodes.<br>2. Inspect rendered DOM to confirm absence of username/password strings.<br>3. Verify account emails and instance count pill render correctly. |
| **REQ-02** | Collapsible Prompt Tree & Automated Filter | `src/components/prompts/tree/PromptTreeViewModal.tsx`<br>`src/components/prompts/tree/TreePanel.tsx`<br>`src/components/prompts/tree/usePromptTreeState.ts` | - Tree sidebar initialized in collapsed/hidden state by default (`isTreeCollapsed = true`).<br>- Automated prompt turns (system messages, goal heartbeats, tool calls) hidden by default (`isAutomatedHidden = true`).<br>- Pill button toggles filter visibility.<br>- CLI flag `--include-system` unhides entries. | 1. Open prompt tree modal; verify sidebar is collapsed and automated turns are filtered.<br>2. Click filter pill; verify system turns appear.<br>3. Run `agm tree --include-system` to verify CLI parity. |
| **REQ-03** | Multimodal Image Clipboard (IDE & OneNote) | `src/components/prompts/DetailHeader.tsx`<br>`src/components/prompts/PromptInstructionView.tsx`<br>`src/utils/clipboard.ts` | - "Copy Text" generates clean Markdown with embedded Base64 data URI (`![Image](data:image/png;base64,...)`).<br>- "Copy Image" writes standard CF_DIB / PNG blob and HTML wrapper (`<img src="...">`) to clipboard.<br>- Paste works seamlessly in VS Code, Cursor, and Microsoft OneNote. | 1. Open prompt turn containing image.<br>2. Click "Copy Single Text"; paste into text editor; confirm valid Markdown format.<br>3. Click "Copy for OneNote/IDE"; paste into OneNote; verify image renders inline. |
| **REQ-04** | Prompt Backup & Restore Verification | `src-tauri/src/modules/repo_db/backup.rs`<br>`src-tauri/src/modules/repo_db/auto_resume.rs`<br>`src-tauri/src/modules/auto_switcher/rotation.rs` | - Account switch snapshots in-flight prompts + image payloads to `repo_prompts.db`.<br>- Structured log emitted: `[PROMPT_BACKUP] Instance: {id} \| Seq: {seq} \| Images: {count}`.<br>- Instance resume restores prompt with log: `[PROMPT_RESTORE] Restored prompt for instance {id}`.<br>- FIFO order strictly preserved. | 1. Enqueue prompt with multimodal image.<br>2. Trigger account fast-forward rotation (`agm ff`).<br>3. Check logs for `[PROMPT_BACKUP]` and verify `active_prompts` table in `repo_prompts.db`.<br>4. Restart instance; verify `[PROMPT_RESTORE]` and FIFO execution. |
| **REQ-05** | Standalone Telegram Hub & Guide | `src/components/navbar/Navbar.tsx`<br>`src/components/navbar/NavDropdowns.tsx`<br>`src/components/telegram/TelegramGuideModal.tsx`<br>`src/pages/Telegram.tsx`<br>`src-tauri/src/bin/agm/telegram_dispatch.rs` | - Telegram removed from cluttered settings modal.<br>- Promoted to top-level navigation route (`/telegram`) and dropdown action beside instances.<br>- Interactive setup guide modal with step-by-step `@BotFather` walkthrough.<br>- CLI commands (`agm telegram status`, `agm telegram send`, `agm telegram listen`) fully operational. | 1. Click Telegram item in navbar dropdown; verify `/telegram` page or setup guide modal opens.<br>2. Verify step-by-step BotFather guide tabs and copy buttons work.<br>3. Run `agm telegram config` and `agm telegram status` in terminal. |
| **REQ-06** | Speed Error Database & AI Self-Healing | `src-tauri/src/modules/error_db/`<br>`src-tauri/src/bin/agm/error_cmds.rs`<br>`src-tauri/src/bin/agm/help_text.rs`<br>`src/stores/error-store.ts`<br>`src/components/errors/error-modal.tsx`<br>`.agents/skills/agm-ai-diagnostics-and-logs/skill.md` | - `errors.db` captures frontend and backend errors in SQLite WAL format.<br>- Error modal tabs include suggested fixes and AI diagnostic report generator.<br>- CLI commands: `agm errors list`, `agm errors fix`, `agm failed-commands`.<br>- AI diagnostics skill provides complete manual and error code resolution guide. | 1. Trigger test error via UI or backend; verify insertion in `~/.antigravity_tools/errors.db`.<br>2. Run `agm errors list` in terminal; confirm table output.<br>3. Run `agm errors fix --dry-run` to test self-healing checks.<br>4. Inspect AI skill reference in `.agents/skills/agm-ai-diagnostics-and-logs/skill.md`. |

---

## 5. Verification Commands & Pre-flight Testing

Execute the following commands to validate all components and prevent regressions before release:

```powershell
# 1. Frontend type-checking and production build
npm run build

# 2. Rust syntax, formatting and clippy gates
cd src-tauri
cargo fmt -- --check
cargo clippy --all-targets --all-features
cd ..

# 3. Targeted Rust unit tests for error_db and repo_db modules
cd src-tauri
cargo test --lib modules::error_db
cargo test --lib modules::repo_db
cd ..

# 4. CLI command syntax and help verification
cargo run --bin agm -- errors --help
cargo run --bin agm -- telegram --help
cargo run --bin agm -- tree --help
```

---

*Maintained by @aukgit*
