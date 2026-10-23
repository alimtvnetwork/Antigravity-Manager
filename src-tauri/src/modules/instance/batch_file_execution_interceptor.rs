//! Batch File Execution Interceptor (`Imp-698`).
//!
//! Enforces AGENTS.md Process Spawning & Job Breakaway Invariant:
//! "Never pass `.cmd` or `.bat` batch files directly as the executable path in
//! `std::process::Command::new(...)`. Multi-instance launchers must implement
//! multi-stage fallback: target binary -> detected base binary -> Windows Shell dispatch (`cmd.exe /c start`)."
//!
//! Windows batch scripts (`.bat` / `.cmd`) are not PE executables and cannot be spawned
//! directly with `CreateProcessW` / `Command::new` without invoking the Windows Command Interpreter (`cmd.exe`).
//! Passing batch scripts directly results in OS error 193 (%1 is not a valid Win32 application)
//! or job breakaway permission denial. This interceptor evaluates execution targets and ensures
//! multi-stage resolution before invocation.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Target classification identified during executable resolution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionTargetKind {
    /// Direct binary executable (e.g. `.exe` or native binary).
    DirectBinary(String),
    /// Windows batch script (`.bat` or `.cmd`). Direct execution forbidden.
    WindowsBatchScript(String),
    /// Delegated invocation via Windows Shell dispatch (`cmd.exe /c start "" ...`).
    WindowsShellDispatch(String),
}

impl ExecutionTargetKind {
    /// Returns true if target is a Windows batch script.
    pub fn is_batch(&self) -> bool {
        matches!(self, Self::WindowsBatchScript(_))
    }

    /// Returns true if target is a direct binary.
    pub fn is_direct_binary(&self) -> bool {
        matches!(self, Self::DirectBinary(_))
    }

    /// Returns true if target is a Windows shell dispatch.
    pub fn is_shell_dispatch(&self) -> bool {
        matches!(self, Self::WindowsShellDispatch(_))
    }

    /// Returns the underlying target string.
    pub fn target_path(&self) -> &str {
        match self {
            Self::DirectBinary(path) => path.as_str(),
            Self::WindowsBatchScript(path) => path.as_str(),
            Self::WindowsShellDispatch(path) => path.as_str(),
        }
    }
}

impl fmt::Display for ExecutionTargetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DirectBinary(path) => write!(f, "DirectBinary({})", path),
            Self::WindowsBatchScript(path) => write!(f, "WindowsBatchScript({})", path),
            Self::WindowsShellDispatch(path) => write!(f, "WindowsShellDispatch({})", path),
        }
    }
}

/// Multi-stage launch resolution strategy determined for process execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaunchResolutionStrategy {
    /// Stage 1: Target binary directly passed to `Command::new`.
    TargetBinaryDirect,
    /// Stage 2: Fall back to detected base binary (e.g. installed IDE executable).
    DetectedBaseBinary { base_path: String },
    /// Stage 3: Wrap execution inside Windows Shell dispatch (`cmd.exe /c start "" <exe> [args]`).
    WindowsShellDispatch { cmd_exe: String, args: Vec<String> },
}

impl LaunchResolutionStrategy {
    /// Returns true if direct binary execution was chosen.
    pub fn is_direct(&self) -> bool {
        matches!(self, Self::TargetBinaryDirect)
    }

    /// Returns true if base binary fallback was chosen.
    pub fn is_detected_base_binary(&self) -> bool {
        matches!(self, Self::DetectedBaseBinary { .. })
    }

    /// Returns true if Windows Shell dispatch was chosen.
    pub fn is_shell_dispatch(&self) -> bool {
        matches!(self, Self::WindowsShellDispatch { .. })
    }

    /// Canonical string identifier for the strategy.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TargetBinaryDirect => "target_binary_direct",
            Self::DetectedBaseBinary { .. } => "detected_base_binary",
            Self::WindowsShellDispatch { .. } => "windows_shell_dispatch",
        }
    }
}

