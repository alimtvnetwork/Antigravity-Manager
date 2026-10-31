//! Caller Process Ancestry Tree Scanner (`Imp-688`).
//!
//! Enforces AGENTS.md Caller IDE Self-Preservation & Transient Test Sandboxing invariant:
//! "Automated tests, CLI commands, and auto-switcher daemons MUST detect the caller's active
//! IDE process (`detect_caller_instance_id()`) to prevent terminating the developer's working
//! IDE session. Commands targeting the caller instance (`agm instances stop`, `rm`, `switch`)
//! must fail-fast with exit code 2 unless `--force` is explicitly provided."
//!
//! Traverses parent process ID (PPID) hierarchy upwards from the current executing CLI process
//! to detect whether invocation originated inside an active Antigravity IDE instance terminal,
//! extracting the caller instance ID and protecting the developer's working session from termination.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Maximum search depth when ascending the process parentage tree.
pub const MAX_ANCESTRY_SCAN_DEPTH: usize = 24;

/// Known IDE binary names that identify a developer's working IDE session.
pub const KNOWN_IDE_BINARIES: &[&str] = &[
    "antigravity.exe",
    "antigravity",
    "cursor.exe",
    "cursor",
    "windsurf.exe",
    "windsurf",
    "code.exe",
    "code",
];

/// Known terminal and shell binaries used to detect standalone terminals.
pub const KNOWN_TERMINAL_BINARIES: &[&str] = &[
    "pwsh.exe",
    "pwsh",
    "powershell.exe",
    "powershell",
    "cmd.exe",
    "cmd",
    "bash.exe",
    "bash",
    "zsh",
    "fish",
    "sh",
    "windowsterminal.exe",
    "wt.exe",
    "alacritty.exe",
    "alacritty",
    "iterm2",
    "kitty",
    "konsole",
    "gnome-terminal",
    "explorer.exe",
];

/// A single process entry in an operating system process table snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessNode {
    /// Process identifier.
    pub pid: u32,
    /// Parent Process identifier.
    pub ppid: u32,
    /// Executable binary filename (e.g. `pwsh.exe`, `antigravity.exe`).
    pub executable_name: String,
    /// Optional command-line invocation string.
    pub command_line: Option<String>,
}

impl ProcessNode {
    /// Creates a new process node.
    pub fn new(pid: u32, ppid: u32, executable_name: impl Into<String>) -> Self {
        Self {
            pid,
            ppid,
            executable_name: executable_name.into(),
            command_line: None,
        }
    }

    /// Attaches an optional command line string.
    pub fn with_command_line(mut self, cmd: impl Into<String>) -> Self {
        self.command_line = Some(cmd.into());
        self
    }

    /// Returns true if this process binary corresponds to the CLI or manager tool rather than the IDE.
    pub fn is_manager_binary(&self) -> bool {
        let lower = self.executable_name.to_ascii_lowercase();
        lower.starts_with("antigravity-manager")
            || lower.contains("agm")
            || lower.starts_with("agm-")
    }

    /// Returns true if this executable represents a known IDE application.
    pub fn is_known_ide(&self) -> bool {
        if self.is_manager_binary() {
            return false;
        }

        let lower = self.executable_name.to_ascii_lowercase();
        if KNOWN_IDE_BINARIES.iter().any(|&known| lower == known) {
            return true;
        }

        // Cloned instance executables: antigravity-<id>.exe or Antigravity-<id>
        if (lower.starts_with("antigravity-") || lower.starts_with("antigravity_"))
            && !lower.contains("manager")
        {
            return true;
        }

        false
    }

    /// Returns true if this executable represents a known shell or terminal application.
    pub fn is_known_terminal(&self) -> bool {
        let lower = self.executable_name.to_ascii_lowercase();
        KNOWN_TERMINAL_BINARIES.iter().any(|&known| lower == known)
    }
}

/// Discovered provenance / origin of the caller process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallerIdeOrigin {
    /// Invocation originated from an active Antigravity IDE terminal session.
    AntigravityIdeTerminal {
        /// Discovered instance ID of the IDE.
        instance_id: String,
        /// Process ID of the IDE ancestor.
        ide_pid: u32,
    },
    /// Invocation originated from a standalone external terminal (e.g. PowerShell, CMD, Bash).
    StandaloneTerminal,
    /// Origin could not be determined.
    Unknown,
}

impl fmt::Display for CallerIdeOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AntigravityIdeTerminal {
                instance_id,
                ide_pid,
            } => {
                write!(
                    f,
                    "Antigravity IDE Terminal (Instance: '{instance_id}', PID: {ide_pid})"
                )
            }
            Self::StandaloneTerminal => write!(f, "Standalone Terminal"),
            Self::Unknown => write!(f, "Unknown Process Origin"),
        }
    }
}

