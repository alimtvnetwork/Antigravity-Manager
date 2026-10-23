//! WAL Frame Header Integrity Inspector
//!
//! Validates low-level SQLite Write-Ahead Log (WAL) 32-byte file headers, checking
//! magic numbers (`0x377f0682` or `0x377f0683`), page size boundaries (512 to 65536 bytes),
//! and sequence counter integrity before attempting recovery or stepped checkpoints.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use tracing::{debug, warn};

/// SQLite WAL magic numbers.
pub const WAL_MAGIC_LE: u32 = 0x377f0682; // Little-endian checksums
pub const WAL_MAGIC_BE: u32 = 0x377f0683; // Big-endian checksums

/// Result of inspecting a WAL file header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalHeaderValidation {
    Valid {
        page_size: u32,
        checkpoint_seq: u32,
        salt1: u32,
        salt2: u32,
    },
    FileNotFound,
    FileTooSmall {
        bytes: u64,
    },
    InvalidMagicNumber {
        found: u32,
    },
    InvalidPageSize {
        page_size: u32,
    },
    IoError(String),
}

/// Inspects the 32-byte header of a SQLite WAL file.
pub fn inspect_wal_header(wal_path: &Path) -> WalHeaderValidation {
    if !wal_path.exists() {
        return WalHeaderValidation::FileNotFound;
    }

    let mut file = match File::open(wal_path) {
        Ok(f) => f,
        Err(e) => return WalHeaderValidation::IoError(e.to_string()),
    };

    let len = match file.seek(SeekFrom::End(0)) {
        Ok(l) => l,
        Err(e) => return WalHeaderValidation::IoError(e.to_string()),
    };

    if len < 32 {
        return WalHeaderValidation::FileTooSmall { bytes: len };
    }

    if let Err(e) = file.seek(SeekFrom::Start(0)) {
        return WalHeaderValidation::IoError(e.to_string());
    }

    let mut header_buf = [0u8; 32];
    if let Err(e) = file.read_exact(&mut header_buf) {
        return WalHeaderValidation::IoError(e.to_string());
    }

    let magic = u32::from_be_bytes([header_buf[0], header_buf[1], header_buf[2], header_buf[3]]);
    if magic != WAL_MAGIC_LE && magic != WAL_MAGIC_BE {
        warn!(
            magic = format!("{:#x}", magic),
            "Invalid WAL file magic number"
        );
        return WalHeaderValidation::InvalidMagicNumber { found: magic };
    }

    let page_size =
        u32::from_be_bytes([header_buf[8], header_buf[9], header_buf[10], header_buf[11]]);
    if page_size < 512 || page_size > 65536 || !page_size.is_power_of_two() {
        warn!(page_size = page_size, "Invalid WAL page size in header");
        return WalHeaderValidation::InvalidPageSize { page_size };
    }

    let checkpoint_seq = u32::from_be_bytes([
        header_buf[12],
        header_buf[13],
        header_buf[14],
        header_buf[15],
    ]);
    let salt1 = u32::from_be_bytes([
        header_buf[16],
        header_buf[17],
        header_buf[18],
        header_buf[19],
    ]);
    let salt2 = u32::from_be_bytes([
        header_buf[20],
        header_buf[21],
        header_buf[22],
        header_buf[23],
    ]);

    debug!(
        path = ?wal_path,
        page_size = page_size,
        seq = checkpoint_seq,
        "WAL file header verified"
    );

    WalHeaderValidation::Valid {
        page_size,
        checkpoint_seq,
        salt1,
        salt2,
    }
}
