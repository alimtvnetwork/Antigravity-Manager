#[cfg(test)]
mod tests {
    use super::super::column_default_validator::ColumnDefaultValidator;
    use rusqlite::Connection;

    #[test]
    fn test_audit_table_detects_missing_defaults_on_not_null_columns() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL,
                email TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                score INTEGER NOT NULL
            );",
        )
        .expect("create table");

        let audit = ColumnDefaultValidator::audit_table(&conn, "users").expect("audit table");
        assert_eq!(audit.table_name, "users");
        assert_eq!(audit.total_columns, 5);

        // id is PK, email is nullable, status has DEFAULT 'active'
        // username and score are NOT NULL without DEFAULT
        assert_eq!(audit.columns_missing_defaults.len(), 2);
        assert!(audit.columns_missing_defaults.contains(&"username".to_string()));
        assert!(audit.columns_missing_defaults.contains(&"score".to_string()));
    }

    #[test]
    fn test_audit_table_all_columns_have_defaults_or_nullable() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "CREATE TABLE configs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL DEFAULT '',
                val TEXT DEFAULT NULL,
                enabled INTEGER NOT NULL DEFAULT 1
            );",
        )
        .expect("create table");

        let audit = ColumnDefaultValidator::audit_table(&conn, "configs").expect("audit table");
        assert_eq!(audit.total_columns, 4);
        assert!(audit.columns_missing_defaults.is_empty());
    }

    #[test]
    fn test_audit_table_nonexistent() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        let audit = ColumnDefaultValidator::audit_table(&conn, "nonexistent").expect("audit table");
        assert_eq!(audit.total_columns, 0);
        assert!(audit.columns.is_empty());
        assert!(audit.columns_missing_defaults.is_empty());
    }
}
