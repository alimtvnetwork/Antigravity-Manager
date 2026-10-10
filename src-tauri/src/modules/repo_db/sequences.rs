//! Repo DB: sequences

use super::schema::init_tables;
use super::text_utils::extract_clean_user_prompt;
use chrono::Utc;
use rusqlite::params;
use rusqlite::Connection;

pub(crate) fn ensure_project_sequence_in_conn(
    conn: &Connection,
    project_key: &str,
    project_id: &str,
    repo_name: &str,
    repo_path: &str,
    instance_id: &str,
) -> i64 {
    let now = Utc::now().timestamp();
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

pub(crate) fn ensure_conversation_sequence_in_conn(
    conn: &Connection,
    conversation_id: &str,
    project_key: &str,
    title: &str,
    instance_id: &str,
) -> i64 {
    let now = Utc::now().timestamp();
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
}
