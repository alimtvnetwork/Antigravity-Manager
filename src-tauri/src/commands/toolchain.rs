//! Toolchain installer backend (plan 150).
//!
//! Exposes the `scripts/install-rust-toolchain.sh` installer to the UI:
//! item inventory with live detection, read-only `check` reporting, and
//! `install` with live progress streaming over the `toolchain-progress` event.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use std::path::PathBuf;
use tauri::{Emitter, Manager};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// One installable toolchain item shown in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub installed: bool,
}

/// One line of the `check` report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckItem {
    pub item: String,
    pub ok: bool,
    pub hint: String,
}

/// Structured result of `toolchain_check`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckReport {
    pub items: Vec<CheckItem>,
    pub success: bool,
}

/// Install request from the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRequest {
    pub items: Vec<String>,
    pub profiles: Vec<String>,
    pub force: bool,
    pub quiet: bool,
    pub skip_native_deps: bool,
    pub ssh_target: Option<String>,
}

/// Progress event payload (`toolchain-progress`).
#[derive(Debug, Clone, Serialize)]
struct ToolchainProgressEvent {
    line: String,
    done: bool,
}

// ---------------------------------------------------------------------------
// Item catalog + detection
// ---------------------------------------------------------------------------

struct ItemDef {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    /// Binaries probed via PATH; installed if ANY is found (except rust).
    bins: &'static [&'static str],
}

const ITEM_CATALOG: &[ItemDef] = &[
    ItemDef {
        id: "rust",
        name: "Rust toolchain",
        description: "rustc + cargo with clippy and rustfmt components",
        bins: &["rustc", "cargo"],
    },
    ItemDef {
        id: "nodejs",
        name: "Node.js",
        description: "JavaScript runtime for the Tauri frontend",
        bins: &["node"],
    },
    ItemDef {
        id: "sccache",
        name: "sccache",
        description: "Shared compilation cache (faster Rust rebuilds)",
        bins: &["sccache"],
    },
    ItemDef {
        id: "cargo-watch",
        name: "cargo-watch",
        description: "Rebuild on file change (Rust dev loop)",
        bins: &["cargo-watch"],
    },
    ItemDef {
        id: "tauri-cli",
        name: "Tauri CLI",
        description: "tauri dev / tauri build commands",
        bins: &["cargo-tauri", "tauri"],
    },
    ItemDef {
        id: "gh",
        name: "GitHub CLI",
        description: "gh command-line for GitHub workflows",
        bins: &["gh"],
    },
    ItemDef {
        id: "pnpm",
        name: "pnpm",
        description: "Fast Node.js package manager",
        bins: &["pnpm"],
    },
];

/// PATH probe for a single binary name (catalog-controlled, no user input).
fn command_exists(name: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", name))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// rustup component probe (clippy / rustfmt are project pre-flight gates).
fn rustup_has_component(component: &str) -> bool {
    let out = std::process::Command::new("rustup")
        .args(["component", "list", "--installed"])
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).contains(component),
        _ => false,
    }
}

fn rust_toolchain_installed() -> bool {
    command_exists("rustc")
        && command_exists("cargo")
        && command_exists("rustup")
        && rustup_has_component("clippy")
        && rustup_has_component("rustfmt")
}

#[tauri::command]
pub async fn toolchain_list_items() -> AppResult<Vec<ToolchainItem>> {
    let items = ITEM_CATALOG
        .iter()
        .map(|def| {
            let installed = if def.id == "rust" {
                rust_toolchain_installed()
            } else {
                def.bins.iter().any(|b| command_exists(b))
            };
            ToolchainItem {
                id: def.id.to_string(),
                name: def.name.to_string(),
                description: def.description.to_string(),
                installed,
            }
        })
        .collect();
    Ok(items)
}

// ---------------------------------------------------------------------------
// Script resolution + check parsing
// ---------------------------------------------------------------------------

/// Locate `scripts/install-rust-toolchain.sh`:
/// bundled resources first, then dev-tree candidates.
fn resolve_script_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(res) = app.path().resource_dir() {
        candidates.push(res.join("scripts/install-rust-toolchain.sh"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("scripts/install-rust-toolchain.sh"));
        if let Some(parent) = cwd.parent() {
            candidates.push(parent.join("scripts/install-rust-toolchain.sh"));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // dev layout: src-tauri/target/debug -> repo root is ../../..
            candidates.push(dir.join("../../../scripts/install-rust-toolchain.sh"));
        }
    }
    candidates.into_iter().find(|p| p.is_file())
}

