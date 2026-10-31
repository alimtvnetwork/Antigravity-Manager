//! Cross-Platform Compatibility Guard.
//!
//! Enforces cross-platform portability per AGENTS.md:
//! "Cross-Platform Compatibility: Evaluate every code addition and dependency change
//! for seamless cross-platform support."
//!
//! Verification rules:
//! 1. Hardcoded drive letter paths (e.g., "C:\\", "D:\\", "c:/") in string literals
//!    must be gated by platform checks.
//! 2. Bare OS shells (e.g., "cmd.exe", "powershell.exe", "/bin/sh", "/bin/bash")
//!    must not be invoked without #[cfg(...)] guards.
//! 3. Unconditional platform-specific APIs (e.g., std::os::windows, std::os::unix,
//!    CREATE_BREAKAWAY_FROM_JOB) must be platform-gated.
//! 4. Literal path separator assumptions (e.g., manual "\\" string joins) must use
//!    PathBuf or Path::join() instead.

use serde::{Deserialize, Serialize};

/// Represents a source file inspected for cross-platform compliance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    pub file_label: String,
    pub content: String,
}

impl SourceFile {
    /// Constructs a new source file entry for evaluation.
    pub fn new(file_label: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            file_label: file_label.into(),
            content: content.into(),
        }
    }
}

/// Specific cross-platform portability rule violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformRuleViolation {
    HardcodedDriveLetterPath {
        line_number: usize,
        path_snippet: String,
    },
    OSSpecificShellInvocation {
        line_number: usize,
        shell_snippet: String,
    },
    UnconditionalPlatformApi {
        line_number: usize,
        api_name: String,
    },
    LiteralPathSeparatorAssumption {
        line_number: usize,
        separator_snippet: String,
    },
}

/// Detailed compatibility finding with line location and actionable remediation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatFinding {
    pub rule_id: String,
    pub severity: String,
    pub line_number: usize,
    pub explanation: String,
    pub remediation_hint: String,
    pub violation: PlatformRuleViolation,
}

/// Compatibility report for a single evaluated source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileCompatReport {
    pub file_label: String,
    pub is_clean: bool,
    pub findings: Vec<CompatFinding>,
}

/// Aggregate compatibility report across all evaluated source files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatReport {
    pub is_clean: bool,
    pub total_findings: usize,
    pub file_reports: Vec<FileCompatReport>,
}

/// Strips line comments while safely preserving string literals that contain '//'.
fn strip_line_comment(line: &str) -> &str {
    let mut in_str = false;
    let mut escape = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if in_str {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
        } else if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            let byte_idx = line
                .char_indices()
                .nth(i)
                .map(|(idx, _)| idx)
                .unwrap_or(line.len());
            return &line[..byte_idx];
        }
        i += 1;
    }

    line
}

/// Checks whether a source line contains an explicit platform cfg guard.
fn check_line_has_platform_gate(line: &str) -> bool {
    let stripped = strip_line_comment(line);
    stripped.contains("#[cfg(windows")
        || stripped.contains("#[cfg(unix")
        || stripped.contains("#[cfg(target_os")
        || stripped.contains("#[cfg(target_family")
        || stripped.contains("#[cfg(all(windows")
        || stripped.contains("#[cfg(any(windows")
        || stripped.contains("#[cfg(not(windows")
        || stripped.contains("#[cfg(not(unix")
        || stripped.contains("cfg!(windows")
        || stripped.contains("cfg!(unix")
        || stripped.contains("cfg!(target_os")
        || stripped.contains("cfg!(target_family")
}

/// Counts opening and closing curly braces outside string literals and line comments.
fn count_net_braces(line: &str) -> (usize, usize) {
    let stripped = strip_line_comment(line);
    let mut opens = 0;
    let mut closes = 0;
    let mut in_str = false;
    let mut escape = false;

    for c in stripped.chars() {
        if in_str {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
        } else if c == '{' {
            opens += 1;
        } else if c == '}' {
            closes += 1;
        }
    }

    (opens, closes)
}

/// Tracks nested curly-brace scope depth and active platform cfg gating.
struct ScopeGateTracker {
    depth: usize,
    gated_depth_stack: Vec<usize>,
    pending_attr_gate: bool,
}

