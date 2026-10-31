//! Database file header checksum and magic byte guard.
//! Inspects raw SQLite 3 file headers before opening connection pools to prevent
//! panics and storage corruption from truncated or partially written databases.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub const SQLITE_MAGIC_HEADER: &[u8; 16] = b"SQLite format 3\0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderValidationResult {
    pub is_valid_sqlite: bool,
    pub page_size: u32,
    pub file_change_counter: u32,
    pub error_detail: Option<String>,
}

pub struct HeaderChecksumGuard;

impl HeaderChecksumGuard {
    /// Inspects the first 100 bytes of a SQLite database file on disk.
    pub fn validate_file(db_path: &Path) -> Result<HeaderValidationResult, AppError> {
        if !db_path.is_file() {
            return Err(AppError::Config(format!(
                "Database file does not exist: {}",
                db_path.display()
            )));
        }

        let mut file = File::open(db_path).map_err(AppError::Io)?;

        let mut header = [0u8; 100];
        let bytes_read = file.read(&mut header).map_err(AppError::Io)?;

        if bytes_read < 100 {
            return Ok(HeaderValidationResult {
                is_valid_sqlite: false,
                page_size: 0,
                file_change_counter: 0,
                error_detail: Some(format!(
                    "File is too short (read {} bytes, expected >= 100)",
                    bytes_read
                )),
            });
        }

        // Validate 16-byte magic header
        if &header[0..16] != SQLITE_MAGIC_HEADER {
            return Ok(HeaderValidationResult {
                is_valid_sqlite: false,
                page_size: 0,
                file_change_counter: 0,
                error_detail: Some("Invalid SQLite magic header bytes".to_string()),
            });
        }

        // Page size is a 2-byte big-endian integer at offset 16
        let raw_page_size = u16::from_be_bytes([header[16], header[17]]);
        let page_size: u32 = if raw_page_size == 1 {
            65536
        } else {
            raw_page_size as u32
        };

        // File change counter is at offset 24 (4 bytes big-endian)
        let change_counter = u32::from_be_bytes([header[24], header[25], header[26], header[27]]);

        Ok(HeaderValidationResult {
            is_valid_sqlite: true,
            page_size,
            file_change_counter: change_counter,
            error_detail: None,
        })
    }
}
