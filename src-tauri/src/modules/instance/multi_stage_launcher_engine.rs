//! Multi-Stage Launcher Engine & Batch Execution Guard (`Imp-650`).
//!
//! Enforces process spawning safety per AGENTS.md:
//! "Multi-instance launchers must implement multi-stage fallback: target binary ->
//! detected base binary -> Windows Shell dispatch (`cmd.exe /c start`). Never pass
//! `.cmd` or `.bat` batch files directly as the executable path in `std::process::Command::new(...)`.
//! If running inside a restricted Windows Job Object, `CreateProcessW` fails with `os error 5: Access is denied`."

use serde::{Deserialize, Serialize};

/// Win32 process creation flag: CREATE_BREAKAWAY_FROM_JOB (0x01000000).
pub const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x01000000;

/// Win32 ERROR_ACCESS_DENIED code (5).
pub const ERROR_ACCESS_DENIED: i32 = 5;

/// Execution stage of the multi-stage launcher fallback ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionStage {
    TargetBinary,
    BaseBinaryFallback,
    ShellDispatchFallback,
}

impl ExecutionStage {
    pub fn is_target_binary(&self) -> bool {
        matches!(self, Self::TargetBinary)
    }

    pub fn is_base_binary_fallback(&self) -> bool {
        matches!(self, Self::BaseBinaryFallback)
    }

    pub fn is_shell_dispatch_fallback(&self) -> bool {
        matches!(self, Self::ShellDispatchFallback)
    }
}

/// Classification of the executable target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutableKind {
    NativeBinary,
    WindowsBatchScript,
    ShellWrapper,
}

impl ExecutableKind {
    pub fn is_native(&self) -> bool {
        matches!(self, Self::NativeBinary)
    }

    pub fn is_batch_script(&self) -> bool {
        matches!(self, Self::WindowsBatchScript)
    }

    pub fn is_shell_wrapper(&self) -> bool {
        matches!(self, Self::ShellWrapper)
    }
}

/// Concrete launch plan resolving executable, arguments, and process flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchPlan {
    pub program: String,
    pub args: Vec<String>,
    pub creation_flags: u32,
    pub stage: ExecutionStage,
    pub executable_kind: ExecutableKind,
    pub is_batch_guarded: bool,
    pub has_breakaway: bool,
}

impl LaunchPlan {
    pub fn has_breakaway(&self) -> bool {
        self.has_breakaway
    }

    pub fn is_batch_guarded(&self) -> bool {
        self.is_batch_guarded
    }
}

/// Evaluates whether an executable target is a Windows batch file (.bat / .cmd).
pub fn is_batch_script(path: &str) -> bool {
    let lower = path
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_ascii_lowercase();
    lower.ends_with(".bat") || lower.ends_with(".cmd")
}

/// Detects the executable kind based on file path and shell wrappers.
pub fn detect_executable_kind(path: &str) -> ExecutableKind {
    let lower = path
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_ascii_lowercase();
    if lower.ends_with(".bat") || lower.ends_with(".cmd") {
        ExecutableKind::WindowsBatchScript
    } else if lower.ends_with("cmd.exe") || lower.ends_with("powershell.exe") {
        ExecutableKind::ShellWrapper
    } else {
        ExecutableKind::NativeBinary
    }
}

/// Evaluates creation flags based on job breakaway allowance and prior OS error codes.
pub fn resolve_breakaway_flags(
    allow_job_breakaway: bool,
    previous_error_code: Option<i32>,
) -> (u32, bool) {
    if !allow_job_breakaway || previous_error_code == Some(ERROR_ACCESS_DENIED) {
        (0, false)
    } else {
        (CREATE_BREAKAWAY_FROM_JOB, true)
    }
}

/// Builds shell dispatch argument list for wrapping Windows batch scripts.
pub fn build_shell_dispatch_args(target_executable: &str, target_args: &[String]) -> Vec<String> {
    let mut args = vec![
        "/c".to_string(),
        "start".to_string(),
        target_executable.trim().to_string(),
    ];
    args.extend(target_args.iter().cloned());
    args
}

/// Constructs a hardened launch plan resolving binary fallbacks and batch script protections.
pub fn build_launch_plan(
    target_executable: &str,
    target_args: &[String],
    base_binary: Option<&str>,
    allow_job_breakaway: bool,
    previous_error_code: Option<i32>,
) -> LaunchPlan {
    let (creation_flags, has_breakaway) =
        resolve_breakaway_flags(allow_job_breakaway, previous_error_code);

    if is_batch_script(target_executable) {
        if let Some(base) = base_binary.filter(|b| !b.trim().is_empty()) {
            LaunchPlan {
                program: base.trim().to_string(),
                args: target_args.to_vec(),
                creation_flags,
                stage: ExecutionStage::BaseBinaryFallback,
                executable_kind: ExecutableKind::NativeBinary,
                is_batch_guarded: true,
                has_breakaway,
            }
        } else {
            LaunchPlan {
                program: "cmd.exe".to_string(),
                args: build_shell_dispatch_args(target_executable, target_args),
                creation_flags,
                stage: ExecutionStage::ShellDispatchFallback,
                executable_kind: ExecutableKind::ShellWrapper,
                is_batch_guarded: true,
                has_breakaway,
            }
        }
    } else {
        LaunchPlan {
            program: target_executable.trim().to_string(),
            args: target_args.to_vec(),
            creation_flags,
            stage: ExecutionStage::TargetBinary,
            executable_kind: ExecutableKind::NativeBinary,
            is_batch_guarded: false,
            has_breakaway,
        }
    }
}