impl ScopeGateTracker {
    fn new() -> Self {
        Self {
            depth: 0,
            gated_depth_stack: Vec::new(),
            pending_attr_gate: false,
        }
    }

    fn is_currently_gated(&self, line: &str) -> bool {
        if !self.gated_depth_stack.is_empty() || self.pending_attr_gate {
            return true;
        }
        check_line_has_platform_gate(line)
    }

    fn advance_line(&mut self, line: &str) {
        let has_gate = check_line_has_platform_gate(line);
        let (opens, closes) = count_net_braces(line);

        if has_gate && opens == 0 {
            self.pending_attr_gate = true;
        }

        if opens > 0 {
            let new_depth = self.depth + opens;
            if self.pending_attr_gate || has_gate {
                self.gated_depth_stack.push(new_depth);
                self.pending_attr_gate = false;
            }
            self.depth = new_depth;
        }

        if closes > 0 {
            self.depth = self.depth.saturating_sub(closes);
            self.gated_depth_stack.retain(|&d| d <= self.depth);
        }

        let trimmed = line.trim();
        if trimmed.ends_with(';') && self.pending_attr_gate {
            self.pending_attr_gate = false;
        }
    }
}

/// Detects hardcoded drive letter path patterns such as "C:\\" or "c:/" in string literals.
fn detect_hardcoded_drive_letter(line: &str) -> Option<String> {
    let stripped = strip_line_comment(line);
    let bytes = stripped.as_bytes();

    for i in 0..bytes.len() {
        if bytes[i] == b':' && i > 0 && bytes[i - 1].is_ascii_alphabetic() {
            let is_isolated = if i >= 2 {
                !bytes[i - 2].is_ascii_alphanumeric()
            } else {
                true
            };

            if is_isolated && i + 1 < bytes.len() {
                let next = bytes[i + 1];
                if next == b'/' || next == b'\\' {
                    let start = if i >= 2 { i - 2 } else { 0 };
                    let end = (i + 15).min(stripped.len());
                    return Some(stripped[start..end].trim().to_string());
                }
            }
        }
    }

    None
}

/// Detects bare OS shells without platform branching.
fn detect_os_specific_shell(line: &str) -> Option<String> {
    let stripped = strip_line_comment(line);
    let shell_targets = [
        "\"cmd.exe\"",
        "\"cmd.exe /c\"",
        "\"powershell.exe\"",
        "\"pwsh.exe\"",
        "\"/bin/sh\"",
        "\"/bin/bash\"",
        "\"/bin/zsh\"",
        "Command::new(\"cmd\")",
        "Command::new(\"powershell\")",
        "Command::new(\"cmd.exe\")",
        "Command::new(\"powershell.exe\")",
        "Command::new(\"/bin/sh\")",
        "Command::new(\"/bin/bash\")",
    ];

    for target in shell_targets {
        if stripped.contains(target) {
            return Some((*target).to_string());
        }
    }

    None
}

