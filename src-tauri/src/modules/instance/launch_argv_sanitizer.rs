//! IDE Launch Argv Security Sanitizer
//!
//! Sanitizes command-line arguments passed to newly spawned IDE instances,
//! stripping hazardous flags (`--inspect`, `--inspect-brk`, `--no-sandbox`,
//! `--remote-debugging-address=0.0.0.0`) to prevent remote code execution and sandbox escapes.

use tracing::{debug, warn};

/// Dangerous argument prefixes that should be stripped unconditionally.
pub const DISALLOWED_ARGV_PREFIXES: &[&str] = &[
    "--inspect",
    "--inspect-brk",
    "--no-sandbox",
    "--disable-web-security",
    "--remote-debugging-address=0.0.0.0",
];

/// Result of argv sanitization.
#[derive(Debug, Clone)]
pub struct SanitizedArgv {
    pub clean_args: Vec<String>,
    pub stripped_args: Vec<String>,
}

/// Sanitizes a vector of command line arguments for spawning an IDE instance.
pub fn sanitize_launch_argv(raw_args: &[String]) -> SanitizedArgv {
    let mut clean_args = Vec::with_capacity(raw_args.len());
    let mut stripped_args = Vec::new();

    for arg in raw_args {
        let is_disallowed = DISALLOWED_ARGV_PREFIXES
            .iter()
            .any(|disallowed| arg == *disallowed || arg.starts_with(&format!("{}=", disallowed)));

        if is_disallowed {
            warn!(arg = %arg, "Stripped unsafe argument from IDE launch command");
            stripped_args.push(arg.clone());
        } else {
            clean_args.push(arg.clone());
        }
    }

    debug!(
        total = raw_args.len(),
        clean = clean_args.len(),
        stripped = stripped_args.len(),
        "Sanitized IDE launch arguments"
    );

    SanitizedArgv {
        clean_args,
        stripped_args,
    }
}
