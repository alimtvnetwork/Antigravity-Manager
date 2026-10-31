#[cfg(test)]
mod tests {
    use super::super::index_coverage_verifier::{
        inspect_table_indices, is_query_covered, TableIndexInfo,
    };
    use rusqlite::Connection;

    #[test]
    fn test_is_query_covered_empty_filters() {
        let indices = vec![TableIndexInfo {
            name: "idx_account".to_string(),
            is_unique: false,
            columns: vec!["account_id".to_string()],
        }];
        assert!(is_query_covered(&indices, &[]));
    }

    #[test]
    fn test_is_query_covered_leading_column_match() {
        let indices = vec![
            TableIndexInfo {
                name: "idx_account_created".to_string(),
                is_unique: false,
                columns: vec!["account_id".to_string(), "created_at".to_string()],
            },
            TableIndexInfo {
                name: "idx_status".to_string(),
                is_unique: false,
                columns: vec!["status".to_string()],
            },
        ];

        // Leading column matches
        assert!(is_query_covered(&indices, &["account_id"]));
        assert!(is_query_covered(&indices, &["ACCOUNT_ID", "created_at"]));
        assert!(is_query_covered(&indices, &["status"]));

        // Non-leading column does not match
        assert!(!is_query_covered(&indices, &["created_at"]));
        assert!(!is_query_covered(&indices, &["unindexed_col"]));
    }

    #[test]
    fn test_inspect_table_indices_sqlite() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "CREATE TABLE sessions (
                id INTEGER PRIMARY KEY,
                token TEXT NOT NULL,
                user_id TEXT NOT NULL,
                expires_at INTEGER NOT NULL
            );
            CREATE UNIQUE INDEX idx_sessions_token ON sessions(token);
            CREATE INDEX idx_sessions_user ON sessions(user_id, expires_at);",
        )
        .expect("create table and indices");

        let indices = inspect_table_indices(&conn, "sessions").expect("inspect indices");
        assert_eq!(indices.len(), 2);

        let token_idx = indices.iter().find(|i| i.name == "idx_sessions_token");
        assert!(token_idx.is_some());
        let token_info = token_idx.unwrap();
        assert!(token_info.is_unique);
        assert_eq!(token_info.columns, vec!["token"]);

        let user_idx = indices.iter().find(|i| i.name == "idx_sessions_user");
        assert!(user_idx.is_some());
        let user_info = user_idx.unwrap();
        assert!(!user_info.is_unique);
        assert_eq!(user_info.columns, vec!["user_id", "expires_at"]);

        // Verify query coverage with inspected indices
        assert!(is_query_covered(&indices, &["token"]));
        assert!(is_query_covered(&indices, &["user_id", "expires_at"]));
        assert!(!is_query_covered(&indices, &["expires_at"]));
    }
}
