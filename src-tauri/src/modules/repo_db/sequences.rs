//! Repo DB: sequences

use super::schema::init_tables;
use super::text_utils::extract_clean_user_prompt;
use chrono::Utc;
use rusqlite::params;
use rusqlite::Connection;

/// Batch item for conversation sequence operations
#[derive(Debug, Clone)]
pub struct ConversationSequenceBatchItem<'a> {
    pub conversation_id: &'a str,
    pub project_key: &'a str,
    pub title: &'a str,
    pub instance_id: &'a str,
}

/// Batch update sequence entries within an atomic SQLite transaction to eliminate multiple fsync stalls.
pub(crate) fn batch_ensure_sequences(
    conn: &mut Connection,
    items: &[ConversationSequenceBatchItem<'_>],
) -> Result<Vec<i64>, rusqlite::Error> {
    let tx = conn.transaction()?;
    let now = Utc::now().timestamp();
    let mut results = Vec::with_capacity(items.len());

    for item in items {
        if let Ok(existing) = tx.query_row(
            "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1",
            params![item.conversation_id],
            |row| row.get::<_, i64>(0),
        ) {
            // Justification: metadata refresh on an existing sequence row in batch; sequence id is retained
            crate::error::record_ignored(
                tx.execute(
                    "UPDATE agm_conversation_sequences SET project_key = ?2, title = ?3, instance_id = ?4, updated_at = ?5 WHERE conversation_id = ?1",
                    params![item.conversation_id, item.project_key, item.title, item.instance_id, now],
                ),
                "refresh conversation sequence metadata in batch",
            );
            results.push(existing);
            continue;
        }

        let next_seq: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(seq_id), 0) + 1 FROM agm_conversation_sequences",
                [],
                |row| row.get(0),
            )
            .unwrap_or(1);

        // Justification: INSERT OR IGNORE is race-safe in transaction batch; follow-up SELECT re-reads the row
        crate::error::record_ignored(
            tx.execute(
                "INSERT OR IGNORE INTO agm_conversation_sequences (conversation_id, seq_id, project_key, title, instance_id, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![item.conversation_id, next_seq, item.project_key, item.title, item.instance_id, now],
            ),
            "insert conversation sequence row in batch",
        );

        let final_seq = tx
            .query_row(
                "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1",
                params![item.conversation_id],
                |row| row.get(0),
            )
            .unwrap_or(next_seq);
        results.push(final_seq);
    }

    tx.commit()?;
    Ok(results)
}

pub(crate) fn ensure_project_sequence_in_conn(
    conn: &Connection,
    project_key: &str,
    project_id: &str,
    repo_name: &str,
    repo_path: &str,
    instance_id: &str,
) -> i64 {
    let now = Utc::now().timestamp();
    if let Ok(tx) = conn.unchecked_transaction() {
        let seq = if let Ok(existing) = tx.query_row(
            "SELECT seq_id FROM agm_project_sequences WHERE project_key = ?1",
            params![project_key],
            |row| row.get::<_, i64>(0),
        ) {
            // Justification: metadata refresh on an existing sequence row; the sequence id is returned regardless
            crate::error::record_ignored(
                tx.execute(
                    "UPDATE agm_project_sequences SET project_id = ?2, repo_name = ?3, repo_path = ?4, instance_id = ?5, updated_at = ?6 WHERE project_key = ?1",
                    params![project_key, project_id, repo_name, repo_path, instance_id, now],
                ),
                "refresh project sequence metadata",
            );
            existing
        } else {
            let next_seq: i64 = tx
                .query_row(
                    "SELECT COALESCE(MAX(seq_id), 0) + 1 FROM agm_project_sequences",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(1);

            // Justification: INSERT OR IGNORE is race-safe; the follow-up SELECT re-reads the row
            crate::error::record_ignored(
                tx.execute(
                    "INSERT OR IGNORE INTO agm_project_sequences (project_key, seq_id, project_id, repo_name, repo_path, instance_id, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![project_key, next_seq, project_id, repo_name, repo_path, instance_id, now],
                ),
                "insert project sequence row",
            );

            tx.query_row(
                "SELECT seq_id FROM agm_project_sequences WHERE project_key = ?1",
                params![project_key],
                |row| row.get(0),
            )
            .unwrap_or(next_seq)
        };
        // Justification: commit project sequence transaction; sequence id is returned regardless
        crate::error::record_ignored(tx.commit(), "commit project sequence transaction");
        seq
    } else {
        if let Ok(existing) = conn.query_row(
            "SELECT seq_id FROM agm_project_sequences WHERE project_key = ?1",
            params![project_key],
            |row| row.get::<_, i64>(0),
        ) {
            // Justification: metadata refresh on an existing sequence row; the sequence id is returned regardless
            crate::error::record_ignored(
                conn.execute(
                    "UPDATE agm_project_sequences SET project_id = ?2, repo_name = ?3, repo_path = ?4, instance_id = ?5, updated_at = ?6 WHERE project_key = ?1",
                    params![project_key, project_id, repo_name, repo_path, instance_id, now],
                ),
                "refresh project sequence metadata",
            );
            return existing;
        }

        let next_seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(seq_id), 0) + 1 FROM agm_project_sequences",
                [],
                |row| row.get(0),
            )
            .unwrap_or(1);

        // Justification: INSERT OR IGNORE is race-safe; the follow-up SELECT re-reads the row
        crate::error::record_ignored(
            conn.execute(
                "INSERT OR IGNORE INTO agm_project_sequences (project_key, seq_id, project_id, repo_name, repo_path, instance_id, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![project_key, next_seq, project_id, repo_name, repo_path, instance_id, now],
            ),
            "insert project sequence row",
        );

        conn.query_row(
            "SELECT seq_id FROM agm_project_sequences WHERE project_key = ?1",
            params![project_key],
            |row| row.get(0),
        )
        .unwrap_or(next_seq)
    }
}