/// Result of scanning the caller process ancestry tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AncestryScanReport {
    /// Process nodes traversed from leaf to ancestor root.
    pub process_chain: Vec<ProcessNode>,
    /// Resolved origin classification.
    pub caller_origin: CallerIdeOrigin,
    /// True if an active Antigravity IDE parent process was detected.
    pub is_caller_ide_detected: bool,
}

impl AncestryScanReport {
    /// Returns the detected caller instance ID if caller is inside an IDE terminal.
    pub fn caller_instance_id(&self) -> Option<&str> {
        match &self.caller_origin {
            CallerIdeOrigin::AntigravityIdeTerminal { instance_id, .. } => {
                Some(instance_id.as_str())
            }
            CallerIdeOrigin::StandaloneTerminal | CallerIdeOrigin::Unknown => None,
        }
    }

    /// Returns the detected caller IDE PID if caller is inside an IDE terminal.
    pub fn ide_pid(&self) -> Option<u32> {
        match &self.caller_origin {
            CallerIdeOrigin::AntigravityIdeTerminal { ide_pid, .. } => Some(*ide_pid),
            CallerIdeOrigin::StandaloneTerminal | CallerIdeOrigin::Unknown => None,
        }
    }
}

impl fmt::Display for AncestryScanReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AncestryScanReport: origin={}, ide_detected={}, chain_length={}",
            self.caller_origin,
            self.is_caller_ide_detected,
            self.process_chain.len()
        )
    }
}

