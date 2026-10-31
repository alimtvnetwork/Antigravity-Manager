//! Windows Batch File Direct Execution Guard.
//!
//! Enforces process spawning safety per AGENTS.md:
//! Never pass `.cmd` or `.bat` batch files directly as the executable path in
//! `std::process::Command::new(...)`. Multi-instance launchers must implement multi-stage
//! fallback: target binary -> detected base binary -> Windows Shell dispatch (`cmd.exe /c start`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutablePathViolation {
    DirectBatchExecutionForbidden { path: String, suggestion: String },
    EmptyExecutablePath,
}

/// Evaluates whether an executable target is a Windows batch file (.bat / .cmd).
pub fn is_batch_file_extension(path: &str) -> bool {
    let lower = path.trim().to_ascii_lowercase();
    lower.ends_with(".bat") || lower.ends_with(".cmd")
}

/// Validates that an executable path is not a raw batch file passed directly to Command::new.
pub fn validate_executable_path(path: &str) -> Result<(), ExecutablePathViolation> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(ExecutablePathViolation::EmptyExecutablePath);
    }

    if is_batch_file_extension(trimmed) {
        return Err(ExecutablePathViolation::DirectBatchExecutionForbidden {
            path: trimmed.to_string(),
            suggestion: format!(
                "Wrap execution via Windows Shell dispatch: cmd.exe /c start \"\" \"{}\"",
                trimmed
            ),
        });
    }

    Ok(())
}

/// Resolves the actual process executable and argument vector, wrapping batch files in shell dispatch.
pub fn resolve_windows_execution_command(exe_path: &str, args: &[&str]) -> (String, Vec<String>) {
    let trimmed = exe_path.trim();

    if is_batch_file_extension(trimmed) {
        let mut shell_args = vec![
            "/c".to_string(),
            "start".to_string(),
            "".to_string(),
            trimmed.to_string(),
        ];
        shell_args.extend(args.iter().map(|s| s.to_string()));
        ("cmd.exe".to_string(), shell_args)
    } else {
        (
            trimmed.to_string(),
            args.iter().map(|s| s.to_string()).collect(),
        )
    }
}
