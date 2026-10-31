//! Multi-Stage Launcher Job Breakaway Spawner (Imp-628).
//!
//! Enforces process spawning safety and multi-stage fallback per AGENTS.md:
//! - Never unconditionally specify `CREATE_BREAKAWAY_FROM_JOB` (`0x01000000`)
//!   without a fallback ladder (`spawn_with_job_breakaway_fallback`).
//! - If running inside a restricted Windows Job Object, `CreateProcessW` fails
//!   with `os error 5: Access is denied`.
//! - Never pass `.cmd` or `.bat` batch files directly as the executable path in
//!   `std::process::Command::new(...)`.
//! - Multi-instance launchers must implement multi-stage fallback:
//!   target binary -> detected base binary -> Windows Shell dispatch (`cmd.exe /c start`).

use serde::{Deserialize, Serialize};

/// Win32 process creation flag: CREATE_BREAKAWAY_FROM_JOB (0x01000000).
pub const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x01000000;

/// Win32 ERROR_ACCESS_DENIED code (5).
pub const ERROR_ACCESS_DENIED: i32 = 5;

/// Classification of the target executable requested for spawning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnerTargetKind {
    /// Directly executable binary (e.g. .exe or ELF binary).
    DirectBinary,
    /// Windows batch script (.bat or .cmd).
    WindowsBatchScript,
    /// Windows shell command or dispatch wrapper (e.g. cmd.exe).
    ShellFallback,
}

impl SpawnerTargetKind {
    /// Returns true if the target is a direct binary.
    pub fn is_direct_binary(&self) -> bool {
        matches!(self, Self::DirectBinary)
    }

    /// Returns true if the target is a Windows batch script.
    pub fn is_windows_batch_script(&self) -> bool {
        matches!(self, Self::WindowsBatchScript)
    }

    /// Returns true if the target is a shell dispatch fallback.
    pub fn is_shell_fallback(&self) -> bool {
        matches!(self, Self::ShellFallback)
    }
}

/// Fallback ladder stage reached for the spawn operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnFallbackLadderStage {
    /// Initial attempt specifying `CREATE_BREAKAWAY_FROM_JOB`.
    BreakawayAttempt,
    /// Step-down fallback without breakaway flag (e.g. after access denied).
    NoBreakawayFallback,
    /// Fallback via Windows shell dispatch (`cmd.exe /c start`).
    ShellDispatchFallback,
}

impl SpawnFallbackLadderStage {
    /// Returns true if this stage attempts job breakaway.
    pub fn is_breakaway(&self) -> bool {
        matches!(self, Self::BreakawayAttempt)
    }

    /// Returns true if this stage bypasses job breakaway.
    pub fn is_no_breakaway(&self) -> bool {
        matches!(self, Self::NoBreakawayFallback)
    }

    /// Returns true if this stage uses shell dispatch fallback.
    pub fn is_shell_dispatch(&self) -> bool {
        matches!(self, Self::ShellDispatchFallback)
    }
}

/// Fully resolved plan for spawning an instance process safely.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessSpawnDecision {
    /// Executable path to invoke.
    pub executable_path: String,
    /// Argument vector for the command.
    pub args: Vec<String>,
    /// Windows process creation flags (e.g. `CREATE_BREAKAWAY_FROM_JOB`).
    pub creation_flags: u32,
    /// Ladder stage corresponding to this decision.
    pub stage: SpawnFallbackLadderStage,
    /// Whether execution was wrapped via Windows shell dispatch.
    pub is_shell_wrapped: bool,
    /// Target kind of the resolved executable.
    pub target_kind: SpawnerTargetKind,
}

impl ProcessSpawnDecision {
    /// Returns true if the creation flags include `CREATE_BREAKAWAY_FROM_JOB`.
    pub fn has_breakaway_flag(&self) -> bool {
        self.creation_flags & CREATE_BREAKAWAY_FROM_JOB != 0
    }

    /// Returns true if execution was wrapped in a shell dispatch.
    pub fn has_shell_wrapper(&self) -> bool {
        self.is_shell_wrapped
    }

    /// Converts this decision into a standard process Command.
    pub fn to_command(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new(&self.executable_path);
        cmd.args(&self.args);
        #[cfg(target_os = "windows")]
        {
            if self.creation_flags != 0 {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(self.creation_flags);
            }
        }
        cmd
    }
}

/// Evaluates whether an executable path is a Windows batch file (.bat or .cmd).
pub fn is_windows_batch_file(path: &str) -> bool {
    let lower = path.trim().to_ascii_lowercase();
    lower.ends_with(".bat") || lower.ends_with(".cmd")
}

/// Classifies the target executable kind based on file extension and path.
pub fn classify_target_kind(path: &str) -> SpawnerTargetKind {
    let lower = path.trim().to_ascii_lowercase();
    if is_windows_batch_file(&lower) {
        SpawnerTargetKind::WindowsBatchScript
    } else if lower == "cmd.exe" || lower == "cmd" || lower.ends_with("\\cmd.exe") {
        SpawnerTargetKind::ShellFallback
    } else {
        SpawnerTargetKind::DirectBinary
    }
}

/// Helper to determine stage and creation flags based on breakaway permissions and errors.
fn determine_flags_and_stage(
    allow_job_breakaway: bool,
    is_access_denied: bool,
) -> (SpawnFallbackLadderStage, u32) {
    if is_access_denied || !allow_job_breakaway {
        (SpawnFallbackLadderStage::NoBreakawayFallback, 0)
    } else {
        (
            SpawnFallbackLadderStage::BreakawayAttempt,
            CREATE_BREAKAWAY_FROM_JOB,
        )
    }
}

/// Resolves the optimal, safe process spawn plan across the multi-stage ladder.
pub fn resolve_spawn_plan(
    executable: &str,
    args: &[String],
    detected_base_binary: Option<&str>,
    allow_job_breakaway: bool,
    previous_os_error: Option<i32>,
) -> ProcessSpawnDecision {
    let is_batch = is_windows_batch_file(executable);
    let is_access_denied = previous_os_error == Some(ERROR_ACCESS_DENIED);

    if is_batch {
        if let Some(base_binary) = detected_base_binary {
            let (stage, creation_flags) =
                determine_flags_and_stage(allow_job_breakaway, is_access_denied);
            return ProcessSpawnDecision {
                executable_path: base_binary.to_string(),
                args: args.to_vec(),
                creation_flags,
                stage,
                is_shell_wrapped: false,
                target_kind: SpawnerTargetKind::DirectBinary,
            };
        }

        let mut shell_args = Vec::with_capacity(args.len() + 3);
        shell_args.push("/c".to_string());
        shell_args.push("start".to_string());
        shell_args.push(executable.to_string());
        shell_args.extend(args.iter().cloned());

        return ProcessSpawnDecision {
            executable_path: "cmd.exe".to_string(),
            args: shell_args,
            creation_flags: 0,
            stage: SpawnFallbackLadderStage::ShellDispatchFallback,
            is_shell_wrapped: true,
            target_kind: SpawnerTargetKind::ShellFallback,
        };
    }

    let (stage, creation_flags) = determine_flags_and_stage(allow_job_breakaway, is_access_denied);
    ProcessSpawnDecision {
        executable_path: executable.to_string(),
        args: args.to_vec(),
        creation_flags,
        stage,
        is_shell_wrapped: false,
        target_kind: SpawnerTargetKind::DirectBinary,
    }
}