impl fmt::Display for LaunchResolutionStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TargetBinaryDirect => write!(f, "TargetBinaryDirect"),
            Self::DetectedBaseBinary { base_path } => {
                write!(f, "DetectedBaseBinary({})", base_path)
            }
            Self::WindowsShellDispatch { cmd_exe, args } => {
                write!(f, "WindowsShellDispatch({} {})", cmd_exe, args.join(" "))
            }
        }
    }
}

/// Interception and resolution report produced before executing a process command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchInterceptionResult {
    /// Whether the target was identified as a batch script and intercepted.
    pub was_batch_intercepted: bool,
    /// The original target string provided by caller.
    pub original_target: String,
    /// The resolved executable to pass as `Command::new(...)`.
    pub resolved_executable: String,
    /// The resolved arguments to pass as `Command::args(...)`.
    pub resolved_args: Vec<String>,
    /// The resolution strategy selected.
    pub strategy: LaunchResolutionStrategy,
    /// Diagnostic notes regarding resolution decisions.
    pub notes: Vec<String>,
}

impl BatchInterceptionResult {
    /// Returns true if batch interception occurred.
    pub fn is_intercepted(&self) -> bool {
        self.was_batch_intercepted
    }

    /// Returns true if the strategy is direct execution.
    pub fn is_direct(&self) -> bool {
        self.strategy.is_direct()
    }

    /// Renders full command line representation for debugging and telemetry.
    pub fn command_line_preview(&self) -> String {
        if self.resolved_args.is_empty() {
            self.resolved_executable.clone()
        } else {
            format!("{} {}", self.resolved_executable, self.resolved_args.join(" "))
        }
    }
}

/// Classification of executable script extensions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScriptExtension {
    /// Windows Batch File (`.bat`)
    Bat,
    /// Windows NT Command Script (`.cmd`)
    Cmd,
    /// Other executable or script extension
    Other(String),
}

/// Verdict returned when inspecting an executable path for batch file risks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchInterceptorVerdict {
    /// Direct execution is strictly forbidden because the file is a batch file.
    DirectExecutionForbidden {
        extension: String,
        reason: String,
    },
    /// The target is a safe native executable or non-batch script.
    SafeNativeExecutable,
    /// The target path provided was empty or whitespace.
    EmptyPath,
}

impl BatchInterceptorVerdict {
    /// Returns true if the verdict strictly forbids direct execution.
    pub fn is_forbidden(&self) -> bool {
        matches!(self, Self::DirectExecutionForbidden { .. })
    }
}

/// Execution dispatch plan containing the resolved executable, arguments, and shell configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionDispatchPlan {
    /// Executable path to invoke (either `cmd.exe` or the original target binary).
    pub executable_path: String,
    /// Arguments to pass to the process.
    pub arguments: Vec<String>,
    /// Whether execution was wrapped inside a shell (e.g. `cmd.exe`).
    pub uses_shell_wrapper: bool,
    /// Shell kind identifier (e.g. `Some("cmd.exe")`), or `None` if direct execution.
    pub shell_kind: Option<String>,
    /// Full sanitized command line string for logging, diagnostics, and debugging.
    pub sanitized_command_line: String,
}

/// Classifies a path's file extension into `ScriptExtension`.
pub fn classify_script_extension(path: &str) -> ScriptExtension {
    let cleaned = path.trim().trim_matches('"').trim_matches('\'');
    let path_obj = std::path::Path::new(cleaned);
    match path_obj.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("bat") => ScriptExtension::Bat,
        Some(ext) if ext.eq_ignore_ascii_case("cmd") => ScriptExtension::Cmd,
        Some(ext) => ScriptExtension::Other(ext.to_string()),
        None => ScriptExtension::Other(String::new()),
    }
}

/// Evaluates whether an executable path is a Windows batch file (`.bat` or `.cmd`).
///
/// Strips leading/trailing whitespace and surrounding quotation marks, then performs
/// a case-insensitive extension check.
pub fn is_batch_file_path(target: &str) -> bool {
    let trimmed = target.trim();
    let stripped = trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(trimmed)
        .trim();

    let lower = stripped.to_ascii_lowercase();
    lower.ends_with(".bat") || lower.ends_with(".cmd")
}

/// Checks if an executable path represents a Windows batch file (`.bat` or `.cmd`).
pub fn is_windows_batch_file(path: &str) -> bool {
    is_batch_file_path(path)
}

