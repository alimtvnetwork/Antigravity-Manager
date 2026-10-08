# Subtask 03 Spec: AGM CLI 46-Command Expansion & Architecture

**Document ID:** `.ai-memory/plans/subtasks/144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement/03-agm-cli-30-to-40-commands-expansion-spec.md`  
**Parent Task:** `144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement`  
**Target Codebase:** `src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/cli.rs`  
**Author:** Author 02 (Antigravity Backend & CLI Architecture Specialist)  
**Status:** Approved Engineering Specification  

---

## 1. Scope & Objective

The objective of Subtask 03 is to formalize, expand, and structure the AGM CLI binary into a unified 46-command suite organized across 7 functional domains:
1. `instances` (list, status, start, stop, restart, switch, copy, delete, logs)
2. `prompts` (list, tree, inspect, send, running, queue, history, export, backup, restore, purge)
3. `quota` / `accounts` (quota, refresh, accounts, add, remove, validate, balance)
4. `fleet` (nodes, ping, sync, broadcast, health)
5. `proxy` (status, pool, model, config, restart, cache-clear)
6. `security` (ip-list, ip-block, ip-unblock, audit)
7. `system` (version, env, doctor, db-stats, vacuum)

This specification defines:
- The command dispatcher architecture and flag parsing in `src-tauri/src/bin/agm.rs`.
- The shared CLI argument processing in `src-tauri/src/modules/cli.rs`.
- Standard `--json` response envelope formatting via generic Rust structs.
- The dual execution model: **Direct SQLite Database Access** vs. **REST Forwarding** to the local runtime daemon.

---

## 2. Command Dispatch Architecture (`agm.rs` & `cli.rs`)

### 2.1 Two-Tier Dispatch Model

Currently, `agm.rs` matches commands primarily through a flat `match subcommand.as_str()` block. The expansion introduces a hierarchical two-tier dispatcher while maintaining 100% backward compatibility with existing shorthand aliases.

```
                  ┌─────────────────────────────────────────┐
                  │          Command Line Input             │
                  │   agm [domain] [subcommand] [flags]     │
                  └────────────────────┬────────────────────┘
                                       │
                                       ▼
                  ┌─────────────────────────────────────────┐
                  │    First-Tier Token Normalizer          │
                  │  (Strip leading '/', '-', to_lowercase) │
                  └────────────────────┬────────────────────┘
                                       │
              ┌────────────────────────┴────────────────────────┐
              ▼                                                 ▼
   [Domain Prefix Detected]                       [Legacy Shorthand Alias]
   e.g. 'instances', 'prompts',                   e.g. 'wpr' -> prompts::running
        'quota', 'proxy', etc.                         'brp' -> prompts::backup
              │                                        'rrp' -> prompts::restore
              ▼                                                 │
   ┌──────────────────────┐                                     │
   │ Domain Sub-Router    │                                     │
   │ e.g. dispatch_prompt │◄────────────────────────────────────┘
   └──────────┬───────────┘
              │
              ▼
   ┌────────────────────────────────────────────────────────────┐
   │ Execution Layer: Direct SQLite Engine OR REST Forwarder   │
   └────────────────────────────────────────────────────────────┘
```

### 2.2 Domain Sub-Routers in `src-tauri/src/bin/agm.rs`

The following domain handler entry points are declared and wired into `fn main()`:

```rust
// In src-tauri/src/bin/agm.rs

fn dispatch_instance_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "list" | "ls" => cmd_instances_list(args),
        "status" => cmd_instances_status(args),
        "start" | "launch" => cmd_instances_start(args),
        "stop" | "kill" => cmd_instances_stop(args),
        "restart" => cmd_instances_restart(args),
        "switch" | "use" => cmd_instances_switch(args),
        "copy" | "clone" | "duplicate" => cmd_instances_copy(args),
        "delete" | "rm" => cmd_instances_delete(args),
        "logs" => cmd_instances_logs(args),
        _ => handle_unknown_domain_command("instances", subcommand, args),
    }
}

fn dispatch_prompt_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "list" | "ls" => cmd_prompts_list(args),
        "tree" => cmd_prompts_tree(args),
        "inspect" | "observe" => cmd_prompts_inspect(args),
        "send" | "dispatch" => cmd_prompts_send(args),
        "running" | "wpr" => cmd_prompts_running(args),
        "queue" => cmd_prompts_queue(args),
        "history" => cmd_prompts_history(args),
        "export" => cmd_prompts_export(args),
        "backup" | "brp" => cmd_prompts_backup(args),
        "restore" | "rrp" => cmd_prompts_restore(args),
        "purge" | "clean" => cmd_prompts_purge(args),
        _ => handle_unknown_domain_command("prompts", subcommand, args),
    }
}

fn dispatch_quota_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "show" | "status" => cmd_quota_show(args),
        "refresh" => cmd_quota_refresh(args),
        "list" => cmd_accounts_list(args),
        "add" => cmd_accounts_add(args),
        "remove" | "rm" => cmd_accounts_remove(args),
        "validate" | "check" => cmd_accounts_validate(args),
        "balance" => cmd_accounts_balance(args),
        _ => handle_unknown_domain_command("quota", subcommand, args),
    }
}

fn dispatch_fleet_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "nodes" => cmd_fleet_nodes(args),
        "ping" => cmd_fleet_ping(args),
        "sync" => cmd_fleet_sync(args),
        "broadcast" => cmd_fleet_broadcast(args),
        "health" => cmd_fleet_health(args),
        _ => handle_unknown_domain_command("fleet", subcommand, args),
    }
}

fn dispatch_proxy_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "status" => cmd_proxy_status(args),
        "pool" => cmd_proxy_pool(args),
        "model" => cmd_proxy_model(args),
        "config" => cmd_proxy_config(args),
        "restart" => cmd_proxy_restart(args),
        "cache-clear" => cmd_proxy_cache_clear(args),
        _ => handle_unknown_domain_command("proxy", subcommand, args),
    }
}

fn dispatch_security_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "ip-list" => cmd_security_ip_list(args),
        "ip-block" => cmd_security_ip_block(args),
        "ip-unblock" => cmd_security_ip_unblock(args),
        "audit" => cmd_security_audit(args),
        _ => handle_unknown_domain_command("security", subcommand, args),
    }
}

fn dispatch_system_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "version" => cmd_system_version(args),
        "env" => cmd_system_env(args),
        "doctor" => cmd_system_doctor(args),
        "db-stats" => cmd_system_db_stats(args),
        "vacuum" => cmd_system_vacuum(args),
        _ => handle_unknown_domain_command("system", subcommand, args),
    }
}
```

