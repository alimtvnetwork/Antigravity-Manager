//! update_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// Normalize version string: e.g., "4.162" or "v4.162" -> "4.162.0",
/// stripping any leading URL prefixes if present.
pub(crate) fn normalize_version_string(raw: &str) -> String {
    let clean = raw.trim().trim_matches(['\'', '"']);
    let stripped_url = if let Some(pos) = clean.find("/releases/tag/") {
        let after = &clean[pos + "/releases/tag/".len()..];
        after.split(['?', '#', '/']).next().unwrap_or(after)
    } else if let Some(pos) = clean.find("/releases/download/") {
        let after = &clean[pos + "/releases/download/".len()..];
        after.split(['?', '#', '/']).next().unwrap_or(after)
    } else if clean.starts_with("http://") || clean.starts_with("https://") {
        let path_only = clean.split(['?', '#']).next().unwrap_or(clean);
        path_only.rsplit('/').next().unwrap_or(clean)
    } else {
        clean
    };

    let clean = stripped_url
        .trim()
        .trim_matches(['\'', '"'])
        .trim_start_matches(['v', 'V'])
        .trim();
    let base = clean.split('-').next().unwrap_or(clean);
    let parts: Vec<&str> = base.split('.').collect();
    if parts.len() == 2
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    {
        if let Some(suffix_idx) = clean.find('-') {
            format!("{}.0{}", &clean[..suffix_idx], &clean[suffix_idx..])
        } else {
            format!("{}.0", clean)
        }
    } else {
        clean.to_string()
    }
}

pub(crate) fn normalize_version_str(raw: &str) -> String {
    normalize_version_string(raw)
}

fn extract_version_from_token(token: &str) -> Option<String> {
    let clean = token.trim().trim_matches(['\'', '"']);
    if clean.is_empty() {
        return None;
    }

    if let Some(pos) = clean.find("/releases/tag/") {
        let after_tag = &clean[pos + "/releases/tag/".len()..];
        let tag_part = after_tag.split(['?', '#', '/']).next().unwrap_or(after_tag);
        let tag_clean = tag_part
            .trim_matches('/')
            .trim_start_matches(['v', 'V'])
            .trim();
        if !tag_clean.is_empty()
            && tag_clean
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            return Some(normalize_version_string(tag_clean));
        }
    }
    if let Some(pos) = clean.find("/releases/download/") {
        let after_dl = &clean[pos + "/releases/download/".len()..];
        let tag_part = after_dl.split(['?', '#', '/']).next().unwrap_or(after_dl);
        let tag_clean = tag_part
            .trim_matches('/')
            .trim_start_matches(['v', 'V'])
            .trim();
        if !tag_clean.is_empty()
            && tag_clean
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            return Some(normalize_version_string(tag_clean));
        }
    }
    if clean.starts_with("http://") || clean.starts_with("https://") {
        let path_only = clean.split(['?', '#']).next().unwrap_or(clean);
        if let Some(last_seg) = path_only.rsplit('/').next() {
            let seg_clean = last_seg.trim().trim_start_matches(['v', 'V']).trim();
            if !seg_clean.is_empty()
                && seg_clean
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                && seg_clean.contains('.')
            {
                return Some(normalize_version_string(seg_clean));
            }
        }
    }
    let v = clean.trim_start_matches(['v', 'V']).trim();
    if v.contains('.')
        && v.chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
    {
        return Some(normalize_version_string(v));
    }
    None
}

