---
plan: auto-switch-and-ui-functionality-issues
subtask: "04"
title: Max Backup Prompts Retention Cap and Settings UI Control
domain: backend-db-and-settings
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/auto-switch-and-ui-functionality-issues/01-architecture-and-quota-autoswitch-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/models/config.rs
  - src-tauri/src/modules/backup_prompts_db/backup.rs
  - src-tauri/src/modules/config/defaults.rs
  - src/components/settings/AutoSwitcher/TaskContinuitySettings.tsx
  - src/types/settings.ts
status: pending
---

# Subtask 04: Max Backup Prompts Retention Cap and Settings UI Control

## 1. Context & Motivation

When Antigravity Manager snapshots running and queued prompts before switching accounts or triggering Fast-Forward, it records them into the split SQLite database (`backup_prompts.db`). Over time, unpruned backup tables grow indefinitely, consuming storage and increasing restore scan latency.

The user explicitly mandated:
> *"When we keep the information for the prompts, make sure that for each one of the prompts, we should not have more than 100 prompts as a backup, but it can be changed from the settings. There should be a setting that would allow us to change that."*

Furthermore, per project maintenance rules:
- **FIFO Restore & Queue Invariant**: All prompt queries, retention caps, and pruning across SQLite databases (`active_prompts`, `prompt_backups`) must enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. When pruning, oldest records must be removed first so that the newest $N$ records are preserved.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src-tauri/src/models/config.rs` | `AutoProfileSwitcherConfig`, `default_max_backup_prompts` | Add `pub max_backup_prompts: u32` (default 100). |
| `src-tauri/src/modules/backup_prompts_db/backup.rs` | `enforce_max_backup_prompts_cap`, `backup_active_running_prompts_for_instance` | Prune oldest prompt backups beyond the configured cap using strict FIFO deletion. |
| `src/components/settings/AutoSwitcher/TaskContinuitySettings.tsx` | `TaskContinuitySettings` | Add a number input / select dropdown for `max_backup_prompts` (50, 100, 200, 500) under the Task Continuity section. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Add `max_backup_prompts` to Config Model
In `src-tauri/src/models/config.rs`:
1. Define default helper:
   ```rust
   fn default_max_backup_prompts() -> u32 {
       100
   }
   ```
2. Add field to `AutoProfileSwitcherConfig`:
   ```rust
   #[serde(default = "default_max_backup_prompts")]
   pub max_backup_prompts: u32,
   ```
3. Initialize in `Default for AutoProfileSwitcherConfig`:
   ```rust
   max_backup_prompts: 100,
   ```

### 3.2 Step 2: Implement FIFO Pruning in `src-tauri/src/modules/backup_prompts_db/backup.rs`
1. Add `enforce_max_backup_prompts_cap(conn: &Connection, instance_id: &str, max_cap: u32) -> Result<usize, String>`:
   ```rust
   pub fn enforce_max_backup_prompts_cap(
       conn: &Connection,
       instance_id: &str,
       max_cap: u32,
   ) -> Result<usize, String> {
       if max_cap == 0 {
           return Ok(0);
       }
       // Count existing records for this instance
       let total_count: i64 = conn
           .query_row(
               "SELECT COUNT(*) FROM prompt_backups WHERE instance_id = ?1 OR instance_id IS NULL OR instance_id = ''",
               params![instance_id],
               |r| r.get(0),
           )
           .unwrap_or(0);

       if total_count <= max_cap as i64 {
           return Ok(0);
       }

       let excess = (total_count - max_cap as i64) as usize;
       // Prune oldest records strictly adhering to FIFO ordering
       let pruned = conn.execute(
           "DELETE FROM prompt_backups WHERE id IN (
               SELECT id FROM prompt_backups 
               WHERE instance_id = ?1 OR instance_id IS NULL OR instance_id = ''
               ORDER BY created_at ASC, id ASC 
               LIMIT ?2
           )",
           params![instance_id, excess as i64],
       ).map_err(|e| format!("Failed to prune excess backup prompts: {}", e))?;

       tracing::info!(
           "[BackupPromptsDB] Pruned {} excess prompt backups (cap: {}, instance: '{}')",
           pruned, max_cap, instance_id
       );
       Ok(pruned)
   }
   ```
2. Call `enforce_max_backup_prompts_cap` after batch insertions in `backup_active_running_prompts_for_instance`. Read `max_cap` from `crate::modules::config::get_auto_switcher_config().max_backup_prompts`, falling back to 100.

### 3.3 Step 3: Add Retention Cap Control in `TaskContinuitySettings.tsx`
In `src/components/settings/AutoSwitcher/TaskContinuitySettings.tsx`:
1. Under the Task Continuity & Watchdog section, add an interactive configuration control:
   ```tsx
   {/* Maximum Backup Prompts Retention Cap */}
   <div className="flex items-center justify-between gap-3 p-3 bg-slate-100/90 dark:bg-slate-900/90 rounded-xl border border-slate-200 dark:border-slate-700/80">
       <div>
           <span className="text-xs font-semibold text-slate-800 dark:text-slate-200 block">
               Prompt Backup Retention Limit
           </span>
           <p className="text-[11px] text-slate-500 dark:text-slate-400">
               Maximum number of recent prompts retained in SQLite backup database per instance.
           </p>
       </div>
       <div className="relative">
           <select
               value={currentConfig.max_backup_prompts || 100}
               onChange={(e) => onChange({
                   ...currentConfig,
                   max_backup_prompts: Number(e.target.value),
               })}
               className="appearance-none px-3 py-1 pr-7 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-700 rounded-lg text-xs font-semibold text-slate-800 dark:text-slate-200 shadow-2xs focus:outline-none focus:ring-2 focus:ring-emerald-500/30 cursor-pointer"
           >
               <option value={50}>50 Prompts</option>
               <option value={100}>100 Prompts (Default)</option>
               <option value={200}>200 Prompts</option>
               <option value={500}>500 Prompts</option>
           </select>
           <ChevronDown className="w-3.5 h-3.5 text-slate-400 pointer-events-none absolute right-2 top-2" />
       </div>
   </div>
   ```

---

## 4. Verification & Quality Gates

- [ ] Check config serialization: run `cargo test --lib test_auto_switcher_config` or verify JSON round-trip of `max_backup_prompts`.
- [ ] Insert 120 synthetic prompt backup records in a test; verify that `enforce_max_backup_prompts_cap` with cap 100 leaves exactly 100 records.
- [ ] Verify that deleted records are the oldest ones (`ORDER BY created_at ASC, id ASC`).
- [ ] Verify that GUI Settings retains changes to `max_backup_prompts` across app reloads.
