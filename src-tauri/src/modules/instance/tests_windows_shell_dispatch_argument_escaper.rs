//! Unit tests for Windows Shell Dispatch Argument Escaper (`Imp-680`).

use super::windows_shell_dispatch_argument_escaper::*;

#[test]
fn test_plain_argument_unmodified() {
    assert_eq!(escape_shell_argument("simple"), "simple");
    assert_eq!(escape_shell_argument("12345"), "12345");
    assert_eq!(escape_shell_argument("--verbose"), "--verbose");
}

#[test]
fn test_argument_with_spaces_quoted() {
    assert_eq!(
        escape_shell_argument("C:\\Program Files\\Antigravity\\app.exe"),
        "\"C:\\Program Files\\Antigravity\\app.exe\""
    );
    assert_eq!(escape_shell_argument("hello world"), "\"hello world\"");
}

#[test]
fn test_argument_with_metacharacters() {
    assert_eq!(escape_shell_argument("foo&bar"), "\"foo&bar\"");
    assert_eq!(escape_shell_argument("calc|cmd"), "\"calc|cmd\"");
    assert_eq!(escape_shell_argument("price%rate"), "\"price%rate\"");
}

#[test]
fn test_empty_argument_quoted() {
    assert_eq!(escape_shell_argument(""), "\"\"");
}

#[test]
fn test_build_windows_shell_start_command() {
    let (exe, args) =
        build_windows_shell_start_command("C:\\app.exe", &["--port", "8080", "--name", "My App"]);
    assert_eq!(exe, "cmd.exe");
    assert_eq!(args[0], "/c");
    assert_eq!(args[1], "start");
    assert_eq!(args[2], ""); // Mandatory empty title
    assert_eq!(args[3], "C:\\app.exe");
    assert_eq!(args[4], "--port");
    assert_eq!(args[5], "8080");
    assert_eq!(args[6], "--name");
    assert_eq!(args[7], "\"My App\"");
}

#[test]
fn test_audit_and_build_shell_dispatch() {
    let report = audit_and_build_shell_dispatch(
        "C:\\Program Files\\App.exe",
        &["--flag", "value with spaces"],
    );

    assert_eq!(report.target_executable, "C:\\Program Files\\App.exe");
    assert_eq!(report.raw_arguments_count, 2);
    assert_eq!(report.escaped_arguments_count, 2); // Exe + "value with spaces"
    assert!(report
        .full_command_line
        .starts_with("cmd.exe /c start \"\""));
    assert!(report
        .full_command_line
        .contains("\"C:\\Program Files\\App.exe\""));
    assert!(report.full_command_line.contains("\"value with spaces\""));
}
