//! Data models and target representations for the multi-IDE installer subsystem.
//!
//! Conforms to `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/03-migration-and-verification-spec.md`.

use serde::{Deserialize, Serialize};

/// Supported IDE targets managed by the installer engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdeTarget {
    Antigravity,
    Cursor,
    VsCode,
}

impl IdeTarget {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().trim() {
            "antigravity" | "agy" | "google-antigravity" | "google_antigravity" => {
                Some(Self::Antigravity)
            }
            "cursor" | "cursor-ai" | "cursor_ai" => Some(Self::Cursor),
            "vscode" | "code" | "vs-code" | "vs_code" => Some(Self::VsCode),
            _ => None,
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Self::Antigravity => "antigravity",
            Self::Cursor => "cursor",
            Self::VsCode => "vscode",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Antigravity => "Google Antigravity IDE",
            Self::Cursor => "Cursor AI",
            Self::VsCode => "Visual Studio Code",
        }
    }
}

/// Options controlling installation execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallOptions {
    pub dry_run: bool,
    pub verbose: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            verbose: false,
        }
    }
}

/// Execution outcome returned upon installation dispatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub target: IdeTarget,
    pub success: bool,
    pub installed_path: Option<String>,
    pub command_executed: String,
    pub output: String,
}

/// Detailed installation discovery report for an IDE target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeStatusInfo {
    pub target: IdeTarget,
    pub display_name: String,
    pub is_installed: bool,
    pub version: Option<String>,
    pub executable_path: Option<String>,
}
