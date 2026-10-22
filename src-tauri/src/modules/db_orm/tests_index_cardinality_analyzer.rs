#[cfg(test)]
mod tests {
    use super::super::index_cardinality_analyzer::IndexCardinalityAnalyzer;
    use rusqlite::Connection;

    #[test]
    fn test_evaluate_column_cardinality_high_selectivity() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT NOT NULL);
            INSERT INTO users (email) VALUES ('a@test.com'), ('b@test.com'), ('c@test.com');",
        )
        .expect("setup table");

        let metric = IndexCardinalityAnalyzer::evaluate_column_cardinality(
            &conn,
            "users",
            "email",
            "idx_users_email",
        )
        .expect("evaluate cardinality");

        assert_eq!(metric.table_name, "users");
        assert_eq!(metric.index_name, "idx_users_email");
        assert_eq!(metric.total_row_count, 3);
        assert_eq!(metric.unique_entry_count, 3);
        assert!((metric.selectivity_ratio - 1.0).abs() < f64::EPSILON);
        assert!(!metric.is_low_selectivity);
    }

    #[test]
    fn test_evaluate_column_cardinality_low_selectivity_flagged() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch("CREATE TABLE logs (id INTEGER PRIMARY KEY, level TEXT NOT NULL);")
            .expect("create table");

        // Insert 150 rows with only 2 distinct levels ('INFO', 'WARN')
        let mut stmt = conn.prepare("INSERT INTO logs (level) VALUES (?);").expect("prepare");
        for i in 0..150 {
            let lvl = if i % 2 == 0 { "INFO" } else { "WARN" };
            stmt.execute([lvl]).expect("insert row");
        }

        let metric = IndexCardinalityAnalyzer::evaluate_column_cardinality(
            &conn,
            "logs",
            "level",
            "idx_logs_level",
        )
        .expect("evaluate cardinality");

        assert_eq!(metric.total_row_count, 150);
        assert_eq!(metric.unique_entry_count, 2);
        // Selectivity is 2 / 150 = ~0.0133 (< 0.05) and rows > 100 -> flagged as low selectivity
        assert!(metric.selectivity_ratio < 0.05);
        assert!(metric.is_low_selectivity);
    }

    #[test]
    fn test_evaluate_column_cardinality_empty_table() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch("CREATE TABLE items (id INTEGER PRIMARY KEY, sku TEXT);")
            .expect("create table");

        let metric = IndexCardinalityAnalyzer::evaluate_column_cardinality(
            &conn,
            "items",
            "sku",
            "idx_items_sku",
        )
        .expect("evaluate cardinality");

        assert_eq!(metric.total_row_count, 0);
        assert_eq!(metric.unique_entry_count, 0);
        assert_eq!(metric.selectivity_ratio, 1.0);
        assert!(!metric.is_low_selectivity);
    }
}
