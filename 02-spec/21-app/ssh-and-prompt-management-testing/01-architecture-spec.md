# Architecture Specification: SSH and Prompt Management Testing

## 1. SSH Fleet Telemetry & Credential Redaction

### 1.1 Requirements & Threat Model
In the Instance view, the SSH fleet cards previously displayed administrative OS usernames (e.g. `Administrator`, `root`) and password hints. In shared or recorded environments, this leaks internal credential metadata.
Users require:
- **Redaction of sensitive credentials**: Never render administrative usernames (`Administrator`, `root`, etc.) or password indicators in the card header or summary view.
- **Display operational attributes**: Highlight Node IP address, Node Alias, running instance counts, and the associated account emails bound to those instances.

### 1.2 Data Contract
The `FleetMachineCard` and `FleetMachinesSection` components shall consume node records and enrich them with instance telemetry:
```typescript
interface FleetNodeTelemetry {
    alias: string;
    ipAddress: string;
    runningInstanceCount: number;
    accountEmails: string[];
    isOnline: boolean;
}
```

---

## 2. Collapsible Prompt Tree & Automated Prompts Filter

### 2.1 Default-Hidden Collapsible Tree View
- The prompt tree view panel shall feature a toggle button (`ChevronLeft` / `ChevronRight`) allowing users to collapse the tree sidebar into a thin icon rail or hide it entirely.
- By default, the prompt tree views list shall be initialized in the collapsed/hidden state to preserve screen space for prompt inspection and results.

### 2.2 Automated Prompts Filter (`isAutomatedHidden`)
- System messages, prompt goal heartbeats, and telemetry entries (`PromptOriginTier.SystemMessage`, `PromptOriginTier.ToolOutput`) shall be filtered out by default (`isAutomatedHidden = true`).
- A quick-toggle pill button with a filter icon shall allow users to unhide automated turns on demand.
- CLI parity: `agm observe --include-system` or `agm prompts list --all` allows inspecting automated entries from the terminal.

---

## 3. Multimodal Prompt Image Clipboard

### 3.1 IDE & OneNote Clipboard Format Matrix
When copying prompt turns that contain multimodal images:
1. **Single Text Copy**: Copies clean Markdown containing either relative paths or embedded Base64 data URIs (`![Image](data:image/png;base64,...)`) so text-only editors or IDE chat inputs receive clean text.
2. **Rich Clipboard HTML with Embedded Image**: Formats an `HTML Format` clipboard payload containing `<img src="data:image/png;base64,...">` along with an optional CF_DIB / image/png blob. This enables direct pasting into Microsoft OneNote, Word, and rich desktop applications.

---

## 4. Prompt Backup & Restore Verification

### 4.1 Backup Pipeline Invariants
- When an account switch or instance rotation occurs, the running prompt (including raw prompt text, multimodal image payloads, active sequence numbers, and goal telemetry) is atomically committed to SQLite `repo_prompts.db` and `.antigravity_resume_task.json`.
- The backup routine records an explicit audit trace:
  `[PROMPT_BACKUP] Instance: {id} | Seq: {seq} | Images: {count} | Bytes: {size}`
- Upon instance resume/restore, the restore runner logs:
  `[PROMPT_RESTORE] Restored prompt for instance {id} | Status: active`

---

## 5. Standalone Telegram Section in Navbar Dropdown

### 5.1 UX & Architecture
- Remove the Telegram inbound/bot settings from inside the cluttered `SettingsModal`.
- Promote Telegram to a top-level action in the navbar settings dropdown (next to the instance selector dropdown).
- Launch a dedicated `TelegramGuideModal` featuring:
  - Step-by-step setup guide for creating a bot via `@BotFather`.
  - Setting up the Telegram bot token and chat ID.
  - Recommended CLI commands (`agm telegram status`, `agm telegram send`, `agm telegram listen`).

---

## 6. Speed Error Database (`errors.db`) & AI Diagnostics

### 6.1 Database Schema (`errors.db`)
Located at `~/.antigravity_tools/errors.db`:
```sql
CREATE TABLE IF NOT EXISTS application_errors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    error_code TEXT NOT NULL,
    severity TEXT NOT NULL, -- 'critical', 'error', 'warn', 'info'
    message TEXT NOT NULL,
    stack_trace TEXT,
    source TEXT NOT NULL,
    trigger_action TEXT,
    metadata_json TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_errors_created_at ON application_errors(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_errors_severity ON application_errors(severity);
```

### 6.2 CLI Parity
- `agm errors list [--limit N] [--severity S] [--json]`: Lists recent errors in tabular or JSON format.
- `agm errors clear [--before DAYS]`: Purges old error logs.
- `agm errors fix`: Runs automated diagnostics on detected error patterns (e.g., stale lockfiles, invalid config, clipboard permissions) to apply self-healing remediations.