pub(crate) fn ensure_conversation_sequence_in_conn(
    conn: &Connection,
    conversation_id: &str,
    project_key: &str,
    title: &str,
    instance_id: &str,
) -> i64 {
    let now = Utc::now().timestamp();
    if let Ok(tx) = conn.unchecked_transaction() {
        let seq = if let Ok(existing) = tx.query_row(
            "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1",
            params![conversation_id],
            |row| row.get::<_, i64>(0),
        ) {
            // Justification: metadata refresh on an existing sequence row; the sequence id is returned regardless
            crate::error::record_ignored(
                tx.execute(
                    "UPDATE agm_conversation_sequences SET project_key = ?2, title = ?3, instance_id = ?4, updated_at = ?5 WHERE conversation_id = ?1",
                    params![conversation_id, project_key, title, instance_id, now],
                ),
                "refresh conversation sequence metadata",
            );
            existing
        } else {
            let next_seq: i64 = tx
                .query_row(
                    "SELECT COALESCE(MAX(seq_id), 0) + 1 FROM agm_conversation_sequences",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(1);

            // Justification: INSERT OR IGNORE is race-safe; the follow-up SELECT re-reads the row
            crate::error::record_ignored(
                tx.execute(
                    "INSERT OR IGNORE INTO agm_conversation_sequences (conversation_id, seq_id, project_key, title, instance_id, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![conversation_id, next_seq, project_key, title, instance_id, now],
                ),
                "insert conversation sequence row",
            );

            tx.query_row(
                "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1",
                params![conversation_id],
                |row| row.get(0),
            )
            .unwrap_or(next_seq)
        };
        // Justification: commit conversation sequence transaction; sequence id is returned regardless
        crate::error::record_ignored(tx.commit(), "commit conversation sequence transaction");
        seq
    } else {
        if let Ok(existing) = conn.query_row(
            "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1",
            params![conversation_id],
            |row| row.get::<_, i64>(0),
        ) {
            // Justification: metadata refresh on an existing sequence row; the sequence id is returned regardless
            crate::error::record_ignored(
                conn.execute(
                    "UPDATE agm_conversation_sequences SET project_key = ?2, title = ?3, instance_id = ?4, updated_at = ?5 WHERE conversation_id = ?1",
                    params![conversation_id, project_key, title, instance_id, now],
                ),
                "refresh conversation sequence metadata",
            );
            return existing;
        }

        let next_seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(seq_id), 0) + 1 FROM agm_conversation_sequences",
                [],
                |row| row.get(0),
            )
            .unwrap_or(1);

        // Justification: INSERT OR IGNORE is race-safe; the follow-up SELECT re-reads the row
        crate::error::record_ignored(
            conn.execute(
                "INSERT OR IGNORE INTO agm_conversation_sequences (conversation_id, seq_id, project_key, title, instance_id, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![conversation_id, next_seq, project_key, title, instance_id, now],
            ),
            "insert conversation sequence row",
        );

        conn.query_row(
            "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1",
            params![conversation_id],
            |row| row.get(0),
        )
        .unwrap_or(next_seq)
    }
}

