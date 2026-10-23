//! Environment Variable Whitelist Filter.
//! Filters out leaking parent process variables (e.g. VS Code IPC variables, Electron debug hooks,
//! Node options) when spawning sandboxed child Antigravity IDE processes.

use std::collections::HashSet;

/// Blacklisted environment variable prefixes and exact names that cause child process leakage.
pub const BLACKLISTED_ENV_PREFIXES: &[&str] = &["VSCODE_", "ELECTRON_", "CHROME_", "ORIGINAL_XDG_"];

pub const BLACKLISTED_EXACT_ENVS: &[&str] = &[
    "NODE_OPTIONS",
    "ELECTRON_RUN_AS_NODE",
    "DEBUG",
    "NODE_PATH",
    "TAURI_ENV_DEBUG",
];

/// Evaluates if an environment variable key is safe to propagate to child instances.
pub fn is_safe_instance_env_var(key: &str) -> bool {
    let upper = key.to_uppercase();

    for exact in BLACKLISTED_EXACT_ENVS {
        if upper == *exact {
            return false;
        }
    }

    for prefix in BLACKLISTED_ENV_PREFIXES {
        if upper.starts_with(prefix) {
            return false;
        }
    }

    true
}

/// Filters an iterator of (key, value) environment pairs into a sanitized collection.
pub fn filter_child_environment<'a, I>(env_vars: I) -> Vec<(String, String)>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let mut safe_envs = Vec::new();
    let mut seen = HashSet::new();

    for (k, v) in env_vars {
        if is_safe_instance_env_var(k) && seen.insert(k.to_string()) {
            safe_envs.push((k.to_string(), v.to_string()));
        }
    }

    safe_envs
}
