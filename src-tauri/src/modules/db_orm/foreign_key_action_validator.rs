//! Foreign Key Action Rule Validator
//!
//! Validates `ON DELETE` and `ON UPDATE` referential actions across foreign key
//! constraints in split SQLite schemas, ensuring relations default to safe cascades
//! or restrictions to prevent orphan records or accidental cascade deletions.

use rusqlite::Connection;
use tracing::debug;

/// Referential action rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FkActionRule {
    NoAction,
    Restrict,
    SetNull,
    SetDefault,
    Cascade,
}

impl FkActionRule {
    pub fn from_pragma_str(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
            "CASCADE" => FkActionRule::Cascade,
            "RESTRICT" => FkActionRule::Restrict,
            "SET NULL" => FkActionRule::SetNull,
            "SET DEFAULT" => FkActionRule::SetDefault,
            _ => FkActionRule::NoAction,
        }
    }
}

/// Foreign key constraint details extracted from PRAGMA foreign_key_list.
#[derive(Debug, Clone)]
pub struct TableForeignKey {
    pub id: i32,
    pub seq: i32,
    pub table: String,
    pub from: String,
    pub to: String,
    pub on_update: FkActionRule,
    pub on_delete: FkActionRule,
}

/// Inspects foreign keys on a table and returns action rules.
pub fn inspect_table_foreign_keys(
    conn: &Connection,
    table_name: &str,
) -> Result<Vec<TableForeignKey>, rusqlite::Error> {
    let sql = format!("PRAGMA foreign_key_list('{}');", table_name);
    let mut stmt = conn.prepare(&sql)?;

    let mut rows = stmt.query([])?;
    let mut fks = Vec::new();

    while let Some(row) = rows.next()? {
        let id: i32 = row.get(0)?;
        let seq: i32 = row.get(1)?;
        let foreign_table: String = row.get(2)?;
        let from_col: String = row.get(3)?;
        let to_col: String = row.get(4)?;
        let on_update_str: String = row.get(5)?;
        let on_delete_str: String = row.get(6)?;

        fks.push(TableForeignKey {
            id,
            seq,
            table: foreign_table,
            from: from_col,
            to: to_col,
            on_update: FkActionRule::from_pragma_str(&on_update_str),
            on_delete: FkActionRule::from_pragma_str(&on_delete_str),
        });
    }

    debug!(
        table = table_name,
        fk_count = fks.len(),
        "Inspected table foreign key action rules"
    );

    Ok(fks)
}
