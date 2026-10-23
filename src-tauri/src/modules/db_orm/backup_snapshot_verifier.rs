//! Database Backup Snapshot Verifier.
//! Validates the integrity of SQLite database backup snapshot files,
//! confirming standard 16-byte magic headers and page sizes before committing restores.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Expected 16-byte magic header for SQLite 3 database files.
pub const SQLITE3_MAGIC_HEADER: &[u8; 16] = b"SQLite format 3\0";

/// Verification outcome for a database backup file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupVerificationOutcome {
    Valid { page_size: u16, file_size: u64 },
    InvalidMagicHeader { found_bytes: Vec<u8> },
    FileTooSmall { size: u64 },
    IoError(String),
}

/// Verifies that a target file is a structurally valid SQLite 3 database.
pub fn verify_sqlite_backup_file(path: &Path) -> BackupVerificationOutcome {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return BackupVerificationOutcome::IoError(e.to_string()),
    };

    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(e) => return BackupVerificationOutcome::IoError(e.to_string()),
    };

    let file_size = metadata.len();
    if file_size < 100 {
        return BackupVerificationOutcome::FileTooSmall { size: file_size };
    }

    let mut header = [0u8; 16];
    if let Err(e) = file.read_exact(&mut header) {
        return BackupVerificationOutcome::IoError(e.to_string());
    }

    if &header != SQLITE3_MAGIC_HEADER {
        return BackupVerificationOutcome::InvalidMagicHeader {
            found_bytes: header.to_vec(),
        };
    }

    // Read page size at offset 16-17 (Big Endian)
    if let Err(e) = file.seek(SeekFrom::Start(16)) {
        return BackupVerificationOutcome::IoError(e.to_string());
    }

    let mut page_size_bytes = [0u8; 2];
    if let Err(e) = file.read_exact(&mut page_size_bytes) {
        return BackupVerificationOutcome::IoError(e.to_string());
    }

    let raw_page_size = u16::from_be_bytes(page_size_bytes);
    let page_size = if raw_page_size == 1 { 65536 } else { raw_page_size as u32 };

    BackupVerificationOutcome::Valid {
        page_size: page_size as u16,
        file_size,
    }
}
