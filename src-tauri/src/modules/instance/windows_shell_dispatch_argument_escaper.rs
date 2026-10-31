//! Windows Shell Dispatch Argument Escaper (`Imp-680`).
//!
//! Enforces AGENTS.md Process Spawning & Job Breakaway Invariant:
//! "Multi-instance launchers must implement multi-stage fallback: target binary → detected
//! base binary → Windows Shell dispatch (`cmd.exe /c start`). Never pass `.cmd` or `.bat`
//! batch files directly as the executable path in `std::process::Command::new(...)`."
//!
//! Constructs safe, injection-resistant command arguments when dispatching process executions
//! via Windows `cmd.exe /c start ""` shell fallback, properly escaping shell metacharacters
//! (`&`, `|`, `(`, `)`, `<`, `>`, `^`, `"`, `%`) and whitespace.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Windows `cmd.exe` shell metacharacters requiring escaping or quoting.
pub const CMD_METACHARACTERS: &[char] = &['&', '|', '(', ')', '<', '>', '^', '"', '%', '!', ' '];

/// Execution dispatch strategy resolved by launcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShellDispatchMode {
    /// Direct binary invocation (Stage 1 or Stage 2).
    DirectBinary,
    /// Windows Command Prompt shell fallback (Stage 3).
    CmdShellStart,
}

impl fmt::Display for ShellDispatchMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DirectBinary => write!(f, "DirectBinary"),
            Self::CmdShellStart => write!(f, "CmdShellStart"),
        }
    }
}

/// Audit report produced after escaping arguments for shell execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellEscapeReport {
    /// Executable target path.
    pub target_executable: String,
    /// Number of raw input arguments.
    pub raw_arguments_count: usize,
    /// Count of arguments that required quoting or escaping.
    pub escaped_arguments_count: usize,
    /// Formatted argument array suitable for `std::process::Command`.
    pub formatted_args: Vec<String>,
    /// Full command line string for logging and telemetry.
    pub full_command_line: String,
}

/// Checks if a string contains any Windows cmd shell metacharacters.
pub fn contains_cmd_metacharacters(arg: &str) -> bool {
    arg.chars().any(|c| CMD_METACHARACTERS.contains(&c))
}

/// Escapes a single argument for Windows command line parsing.
///
/// Wraps arguments containing spaces or metacharacters in double quotes, escaping embedded quotes.
pub fn escape_shell_argument(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }

    if !contains_cmd_metacharacters(arg) {
        return arg.to_string();
    }

    let mut escaped = String::with_capacity(arg.len() + 8);
    escaped.push('"');

    for c in arg.chars() {
        if c == '"' {
            escaped.push_str("\\\"");
        } else {
            escaped.push(c);
        }
    }

    escaped.push('"');
    escaped
}

/// Builds arguments for invoking a binary via `cmd.exe /c start "" <exe> [args...]`.
///
/// Note: The empty double quotes `""` immediately following `start` are mandatory in Windows `cmd.exe`
/// syntax to prevent the target executable path from being misinterpreted as the console window title.
pub fn build_windows_shell_start_command(
    executable_path: &str,
    args: &[&str],
) -> (String, Vec<String>) {
    let mut cmd_args = vec![
        "/c".to_string(),
        "start".to_string(),
        "".to_string(), // Empty window title
        escape_shell_argument(executable_path),
    ];

    for arg in args {
        cmd_args.push(escape_shell_argument(arg));
    }

    ("cmd.exe".to_string(), cmd_args)
}

/// Generates a comprehensive escape audit report for a target execution.
pub fn audit_and_build_shell_dispatch(executable_path: &str, args: &[&str]) -> ShellEscapeReport {
    let mut escaped_count = 0;
    let mut formatted_args = Vec::with_capacity(args.len() + 4);

    formatted_args.push("/c".to_string());
    formatted_args.push("start".to_string());
    formatted_args.push("\"\"".to_string());

    let escaped_exe = escape_shell_argument(executable_path);
    if escaped_exe != executable_path {
        escaped_count += 1;
    }
    formatted_args.push(escaped_exe);

    for arg in args {
        let escaped = escape_shell_argument(arg);
        if &escaped != arg {
            escaped_count += 1;
        }
        formatted_args.push(escaped);
    }

    let full_command_line = format!("cmd.exe {}", formatted_args.join(" "));

    ShellEscapeReport {
        target_executable: executable_path.to_string(),
        raw_arguments_count: args.len(),
        escaped_arguments_count: escaped_count,
        formatted_args,
        full_command_line,
    }
}
