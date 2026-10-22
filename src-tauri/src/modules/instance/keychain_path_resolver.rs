//! Platform-Agnostic Sandboxed Keychain Path Resolver
//!
//! Resolves sandboxed mock keyring file paths and command-line flags across Windows, Linux,
//! and macOS runtimes to ensure instance profiles do not collide with or access the host OS keyring.

use std::path::{Path, PathBuf};
use tracing::debug;

/// Platform-specific sandboxed keyring flags and configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxedKeychainProfile {
    pub flags: Vec<String>,
    pub mock_keyring_path: PathBuf,
    pub platform: &'static str,
}

/// Resolves the sandboxed mock keyring profile for a given instance data directory.
pub fn resolve_sandboxed_keychain(instance_dir: &Path) -> SandboxedKeychainProfile {
    let mock_path = instance_dir.join("antigravity-mock-keychain.json");

    #[cfg(target_os = "linux")]
    {
        debug!(instance_dir = ?instance_dir, "Configured Linux basic password store");
        SandboxedKeychainProfile {
            flags: vec![
                "--password-store=basic".to_string(),
                format!("--mock-keychain-path={}", mock_path.display()),
            ],
            mock_keyring_path: mock_path,
            platform: "linux",
        }
    }

    #[cfg(target_os = "macos")]
    {
        debug!(instance_dir = ?instance_dir, "Configured macOS mock keychain");
        SandboxedKeychainProfile {
            flags: vec![
                "--use-mock-keychain".to_string(),
                format!("--mock-keychain-path={}", mock_path.display()),
            ],
            mock_keyring_path: mock_path,
            platform: "macos",
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        debug!(instance_dir = ?instance_dir, "Configured Windows sandboxed credential locker");
        SandboxedKeychainProfile {
            flags: vec![
                "--disable-features=Win32VaultStorage".to_string(),
                format!("--mock-keychain-path={}", mock_path.display()),
            ],
            mock_keyring_path: mock_path,
            platform: "windows",
        }
    }
}