/// Helper function to extract an instance identifier from command line flags or executable name.
fn extract_instance_id_from_node(node: &ProcessNode) -> Option<String> {
    // 1. Inspect command line arguments if available
    if let Some(cmd) = &node.command_line {
        // Match --instance-id=<id> or -instance-id <id>
        if let Some(idx) = cmd.find("--instance-id=") {
            let rest = &cmd[idx + "--instance-id=".len()..];
            let token = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches('"');
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
        if let Some(idx) = cmd.find("--instance-id ") {
            let rest = &cmd[idx + "--instance-id ".len()..];
            let token = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches('"');
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
        // Match --user-data-dir containing instances/<id>
        if let Some(idx) = cmd.find("instances/") {
            let rest = &cmd[idx + "instances/".len()..];
            let token = rest
                .split(|c: char| c == '/' || c == '\\' || c.is_whitespace() || c == '"')
                .next()
                .unwrap_or("");
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
        if let Some(idx) = cmd.find("instances\\") {
            let rest = &cmd[idx + "instances\\".len()..];
            let token = rest
                .split(|c: char| c == '/' || c == '\\' || c.is_whitespace() || c == '"')
                .next()
                .unwrap_or("");
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
        // Match --profile <id> or --profile=<id>
        if let Some(idx) = cmd.find("--profile=") {
            let rest = &cmd[idx + "--profile=".len()..];
            let token = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches('"');
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
        if let Some(idx) = cmd.find("--profile ") {
            let rest = &cmd[idx + "--profile ".len()..];
            let token = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches('"');
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }

    // 2. Inspect cloned executable name: e.g. antigravity-worker-1.exe -> worker-1
    let lower_exe = node.executable_name.to_ascii_lowercase();
    let clean_exe = lower_exe.strip_suffix(".exe").unwrap_or(&lower_exe);

    if let Some(inst) = clean_exe.strip_prefix("antigravity-") {
        if !inst.is_empty() && inst != "manager" {
            return Some(inst.to_string());
        }
    }
    if let Some(inst) = clean_exe.strip_prefix("antigravity_") {
        if !inst.is_empty() && inst != "manager" {
            return Some(inst.to_string());
        }
    }

    None
}

/// Ascends the process tree using a provided process table and optional environment variables.
///
/// Designed for deterministic unit testing and runtime process table traversal.
pub fn scan_process_ancestry_with_table(
    start_pid: u32,
    process_table: &[ProcessNode],
    env_vars: Option<&[(&str, &str)]>,
) -> AncestryScanReport {
    // 1. Resolve environment instance override if specified
    let env_instance_id = env_vars.and_then(|vars| {
        vars.iter()
            .find(|(k, v)| {
                (*k == "ANTIGRAVITY_INSTANCE_ID" || *k == "AGM_INSTANCE_ID") && !v.trim().is_empty()
            })
            .map(|(_, v)| v.trim().to_string())
    });

    let env_vscode_pid = env_vars.and_then(|vars| {
        vars.iter()
            .find(|(k, v)| *k == "VSCODE_PID" && !v.trim().is_empty())
            .and_then(|(_, v)| v.trim().parse::<u32>().ok())
    });

    let mut current_pid = start_pid;
    let mut process_chain = Vec::new();
    let mut visited_pids = HashSet::new();
    let mut depth = 0;

    let mut detected_ide_node: Option<ProcessNode> = None;
    let mut has_terminal_ancestor = false;

    while depth < MAX_ANCESTRY_SCAN_DEPTH {
        if current_pid == 0 || visited_pids.contains(&current_pid) {
            break;
        }
        visited_pids.insert(current_pid);
        depth += 1;

        let node = match process_table.iter().find(|p| p.pid == current_pid) {
            Some(n) => n.clone(),
            None => break,
        };

        if node.is_known_terminal() {
            has_terminal_ancestor = true;
        }

        if node.is_known_ide() {
            detected_ide_node = Some(node.clone());
            process_chain.push(node);
            break;
        }

        let ppid = node.ppid;
        process_chain.push(node);

        if ppid == 0 || ppid == current_pid {
            break;
        }
        current_pid = ppid;
    }

    // 2. Determine caller origin
    if let Some(ide_node) = detected_ide_node {
        let instance_id = env_instance_id
            .or_else(|| extract_instance_id_from_node(&ide_node))
            .unwrap_or_else(|| "default".to_string());

        return AncestryScanReport {
            process_chain,
            caller_origin: CallerIdeOrigin::AntigravityIdeTerminal {
                instance_id,
                ide_pid: ide_node.pid,
            },
            is_caller_ide_detected: true,
        };
    }

    // Check if environment indicated an active IDE terminal even if parent traversal was truncated
    if let Some(instance_id) = env_instance_id {
        let ide_pid = env_vscode_pid.unwrap_or(start_pid);
        return AncestryScanReport {
            process_chain,
            caller_origin: CallerIdeOrigin::AntigravityIdeTerminal {
                instance_id,
                ide_pid,
            },
            is_caller_ide_detected: true,
        };
    }

    // Check if terminal was discovered without an IDE root
    let caller_origin = if has_terminal_ancestor || !process_chain.is_empty() {
        CallerIdeOrigin::StandaloneTerminal
    } else {
        CallerIdeOrigin::Unknown
    };

    AncestryScanReport {
        process_chain,
        caller_origin,
        is_caller_ide_detected: false,
    }
}

/// Traverses process tree upwards via PPID, detects Antigravity IDE parent process or IDE instance markers.
pub fn scan_process_ancestry(start_pid: u32) -> AncestryScanReport {
    let effective_pid = if start_pid == 0 {
        std::process::id()
    } else {
        start_pid
    };

    // Collect operating system process table snapshot via sysinfo
    let mut system = sysinfo::System::new();
    let refresh_kind = sysinfo::ProcessRefreshKind::new()
        .with_cmd(sysinfo::UpdateKind::Always)
        .with_exe(sysinfo::UpdateKind::Always);
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, refresh_kind);

    let mut table = Vec::new();
    for (pid, proc) in system.processes() {
        let exe_name = proc
            .exe()
            .and_then(|p| p.file_name())
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| proc.name().to_string_lossy().to_string());

        let cmd = if !proc.cmd().is_empty() {
            Some(
                proc.cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        } else {
            None
        };

        table.push(ProcessNode {
            pid: pid.as_u32(),
            ppid: proc.parent().map(|p| p.as_u32()).unwrap_or(0),
            executable_name: exe_name,
            command_line: cmd,
        });
    }

    // Query active environment variables
    let env_inst = std::env::var("ANTIGRAVITY_INSTANCE_ID")
        .or_else(|_| std::env::var("AGM_INSTANCE_ID"))
        .ok();
    let env_vpid = std::env::var("VSCODE_PID").ok();

    let mut env_pairs = Vec::new();
    if let Some(ref inst) = env_inst {
        env_pairs.push(("ANTIGRAVITY_INSTANCE_ID", inst.as_str()));
    }
    if let Some(ref vpid) = env_vpid {
        env_pairs.push(("VSCODE_PID", vpid.as_str()));
    }

    scan_process_ancestry_with_table(effective_pid, &table, Some(&env_pairs))
}

/// Extracts caller instance ID if caller process ancestry report indicates an Antigravity IDE origin.
pub fn detect_caller_instance_id_from_ancestry(report: &AncestryScanReport) -> Option<String> {
    report.caller_instance_id().map(|s| s.to_string())
}
