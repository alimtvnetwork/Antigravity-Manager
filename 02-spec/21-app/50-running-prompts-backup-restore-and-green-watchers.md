# Specification 50: Running Prompts Split SQLite Backup/Restore, FPUG/SUG Green Watchers, Broadcast Email & Telegram CLI

> **Module ID:** `02-spec/21-app/50-running-prompts-backup-restore-and-green-watchers.md`  
> **Status:** Normative / Active  
> **Target Release:** v4.77.0  
> **Target Subsystems:** `src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/backup_prompts_db.rs`, `src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/email_vault_db.rs`

---

## 1. User Request (Verbatim)

```text
<prefix cli> = agm

<prefix cli> backup-running-prompts [-file/-f "filepath or file name.db"] # if no 
<prefix cli> backup-running-prompts ls

<prefix cli> running-prompts backup ls
<prefix cli> running-prompts restore --keep/k 
<prefix cli> restore-running-prompts --keep/k

<prefix cli> running-prompts backup ls --json
<prefix cli> running-prompts restore --json
<prefix cli> running-prompts backup --json

<prefix cli> running-prompts help
<prefix cli> running-prompts backup help
<prefix cli> backup-running-prompts help
<prefix cli> restore-running-prompts help

<prefix cli> running-prompts ls
<prefix cli> running-prompts ls --json --full
<prefix cli> running-prompts ls --json [--wordcount (wc) N] # N = 100

<prefix cli> auto-swtich thresehold 25 # sets the auto switch threshold to 25%, by default keep it to 25% please

<prefix cli> broadcast-email add/ls/edit/rm/help/send-to-all/send/send help/send-help(sh)
<prefix cli> broadcast-email test

<prefix cli> running-prompts ls --limit/l Y --json [--wordcount (wc) N] --full # N = 100, Y = 8
<prefix cli> running-prompts export [-file "file abs path or relative path or nothing , format is file.db, file.json"] [--wc 200] # default file <cli>-running-prompts.db
<prefix cli> running-prompts import [-file "file abs path or relative path or nothing , format is file.db, file.json"] [--wc 200] # default file <cli>-running-prompts.db

<prefix cli> running-projects help/ls --json [-file/-f -file "file abs path or relative path or nothing , format is file.db, default file.json"] 
<prefix cli> running-projects help/ls --json [-file/-f -file "file abs path or relative path or nothing , format is file.db, default file.json"] --ssh
<prefix cli> running-projects help/ls [-file/-f -file "file abs path or relative path or nothing , format is file.db, default file.json"] --ssh

<prefix cli> finish-prompts-until-green(fpug) <project path, id, alias, seq>, <project path, id, alias, seq>,  [-t 5m]
<prefix cli> finish-prompts-until-green(fpug) running-projects  [-t 5m] # enquee all the running projects which has running prompts or queue prompts will ensure the running is done, clear???

<prefix cli> shutdown-until-green(sug) ls/help/run/add-projects/rm/agy-running-projects [-t 5m]

<prefix cli> telegram ls/set/help/set help/ping/cmds/commands



Okay. So let's get into some of the prompts and commands that I want to integrate. So one of them is backup running prompts. So that is basically going to back up all the running and queued prompts. Okay, so help text also say the same thing. And the backup running prompts will create a split DB close to the CLI where the data folder is. Inside the data folder, it will create a backup prompts folder. Inside this, it would have SQL.db. That would be the backup prompts db. So anytime you say backup prompts, it's going to back up the prompts with date, time, project name, project path, project ID, conversation ID, everything. Sequence ID, everything. So you could do a LS to see how many backup prompts are there. If no file path is given, if you provide a file path, then the backup or current backup will be created on that file path or on that location, or if you just give a name.db, it will just create that backup on that instance backup on that location. Okay? If you provide that. If you don't provide, then that is the default location as I have described. Okay. Now, we could do the similar way, like LS. LS will tell us what the backup prompts does. Okay. The same thing can also be accessed using some control. LS will do the same thing as well. We could do restore as well. So we can do restore from this. We could do restore from... Stop sharing all of these. So we could add a flag with restore, like... Okay. So if we do not use anything, the usual process of the backup is it's going to keep the backup for, let's say, one day. So if any backup, not one day, sorry. So after the restore, it will keep inside the database that it has been restored. We mark all these IDs with a sub-table, right? So sub-table relationship needs to be there. It would mark like these are restored. So after one restore, it would wait for one day or more. Okay? So if next time we do the restore or backup LS or anything we wanted to run, then it will do the cleanup, old cleanup. So in old cleanup, if the data is already restored and more than one day, so this is a setting you need to also keep in settings that can be changed, it will be automatically removed, and it would mention that old data has been removed, a single line, not anything details. Okay. Yeah. So the LS one, we can also do it using JSON. That means we could do JSON to get the output. We could do running prompts backup, running prompts restore using JSON. So the JSON would just show us what's going on. Okay? Backup status and what the backup actually happened. And also all of these should have the help command. So we should have help, running prompts help. Okay, so every one of these cases would be very delicate, and it's going to give us the command information, help text, and everything. Okay, backup. If we do the backup LS, it would show the databases, how many data are there, also the storage information, where the database is, what the size of the database, how user can clean it. So there would be a clean command like for the backup, so user can force to clean it around the clean command. Usually, the clean command would just follow the procedure as I mentioned, but user can do a force clean to clean everything. Okay? That is something you need to mention in the help text, also in the UI, and also in the terminal, and also the docs in the prompt. Do you understand? Yeah. If you do all these things, I think then you will have a good thing going on. Do you understand? Any question and confusion?

Make a minor bump and release
```

