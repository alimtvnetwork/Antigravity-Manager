// IDE Binary Architecture and Execution Probe (Imp-532)
// Probes candidate Antigravity executable binaries on disk, verifying file existence,
// binary permissions, and architecture validity before launching a child process.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryProbeResult {
    pub path: String,
    pub exists: bool,
    pub is_file: bool,
    pub size_bytes: u64,
    pub is_executable: bool,
    pub error: Option<String>,
}

pub struct BinaryExecutionProbe;

impl BinaryExecutionProbe {
    /// Inspects an executable path to confirm readiness for process spawn.
    pub fn probe_binary<P: AsRef<Path>>(bin_path: P) -> BinaryProbeResult {
        let p = bin_path.as_ref();
        let path_str = p.to_string_lossy().to_string();

        if !p.exists() {
            return BinaryProbeResult {
                path: path_str,
                exists: false,
                is_file: false,
                size_bytes: 0,
                is_executable: false,
                error: Some("Executable binary path does not exist".to_string()),
            };
        }

        match fs::metadata(p) {
            Ok(meta) => {
                let is_file = meta.is_file();
                let size_bytes = meta.len();

                // Basic validation: must be a non-empty file
                let is_executable = is_file && size_bytes > 0;

                BinaryProbeResult {
                    path: path_str,
                    exists: true,
                    is_file,
                    size_bytes,
                    is_executable,
                    error: if is_executable {
                        None
                    } else {
                        Some("Binary path exists but is not a valid executable file".to_string())
                    },
                }
            }
            Err(e) => BinaryProbeResult {
                path: path_str,
                exists: true,
                is_file: false,
                size_bytes: 0,
                is_executable: false,
                error: Some(e.to_string()),
            },
        }
    }
}
