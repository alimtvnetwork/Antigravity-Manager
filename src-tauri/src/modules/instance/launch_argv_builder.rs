//! Modular Instance Launch Argv Builder
//! Constructs normalized, security-hardened CLI argument lists for spawning
//! child Antigravity IDE instances with sandbox isolation flags and workspace paths.

use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct LaunchArgvBuilder {
    user_data_dir: Option<String>,
    remote_debugging_port: Option<u16>,
    is_mock_keyring: bool,
    workspace_paths: Vec<String>,
    extra_flags: Vec<String>,
}

impl LaunchArgvBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_user_data_dir(mut self, data_dir: &Path) -> Self {
        self.user_data_dir = Some(data_dir.to_string_lossy().to_string());
        self
    }

    pub fn with_remote_debugging_port(mut self, port: u16) -> Self {
        self.remote_debugging_port = Some(port);
        self
    }

    pub fn with_mock_keyring(mut self, enable: bool) -> Self {
        self.is_mock_keyring = enable;
        self
    }

    pub fn with_workspace(mut self, path: impl Into<String>) -> Self {
        self.workspace_paths.push(path.into());
        self
    }

    pub fn with_extra_flag(mut self, flag: impl Into<String>) -> Self {
        self.extra_flags.push(flag.into());
        self
    }

    /// Assembles the complete argument list into a vector of strings.
    pub fn build(self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(data_dir) = self.user_data_dir {
            args.push(format!("--user-data-dir={}", data_dir));
        }

        if let Some(port) = self.remote_debugging_port {
            args.push(format!("--remote-debugging-port={}", port));
        }

        if self.is_mock_keyring {
            args.push("--password-store=basic".to_string());
            args.push("--use-mock-keychain".to_string());
        }

        args.extend(self.extra_flags);
        args.extend(self.workspace_paths);

        args
    }
}