/// Query the sequence code for a conversation (formatted as "C{:03}") from `agm_conversation_sequences`.
pub fn get_sequence_code_for_conversation(conv_id: &str) -> Option<String> {
    let clean = conv_id.trim();
    if clean.is_empty() {
        return None;
    }
    if let Ok(conn) = super::schema::connect_db() {
        get_sequence_code_in_conn(&conn, clean)
    } else {
        None
    }
}

/// Query the sequence code for a conversation using an existing connection.
pub fn get_sequence_code_in_conn(conn: &Connection, conv_id: &str) -> Option<String> {
    let clean = conv_id.trim();
    if clean.is_empty() {
        return None;
    }
    conn.query_row(
        "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1",
        params![clean],
        |row| row.get::<_, i64>(0),
    )
    .ok()
    .map(|s| format!("C{:03}", s))
}

/// Extract clean user prompt up to `max_words` (e.g. 200 words) and return `(preview_text, total_word_count)`.
/// Preserves line breaks (`\n`) and vertical paragraph gaps rather than flattening all text into a single line.
pub fn extract_prompt_words_preview(raw_text: &str, max_words: usize) -> (String, usize) {
    let cleaned = extract_clean_user_prompt(raw_text);
    let mut total_words = 0;
    let mut selected_lines: Vec<String> = Vec::new();
    let mut words_collected = 0;
    let limit = max_words.max(1);
    let mut reached_limit = false;

    for line in cleaned.lines() {
        let t = line.trim();
        // Skip standalone 40-char git SHA lines
        if t.len() == 40 && t.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        let line_words: Vec<&str> = t.split_whitespace().collect();
        let count = line_words.len();
        total_words += count;

        if !reached_limit {
            if count == 0 {
                // Preserve blank line between paragraphs if previous line wasn't blank
                if !selected_lines.is_empty()
                    && !selected_lines.last().map(|s| s.is_empty()).unwrap_or(false)
                {
                    selected_lines.push(String::new());
                }
            } else if words_collected + count <= limit {
                selected_lines.push(line.to_string());
                words_collected += count;
            } else {
                let remaining = limit.saturating_sub(words_collected);
                if remaining > 0 {
                    let truncated_line = line_words[..remaining].join(" ");
                    selected_lines.push(format!("{} ...", truncated_line));
                } else if !selected_lines.is_empty() {
                    let last_idx = selected_lines.len() - 1;
                    selected_lines[last_idx] = format!("{} ...", selected_lines[last_idx]);
                }
                words_collected = limit;
                reached_limit = true;
            }
        }
    }

    if total_words == 0 {
        return (String::new(), 0);
    }

    let preview = selected_lines.join("\n").trim().to_string();
    (preview, total_words)
}

/// Extract trailing snippet (last 10-15 words) of user prompt
pub fn extract_prompt_tail_snippet(raw_text: &str, tail_word_count: usize) -> String {
    let cleaned = extract_clean_user_prompt(raw_text);
    let mut meaningful_tokens: Vec<&str> = Vec::new();
    for line in cleaned.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.len() == 40 && t.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        for word in t.split_whitespace() {
            meaningful_tokens.push(word);
        }
    }
    let total = meaningful_tokens.len();
    if total == 0 {
        return String::new();
    }
    let count = tail_word_count.clamp(1, 15);
    if total <= count {
        meaningful_tokens.join(" ")
    } else {
        let tail_slice = &meaningful_tokens[total - count..];
        tail_slice.join(" ")
    }
}

