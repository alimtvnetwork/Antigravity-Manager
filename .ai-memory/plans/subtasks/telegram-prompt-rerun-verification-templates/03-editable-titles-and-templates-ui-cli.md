---
plan: telegram-prompt-rerun-verification-templates
subtask: "03"
title: Editable Resend As Titles and Templates Architecture (UI & CLI Parity)
domain: backend-cli-and-ui
depends_on: []
citations:
  architecture_spec: ../../../../02-spec/21-app/telegram-prompt-rerun-verification-templates/03-editable-titles-and-templates-ui-cli-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/suffix_templates.rs
  - src-tauri/src/modules/repo_db/schema.rs
  - src-tauri/src/bin/agm/prompt_cmds.rs
  - src-tauri/src/bin/agm/common.rs
  - src-tauri/src/commands/project_cmds.rs
  - src/components/instances/PromptTreeView/SuffixTemplateModal.tsx
  - src/components/instances/PromptTreeView/ResendAsModal.tsx
  - src/components/instances/PromptTreeView/DetailHeader.tsx
  - src/components/instances/PromptTreeView/ConversationContextMenu.tsx
status: pending
---

# Subtask 03: Editable Resend As Titles and Templates Architecture (UI & CLI Parity)

## 1. Context & Motivation

During conversation re-dispatch and verification workflows in Antigravity-Manager, users frequently use the "Resend As" action to create targeted follow-up threads or inject verification nudges. Previously, custom verification templates existed only in the browser's `localStorage` (`verificationTemplates.ts`), preventing terminal CLI operators and headless environments from querying or modifying templates. Furthermore, title prefixes (such as `[Verify]`, `[Fix]`, `[Rerun]`) were hardcoded or manually typed without a persistent preset engine or pattern generator.

Additionally, as identified in the root cause analysis, invoking CLI commands such as `agm prompt template list` or `agm prompt resend-as list` risks catastrophic accidental prompt dispatch into open IDE workspaces if those subcommands are not explicitly registered in `is_prompt_subcommand` in `src-tauri/src/bin/agm/common.rs`.

