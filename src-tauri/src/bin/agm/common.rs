//! common — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::env;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn handle_unknown_domain_command(domain: &str, subcommand: &str, _args: &[String]) {
    eprintln!(
        "[ERROR] Unknown subcommand '{}' for domain '{}'. Run 'agm {} --help' for available commands.",
        subcommand, domain, domain
    );
    std::process::exit(1);
}

pub(crate) fn is_prompt_subcommand(sub: &str) -> bool {
    let s = sub
        .trim_start_matches('/')
        .trim_start_matches('-')
        .to_lowercase();
    matches!(
        s.as_str(),
        "list"
            | "ls"
            | "tree"
            | "inspect"
            | "observe"
            | "send"
            | "dispatch"
            | "running"
            | "wpr"
            | "queue"
            | "history"
            | "export"
            | "backup"
            | "brp"
            | "restore"
            | "rrp"
            | "purge"
            | "clean"
    )
}

pub(crate) fn derive_current_repo_slug() -> String {
    env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .map(|s| {
            s.to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>()
                .trim_matches('-')
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "workspace".to_string())
}

pub(crate) fn truncate_words(text: &str, max_words: usize) -> (String, usize) {
    let words: Vec<&str> = text.split_whitespace().collect();
    let total = words.len();
    if total <= max_words {
        (words.join(" "), total)
    } else {
        (format!("{} ...", words[..max_words].join(" ")), total)
    }
}

pub(crate) fn parse_duration_to_seconds(arg: &str, default_sec: u64) -> u64 {
    let s = arg.trim().to_lowercase();
    if s.ends_with('m') {
        s.trim_end_matches('m')
            .parse::<u64>()
            .map(|m| m * 60)
            .unwrap_or(default_sec)
    } else if s.ends_with('s') {
        s.trim_end_matches('s')
            .parse::<u64>()
            .unwrap_or(default_sec)
    } else if s.ends_with('h') {
        s.trim_end_matches('h')
            .parse::<u64>()
            .map(|h| h * 3600)
            .unwrap_or(default_sec)
    } else {
        s.parse::<u64>().unwrap_or(default_sec)
    }
}

pub(crate) fn is_safe_to_delete(path: &Path) -> bool {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    if name.contains("vault")
        || name.contains("account")
        || name.contains(".db")
        || name.contains("config")
    {
        return false;
    }
    true
}

pub(crate) fn check_is_in_path() -> bool {
    let path_var = env::var("PATH").unwrap_or_default();
    let separator = if cfg!(windows) { ';' } else { ':' };
    let current_exe_name = if cfg!(windows) { "agm.exe" } else { "agm" };

    for dir in path_var.split(separator) {
        let p = Path::new(dir).join(current_exe_name);
        if p.exists() {
            return true;
        }
    }
    false
}

pub(crate) fn read_password_masked() -> String {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "$p = Read-Host -Prompt '' -AsSecureString; \
                 [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($p))",
            ])
            .output();

        if let Ok(out) = output {
            let res = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return res;
        }
    }

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input.trim().to_string()
}

pub(crate) fn resolve_switch_filename(custom_path: Option<&str>, node_alias: &str) -> String {
    if let Some(p) = custom_path {
        let trimmed = p.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    let raw_alias = if !node_alias.trim().is_empty() {
        node_alias.trim()
    } else {
        "node"
    };
    let clean_alias: String = raw_alias
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("agm-{}-switch.json", clean_alias)
}

pub(crate) fn strsim_levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let (m, n) = (a_chars.len(), b_chars.len());
    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[m][n]
}

pub(crate) fn suggest_agm_commands(input: &str) -> Vec<String> {
    let known_commands = [
        "accounts",
        "email",
        "telegram",
        "instances",
        "supabase",
        "config",
        "ssh",
        "sj",
        "se",
        "proxy",
        "doctor",
        "version",
        "help",
        "which-format",
        "clear-terminal",
        "clean",
        "prune",
        "clear-cache",
        "clear",
        "failed-commands",
        "fc",
        "install",
        "nodes",
        "deploy-keys",
        "add-key",
        "gitignore",
        "gitignore-agm",
    ];

    let low = input.to_lowercase();
    let mut scored: Vec<(usize, &str)> = Vec::new();

    for &cmd in &known_commands {
        let cmd_low = cmd.to_lowercase();
        if cmd_low == low {
            return vec![cmd.to_string()];
        }
        if cmd_low.starts_with(&low) || low.starts_with(&cmd_low) {
            scored.push((1, cmd));
            continue;
        }
        if cmd_low.contains(&low) || low.contains(&cmd_low) {
            scored.push((2, cmd));
            continue;
        }
        let dist = strsim_levenshtein(&low, &cmd_low);
        if dist <= 2 || (low.len() > 4 && dist <= 3) {
            scored.push((10 + dist, cmd));
        }
    }

    scored.sort_by_key(|&(score, cmd)| (score, cmd.len()));
    scored
        .into_iter()
        .map(|(_, cmd)| cmd.to_string())
        .take(4)
        .collect()
}

pub(crate) fn handle_unknown_command(cmd: &str, full_args: &[String]) {
    let suggestions = suggest_agm_commands(cmd);
    let full_str = full_args.join(" ");
    let msg = format!("Unknown command: 'agm {}'", cmd);
    let _ = repo_db::log_failed_command(cmd, &full_str, "root", "E1001", &msg, &suggestions);

    eprintln!("\n❌ Unknown command: 'agm {}'", cmd);
    if !suggestions.is_empty() {
        eprintln!("\n  💡 It is not there, but here is a suggestion you can try:");
        for s in &suggestions {
            eprintln!("    • agm {}", s);
        }
    }
    eprintln!("\n  Run 'agm help' for available commands.");
    eprintln!(
        "  Run 'agm failed-commands' (or 'agm fc') to view failed command history & suggestions.\n"
    );
    std::process::exit(1);
}

pub(crate) fn extract_immediate_and_weekly_credits(
    acc: &antigravity_tools_lib::models::Account,
) -> (f64, f64, String) {
    let tier = acc
        .quota
        .as_ref()
        .and_then(|q| q.subscription_tier.clone())
        .unwrap_or_else(|| "FREE".to_string());

    let app_cfg = config::load_app_config().unwrap_or_default();
    let target_model = &app_cfg.auto_profile_switcher.target_model;

    let immediate_pct = auto_switcher::calculate_account_quota(acc, target_model).unwrap_or(100.0);

    let mut weekly_pct = immediate_pct;
    if let Some(ref q) = acc.quota {
        if let Some(ref groups) = q.quota_groups {
            let mut weekly_vals = Vec::new();
            for g in groups {
                for b in &g.buckets {
                    let w = b.window.to_lowercase();
                    let bid = b.bucket_id.to_lowercase();
                    if w.contains("week") || bid.contains("week") {
                        weekly_vals.push((b.remaining_fraction * 100.0).round());
                    }
                }
            }
            if !weekly_vals.is_empty() {
                weekly_pct = weekly_vals.into_iter().fold(100.0, f64::min);
            }
        }
    }

    (immediate_pct, weekly_pct, tier)
}