/// Detects literal path separator assumptions where PathBuf or Path::join should be used.
fn detect_literal_path_separator(line: &str) -> Option<String> {
    let stripped = strip_line_comment(line);

    if stripped.contains(".join(\"\\\\") || stripped.contains(".join(r\"\\") {
        return Some(".join(\"\\\\\")".to_string());
    }
    if stripped.contains("+ \"\\\\\"") || stripped.contains("+ r\"\\\"") {
        return Some("+ \"\\\\\"".to_string());
    }
    if stripped.contains(".push_str(\"\\\\") || stripped.contains(".push_str(r\"\\") {
        return Some(".push_str(\"\\\\\")".to_string());
    }
    if stripped.contains(".split('\\\\')") || stripped.contains(".split(\"\\\\\")") {
        return Some(".split('\\\\')".to_string());
    }
    if stripped.contains("format!")
        && (stripped.contains("{}\\\\")
            || stripped.contains("\\\\{}")
            || stripped.contains(r"{}\")
            || stripped.contains(r"\{}"))
    {
        return Some("format! with literal '\\\\' separator".to_string());
    }

    None
}

/// Detects unconditional platform-specific APIs.
fn detect_unconditional_platform_api(line: &str) -> Option<String> {
    let stripped = strip_line_comment(line);
    let platform_apis = [
        "std::os::windows",
        "std::os::unix",
        "CREATE_BREAKAWAY_FROM_JOB",
        "CreateProcessW",
        "SetInformationJobObject",
    ];

    for api in platform_apis {
        if stripped.contains(api) {
            return Some((*api).to_string());
        }
    }

    None
}

/// Evaluates a single source file for cross-platform portability violations.
pub fn evaluate_file(file: &SourceFile) -> FileCompatReport {
    let mut findings = Vec::new();
    let mut tracker = ScopeGateTracker::new();

    for (idx, line) in file.content.lines().enumerate() {
        let line_number = idx + 1;
        let is_gated = tracker.is_currently_gated(line);

        if !is_gated {
            if let Some(path_snippet) = detect_hardcoded_drive_letter(line) {
                findings.push(CompatFinding {
                    rule_id: "CROSS_PLATFORM_NO_HARDCODED_DRIVE".to_string(),
                    severity: "error".to_string(),
                    line_number,
                    explanation: "Hardcoded drive letter path detected without platform cfg gate"
                        .to_string(),
                    remediation_hint:
                        "Use repository-relative paths, dirs crate, or PathBuf instead of hardcoded drive letters"
                            .to_string(),
                    violation: PlatformRuleViolation::HardcodedDriveLetterPath {
                        line_number,
                        path_snippet,
                    },
                });
            }

            if let Some(shell_snippet) = detect_os_specific_shell(line) {
                findings.push(CompatFinding {
                    rule_id: "CROSS_PLATFORM_NO_BARE_SHELL".to_string(),
                    severity: "error".to_string(),
                    line_number,
                    explanation: "OS-specific shell invocation without platform branch detected"
                        .to_string(),
                    remediation_hint:
                        "Use cross-platform process dispatch or gate with #[cfg(windows)] / #[cfg(unix)]"
                            .to_string(),
                    violation: PlatformRuleViolation::OSSpecificShellInvocation {
                        line_number,
                        shell_snippet,
                    },
                });
            }

            if let Some(api_name) = detect_unconditional_platform_api(line) {
                findings.push(CompatFinding {
                    rule_id: "CROSS_PLATFORM_NO_UNCONDITIONAL_PLATFORM_API".to_string(),
                    severity: "error".to_string(),
                    line_number,
                    explanation: "Unconditional platform API usage detected without cfg gate"
                        .to_string(),
                    remediation_hint:
                        "Gate platform-specific APIs with #[cfg(windows)] / #[cfg(unix)] or use multi-stage fallback ladder"
                            .to_string(),
                    violation: PlatformRuleViolation::UnconditionalPlatformApi {
                        line_number,
                        api_name,
                    },
                });
            }

            if let Some(separator_snippet) = detect_literal_path_separator(line) {
                findings.push(CompatFinding {
                    rule_id: "CROSS_PLATFORM_NO_LITERAL_SEPARATOR".to_string(),
                    severity: "error".to_string(),
                    line_number,
                    explanation: "Literal path separator assumption detected in path manipulation"
                        .to_string(),
                    remediation_hint:
                        "Use PathBuf or Path::join() instead of manual backslash string concatenation"
                            .to_string(),
                    violation: PlatformRuleViolation::LiteralPathSeparatorAssumption {
                        line_number,
                        separator_snippet,
                    },
                });
            }
        }

        tracker.advance_line(line);
    }

    let is_clean = findings.is_empty();
    FileCompatReport {
        file_label: file.file_label.clone(),
        is_clean,
        findings,
    }
}

/// Evaluates a slice of source files and aggregates findings into a CompatReport.
pub fn evaluate_files(files: &[SourceFile]) -> CompatReport {
    let mut file_reports = Vec::new();
    let mut total_findings = 0;

    for file in files {
        let report = evaluate_file(file);
        total_findings += report.findings.len();
        file_reports.push(report);
    }

    let is_clean = total_findings == 0;
    CompatReport {
        is_clean,
        total_findings,
        file_reports,
    }
}