/// Strip ANSI color codes so parsing works even on colorized output.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // consume until the terminating letter (e.g. 'm')
            for nc in chars.by_ref() {
                if nc.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Parse one `check` output line: `[OK] <item>` / `[MISSING] <item> (hint)`.
fn parse_check_line(line: &str) -> Option<CheckItem> {
    let line = strip_ansi(line);
    let (ok, rest) = if let Some(r) = line.strip_prefix("[OK]") {
        (true, r)
    } else if let Some(r) = line.strip_prefix("[MISSING]") {
        (false, r)
    } else {
        return None;
    };
    let rest = rest.trim();
    let (item, hint) = match rest.rfind(" (") {
        Some(i) if rest.ends_with(')') => (
            rest[..i].trim().to_string(),
            rest[i + 2..rest.len() - 1].trim().to_string(),
        ),
        _ => (rest.to_string(), String::new()),
    };
    if item.is_empty() {
        return None;
    }
    Some(CheckItem { item, ok, hint })
}

#[tauri::command]
pub async fn toolchain_check(app: tauri::AppHandle) -> AppResult<CheckReport> {
    let script = resolve_script_path(&app)
        .ok_or_else(|| AppError::Config("install-rust-toolchain.sh not found".to_string()))?;

    let output = std::process::Command::new("sh")
        .arg(&script)
        .arg("check")
        .output()
        .map_err(|e| AppError::Process(format!("failed to run toolchain check: {}", e)))?;

    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let items: Vec<CheckItem> = combined.lines().filter_map(parse_check_line).collect();
    let success = output.status.success();
    Ok(CheckReport { items, success })
}

// ---------------------------------------------------------------------------
// Install (streaming)
// ---------------------------------------------------------------------------

/// Validate `user@host`: exactly one '@', non-empty parts, no whitespace or
/// shell metacharacters. Keys must be pre-configured; never passwords.
fn valid_ssh_target(s: &str) -> bool {
    let mut parts = s.split('@');
    let (user, host) = match (parts.next(), parts.next(), parts.next()) {
        (Some(u), Some(h), None) => (u, h),
        _ => return false,
    };
    if user.is_empty() || host.is_empty() {
        return false;
    }
    let forbidden = |c: char| {
        c.is_whitespace()
            || matches!(
                c,
                '/' | '\\'
                    | ';'
                    | '&'
                    | '|'
                    | '`'
                    | '$'
                    | '('
                    | ')'
                    | '<'
                    | '>'
                    | '"'
                    | '\''
                    | '!'
                    | '*'
                    | '~'
            )
    };
    !s.chars().any(forbidden)
}

fn emit_progress(app: &tauri::AppHandle, line: String, done: bool) {
    let _ = app.emit("toolchain-progress", ToolchainProgressEvent { line, done });
}

#[tauri::command]
pub async fn toolchain_install(app: tauri::AppHandle, req: InstallRequest) -> AppResult<bool> {
    if let Some(ref target) = req.ssh_target {
        if !valid_ssh_target(target) {
            return Err(AppError::Config(format!(
                "Invalid SSH target '{}': expected user@host",
                target
            )));
        }
    }

    let script = resolve_script_path(&app)
        .ok_or_else(|| AppError::Config("install-rust-toolchain.sh not found".to_string()))?;

    // argv contract (script v3): install [--profile P]... [--item I]...
    //   [--force] [--quiet] [--skip-native-deps] [--ssh user@host]
    let mut args: Vec<String> = vec!["install".to_string()];
    for profile in &req.profiles {
        let p = profile.trim();
        if !p.is_empty() {
            args.push("--profile".to_string());
            args.push(p.to_string());
        }
    }
    for item in &req.items {
        let item = item.trim();
        if !item.is_empty() {
            args.push("--item".to_string());
            args.push(item.to_string());
        }
    }
    if req.force {
        args.push("--force".to_string());
    }
    if req.quiet {
        args.push("--quiet".to_string());
    }
    if req.skip_native_deps {
        args.push("--skip-native-deps".to_string());
    }
    if let Some(target) = req.ssh_target {
        args.push("--ssh".to_string());
        args.push(target);
    }

    let mut child = std::process::Command::new("sh")
        .arg(&script)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Process(format!("failed to spawn installer: {}", e)))?;

    // Stream stdout lines as progress events (blocking reader on a worker thread).
    let stdout = child.stdout.take();
    let app_out = app.clone();
    let out_task = tokio::task::spawn_blocking(move || {
        if let Some(out) = stdout {
            let reader = std::io::BufReader::new(out);
            for line in reader.lines() {
                match line {
                    Ok(l) => emit_progress(&app_out, l, false),
                    Err(_) => break,
                }
            }
        }
    });

    // Drain stderr concurrently so a full pipe can never deadlock the child.
    let stderr = child.stderr.take();
    let app_err = app.clone();
    let err_task = tokio::task::spawn_blocking(move || {
        if let Some(err) = stderr {
            let reader = std::io::BufReader::new(err);
            for line in reader.lines() {
                if let Ok(l) = line {
                    emit_progress(&app_err, l, false);
                } else {
                    break;
                }
            }
        }
    });

    let status = child
        .wait()
        .map_err(|e| AppError::Process(format!("installer wait failed: {}", e)))?;
    let _ = out_task.await;
    let _ = err_task.await;

    let success = status.success();
    emit_progress(&app, String::new(), true);
    Ok(success)
}