/// Helper to update conversation_summaries.db and conversations_fts.
fn update_summaries_in_db(
    db_path: &std::path::Path,
    clean_title: &str,
    now_iso: &str,
    clean_cid: &str,
) -> usize {
    let Ok(conn) = Connection::open(db_path) else {
        return 0;
    };
    // Justification: busy_timeout on summaries database is a nicety; continues with default if failed
    crate::error::record_ignored(
        conn.pragma_update(None, "busy_timeout", 5000),
        "set busy_timeout on conversation_summaries.db",
    );
    // Justification: journal_mode WAL on summaries database is a performance optimization; continues with default if failed
    crate::error::record_ignored(
        conn.pragma_update(None, "journal_mode", "WAL"),
        "set journal_mode WAL on conversation_summaries.db",
    );

    let mut count = conn
        .execute(
            "UPDATE conversation_summaries SET title = ?1, last_modified_time = ?2 WHERE conversation_id = ?3",
            params![clean_title, now_iso, clean_cid],
        )
        .or_else(|_| {
            conn.execute(
                "UPDATE conversation_summaries SET title = ?1 WHERE conversation_id = ?2",
                params![clean_title, clean_cid],
            )
        })
        .unwrap_or(0);

    if count == 0 {
        let pattern = format!("%{}%", clean_cid);
        count = conn
            .execute(
                "UPDATE conversation_summaries SET title = ?1, last_modified_time = ?2 WHERE conversation_id LIKE ?3",
                params![clean_title, now_iso, &pattern],
            )
            .or_else(|_| {
                conn.execute(
                    "UPDATE conversation_summaries SET title = ?1 WHERE conversation_id LIKE ?2",
                    params![clean_title, &pattern],
                )
            })
            .unwrap_or(0);
    }

    // Also update conversations_fts virtual table if present
    let fts_res = conn.execute(
        "UPDATE conversations_fts SET title = ?1 WHERE conversation_id = ?2",
        params![clean_title, clean_cid],
    );
    match fts_res {
        Ok(fts_count) => {
            if fts_count == 0 {
                let pattern = format!("%{}%", clean_cid);
                // Justification: updating conversations_fts via LIKE fallback is best-effort full-text sync
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE conversations_fts SET title = ?1 WHERE conversation_id LIKE ?2",
                        params![clean_title, &pattern],
                    ),
                    "update title in conversations_fts like fallback",
                );
            }
        }
        Err(e) => {
            // Justification: conversations_fts is an optional virtual table not present in all SQLite versions or databases
            crate::error::record_ignored(
                Err::<(), _>(e),
                "update title in conversations_fts (table may not exist)",
            );
        }
    }

    count
}

/// Update conversation title across both instance conversation_summaries.db and AGM repo_prompts.db.
/// Invalidates the prompt tree cache upon success.
pub fn update_conversation_title(
    instance_id: Option<&str>,
    conversation_id: &str,
    new_title: &str,
) -> Result<(), crate::error::AppError> {
    let clean_title = new_title.trim();
    if clean_title.is_empty() {
        return Err(crate::error::AppError::Config(
            "Conversation title cannot be empty".to_string(),
        ));
    }
    let clean_cid = conversation_id.trim();
    if clean_cid.is_empty() {
        return Err(crate::error::AppError::Config(
            "Conversation ID cannot be empty".to_string(),
        ));
    }

    let resolved_inst = instance_id
        .filter(|s| !s.is_empty())
        .map(|s| crate::modules::instance::resolve_instance_id(s).unwrap_or_else(|_| s.to_string()))
        .unwrap_or_else(|| "default".to_string());

    let mut updated_summaries_count = 0;

    // 1. Locate instance's conversation_summaries.db and update
    let candidate_dirs = super::gemini_dirs::gemini_dirs_for_instance(&resolved_inst);
    let now_iso = Utc::now().to_rfc3339();

    for dir in &candidate_dirs {
        let db_candidates = [
            dir.join("conversation_summaries.db"),
            dir.join("antigravity").join("conversation_summaries.db"),
        ];

        for db_path in db_candidates {
            if db_path.exists() {
                updated_summaries_count +=
                    update_summaries_in_db(&db_path, clean_title, &now_iso, clean_cid);
            }
        }
    }

    // Fallback: If 0 rows updated in the target instance dirs, scan across all candidate dirs
    if updated_summaries_count == 0 {
        for (_tag, dir) in super::gemini_dirs::gemini_dirs_tagged(None) {
            let db_candidates = [
                dir.join("conversation_summaries.db"),
                dir.join("antigravity").join("conversation_summaries.db"),
            ];
            for db_path in db_candidates {
                if db_path.exists() {
                    let count = update_summaries_in_db(&db_path, clean_title, &now_iso, clean_cid);
                    if count > 0 {
                        updated_summaries_count += count;
                        break;
                    }
                }
            }
            if updated_summaries_count > 0 {
                break;
            }
        }
    }

    // 2. Open repo_prompts.db, execute: UPDATE agm_conversation_sequences SET title = ?1, updated_at = ?2 WHERE conversation_id = ?3
    let now_ts = Utc::now().timestamp();
    if let Ok(repo_conn) = super::schema::connect_db() {
        // Justification: busy_timeout on repo_prompts.db is a nicety; continues with default if failed
        crate::error::record_ignored(
            repo_conn.pragma_update(None, "busy_timeout", 5000),
            "set busy_timeout on repo_prompts.db",
        );
        // Justification: updating conversation sequence title is best-effort metadata synchronization
        crate::error::record_ignored(
            repo_conn.execute(
                "UPDATE agm_conversation_sequences SET title = ?1, updated_at = ?2 WHERE conversation_id = ?3",
                params![clean_title, now_ts, clean_cid],
            ),
            "update title in agm_conversation_sequences",
        );
        // Also update active_prompts if untitled
        // Justification: updating active prompt preview is best-effort synchronization
        crate::error::record_ignored(
            repo_conn.execute(
                "UPDATE active_prompts SET preview_raw = ?1, preview_clean = ?1 WHERE session_id = ?2 AND (preview_raw IS NULL OR preview_raw = '' OR preview_raw LIKE 'Untitled%')",
                params![clean_title, clean_cid],
            ),
            "update active_prompts preview if untitled",
        );
    }

    // 3. Invalidate prompt tree cache
    super::tree::invalidate_prompt_tree_cache(Some(&resolved_inst));
    if resolved_inst != "all" {
        super::tree::invalidate_prompt_tree_cache(Some("all"));
    }

    Ok(())
}

