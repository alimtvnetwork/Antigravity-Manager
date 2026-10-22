//! Split Database Health Probe Aggregator.
//! Performs non-intrusive ping probes across all split databases,
//! gathering responsiveness, file size, and quick integrity checks into a unified health status.

use rusqlite::Connection;
use std::path::Path;
use std::time::Instant;

/// Individual database health report.
#[derive(Debug, Clone)]
pub struct SingleDbHealth {
    pub db_name: String,
    pub is_responsive: bool,
    pub latency_ms: f64,
    pub integrity_ok: bool,
    pub file_size_bytes: u64,
    pub error_message: Option<String>,
}

/// Global health summary across all split databases.
#[derive(Debug, Clone)]
pub struct SplitDbHealthSummary {
    pub all_healthy: bool,
    pub databases: Vec<SingleDbHealth>,
}

/// Probes a single database connection and path.
pub fn probe_database(
    conn: &Connection,
    db_name: &str,
    file_path: &Path,
) -> SingleDbHealth {
    let start = Instant::now();
    let file_size = file_path.metadata().map(|m| m.len()).unwrap_or(0);

    // Run quick check
    let probe_res: Result<String, rusqlite::Error> = conn.query_row("PRAGMA quick_check(1);", [], |row| row.get(0));
    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    match probe_res {
        Ok(status) => {
            let integrity_ok = status.eq_ignore_ascii_case("ok");
            SingleDbHealth {
                db_name: db_name.to_string(),
                is_responsive: true,
                latency_ms,
                integrity_ok,
                file_size_bytes: file_size,
                error_message: if integrity_ok { None } else { Some(status) },
            }
        }
        Err(e) => SingleDbHealth {
            db_name: db_name.to_string(),
            is_responsive: false,
            latency_ms,
            integrity_ok: false,
            file_size_bytes: file_size,
            error_message: Some(e.to_string()),
        },
    }
}
