# 4-Part Root Cause Analysis: Media Vault Missing `instance_id` Column on Per-Instance Tree Cache Wipe

- **Issue Code**: `E9001`
- **Scope**: `src-tauri/src/modules/repo_db/agy.rs`, `src-tauri/src/modules/repo_db/schema.rs`
- **Related Spec**: `02-spec/21-app/telegram-backup-and-fleet-management-enhancements/`
- **Author**: Maintainer Team (@aukgit)
- **Status**: Resolved

---

## Part 1: Problem Symptom & Classification

### 1.1 Observable Failure
When an operator or frontend view triggers `reset_prompt_tree_cache` or `invalidate_prompt_tree_cache` scoped to an individual instance ID (e.g. from `/instances` route), the operation immediately aborts with diagnostic error `E9001`:

```text
E9001 — Failed to clear media_vault for instance: no such column: instance_id in DELETE FROM media_vault WHERE instance_id = ?1 OR instance_id = ?2 at offset 30
    Route / Page: /instances
    Trigger: (reset_prompt_tree_cache)
```

This error occurred repeatedly (captured 6 times in the Diagnostics History Report) whenever the instance prompt tree cache was reset or refreshed for a specific instance.

### 1.2 Impact
1. **Frontend Tree Stalling**: The prompt tree view on the frontend received a rejected promise during cache reset, leaving the user with an error notification.
2. **Aborted Cache Regeneration**: Because SQLite returned a statement error within `wipe_and_rescan_repo_db`, downstream rescanning and sequence rebuilding for the target instance were disrupted.

---

## Part 2: Root Cause Analysis (Mechanisms & Trace)

### 2.1 Schema Definition of `media_vault`
In `src-tauri/src/modules/repo_db/schema.rs:171-183`, the `media_vault` table is defined as:

```sql
CREATE TABLE IF NOT EXISTS media_vault (
    id TEXT PRIMARY KEY,
    sha256 TEXT NOT NULL UNIQUE,
    local_path TEXT,
    public_url TEXT,
    telegram_file_id TEXT,
    mime_type TEXT,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_media_vault_sha256 ON media_vault(sha256);
```

The table is designed as a **global, content-addressed media cache** (keyed by `sha256`) used by `src-tauri/src/modules/telegram_inbound/media.rs` to deduplicate photo downloads, local storage, and CDN mirror links across all sessions. It possesses **no** `instance_id` column.

### 2.2 Execution Path in `wipe_and_rescan_repo_db`
In `src-tauri/src/modules/repo_db/agy.rs:1140-1192`:
1. When `is_all == true` (global reset), line 1149 executes:
   ```rust
   conn.execute("DELETE FROM media_vault", [])
   ```
   This succeeds because `DELETE FROM media_vault` has no column filter.
2. When `is_all == false` (per-instance reset), lines 1187-1191 incorrectly attempted:
   ```rust
   conn.execute(
       "DELETE FROM media_vault WHERE instance_id = ?1 OR instance_id = ?2",
       rusqlite::params![raw_id, &norm_id],
   )
   .map_err(|e| format!("Failed to clear media_vault for instance: {e}"))?;
   ```
   SQLite parsed the SQL query, found no `instance_id` column in `media_vault`, and immediately returned `no such column: instance_id at offset 30`.

### 2.3 Conceptual Mismatch
Deleting media vault entries on a single-instance cache reset is both syntactically invalid and semantically wrong:
- **Syntactic invalidity**: `media_vault` lacks an `instance_id` column.
- **Semantic violation**: `media_vault` caches global media blobs by SHA-256 hash. If an instance prompt references an image downloaded via Telegram or mirrored to a CDN, other instances or global views referencing the same hash would have their cached media link unnecessarily destroyed or corrupted if instance-level purging were attempted.

---

## Part 3: Surgical Resolution & Invariants

### 3.1 Code Changes
In `src-tauri/src/modules/repo_db/agy.rs`:
- Removed the erroneous `DELETE FROM media_vault WHERE instance_id = ?1 OR instance_id = ?2` execution block from the `!is_all` branch of `wipe_and_rescan_repo_db`.
- Added clear architectural documentation clarifying that `media_vault` is an immutable content-addressed cache keyed by SHA-256 without an `instance_id` column, and is intentionally preserved during per-instance resets.

### 3.2 System Invariants Enforced
- **Invariant 1: Content-Addressed Global Stores**: Tables keyed by cryptographic hashes (`sha256`) that lack tenancy columns must never be targeted by tenancy-scoped queries (`WHERE instance_id = ?`).
- **Invariant 2: Non-Destructive Instance Invalidation**: Resetting an instance's prompt tree cache must only invalidate instance-scoped metadata (`prompt_tree_cache`, `active_prompts`, `running_projects`, `agm_conversation_sequences`, `agm_project_sequences`).
- **Invariant 3: Zero Silent Failures**: Any database statement errors must continue to be explicitly surfaced through `AppError` and logged via diagnostics.

---

## Part 4: Verification & Prevention Gate

### 4.1 Verification Plan
1. **Compilation & Clippy**:
   - `cargo check --manifest-path src-tauri/Cargo.toml --lib`
   - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
2. **Targeted Unit Tests**:
   - `cargo test --manifest-path src-tauri/Cargo.toml --lib modules::repo_db`
3. **Database Integrity Verification**:
   - Verify that calling `wipe_and_rescan_repo_db(Some("default"))` executes cleanly without throwing `E9001` or any SQLite column errors.
   - Verify that global wipe (`wipe_and_rescan_repo_db(None)` or `is_all == true`) continues to work as intended.