---

## 2. Technical Architecture & Data Contracts

### 2.1 Auto-Switch 25% Default Threshold
- **Default Value:** 25.0% (previously 15.0%).
- **CLI Commands:**
  - `agm auto-switch threshold [N]` (with aliases `auto-switch thresehold`, `auto-swtich threshold`, `auto-swtich thresehold`).
  - Without arguments: prints current threshold percentage.
  - With argument `N`: parses numeric value, clamps to `0.0..100.0`, persists into settings database, and prints confirmation.

### 2.2 Split SQLite Backup Engine (`backup-prompts.db`)
- **Default Database Location:**
  `~/.gemini/antigravity/data/backup-prompts/backup-prompts.db` (or relative `data/backup-prompts/backup-prompts.db`).
- **Table Schema 1: `prompt_backups`**:
  ```sql
  CREATE TABLE IF NOT EXISTS prompt_backups (
      id TEXT PRIMARY KEY,
      backup_batch_id TEXT NOT NULL,
      prompt_id TEXT NOT NULL,
      project_name TEXT NOT NULL,
      project_path TEXT NOT NULL,
      project_id TEXT NOT NULL,
      conversation_id TEXT NOT NULL,
      conversation_name TEXT,
      sequence_id INTEGER NOT NULL,
      prompt_text TEXT NOT NULL,
      has_images BOOLEAN NOT NULL DEFAULT 0,
      images_payload TEXT,
      status TEXT NOT NULL DEFAULT 'queued',
      created_at INTEGER NOT NULL,
      is_restored BOOLEAN NOT NULL DEFAULT 0,
      restored_at INTEGER
  );
  CREATE INDEX IF NOT EXISTS idx_prompt_backups_batch ON prompt_backups(backup_batch_id);
  CREATE INDEX IF NOT EXISTS idx_prompt_backups_restored ON prompt_backups(is_restored, restored_at);
  ```
- **Table Schema 2: `backup_batches`**:
  ```sql
  CREATE TABLE IF NOT EXISTS backup_batches (
      id TEXT PRIMARY KEY,
      created_at INTEGER NOT NULL,
      prompts_count INTEGER NOT NULL,
      file_path TEXT NOT NULL,
      retention_days INTEGER NOT NULL DEFAULT 1,
      is_fully_restored BOOLEAN NOT NULL DEFAULT 0
  );
  ```
- **Retention & Auto-Cleanup Lifecycle:**
  - When prompts are restored, they are marked `is_restored = 1`, `restored_at = <unix_timestamp>`.
  - On every invocation of `ls`, `backup`, or `restore`, evaluate records where `is_restored == 1` and `restored_at <= (now - retention_seconds)` (default 1 day = 86400s).
  - Delete expired restored records and output single concise notice:
    `[INFO] Cleaned up N expired restored prompt backup record(s).`
  - Support force clean: `agm backup-running-prompts clean --force` / `agm running-prompts backup clean --force`.

### 2.3 Running Prompts Commands Matrix
- `agm running-prompts ls [--limit/-l Y] [--wordcount/--wc N] [--full] [--json]`
- `agm running-prompts backup [-f/--file <path>] [--json]`
- `agm running-prompts backup ls [--json]`
- `agm running-prompts restore [--keep/-k] [--json]`
- `agm running-prompts export [-f/--file <path>] [--wc N]` (defaults to `agm-running-prompts.db`)
- `agm running-prompts import [-f/--file <path>] [--wc N]` (defaults to `agm-running-prompts.db`, re-runs imported prompts)
- `agm running-prompts help` / `agm backup-running-prompts help` / `agm restore-running-prompts help`

### 2.4 Running Projects & Multi-Node SSH
- `agm running-projects ls/help [--json] [-f/--file <path>] [--ssh]`
- Aggregates active projects across local instance sandboxes and remote SSH nodes (when `--ssh` is specified).

### 2.5 Finish Prompts Until Green (FPUG)
- `agm finish-prompts-until-green <targets...> [-t 5m]`
- `agm finish-prompts-until-green running-projects [-t 5m]`
- Short alias: `agm fpug`.
- Polling watcher with configurable interval `-t` (default 5 minutes = 300s). Checks queue and execution status until all targeted projects have 0 pending/running prompts and exit successfully.

### 2.6 Shutdown Until Green (SUG)
- `agm shutdown-until-green <subcommand> [-t 5m]`
- Short alias: `agm sug`.
- Subcommands: `ls`, `help`, `run`, `add-projects <targets...>`, `rm <targets...>`, `agy-running-projects`.
- When green conditions are fulfilled across all configured projects:
  - Windows: `shutdown /s /t 60`
  - Linux: `systemctl poweroff` (or fallback `shutdown -h now`)
  - macOS: `osascript -e 'tell app "System Events" to shut down'`

### 2.7 Broadcast Email Subsystem
- `agm broadcast-email add/ls/edit/rm/help/send-to-all/send/send help/send-help(sh)`
- `agm broadcast-email test`

### 2.8 Telegram CLI Remote
- `agm telegram ls/set/help/set help/ping/cmds/commands`