/// Evaluates an executable path and returns a `BatchInterceptorVerdict`.
pub fn evaluate_verdict(path: &str) -> BatchInterceptorVerdict {
    let cleaned = path.trim().trim_matches('"').trim_matches('\'');
    if cleaned.is_empty() {
        return BatchInterceptorVerdict::EmptyPath;
    }

    match classify_script_extension(cleaned) {
        ScriptExtension::Bat => BatchInterceptorVerdict::DirectExecutionForbidden {
            extension: "bat".to_string(),
            reason: "Never pass .bat batch files directly to Command::new; wrap with cmd.exe".to_string(),
        },
        ScriptExtension::Cmd => BatchInterceptorVerdict::DirectExecutionForbidden {
            extension: "cmd".to_string(),
            reason: "Never pass .cmd batch files directly to Command::new; wrap with cmd.exe".to_string(),
        },
        ScriptExtension::Other(_) => BatchInterceptorVerdict::SafeNativeExecutable,
    }
}

/// Classifies an execution target into its corresponding `ExecutionTargetKind`.
pub fn classify_execution_target(target: &str) -> ExecutionTargetKind {
    let trimmed = target.trim();
    let lower = trimmed.to_ascii_lowercase();

    if lower.starts_with("cmd.exe") || lower.starts_with("cmd /c") || lower.contains("/c start") {
        ExecutionTargetKind::WindowsShellDispatch(target.to_string())
    } else if is_batch_file_path(trimmed) {
        ExecutionTargetKind::WindowsBatchScript(target.to_string())
    } else {
        ExecutionTargetKind::DirectBinary(target.to_string())
    }
}

/// Quotes and escapes a single command line argument for safe Windows shell dispatch.
///
/// Wraps empty arguments or arguments containing spaces and shell metacharacters
/// (`&`, `|`, `(`, `)`, `<`, `>`, `^`, `"`, `%`, `!`) in double quotes, escaping embedded quotes.
pub fn quote_shell_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }

    let needs_quotes = arg
        .chars()
        .any(|c| c.is_whitespace() || "&|()<>^\"%!".contains(c));

    if !needs_quotes {
        return arg.to_string();
    }

    let mut quoted = String::with_capacity(arg.len() + 8);
    quoted.push('"');

    for c in arg.chars() {
        if c == '"' {
            quoted.push_str("\\\"");
        } else {
            quoted.push(c);
        }
    }

    quoted.push('"');
    quoted
}

/// Sanitizes a command line argument for Windows cmd.exe batch dispatch.
pub fn sanitize_batch_arg(arg: &str) -> String {
    if arg.is_empty() {
        return String::new();
    }

    let mut escaped = String::with_capacity(arg.len() + 8);
    for ch in arg.chars() {
        match ch {
            '%' => escaped.push_str("%%"),
            '^' => escaped.push_str("^^"),
            '&' => escaped.push_str("^&"),
            '<' => escaped.push_str("^<"),
            '>' => escaped.push_str("^>"),
            '|' => escaped.push_str("^|"),
            '"' => escaped.push_str("\\\""),
            other => escaped.push(other),
        }
    }
    escaped
}

/// Quotes a batch file path safely for Windows shell execution.
fn quote_batch_path(path: &str) -> String {
    let trimmed = path.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed.to_string()
    } else {
        format!("\"{}\"", trimmed)
    }
}

