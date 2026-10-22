// Multi-Stage Process Spawn Ladder (Imp-502)
// Provides a 3-stage fallback ladder for spawning sandboxed IDE processes:
// Stage 1: Explicit target executable path
// Stage 2: System detected base binary path
// Stage 3: Windows shell dispatch fallback (`cmd.exe /c start`)

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnStage {
    DirectTarget,
    DetectedBase,
    ShellDispatch,
}

pub struct ProcessSpawnLadder;

impl ProcessSpawnLadder {
    /// Attempts to spawn the process through the 3-stage ladder.
    pub fn spawn_with_fallback(
        target_bin: &Path,
        fallback_base_bin: Option<&Path>,
        args: &[String],
    ) -> Result<(Child, SpawnStage), std::io::Error> {
        // Stage 1: Try direct target executable
        if target_bin.exists() {
            match Command::new(target_bin).args(args).spawn() {
                Ok(child) => {
                    info!(stage = "DirectTarget", bin = %target_bin.display(), "Spawned process successfully");
                    return Ok((child, SpawnStage::DirectTarget));
                }
                Err(e) => {
                    warn!(
                        stage = "DirectTarget",
                        bin = %target_bin.display(),
                        error = %e,
                        "Direct target spawn failed, falling back to Stage 2"
                    );
                }
            }
        }

        // Stage 2: Try detected base binary if provided and exists
        if let Some(base_bin) = fallback_base_bin {
            if base_bin.exists() {
                match Command::new(base_bin).args(args).spawn() {
                    Ok(child) => {
                        info!(stage = "DetectedBase", bin = %base_bin.display(), "Spawned process via detected base binary");
                        return Ok((child, SpawnStage::DetectedBase));
                    }
                    Err(e) => {
                        warn!(
                            stage = "DetectedBase",
                            bin = %base_bin.display(),
                            error = %e,
                            "Base binary spawn failed, falling back to Stage 3"
                        );
                    }
                }
            }
        }

        // Stage 3: Windows Shell dispatch fallback
        #[cfg(target_os = "windows")]
        {
            let mut shell_cmd = Command::new("cmd.exe");
            shell_cmd.arg("/c").arg("start").arg("").arg(target_bin);
            for arg in args {
                shell_cmd.arg(arg);
            }
            match shell_cmd.spawn() {
                Ok(child) => {
                    info!(stage = "ShellDispatch", "Spawned process via Windows Shell dispatch fallback");
                    return Ok((child, SpawnStage::ShellDispatch));
                }
                Err(e) => {
                    warn!(stage = "ShellDispatch", error = %e, "Shell dispatch fallback failed");
                    return Err(e);
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Process could not be spawned via any stage in the ladder",
            ))
        }
    }
}
