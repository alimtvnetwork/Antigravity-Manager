//! Launch Environment Variable Sanitizer
//! Strips dangerous and contaminating environment variables (NODE_OPTIONS, ELECTRON_RUN_AS_NODE)
//! before spawning child Antigravity IDE instance processes.

use std::collections::HashMap;

pub struct LaunchEnvSanitizer;

impl LaunchEnvSanitizer {
    /// List of environment variable keys that should be stripped to ensure child process isolation.
    pub const STRIPPED_VARS: &'static [&'static str] = &[
        "NODE_OPTIONS",
        "ELECTRON_RUN_AS_NODE",
        "VSCODE_CWD",
        "VSCODE_IPC_HOOK",
        "VSCODE_NLS_CONFIG",
        "VSCODE_PORTABLE",
        "ELECTRON_NO_ASAR",
    ];

    /// Filters an environment map, removing all contaminating keys.
    pub fn sanitize_env_map(env: &mut HashMap<String, String>) {
        for &var in Self::STRIPPED_VARS {
            env.remove(var);
        }
    }

    /// Applies environment exclusions directly to a std::process::Command.
    pub fn apply_to_command(cmd: &mut std::process::Command) {
        for &var in Self::STRIPPED_VARS {
            cmd.env_remove(var);
        }
    }
}