/// Intercepts a potential batch file execution and produces a safe `ExecutionDispatchPlan`.
pub fn intercept_and_plan_execution(
    raw_executable: &str,
    args: &[String],
    use_start_window: bool,
) -> Result<ExecutionDispatchPlan, String> {
    let trimmed = raw_executable.trim();
    if trimmed.is_empty() {
        return Err("Executable path cannot be empty or whitespace".to_string());
    }

    if is_windows_batch_file(trimmed) {
        let op = if use_start_window { "start" } else { "call" };
        let quoted_path = quote_batch_path(trimmed);

        let mut arguments = Vec::with_capacity(args.len() + 3);
        arguments.push("/c".to_string());
        arguments.push(op.to_string());
        arguments.push(quoted_path);

        for arg in args {
            arguments.push(sanitize_batch_arg(arg));
        }

        let executable_path = "cmd.exe".to_string();
        let sanitized_command_line = format!("{} {}", executable_path, arguments.join(" "));

        Ok(ExecutionDispatchPlan {
            executable_path,
            arguments,
            uses_shell_wrapper: true,
            shell_kind: Some("cmd.exe".to_string()),
            sanitized_command_line,
        })
    } else {
        let executable_path = trimmed.to_string();
        let arguments = args.to_vec();
        let sanitized_command_line = if arguments.is_empty() {
            executable_path.clone()
        } else {
            format!("{} {}", executable_path, arguments.join(" "))
        };

        Ok(ExecutionDispatchPlan {
            executable_path,
            arguments,
            uses_shell_wrapper: false,
            shell_kind: None,
            sanitized_command_line,
        })
    }
}

/// Resolves an executable launch target and arguments, intercepting `.bat` and `.cmd` scripts.
///
/// Multi-stage fallback behavior per AGENTS.md:
/// 1. If `target` is `.exe` or direct binary: passes through directly with `TargetBinaryDirect`.
/// 2. If `target` is `.bat` or `.cmd`:
///    a. If `detected_base_binary` is available: redirects to `DetectedBaseBinary`.
///    b. If no base binary detected: wraps into `WindowsShellDispatch` using `cmd.exe /c start ""`
///       with properly quoted arguments.
pub fn resolve_executable_launch(
    target: &str,
    raw_args: &[&str],
    detected_base_binary: Option<&str>,
) -> BatchInterceptionResult {
    let trimmed_target = target.trim();

    if !is_batch_file_path(trimmed_target) {
        return BatchInterceptionResult {
            was_batch_intercepted: false,
            original_target: target.to_string(),
            resolved_executable: trimmed_target.to_string(),
            resolved_args: raw_args.iter().map(|s| s.to_string()).collect(),
            strategy: LaunchResolutionStrategy::TargetBinaryDirect,
            notes: vec![format!(
                "Direct binary target '{}' resolved without batch interception.",
                trimmed_target
            )],
        };
    }

    // Target is a Windows batch script (.bat or .cmd).
    // Stage 2 Fallback: Check if detected base binary is available and not a batch script itself.
    if let Some(base_bin) = detected_base_binary {
        let trimmed_base = base_bin.trim();
        if !trimmed_base.is_empty() && !is_batch_file_path(trimmed_base) {
            let base_path = trimmed_base.to_string();
            return BatchInterceptionResult {
                was_batch_intercepted: true,
                original_target: target.to_string(),
                resolved_executable: base_path.clone(),
                resolved_args: raw_args.iter().map(|s| s.to_string()).collect(),
                strategy: LaunchResolutionStrategy::DetectedBaseBinary {
                    base_path: base_path.clone(),
                },
                notes: vec![format!(
                    "Windows batch script '{}' intercepted; redirected to detected base binary '{}'.",
                    trimmed_target, base_path
                )],
            };
        }
    }

    // Stage 3 Fallback: Windows Shell Dispatch (`cmd.exe /c start "" <exe> [args]`).
    let cmd_exe = "cmd.exe".to_string();
    let mut shell_args = Vec::with_capacity(raw_args.len() + 4);
    shell_args.push("/c".to_string());
    shell_args.push("start".to_string());
    shell_args.push("\"\"".to_string()); // Mandatory empty title parameter for Windows cmd `start`
    shell_args.push(quote_shell_arg(trimmed_target));

    for arg in raw_args {
        shell_args.push(quote_shell_arg(arg));
    }

    BatchInterceptionResult {
        was_batch_intercepted: true,
        original_target: target.to_string(),
        resolved_executable: cmd_exe.clone(),
        resolved_args: shell_args.clone(),
        strategy: LaunchResolutionStrategy::WindowsShellDispatch {
            cmd_exe,
            args: shell_args,
        },
        notes: vec![format!(
            "Windows batch script '{}' intercepted; no base binary detected; delegated to Windows Shell dispatch ('cmd.exe /c start \"\"').",
            trimmed_target
        )],
    }
}
