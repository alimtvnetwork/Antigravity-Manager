use crate::modules::{account, auto_switcher, config, instance, repo_db};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::*;

#[cfg(target_os = "windows")]
pub(crate) fn attach_parent_console() {
    #[link(name = "Kernel32")]
    extern "system" {
        fn AttachConsole(dw_process_id: u32) -> i32;
    }
    const ATTACH_PARENT_PROCESS: u32 = 0xFFFFFFFF;
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

/// Unified argument extractor for AGM CLI commands
#[derive(Debug, Clone, Default)]
pub struct CliContext {
    pub instance_id: Option<String>,
    pub repo_path: Option<String>,
    pub json_output: bool,
    pub all_instances: bool,
    pub verbose: bool,
    pub dry_run: bool,
    pub model: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub fifo: bool,
    pub positional_args: Vec<String>,
}

impl CliContext {
    pub fn parse(args: &[String]) -> Self {
        let mut ctx = Self::default();
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-i" | "--instance" | "--profile" => {
                    if i + 1 < args.len() {
                        ctx.instance_id = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "-r" | "--repo" | "--workspace" => {
                    if i + 1 < args.len() {
                        ctx.repo_path = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "-m" | "--model" => {
                    if i + 1 < args.len() {
                        ctx.model = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "--prefix" => {
                    if i + 1 < args.len() {
                        ctx.prefix = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "--suffix" => {
                    if i + 1 < args.len() {
                        ctx.suffix = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "--fifo" => {
                    ctx.fifo = true;
                }
                "--json" | "-j" => {
                    ctx.json_output = true;
                }
                "-a" | "--all" => {
                    ctx.all_instances = true;
                }
                "-v" | "--verbose" => {
                    ctx.verbose = true;
                }
                "--dry-run" => {
                    ctx.dry_run = true;
                }
                arg if !arg.starts_with('-') => {
                    ctx.positional_args.push(arg.to_string());
                }
                _ => {}
            }
            i += 1;
        }
        ctx
    }

    /// Resolves canonical instance ID or falls back to active instance.
    /// Checks explicit `--instance` flag first, then first positional argument if it resolves to an instance.
    pub fn resolve_target_instance(&self) -> Result<String, String> {
        if let Some(ref raw_id) = self.instance_id {
            crate::modules::instance::resolve_instance_id(raw_id)
        } else if let Some(first) = self.positional_args.first() {
            if let Ok(resolved) = crate::modules::instance::resolve_instance_id(first) {
                Ok(resolved)
            } else {
                crate::modules::instance::get_active_instance_id()
            }
        } else {
            crate::modules::instance::get_active_instance_id()
        }
    }
}

/// Generic JSON envelope for standard CLI responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliEnvelope<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CliErrorPayload>,
    pub meta: CliMetaPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliMetaPayload {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    pub timestamp: String,
    pub version: String,
}

impl<T: Serialize> CliEnvelope<T> {
    pub fn ok(command: &str, instance_id: Option<String>, data: T) -> Self {
        let now_unix = Utc::now().timestamp();
        Self {
            success: true,
            status: Some("success".to_string()),
            command: Some(command.to_string()),
            instance_id: instance_id.clone(),
            timestamp: Some(now_unix),
            data: Some(data),
            error: None,
            meta: CliMetaPayload {
                command: command.to_string(),
                instance_id,
                timestamp: Utc::now().to_rfc3339(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    pub fn err(command: &str, instance_id: Option<String>, code: &str, message: &str) -> Self {
        let now_unix = Utc::now().timestamp();
        Self {
            success: false,
            status: Some("error".to_string()),
            command: Some(command.to_string()),
            instance_id: instance_id.clone(),
            timestamp: Some(now_unix),
            data: None,
            error: Some(CliErrorPayload {
                code: code.to_string(),
                message: message.to_string(),
            }),
            meta: CliMetaPayload {
                command: command.to_string(),
                instance_id,
                timestamp: Utc::now().to_rfc3339(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    pub fn print_and_exit(&self) -> ! {
        let json_str = serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string());
        println!("{}", json_str);
        if self.success {
            std::process::exit(0);
        } else {
            std::process::exit(1);
        }
    }
}