/// Extract target version string from command-line arguments.
/// Supports: 'to v4.162', '--to v4.162', '-to v4.162', 'revert v4.162', 'downgrade 4.162', '4.162', 'v4.162', '--version 4.162', '-v 4.162',
/// as well as GitHub release tag URLs (e.g. 'https://github.com/.../releases/tag/v4.226.3').
pub(crate) fn extract_target_version(args: &[String]) -> Option<String> {
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        let clean = arg.trim().trim_matches(['\'', '"']);
        let is_prefix_keyword = clean.eq_ignore_ascii_case("to")
            || clean.eq_ignore_ascii_case("--to")
            || clean.eq_ignore_ascii_case("-to")
            || clean.eq_ignore_ascii_case("revert")
            || clean.eq_ignore_ascii_case("--revert")
            || clean.eq_ignore_ascii_case("-revert")
            || clean.eq_ignore_ascii_case("downgrade")
            || clean.eq_ignore_ascii_case("--downgrade")
            || clean.eq_ignore_ascii_case("-downgrade")
            || clean == "--version"
            || clean == "-v";

        if is_prefix_keyword {
            if let Some(next_arg) = iter.next() {
                if let Some(ver) = extract_version_from_token(next_arg) {
                    return Some(ver);
                }
            }
        } else if let Some(stripped) = clean
            .strip_prefix("--version=")
            .or_else(|| clean.strip_prefix("-v="))
            .or_else(|| clean.strip_prefix("--to="))
            .or_else(|| clean.strip_prefix("-to="))
            .or_else(|| clean.strip_prefix("--revert="))
            .or_else(|| clean.strip_prefix("-revert="))
            .or_else(|| clean.strip_prefix("--downgrade="))
            .or_else(|| clean.strip_prefix("-downgrade="))
        {
            if let Some(ver) = extract_version_from_token(stripped) {
                return Some(ver);
            }
        } else {
            let is_special_cmd = clean.starts_with('-')
                || clean.eq_ignore_ascii_case("all")
                || clean.eq_ignore_ascii_case("gitmap")
                || clean.eq_ignore_ascii_case("gm")
                || clean.eq_ignore_ascii_case("ssh")
                || clean.eq_ignore_ascii_case("export-zip")
                || clean.eq_ignore_ascii_case("help");

            if is_special_cmd {
                continue;
            }
            if let Some(ver) = extract_version_from_token(clean) {
                return Some(ver);
            }
        }
    }
    None
}

