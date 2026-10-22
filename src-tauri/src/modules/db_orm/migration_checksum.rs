//! Database Migration Checksum Verifier.
//! Verifies SHA-256 checksums of applied schema migrations against canonical DDL models to detect schema drift.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Report on migration checksum verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationChecksumReport {
    pub version_number: i32,
    pub is_valid: bool,
    pub stored_checksum: Option<String>,
    pub expected_checksum: String,
}

/// Computes SHA-256 hex digest of a DDL string.
pub fn compute_ddl_checksum(ddl: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ddl.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Validates stored checksum in `SchemaVersion` against expected DDL checksum.
pub fn verify_migration_checksum(
    conn: &Connection,
    version_number: i32,
    canonical_ddl: &str,
) -> Result<MigrationChecksumReport> {
    let expected_checksum = compute_ddl_checksum(canonical_ddl);

    let stored_checksum: Option<String> = conn
        .query_row(
            "SELECT Checksum FROM SchemaVersion WHERE VersionNumber = ?1 LIMIT 1",
            [version_number],
            |r| r.get(0),
        )
        .ok();

    let is_valid = match &stored_checksum {
        Some(stored) => stored.eq_ignore_ascii_case(&expected_checksum),
        None => false,
    };

    Ok(MigrationChecksumReport {
        version_number,
        is_valid,
        stored_checksum,
        expected_checksum,
    })
}