This subtask guides Worker 03 through implementing:
1. Two SQLite tables in `repo_prompts.db`: `prompt_suffix_templates` and `prompt_resend_as_templates` with default seeds.
2. A dedicated Rust business module `src-tauri/src/modules/suffix_templates.rs` with CRUD and title pattern interpolation (`{title}`, `{date}`, `{time}`, `{seq}`).
3. Comprehensive Tauri IPC command handlers in `src-tauri/src/commands/project_cmds.rs`.
4. CLI Subcommand Whitelist Invariant protection in `src-tauri/src/bin/agm/common.rs`.
5. Full terminal CLI suite in `src-tauri/src/bin/agm/prompt_cmds.rs` (`agm prompt template ...` and `agm prompt resend-as ...`).
6. Frontend bidirectional synchronization in `SuffixTemplateModal.tsx` and a new `ResendAsModal.tsx`.
7. UI integration into `DetailHeader.tsx` and `ConversationContextMenu.tsx`.
8. Rigorous 5-stage write verification protocol (validation, atomic transaction, read-back verification, cross-surface event broadcast, and audit logging).

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src-tauri/src/modules/repo_db/schema.rs` | `init_tables` | Create `prompt_suffix_templates` and `prompt_resend_as_templates` tables with indices and default record seeding. |
| `src-tauri/src/modules/suffix_templates.rs` | `SuffixTemplateRow`, `ResendAsTemplateRow`, CRUD functions | Implement SQLite operations, validation logic, read-back verification, and `apply_title_pattern`. |
| `src-tauri/src/modules/mod.rs` | Module exports | Export `pub mod suffix_templates;`. |
| `src-tauri/src/commands/project_cmds.rs` | Tauri commands | Implement 10 async IPC commands for listing, getting, saving, deleting, and setting defaults for templates and presets. |
| `src-tauri/src/commands/mod.rs` & `src-tauri/src/lib/run.rs` | Command exports & handler table | Re-export and register all 10 IPC commands in `generate_handler!`. |
| `src-tauri/src/bin/agm/common.rs` | `is_prompt_subcommand` | Whitelist `template`, `templates`, `suffix`, `resend-as`, `resend-titles` to prevent raw prompt IDE leakage. |
| `src-tauri/src/bin/agm/prompt_cmds.rs` | `dispatch_prompt_domain`, `cmd_prompts_template`, `cmd_prompts_resend_as` | Dispatch and execute template and resend-as CLI subcommands with `--json` support. |
| `src/components/instances/PromptTreeView/SuffixTemplateModal.tsx` | `SuffixTemplateModal` | Transition from pure `localStorage` to Tauri IPC first with graceful fallback, optimistic updates, and write verification. |
| `src/components/instances/PromptTreeView/ResendAsModal.tsx` | `ResendAsModal` (New) | Build preset manager with live title preview, variable token tags, target mode switch, and bound suffix dropdown. |
| `src/components/instances/PromptTreeView/DetailHeader.tsx` | `DetailHeader` | Add "Resend As" preset dropdown menu inside segmented action capsule, wiring selection to `newConversationTitle`. |
| `src/components/instances/PromptTreeView/ConversationContextMenu.tsx` | `ConversationContextMenu` | Add "Resend As..." submenu populated with active title presets. |

---

## 3. Step-by-Step Implementation Checklist

### Step 1: Database Migration & Schema Seeding (`src-tauri/src/modules/repo_db/schema.rs`)
- [ ] In `init_tables()`, execute DDL for `prompt_suffix_templates`:
  ```sql
  CREATE TABLE IF NOT EXISTS prompt_suffix_templates (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      description TEXT,
      category TEXT NOT NULL DEFAULT 'custom',
      template TEXT NOT NULL,
      is_default INTEGER NOT NULL DEFAULT 0,
      created_at TEXT NOT NULL,
      updated_at TEXT NOT NULL
  );
  CREATE INDEX IF NOT EXISTS idx_prompt_suffix_templates_category ON prompt_suffix_templates(category);
  CREATE INDEX IF NOT EXISTS idx_prompt_suffix_templates_default ON prompt_suffix_templates(is_default);
  ```
- [ ] Execute DDL for `prompt_resend_as_templates`:
  ```sql
  CREATE TABLE IF NOT EXISTS prompt_resend_as_templates (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      title_pattern TEXT NOT NULL,
      target_mode TEXT NOT NULL DEFAULT 'new',
      suffix_id TEXT,
      is_default INTEGER NOT NULL DEFAULT 0,
      created_at TEXT NOT NULL,
      updated_at TEXT NOT NULL
  );
  CREATE INDEX IF NOT EXISTS idx_prompt_resend_as_default ON prompt_resend_as_templates(is_default);
  ```
- [ ] Seed default records if the tables are empty:
  - Resend As Presets: `[Verify] {title}`, `[Fix] {title}`, `[Rerun] {title}`, `[Audit] {title}`, `[Follow-up] {title}`.
  - Suffix Templates: `is-it-done` (default), `is-it-released`, `are-you-confident`, `edge-cases-and-tests`, `security-audit`, `pr-review-ready`, `test-and-coverage`, `refactor-dry`.

### Step 2: Rust Backend Subsystem (`src-tauri/src/modules/suffix_templates.rs`)
- [ ] Create `src-tauri/src/modules/suffix_templates.rs`.
- [ ] Define serialization structs: `SuffixTemplateRow`, `ResendAsTemplateRow`, `CreateSuffixTemplateInput`, `CreateResendAsTemplateInput`.
- [ ] Implement CRUD functions using `repo_db::connect_db()` with WAL mode:
  - `list_suffix_templates() -> Result<Vec<SuffixTemplateRow>, String>`
  - `get_suffix_template(id: &str) -> Result<Option<SuffixTemplateRow>, String>`
  - `save_suffix_template(input: &CreateSuffixTemplateInput) -> Result<SuffixTemplateRow, String>`
  - `delete_suffix_template(id: &str) -> Result<bool, String>`
  - `set_default_suffix_template(id: &str) -> Result<(), String>`
  - `list_resend_as_templates() -> Result<Vec<ResendAsTemplateRow>, String>`
  - `get_resend_as_template(id: &str) -> Result<Option<ResendAsTemplateRow>, String>`
  - `save_resend_as_template(input: &CreateResendAsTemplateInput) -> Result<ResendAsTemplateRow, String>`
  - `delete_resend_as_template(id: &str) -> Result<bool, String>`
  - `set_default_resend_as_template(id: &str) -> Result<(), String>`
- [ ] Implement `apply_title_pattern(pattern: &str, current_title: &str, date_str: Option<&str>, seq_num: Option<u64>) -> String`.
- [ ] Export module in `src-tauri/src/modules/mod.rs`.
- [ ] Write targeted unit tests covering CRUD, default toggling, and pattern interpolation.

### Step 3: Implement Post-Write Verification Protocol
- [ ] In `save_suffix_template` and `save_resend_as_template`:
  - **Pre-write validation**: Check ID regex (`^[a-zA-Z0-9_\-\.]{2,64}$`), non-empty strings, valid category.
  - **Transaction**: Execute write inside SQLite transaction.
  - **Read-back verification**: Query the inserted row immediately and assert all fields match. Return explicit `Err` on mismatch.
  - **Audit Logging**: Log write event with template ID and content hash.

### Step 4: Tauri IPC Commands (`src-tauri/src/commands/project_cmds.rs`)
- [ ] Expose 10 async commands in `src-tauri/src/commands/project_cmds.rs` wrapping the module functions with `tokio::task::spawn_blocking`.
- [ ] Re-export commands in `src-tauri/src/commands/mod.rs`.
- [ ] Register all 10 commands in `src-tauri/src/lib/run.rs` inside `generate_handler![...]`.

### Step 5: CLI Subcommand Whitelist Invariant (`src-tauri/src/bin/agm/common.rs`)
- [ ] In `is_prompt_subcommand`, add whitelist entries:
  - `"template" | "templates" | "suffix" | "resend-as" | "resend-titles"`
- [ ] Verify that running `agm prompt template list` does NOT execute `cmd_prompts_send` or leak to the Antigravity IDE.

### Step 6: CLI Command Implementation (`src-tauri/src/bin/agm/prompt_cmds.rs`)
- [ ] In `dispatch_prompt_domain`, add routes for:
  - `"template" | "templates" | "suffix" => cmd_prompts_template(args)`
  - `"resend-as" | "resend-titles" => cmd_prompts_resend_as(args)`
- [ ] Implement `cmd_prompts_template(args)`:
  - Subcommands: `list`, `get`, `add`, `remove`, `set-default`.
  - Flags: `--name`, `--category`, `--file`, `--default`, `--json`.
  - Output: Formatted terminal tables or compact JSON.
- [ ] Implement `cmd_prompts_resend_as(args)`:
  - Subcommands: `list`, `get`, `add`, `remove`, `set-default`.
  - Flags: `--name`, `--pattern`, `--mode`, `--suffix`, `--default`, `--json`.
  - Output: Formatted terminal tables or compact JSON.
- [ ] Update `cmd_prompts_help` to document template and resend-as syntax.

### Step 7: Update `SuffixTemplateModal.tsx` for Tauri IPC
- [ ] In `SuffixTemplateModal.tsx`:
  - Replace direct synchronous `getCustomSuffixTemplates()` with async fetch: call `invoke('list_prompt_suffix_templates')`.
  - If `invoke` rejects (e.g. running outside Tauri in browser), fall back to `localStorage`.
  - On save: invoke `save_prompt_suffix_template`.
  - On delete: invoke `delete_prompt_suffix_template`.
  - On set default: invoke `set_default_prompt_suffix_template`.
  - Wire error handling through `useErrorStore.getState().captureException`.

### Step 8: Build `ResendAsModal.tsx`
- [ ] Create `src/components/instances/PromptTreeView/ResendAsModal.tsx`.
- [ ] Implement interactive preset editor:
  - Preset ID and Name inputs.
  - Pattern input with quick-click token pills: `{title}`, `{date}`, `{time}`, `{seq}`.
  - Live interactive preview showing formatted title dynamically.
  - Target Mode selector (`New Conversation` vs `Current Conversation`).
  - Suffix Template binder dropdown (populated from active suffix templates).
  - List of existing presets with Edit, Delete, Set Default actions.
  - Accessible dialog controls (Escape to close, autofocus, dark-glass theme).

### Step 9: Integrate into `DetailHeader.tsx` & `ConversationContextMenu.tsx`
- [ ] In `DetailHeader.tsx`:
  - Add "Resend As" dropdown in action capsule displaying title presets.
  - Clicking a preset evaluates `applyTitlePattern`, sets `newConversationTitle`, switches mode, sets `confirmationSuffix`, and focuses input.
  - Add "Configure Presets..." opening `ResendAsModal`.
- [ ] In `ConversationContextMenu.tsx`:
  - Add "Resend As..." submenu with preset list and click handlers.

---

## 4. Verification & Quality Gate Checklist

| Verification Gate | Command / Test | Success Standard |
| :--- | :--- | :--- |
| **Rust Unit Tests** | `cd src-tauri && cargo test suffix_templates` | All CRUD, pattern interpolation, and validation tests pass. |
| **Rust Formatting** | `cd src-tauri && cargo fmt -- --check` | Clean zero-diff code style. |
| **Rust Clippy** | `cd src-tauri && cargo clippy --lib --all-features` | Zero warnings, zero errors. |
| **Frontend Build** | `npm run build` | Clean Vite/TypeScript compilation. |
| **CLI Whitelist Verification** | `cargo run --bin agm -- prompt template list` | Renders template list without attempting IDE dispatch. |
| **Write Verification Test** | Write template via CLI and UI | Read-back verification passes; row written to `repo_prompts.db`. |

---

## 5. Rollback & Safety Invariants

- **Zero Data Loss**: `init_tables` uses `CREATE TABLE IF NOT EXISTS` and does not overwrite existing data.
- **Graceful Fallback**: If SQLite database access fails or runs in a browser web context, UI seamlessly degrades to `localStorage` without throwing uncaught exceptions.
- **Strict Attribution**: Changelogs and commits must credit strictly `@aukgit` (`(Thanks to @aukgit)`).