use antigravity_tools_lib::modules::update_checker::{
    compare_versions, resolve_installable_release, GitHubRelease, ReleaseAssetInfo,
    SkippedVersion, UpdateChannel,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpdateExecutionMode {
    CheckOnly,
    Execute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpdateTargetScope {
    Standard,
    All,
    Gitmap,
    SshFleet,
    ExportZip,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputFormat {
    Interactive,
    Json,
}

#[derive(Debug, Clone)]
struct ReleaseMeta {
    tag_name: String,
    release_html_url: String,
    release_name: String,
}

fn query_tag_api(client: &reqwest::blocking::Client, cand_tag: &str) -> Option<ReleaseMeta> {
    let api_url = format!(
        "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/tags/{}",
        cand_tag
    );
    let resp = client.get(&api_url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json = resp.json::<serde_json::Value>().ok()?;
    let tag = json["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches(['v', 'V'])
        .to_string();
    if tag.is_empty() {
        return None;
    }
    let html_url = json["html_url"]
        .as_str()
        .unwrap_or("https://github.com/alimtvnetwork/Antigravity-Manager/releases")
        .to_string();
    let name = json["name"].as_str().unwrap_or("").to_string();
    Some(ReleaseMeta {
        tag_name: tag,
        release_html_url: html_url,
        release_name: name,
    })
}

fn query_cdn_updater(client: &reqwest::blocking::Client, cand_tag: &str) -> Option<ReleaseMeta> {
    let cdn_url = format!(
        "https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/{}/updater.json",
        cand_tag
    );
    let resp = client.get(&cdn_url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json = resp.json::<serde_json::Value>().ok()?;
    let tag = json["version"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches(['v', 'V'])
        .to_string();
    if tag.is_empty() {
        return None;
    }
    Some(ReleaseMeta {
        tag_name: tag,
        release_html_url: format!(
            "https://github.com/alimtvnetwork/Antigravity-Manager/releases/tag/{}",
            cand_tag
        ),
        release_name: cand_tag.to_string(),
    })
}

fn query_releases_list(client: &reqwest::blocking::Client, target: &str) -> Option<ReleaseMeta> {
    let list_url =
        "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases?per_page=100";
    let resp = client.get(list_url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let releases = resp.json::<Vec<serde_json::Value>>().ok()?;
    let target_short = target.strip_suffix(".0");
    for rel in releases {
        let rel_tag = rel["tag_name"]
            .as_str()
            .unwrap_or("")
            .trim_start_matches(['v', 'V']);
        let is_match = rel_tag == target
            || target_short.map(|s| rel_tag == s).unwrap_or(false)
            || rel_tag.starts_with(target);
        if is_match {
            let html_url = rel["html_url"]
                .as_str()
                .unwrap_or("https://github.com/alimtvnetwork/Antigravity-Manager/releases")
                .to_string();
            let name = rel["name"].as_str().unwrap_or("").to_string();
            return Some(ReleaseMeta {
                tag_name: rel_tag.to_string(),
                release_html_url: html_url,
                release_name: name,
            });
        }
    }
    None
}

fn query_latest_release(
    client: &reqwest::blocking::Client,
    channel: UpdateChannel,
) -> Option<ReleaseMeta> {
    // Plan 86, subtask 007: resolve the newest release that actually has a
    // platform asset; a newer release without one is skipped (and reported
    // by fetch_skipped_versions) instead of being offered.
    let list_url =
        "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases?per_page=15";
    if let Ok(resp) = client.get(list_url).send() {
        if resp.status().is_success() {
            if let Ok(releases) = resp.json::<Vec<GitHubRelease>>() {
                let infos: Vec<ReleaseAssetInfo> =
                    releases.iter().map(ReleaseAssetInfo::from).collect();
                let resolution = resolve_installable_release(&infos, channel);
                if let Some(selected) = resolution.selected {
                    return Some(ReleaseMeta {
                        tag_name: selected.version.clone(),
                        release_html_url: selected.html_url.clone(),
                        release_name: format!("v{}", selected.version),
                    });
                }
            }
        }
    }

    let api_url = "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/latest";
    if let Ok(resp) = client.get(api_url).send() {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>() {
                let tag = json["tag_name"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches(['v', 'V'])
                    .to_string();
                if !tag.is_empty() {
                    let html_url = json["html_url"]
                        .as_str()
                        .unwrap_or("https://github.com/alimtvnetwork/Antigravity-Manager/releases")
                        .to_string();
                    let name = json["name"].as_str().unwrap_or("").to_string();
                    return Some(ReleaseMeta {
                        tag_name: tag,
                        release_html_url: html_url,
                        release_name: name,
                    });
                }
            }
        }
    }

    let updater_cdn_url = match channel {
        UpdateChannel::Beta => {
            "https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/preview/updater.json"
        }
        UpdateChannel::Stable => {
            "https://github.com/alimtvnetwork/Antigravity-Manager/releases/latest/download/updater.json"
        }
    };
    let resp = client.get(updater_cdn_url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json = resp.json::<serde_json::Value>().ok()?;
    let tag = json["version"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches(['v', 'V'])
        .to_string();
    if tag.is_empty() {
        return None;
    }
    Some(ReleaseMeta {
        tag_name: tag.clone(),
        release_html_url: format!(
            "https://github.com/alimtvnetwork/Antigravity-Manager/releases/tag/v{}",
            tag
        ),
        release_name: format!("v{}", tag),
    })
}

/// Newer releases that cannot be installed on this platform (plan 86
/// skipped-version reporting for the CLI update path). Best-effort: a
/// failed list query yields an empty report, never a failed update.
fn fetch_skipped_versions(
    client: &reqwest::blocking::Client,
    channel: UpdateChannel,
    selected_tag: &str,
) -> Vec<SkippedVersion> {
    let list_url =
        "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases?per_page=15";
    let resp = match client.get(list_url).send() {
        Ok(r) if r.status().is_success() => r,
        _ => return Vec::new(),
    };
    let releases: Vec<GitHubRelease> = match resp.json() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let infos: Vec<ReleaseAssetInfo> = releases.iter().map(ReleaseAssetInfo::from).collect();
    let resolution = resolve_installable_release(&infos, channel);
    resolution
        .skipped
        .into_iter()
        .filter(|s| s.version != selected_tag)
        .filter(|s| compare_versions(&s.version, crate::common::VERSION))
        .collect()
}

pub(crate) fn cmd_update(args: &[String]) {
    let output_format = if args.iter().any(|a| a == "--json" || a == "-j") {
        OutputFormat::Json
    } else {
        OutputFormat::Interactive
    };
    let is_json = output_format == OutputFormat::Json;
    let is_interactive = output_format == OutputFormat::Interactive;

    let execution_mode = if args.iter().any(|a| a == "--check" || a == "-c") {
        UpdateExecutionMode::CheckOnly
    } else {
        UpdateExecutionMode::Execute
    };
    let is_execute_mode = execution_mode == UpdateExecutionMode::Execute;

    let target_scope = if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        UpdateTargetScope::Help
    } else if args
        .iter()
        .any(|a| a == "export-zip" || a == "--export-zip")
    {
        UpdateTargetScope::ExportZip
    } else if args
        .iter()
        .any(|a| a.eq_ignore_ascii_case("gitmap") || a.eq_ignore_ascii_case("gm"))
    {
        UpdateTargetScope::Gitmap
    } else if args.iter().any(|a| a.eq_ignore_ascii_case("ssh")) {
        UpdateTargetScope::SshFleet
    } else if args.iter().any(|a| a == "all" || a == "--all" || a == "-a")
        || env::args().any(|a| a == "update-all" || a == "ua")
    {
        UpdateTargetScope::All
    } else {
        UpdateTargetScope::Standard
    };

    let is_all = target_scope == UpdateTargetScope::All;
    let is_force = args.iter().any(|a| a == "--force" || a == "-f");
    let requested_version = extract_target_version(args);
    let has_explicit_target = requested_version.is_some();
    let is_implicit_target = requested_version.is_none();

    let channel = if args
        .iter()
        .any(|a| a == "--beta" || a == "-b" || a == "beta")
    {
        UpdateChannel::Beta
    } else {
        UpdateChannel::default()
    };

    if target_scope == UpdateTargetScope::ExportZip {
        crate::update_helpers::cmd_update_export_zip(args);
        return;
    }

    if target_scope == UpdateTargetScope::Help {
        if is_json {
            let help_obj = serde_json::json!({
                "command": "update",
                "aliases": ["update-all", "ua"],
                "syntax": "agm update [all|gitmap|ssh] [--json] [--check] [--force]",
                "options": {
                    "--json, -j": "Output pure machine-readable JSON payload (zero banners)",
                    "all, --all, -a": "Update AGM binary, GitMap CLI, pull latest repo code, and sync fleet",
                    "gitmap, gm": "Update GitMap CLI via gitmap self-update",
                    "ssh": "Update AGM across all registered SSH cluster nodes (gitmap ssh update agm)",
                    "--check, -c": "Query and compare releases without installing updates",
                    "--force, -f": "Force re-installation even if already at latest version"
                }
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&help_obj).unwrap_or_default()
            );
        } else {
            println!("AGM Update & Fleet Synchronization:");
            println!("  agm update export-zip [-o path]            Download the release zip and print its path. Does not install.");
            println!("  agm update [--json] [--check] [--force]    Check and update AGM binary from GitHub");
            println!(
                "  agm update gitmap                          Update GitMap CLI to latest release"
            );
            println!("  agm update ssh                             Update AGM across all SSH cluster machines");
            println!("  agm update all [--json] [--check]          Update AGM binary, GitMap, pull repo, and sync fleet");
            println!("  agm update-all, agm ua                     Aliases for 'agm update all'");
            println!();
            println!("Options:");
            println!("  --json, -j      Output pure machine-readable JSON payload (zero banners)");
            println!("  --check, -c     Query and compare releases without installing updates");
            println!("  --force, -f     Force re-installation even if already at latest version");
            println!();
            println!("Examples:");
            println!("  agm update                       # Update AGM binary interactively");
            println!("  agm update gitmap                # Update GitMap CLI (gitmap self-update)");
            println!("  agm update ssh                   # Update AGM across SSH fleet (gitmap ssh update agm)");
            println!("  agm update all                   # Update AGM binary, GitMap, and sync local repository");
            println!("  agm update all --json            # Remote machine automation via JSON");
        }
        return;
    }

    if target_scope == UpdateTargetScope::Gitmap {
        println!("[*] Updating GitMap CLI via 'gitmap self-update'...");
        // Justification: interactive gitmap self-update inherits stdio; spawn failure is non-fatal to caller
        antigravity_tools_lib::error::record_ignored(
            Command::new("gitmap")
                .arg("self-update")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status(),
            "gitmap self-update",
        );
        return;
    }

    if target_scope == UpdateTargetScope::SshFleet {
        println!("[*] Updating AGM across SSH cluster machines via 'gitmap ssh update agm'...");
        // Justification: interactive gitmap ssh update inherits stdio; spawn failure is non-fatal to caller
        antigravity_tools_lib::error::record_ignored(
            Command::new("gitmap")
                .args(["ssh", "update", "agm"])
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status(),
            "gitmap ssh update agm",
        );
        return;
    }

    if is_all && is_execute_mode {
        if is_interactive {
            println!("[*] Checking GitMap CLI for updates (gitmap self-update)...");
        }
        // Justification: optional gitmap self-update during update-all is best-effort
        antigravity_tools_lib::error::record_ignored(
            Command::new("gitmap").arg("self-update").output(),
            "gitmap self-update in update-all",
        );
    }

    if is_interactive {
        println!("[*] Checking GitHub for AGM updates...");
    }

    let client = match reqwest::blocking::Client::builder()
        .user_agent("AGM-CLI-Updater")
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            if is_json {
                let err_obj = serde_json::json!({
                    "success": false,
                    "error": format!("Failed to initialize HTTP client: {}", e),
                    "current_version": crate::common::VERSION,
                    "timestamp": chrono::Utc::now().timestamp(),
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&err_obj).unwrap_or_default()
                );
            } else {
                eprintln!("Failed to initialize HTTP client: {}", e);
            }
            return;
        }
    };

    let is_delegated_stage2 = args.iter().any(|a| {
        a == "--no-launch"
            || a == "--no-relaunch"
            || a == "--delegated-worker"
            || a == "--install-dir"
    });
    if is_delegated_stage2
        && is_interactive
        && is_execute_mode
        && target_scope == UpdateTargetScope::Standard
    {
        let ok = antigravity_tools_lib::modules::delegate_updater::run_cli_update(args);
        std::process::exit(if ok { 0 } else { 1 });
    }

    let mut candidate_tags = Vec::new();
    if let Some(ref target) = requested_version {
        candidate_tags.push(format!("v{}", target));
        if let Some(short_ver) = target.strip_suffix(".0") {
            if short_ver.contains('.') {
                candidate_tags.push(format!("v{}", short_ver));
            }
        }
    }

    let release_meta = if let Some(ref target) = requested_version {
        // Tier 1: Direct GitHub Release API Tag Query
        let mut found = None;
        for cand_tag in &candidate_tags {
            if let Some(meta) = query_tag_api(&client, cand_tag) {
                found = Some(meta);
                break;
            }
        }
        // Tier 2: Release Asset CDN updater.json Query
        if found.is_none() {
            for cand_tag in &candidate_tags {
                if let Some(meta) = query_cdn_updater(&client, cand_tag) {
                    found = Some(meta);
                    break;
                }
            }
        }
        // Tier 3: Releases Index Pagination Fallback
        if found.is_none() {
            found = query_releases_list(&client, target);
        }
        // Tier 4: Direct fallback to normalized requested version if network queries were rate-limited
        found.unwrap_or_else(|| ReleaseMeta {
            tag_name: target.clone(),
            release_html_url: format!(
                "https://github.com/alimtvnetwork/Antigravity-Manager/releases/tag/v{}",
                target
            ),
            release_name: format!("v{}", target),
        })
    } else {
        query_latest_release(&client, channel).unwrap_or_else(|| ReleaseMeta {
            tag_name: String::new(),
            release_html_url: "https://github.com/alimtvnetwork/Antigravity-Manager/releases"
                .to_string(),
            release_name: String::new(),
        })
    };

    let tag_name = release_meta.tag_name;
    let release_html_url = release_meta.release_html_url;
    let release_name = release_meta.release_name;

    // Skipped-version reporting (plan 86): name the newer releases that are
    // not installable here instead of dropping them silently.
    let skipped_versions: Vec<SkippedVersion> = if has_explicit_target {
        Vec::new()
    } else {
        fetch_skipped_versions(&client, channel, &tag_name)
    };
    if is_interactive {
        for skipped in &skipped_versions {
            println!("[!] Skipped v{}: {}", skipped.version, skipped.reason);
        }
    }

    if tag_name.is_empty() {
        if is_json {
            let err_obj = serde_json::json!({
                "success": false,
                "error": "Failed to fetch latest release metadata from GitHub API and CDN updater.json",
                "current_version": crate::common::VERSION,
                "timestamp": chrono::Utc::now().timestamp(),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&err_obj).unwrap_or_default()
            );
        } else {
            eprintln!("Failed to fetch latest release metadata from GitHub API and CDN.");
        }
        return;
    }

    let is_up_to_date = tag_name == crate::common::VERSION;
    let is_version_different = tag_name != crate::common::VERSION;
    let is_downgrade = has_explicit_target && is_version_different;
    let mut updated = false;

    // Repo check if in workspace
    let is_git_repo = Path::new(".git").exists();
    let git_branch = if is_git_repo {
        antigravity_tools_lib::modules::git_info::get_git_branch()
    } else {
        "N/A".to_string()
    };
    let git_hash = if is_git_repo {
        antigravity_tools_lib::modules::git_info::get_git_hash()
    } else {
        "N/A".to_string()
    };
    let mut repo_pulled = false;

    if is_all && is_git_repo && is_execute_mode {
        if is_interactive {
            println!(
                "[*] Pulling latest repository commits (git pull origin {})...",
                git_branch
            );
        }
        let pull_res = Command::new("git")
            .args(["pull", "origin", &git_branch])
            .output();
        if let Ok(out) = pull_res {
            repo_pulled = out.status.success();
        }
    }

    // Binary update if needed
    let needs_update = is_version_different || is_force || has_explicit_target;
    let should_proceed_update = needs_update && is_execute_mode;
    if should_proceed_update {
        let export_path = crate::update_helpers::default_update_export_path();
        match crate::update_helpers::download_release_zip(&export_path, Some(&tag_name)) {
            Ok(()) => println!("EXPORT_ZIP={}", export_path.display()),
            Err(err) => eprintln!("[WARN] update zip was not saved: {}", err),
        }
        if is_interactive {
            if is_downgrade {
                println!(
                    "[*] Target release requested: v{} -> v{} (revert/downgrade)",
                    crate::common::VERSION,
                    tag_name
                );
            } else if is_up_to_date {
                println!("[*] Reinstalling requested release: v{}", tag_name);
            } else {
                println!(
                    "[*] A new version is available: v{} -> v{}",
                    crate::common::VERSION,
                    tag_name
                );
            }
            println!("[*] Triggering automatic update installation...");
        }

        let mut explicit_install_dir = None;
        for i in 0..args.len() {
            if args[i] == "--install-dir" && i + 1 < args.len() {
                explicit_install_dir = Some(PathBuf::from(&args[i + 1]));
                break;
            }
        }
        let resolved_install_dir =
            antigravity_tools_lib::modules::delegate_updater::resolve_default_install_dir(
                explicit_install_dir,
                None,
            );

        #[cfg(target_os = "windows")]
        {
            let current_exe = std::env::current_exe().unwrap_or_default();
            let exe_str = current_exe.to_string_lossy().to_lowercase();
            let is_isolated_temp = exe_str.contains("agm-updater");
            let is_host_binary = !is_isolated_temp;
            if is_host_binary && is_interactive {
                println!(
                    "[*] Windows executable lock guard: Self-delegating to isolated updater in %TEMP%..."
                );
                let mut delegate_args = vec![
                    "--relaunch".to_string(),
                    "--version".to_string(),
                    tag_name.clone(),
                    "--install-dir".to_string(),
                    resolved_install_dir.to_string_lossy().to_string(),
                ];
                if is_force || has_explicit_target {
                    delegate_args.push("--force".to_string());
                }
                if has_explicit_target {
                    delegate_args.push("-isPinned".to_string());
                }
                antigravity_tools_lib::modules::delegate_updater::run(&delegate_args);
                std::process::exit(0);
            }
        }

        let ui_running =
            antigravity_tools_lib::modules::delegate_updater::is_ui_process_running_excluding(
                std::process::id(),
            );

        if ui_running && is_interactive {
            println!(
                "[*] Antigravity Manager UI is currently running. Delegating to 3-stage Update CLI (agm-update-cli -> agm update -> agm open-ui)..."
            );
            let mut delegate_args = vec![
                "--relaunch".to_string(),
                "--version".to_string(),
                tag_name.clone(),
                "--install-dir".to_string(),
                resolved_install_dir.to_string_lossy().to_string(),
            ];
            if is_force || has_explicit_target {
                delegate_args.push("--force".to_string());
            }
            if has_explicit_target {
                delegate_args.push("-isPinned".to_string());
            }
            antigravity_tools_lib::modules::delegate_updater::run(&delegate_args);
            return;
        }

        let mut cli_update_args = vec![
            "--force".to_string(),
            "--no-launch".to_string(),
            "--version".to_string(),
            tag_name.clone(),
            "--install-dir".to_string(),
            resolved_install_dir.to_string_lossy().to_string(),
        ];
        if has_explicit_target {
            cli_update_args.push("-isPinned".to_string());
        }
        updated =
            antigravity_tools_lib::modules::delegate_updater::run_cli_update(&cli_update_args);
    }

    let (node_alias, local_ip) =
        antigravity_tools_lib::modules::email_sender::get_local_node_identity();
    let instance_count = antigravity_tools_lib::modules::instance::list_instances()
        .map(|i| i.len())
        .unwrap_or(1);

    if is_json {
        let payload = serde_json::json!({
            "success": true,
            "command": if is_all { "update-all" } else { "update" },
            "current_version": format!("v{}", crate::common::VERSION),
            "latest_release": format!("v{}", tag_name),
            "target_version": format!("v{}", tag_name),
            "is_downgrade": is_downgrade,
            "release_name": release_name,
            "release_url": release_html_url,
            "is_up_to_date": is_up_to_date,
            "skipped_versions": skipped_versions,
            "updated": updated,
            "scope": if is_all { "fleet_and_repo" } else { "binary_only" },
            "repo": {
                "is_git_repo": is_git_repo,
                "branch": git_branch,
                "commit": git_hash,
                "pulled": repo_pulled
            },
            "fleet": {
                "node_alias": node_alias,
                "local_ip": local_ip,
                "instance_count": instance_count,
                "status": "synchronized"
            },
            "timestamp": chrono::Utc::now().timestamp()
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else {
        println!(
            "  ================================================================================"
        );
        println!("    AGM SYSTEM UPDATE & FLEET SYNCHRONIZATION");
        println!(
            "  ================================================================================"
        );
        println!("    ● Binary Target:     agm (Antigravity-Manager)");
        println!("    ● Current Version:   v{}", crate::common::VERSION);
        println!("    ● Latest Release:    v{}", tag_name);
        let update_status_desc = if is_downgrade && updated {
            format!(
                "[REVERTED/DOWNGRADED] (Successfully downgraded to v{})",
                tag_name
            )
        } else if is_up_to_date && is_implicit_target {
            "[UP TO DATE] (System is currently running the latest release)".to_string()
        } else if updated {
            format!("[UPDATED] (Successfully updated to v{})", tag_name)
        } else {
            format!("[AVAILABLE] (v{} is available for installation)", tag_name)
        };
        println!("    ● Update Status:     {}", update_status_desc);
        println!("    ● Release URL:       {}", release_html_url);
        if is_all {
            println!(
                "    ────────────────────────────────────────────────────────────────────────────"
            );
            println!(
                "    ● Scope:             Full Fleet Synchronization (Binary + Repo + Instances)"
            );
            if is_git_repo {
                println!(
                    "    ● Local Workspace:   {} (Branch: {}, Commit: {})",
                    env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| ".".to_string()),
                    git_branch,
                    git_hash
                );
                println!(
                    "    ● Repo Git Pull:     {}",
                    if repo_pulled {
                        "Updated (git pull successful)"
                    } else {
                        "Up to date / skipped"
                    }
                );
            }
            println!(
                "    ● Fleet Node:        {} / {} (Instances: {})",
                node_alias, local_ip, instance_count
            );
        }
        println!(
            "  ================================================================================"
        );
    }
}

pub(crate) fn cmd_delegate_update(args: &[String]) {
    antigravity_tools_lib::modules::delegate_updater::run(args);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_target_version() {
        assert_eq!(
            extract_target_version(&["to".to_string(), "v4.162".to_string()]),
            Some("4.162.0".to_string())
        );
        assert_eq!(
            extract_target_version(&["to".to_string(), "4.162".to_string()]),
            Some("4.162.0".to_string())
        );
        assert_eq!(
            extract_target_version(&["revert".to_string(), "v4.162".to_string()]),
            Some("4.162.0".to_string())
        );
        assert_eq!(
            extract_target_version(&["downgrade".to_string(), "4.162".to_string()]),
            Some("4.162.0".to_string())
        );
        assert_eq!(
            extract_target_version(&["4.162".to_string()]),
            Some("4.162.0".to_string())
        );
        assert_eq!(
            extract_target_version(&["v4.162.0".to_string()]),
            Some("4.162.0".to_string())
        );
        assert_eq!(
            extract_target_version(&["--version".to_string(), "4.162".to_string()]),
            Some("4.162.0".to_string())
        );
        assert_eq!(
            extract_target_version(&["-v".to_string(), "v4.162".to_string()]),
            Some("4.162.0".to_string())
        );
        assert_eq!(
            extract_target_version(&["--version=4.162".to_string()]),
            Some("4.162.0".to_string())
        );
        assert_eq!(
            extract_target_version(&[
                "https://github.com/alimtvnetwork/Antigravity-Manager/releases/tag/v4.226.3"
                    .to_string()
            ]),
            Some("4.226.3".to_string())
        );
        assert_eq!(
            extract_target_version(&[
                "to".to_string(),
                "https://github.com/alimtvnetwork/Antigravity-Manager/releases/tag/v4.226.3"
                    .to_string()
            ]),
            Some("4.226.3".to_string())
        );
        assert_eq!(
            extract_target_version(&[
                "--version".to_string(),
                "https://github.com/alimtvnetwork/Antigravity-Manager/releases/tag/4.226.3"
                    .to_string()
            ]),
            Some("4.226.3".to_string())
        );
        assert_eq!(
            extract_target_version(&["all".to_string(), "--force".to_string()]),
            None
        );
    }

    #[test]
    fn test_normalize_version_string() {
        assert_eq!(normalize_version_string("4.162"), "4.162.0");
        assert_eq!(normalize_version_string("v4.162"), "4.162.0");
        assert_eq!(normalize_version_string("4.162.0"), "4.162.0");
        assert_eq!(normalize_version_string("v4.185.0"), "4.185.0");
        assert_eq!(normalize_version_string("4.162.0-beta.1"), "4.162.0-beta.1");
    }
}