---

## 3. Flag Parsing & Shared Parameter Resolution

### 3.1 Unified Argument Extractor

To avoid boilerplate and fragile index slicing across all 46 commands, a standard `CliContext` helper struct is implemented in `src-tauri/src/modules/cli.rs`:

```rust
// In src-tauri/src/modules/cli.rs

#[derive(Debug, Clone, Default)]
pub struct CliContext {
    pub instance_id: Option<String>,
    pub repo_path: Option<String>,
    pub json_output: bool,
    pub all_instances: bool,
    pub verbose: bool,
    pub dry_run: bool,
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

    /// Resolves canonical instance ID or falls back to active instance
    pub fn resolve_target_instance(&self) -> Result<String, String> {
        if let Some(ref raw_id) = self.instance_id {
            crate::modules::instance::resolve_instance_id(raw_id)
        } else {
            crate::modules::instance::get_active_instance_id()
        }
    }
}
```

---

## 4. Generic `--json` Envelope Struct

Every command invoked with `--json` outputs a serialized instance of `CliEnvelope<T>` to `stdout`:

```rust
// In src-tauri/src/modules/cli.rs

use serde::Serialize;
use chrono::Utc;

#[derive(Serialize)]
pub struct CliEnvelope<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CliErrorPayload>,
    pub meta: CliMetaPayload,
}

#[derive(Serialize)]
pub struct CliErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct CliMetaPayload {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    pub timestamp: String,
    pub version: String,
}

impl<T: Serialize> CliEnvelope<T> {
    pub fn ok(command: &str, instance_id: Option<String>, data: T) -> Self {
        Self {
            success: true,
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
        Self {
            success: false,
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
```

---

## 5. Execution Routing Strategy: Direct SQLite vs. REST Forwarding

To ensure the CLI remains fast, responsive, and resilient:

```
                      CLI Command Execution
                               │
               Requires Live Runtime Daemon?
                  ├── NO (e.g. list, inspect, tree, stats, vacuum)
                  │     └───► Direct SQLite Engine (read-only SQLite mode)
                  └── YES (e.g. interrupt in-flight turn, proxy config reload)
                        └───► Check if Daemon Alive (port 8045)
                               ├── YES: Authenticated REST HTTP Forward
                               └── NO : Graceful Standalone Local Handler
```

### 5.1 Direct SQLite Engine Access

Read-only commands (`agm instances list`, `agm prompts tree`, `agm prompts history`, `agm system db-stats`) open database files directly using SQLite URI read-only flags:
```rust
let conn = rusqlite::Connection::open_with_flags(
    &db_path,
    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
)?;
```
**Benefits:**
- 0ms network latency.
- Runs when proxy server or GUI is completely stopped.
- Guaranteed zero lock contention against active writers.

### 5.2 REST Forwarding Helper

When runtime state synchronization is mandatory (e.g. live prompt dispatch or proxy cache invalidation) and the daemon is detected on port 8045:
```rust
pub fn forward_to_local_rest<T: serde::de::DeserializeOwned>(
    method: reqwest::Method,
    path: &str,
    body: Option<serde_json::Value>,
) -> Result<T, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let admin_token = std::env::var("AGM_ADMIN_TOKEN")
        .or_else(|_| std::env::var("ABV_API_KEY"))
        .unwrap_or_default();

    let url = format!("http://127.0.0.1:8045/api{}", path);
    let mut req = client.request(method, &url);
    if !admin_token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", admin_token));
    }
    if let Some(json_payload) = body {
        req = req.json(&json_payload);
    }

    let resp = req.send().map_err(|e| format!("Daemon communication failed: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("Daemon returned HTTP status {}", resp.status()));
    }

    resp.json::<T>().map_err(|e| format!("Failed to parse response: {}", e))
}
```

---

## 6. Implementation Checklist & Verification Gates

- [ ] **Step 3.1:** Implement `CliContext` and `CliEnvelope<T>` in `src-tauri/src/modules/cli.rs`.
- [ ] **Step 3.2:** Refactor `src-tauri/src/bin/agm.rs` `fn main()` to route domain prefixes (`instances`, `prompts`, `quota`, `fleet`, `proxy`, `security`, `system`).
- [ ] **Step 3.3:** Implement all 46 domain subcommand handler functions.
- [ ] **Step 3.4:** Ensure backward compatibility aliases (`wpr`, `brp`, `rrp`, `pe`, `tif`, `sug`, `fpug`) delegate seamlessly to the domain handlers.
- [ ] **Step 3.5:** Validate `--json` formatting across all 7 domains using unit tests and pre-flight checks:
  - `cd src-tauri && cargo fmt -- --check`
  - `cd src-tauri && cargo clippy --all-targets --all-features`
  - Targeted unit tests: `cargo test --bin agm`
