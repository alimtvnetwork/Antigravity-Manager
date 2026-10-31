use std::process::Command as StdCommand;
use tokio::process::Command as TokioCommand;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// CREATE_NO_WINDOW (0x08000000) prevents console window creation on Windows
#[cfg(target_os = "windows")]
const WINDOWS_NO_WINDOW_FLAGS: u32 = 0x08000000;

pub trait CommandExtWrapper {
    /// Add creation flags on Windows to suppress console window flashing
    fn creation_flags_windows(&mut self) -> &mut Self;
}

impl CommandExtWrapper for StdCommand {
    fn creation_flags_windows(&mut self) -> &mut Self {
        #[cfg(target_os = "windows")]
        self.creation_flags(WINDOWS_NO_WINDOW_FLAGS);

        self
    }
}

impl CommandExtWrapper for TokioCommand {
    fn creation_flags_windows(&mut self) -> &mut Self {
        #[cfg(target_os = "windows")]
        self.creation_flags(WINDOWS_NO_WINDOW_FLAGS);

        self
    }
}

/// Imp-65: Cross-Platform Shell Argument Escaping
/// Escapes an argument for Windows cmd.exe / PowerShell execution.
pub fn escape_cmd_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }
    if !arg.contains([' ', '\t', '\n', '\x0b', '\"']) {
        return arg.to_string();
    }
    let mut escaped = String::from("\"");
    let mut backslashes = 0;
    for c in arg.chars() {
        if c == '\\' {
            backslashes += 1;
        } else if c == '\"' {
            escaped.push_str(&"\\".repeat(backslashes * 2 + 1));
            escaped.push('\"');
            backslashes = 0;
        } else {
            escaped.push_str(&"\\".repeat(backslashes));
            escaped.push(c);
            backslashes = 0;
        }
    }
    escaped.push_str(&"\\".repeat(backslashes * 2));
    escaped.push('\"');
    escaped
}

/// Escapes an argument for POSIX shells (sh, bash, zsh).
pub fn escape_posix_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "''".to_string();
    }
    if arg
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':' | '='))
    {
        return arg.to_string();
    }
    format!("'{}'", arg.replace('\'', "'\\''"))
}

/// Escapes an argument according to the current host operating system.
pub fn escape_shell_arg(arg: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        escape_cmd_arg(arg)
    }
    #[cfg(not(target_os = "windows"))]
    {
        escape_posix_arg(arg)
    }
}

/// Joins a collection of arguments safely with platform-appropriate escaping.
pub fn join_shell_args(args: &[String]) -> String {
    args.iter()
        .map(|a| escape_shell_arg(a))
        .collect::<Vec<_>>()
        .join(" ")
}
