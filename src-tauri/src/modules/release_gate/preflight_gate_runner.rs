//! Pre-flight Checks Gate Runner.
//!
//! Enforces CI and local pre-flight gates per AGENTS.md:
//! - Rust edits (`src-tauri/`):
//!   - `cd src-tauri && cargo fmt -- --check`
//!   - `cd src-tauri && cargo clippy --all-targets --all-features`
//! - Frontend edits (`src/` or frontend configs):
//!   - `npm run build`
//! - Trivial edits (constants, prompts, specs, or config tweaks): skip compile checks.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PreflightRequirements {
    pub requires_rust_fmt: bool,
    pub requires_cargo_clippy: bool,
    pub requires_frontend_build: bool,
    pub is_trivial_only: bool,
    pub planned_commands: Vec<String>,
}

/// Classifies a list of repository-relative changed paths into required preflight gates.
pub fn classify_preflight_requirements(changed_paths: &[&str]) -> PreflightRequirements {
    let mut requires_rust = false;
    let mut requires_frontend = false;
    let mut has_non_trivial = false;

    for &path in changed_paths {
        let p = path.trim().replace('\\', "/");

        if p.starts_with("src-tauri/") {
            requires_rust = true;
            has_non_trivial = true;
        } else if p.starts_with("src/")
            || p == "package.json"
            || p == "package-lock.json"
            || p == "vite.config.ts"
            || p == "tailwind.config.js"
            || p == "tsconfig.json"
        {
            requires_frontend = true;
            has_non_trivial = true;
        } else if p.starts_with("02-spec/")
            || p.starts_with(".ai-memory/")
            || p.starts_with("01-prompts/")
            || p.starts_with("docs/")
            || p.ends_with(".md")
            || p.ends_with(".txt")
        {
            // Documentation or spec change - trivial with respect to compile gates
        } else {
            // Unknown file or root config, treat conservatively as non-trivial
            has_non_trivial = true;
        }
    }

    let mut planned_commands = Vec::new();
    if requires_rust {
        planned_commands.push("cd src-tauri && cargo fmt -- --check".to_string());
        planned_commands.push("cd src-tauri && cargo clippy --all-targets --all-features".to_string());
    }
    if requires_frontend {
        planned_commands.push("npm run build".to_string());
    }

    PreflightRequirements {
        requires_rust_fmt: requires_rust,
        requires_cargo_clippy: requires_rust,
        requires_frontend_build: requires_frontend,
        is_trivial_only: !has_non_trivial,
        planned_commands,
    }
}