#[inline]
pub fn rename_conversation_dual_store(
    instance_id: &str,
    conversation_id: &str,
    new_title: &str,
) -> Result<(), crate::error::AppError> {
    update_conversation_title(Some(instance_id), conversation_id, new_title)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::repo_db::init_tables;
    use rusqlite::Connection;

    #[test]
    fn test_agm_sequences_and_200_word_preview() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        let p1 = ensure_project_sequence_in_conn(
            &conn,
            "d:/work/antigravity-manager",
            "agm-1",
            "Antigravity-Manager",
            "d:/work/Antigravity-Manager",
            "default",
        );
        let p1_repeat = ensure_project_sequence_in_conn(
            &conn,
            "d:/work/antigravity-manager",
            "agm-1",
            "Antigravity-Manager",
            "d:/work/Antigravity-Manager",
            "default",
        );
        let p2 = ensure_project_sequence_in_conn(
            &conn,
            "d:/work/gitmap",
            "gm-1",
            "gitmap",
            "d:/work/gitmap",
            "default",
        );
        assert_eq!(p1, 1);
        assert_eq!(p1_repeat, 1);
        assert_eq!(p2, 2);

        let c1 = ensure_conversation_sequence_in_conn(
            &conn,
            "d58c5517-d8ad-437e-ab7c-e506b0322383",
            "d:/work/antigravity-manager",
            "Telegram & AGM Tree View",
            "default",
        );
        assert_eq!(c1, 1);

        let two_hundred_fifty_words = (1..=250)
            .map(|i| format!("word{}", i))
            .collect::<Vec<_>>()
            .join(" ");
        let (preview, count) = extract_prompt_words_preview(&two_hundred_fifty_words, 200);
        assert_eq!(count, 250);
        assert!(preview.ends_with("word200 ..."));
    }

    #[test]
    fn test_extract_prompt_tail_snippet() {
        let text = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen";
        let snippet = extract_prompt_tail_snippet(text, 12);
        assert_eq!(
            snippet,
            "five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen"
        );

        let short_text = "hello world";
        let short_snippet = extract_prompt_tail_snippet(short_text, 12);
        assert_eq!(short_snippet, "hello world");

        let empty_snippet = extract_prompt_tail_snippet("   ", 12);
        assert_eq!(empty_snippet, "");
    }

    #[test]
    fn test_batch_ensure_sequences() {
        let mut conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        let items = vec![
            ConversationSequenceBatchItem {
                conversation_id: "batch-c1",
                project_key: "d:/work/test",
                title: "Task 1",
                instance_id: "default",
            },
            ConversationSequenceBatchItem {
                conversation_id: "batch-c2",
                project_key: "d:/work/test",
                title: "Task 2",
                instance_id: "default",
            },
        ];

        let results = batch_ensure_sequences(&mut conn, &items).expect("batch sequence update");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], 1);
        assert_eq!(results[1], 2);

        // Repeat batch to verify idempotency and metadata refresh
        let repeat = batch_ensure_sequences(&mut conn, &items).expect("repeat batch update");
        assert_eq!(repeat, vec![1, 2]);
    }

    #[test]
    fn test_get_sequence_code_in_conn() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        assert!(get_sequence_code_in_conn(&conn, "nonexistent").is_none());
        assert!(get_sequence_code_in_conn(&conn, "").is_none());

        let seq_num = ensure_conversation_sequence_in_conn(
            &conn,
            "conv-seq-test-01",
            "d:/work/test",
            "Test Sequence",
            "default",
        );
        assert_eq!(seq_num, 1);

        let code = get_sequence_code_in_conn(&conn, "conv-seq-test-01");
        assert_eq!(code, Some("C001".to_string()));
    }
}
