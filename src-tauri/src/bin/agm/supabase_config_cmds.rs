//! supabase_config_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::fs;

pub(crate) fn cmd_supabase_set_config(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("Usage: agm supabase set-config [--cooldown <minutes>] [--interval <seconds>] [--alias <alias>]");
        println!();
        println!("Options:");
        println!("  --cooldown, -c <minutes>    Set account lockout cooldown window (in minutes)");
        println!("  --interval, -i <seconds>    Set Supabase heartbeat interval & auto-switcher check interval");
        println!("  --alias, -a <alias>         Set local machine node alias");
        println!();
        println!("Examples:");
        println!("  agm supabase set-config --cooldown 45");
        println!("  agm supabase set-config --interval 15 --alias \"Node-Alpha\"");
        println!(
            "  agm supabase set-config --cooldown 30 --interval 20 --alias \"Fleet-Worker-01\""
        );
        return;
    }

    let mut cooldown_mins: Option<u32> = None;
    let mut interval_secs: Option<u64> = None;
    let mut alias_opt: Option<String> = None;

    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        if arg == "--cooldown" || arg == "-c" {
            if idx + 1 < args.len() {
                idx += 1;
                match args[idx].parse::<u32>() {
                    Ok(v) => cooldown_mins = Some(v),
                    Err(_) => {
                        eprintln!("[ERROR] Invalid cooldown minutes: '{}'", args[idx]);
                        return;
                    }
                }
            } else {
                eprintln!("[ERROR] Missing value for --cooldown");
                return;
            }
        } else if let Some(val_str) = arg.strip_prefix("--cooldown=") {
            match val_str.parse::<u32>() {
                Ok(v) => cooldown_mins = Some(v),
                Err(_) => {
                    eprintln!("[ERROR] Invalid cooldown minutes: '{}'", val_str);
                    return;
                }
            }
        } else if arg == "--interval" || arg == "-i" {
            if idx + 1 < args.len() {
                idx += 1;
                match args[idx].parse::<u64>() {
                    Ok(v) => interval_secs = Some(v),
                    Err(_) => {
                        eprintln!("[ERROR] Invalid interval seconds: '{}'", args[idx]);
                        return;
                    }
                }
            } else {
                eprintln!("[ERROR] Missing value for --interval");
                return;
            }
        } else if let Some(val_str) = arg.strip_prefix("--interval=") {
            match val_str.parse::<u64>() {
                Ok(v) => interval_secs = Some(v),
                Err(_) => {
                    eprintln!("[ERROR] Invalid interval seconds: '{}'", val_str);
                    return;
                }
            }
        } else if arg == "--alias" || arg == "-a" {
            if idx + 1 < args.len() {
                idx += 1;
                alias_opt = Some(args[idx].clone());
            } else {
                eprintln!("[ERROR] Missing value for --alias");
                return;
            }
        } else if let Some(val_str) = arg.strip_prefix("--alias=") {
            alias_opt = Some(val_str.to_string());
        } else {
            eprintln!("[WARNING] Unrecognized argument: '{}'", arg);
        }
        idx += 1;
    }

    if cooldown_mins.is_none() && interval_secs.is_none() && alias_opt.is_none() {
        println!("No configuration updates specified.");
        println!("Usage: agm supabase set-config [--cooldown <minutes>] [--interval <seconds>] [--alias <alias>]");
        return;
    }

    // 1. Update SupabaseConfig
    let mut sb_cfg = supabase_sync::load_config().unwrap_or_default();
    let mut sb_changed = false;

    if let Some(ref alias) = alias_opt {
        sb_cfg.node_alias = alias.clone();
        sb_changed = true;
    }

    if let Some(iv) = interval_secs {
        sb_cfg.heartbeat_interval_secs = iv;
        sb_changed = true;
    }

    if sb_changed {
        if let Err(e) = supabase_sync::save_config(&sb_cfg) {
            eprintln!("[ERROR] Failed to save Supabase config: {}", e);
        } else {
            println!("✅ Supabase configuration saved successfully.");
        }
    }

    // 2. Update AutoProfileSwitcherConfig (in AppConfig)
    let mut app_cfg = config::load_app_config().unwrap_or_default();
    let mut app_changed = false;

    if let Some(cd) = cooldown_mins {
        app_cfg.auto_profile_switcher.account_lockout_window_minutes = cd;
        app_changed = true;
    }

    if let Some(iv) = interval_secs {
        app_cfg.auto_profile_switcher.check_interval_seconds = iv as u32;
        app_changed = true;
    }

    if app_changed {
        if let Err(e) = config::save_app_config(&app_cfg) {
            eprintln!("[ERROR] Failed to save AppConfig: {}", e);
        } else {
            println!("✅ Auto-Profile Switcher configuration saved successfully.");
        }
    }

    println!("\nUpdated Configuration Summary:");
    if let Some(alias) = alias_opt {
        println!("  • Node Alias               : {}", alias);
    }
    if let Some(iv) = interval_secs {
        println!("  • Heartbeat Interval       : {}s", iv);
        println!("  • Switcher Check Interval  : {}s", iv);
    }
    if let Some(cd) = cooldown_mins {
        println!("  • Account Lockout Window   : {}m", cd);
    }
}

pub(crate) fn cmd_supabase_load_json(args: &[String]) {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;

    let paths: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if paths.is_empty() {
        eprintln!("Usage: agm supabase load-json <file_path...> [-y]");
        return;
    }

    let mut cfg = supabase_sync::load_config().unwrap_or_default();
    let mut total_endpoints_added = 0;
    let mut total_creds_added = 0;

    for path_str in paths {
        let resolved_path = json_envelope::resolve_relative_json_path(path_str);
        let content = match fs::read_to_string(&resolved_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[ERROR] Failed to read file '{}': {}", path_str, e);
                continue;
            }
        };

        let raw_val: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[ERROR] Failed to parse JSON in '{}': {}", path_str, e);
                continue;
            }
        };

        // Unpack envelope if present
        let (attrs_opt, val) = json_envelope::unpack_envelope(raw_val.clone());
        let is_b64 = attrs_opt
            .as_ref()
            .and_then(|a| a.encoding.as_deref())
            .map(|enc| enc.eq_ignore_ascii_case("base64"))
            .unwrap_or(false)
            || raw_val.get("encoding_format").and_then(|f| f.as_str()) == Some("base64");

        let decode_val = |raw: Option<&serde_json::Value>| -> Option<String> {
            let s = raw.and_then(|v| v.as_str())?;
            if is_b64 {
                if let Ok(b) = STANDARD.decode(s.trim()) {
                    if let Ok(utf) = String::from_utf8(b) {
                        return Some(utf.trim().to_string());
                    }
                }
            }
            Some(s.trim().to_string())
        };

        if let Some(arr) = val.get("endpoints").and_then(|v| v.as_array()) {
            if let Ok(eps) = serde_json::from_value::<Vec<supabase_client::SupabaseEndpoint>>(
                serde_json::Value::Array(arr.clone()),
            ) {
                for new_ep in eps {
                    if let Some(pos) = cfg.endpoints.iter().position(|e| e.id == new_ep.id) {
                        cfg.endpoints[pos] = new_ep;
                    } else {
                        cfg.endpoints.push(new_ep);
                    }
                    total_endpoints_added += 1;
                }
                println!("✅ Merged endpoints from '{}'.", path_str);
            }
        } else if let Ok(eps) =
            serde_json::from_value::<Vec<supabase_client::SupabaseEndpoint>>(val.clone())
        {
            for new_ep in eps {
                if let Some(pos) = cfg.endpoints.iter().position(|e| e.id == new_ep.id) {
                    cfg.endpoints[pos] = new_ep;
                } else {
                    cfg.endpoints.push(new_ep);
                }
                total_endpoints_added += 1;
            }
            println!("✅ Merged endpoints array from '{}'.", path_str);
        } else if let Some(creds) = val
            .get("credentials")
            .and_then(|v| v.as_object())
            .or_else(|| val.as_object())
        {
            let endpoint_url = decode_val(creds.get("endpoint"));
            let token = decode_val(creds.get("token"));
            let service =
                decode_val(creds.get("service")).unwrap_or_else(|| "supabase-service".to_string());
            if let (Some(url), Some(tok)) = (endpoint_url, token) {
                let clean_id = format!("ep-{}", service.to_lowercase().replace([' ', '_'], "-"));
                let role = if service.to_lowercase().contains("root")
                    || service.to_lowercase().contains("lovable")
                {
                    "root"
                } else {
                    "secondary"
                };
                let norm_url = supabase_client::normalize_supabase_url(&url);
                let new_ep = supabase_client::SupabaseEndpoint {
                    id: clean_id.clone(),
                    name: format!("Supabase ({})", service),
                    url: norm_url.clone(),
                    api_key: tok,
                    role: role.to_string(),
                    is_enabled: true,
                    prune_threshold_mb: if role == "root" { 400 } else { 200 },
                    priority: if role == "root" { 1 } else { 2 },
                    notes: Some(format!("Imported from credentials JSON ({})", path_str)),
                    tags: vec![role.to_string(), service],
                };
                if let Some(pos) = cfg
                    .endpoints
                    .iter()
                    .position(|e| e.id == clean_id || e.url == norm_url)
                {
                    cfg.endpoints[pos] = new_ep;
                } else {
                    cfg.endpoints.push(new_ep);
                }
                println!(
                    "✅ Ingested credentials database '{}' ({}) from '{}'",
                    clean_id, norm_url, path_str
                );
                total_creds_added += 1;
            }
        }
    }

    cfg.is_sync_enabled = true;
    if let Err(e) = supabase_sync::save_config(&cfg) {
        eprintln!("[ERROR] Failed to save config: {}", e);
    } else {
        println!(
            "✅ Saved {} active endpoint(s) to supabase_config.json (added: {} endpoints, {} credentials)",
            cfg.endpoints.len(),
            total_endpoints_added,
            total_creds_added
        );
    }
}

pub(crate) fn cmd_supabase_schema(args: &[String]) {
    let role = args
        .first()
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "root".to_string());
    let sql = supabase_schema::get_schema_sql(&role);
    println!(
        "-- SQL Schema for Supabase {} Database --",
        role.to_uppercase()
    );
    println!("{}", sql);
}

pub(crate) fn cmd_supabase_sync(rt: &tokio::runtime::Runtime) {
    println!("Synchronizing local machine node and instance profiles to Supabase...");
    match rt.block_on(supabase_sync::sync_local_node_now()) {
        Ok(_) => println!("✅ Sync completed successfully."),
        Err(e) => eprintln!("[ERROR] Sync failed: {}", e),
    }
}

pub(crate) fn cmd_supabase_confirm(rt: &tokio::runtime::Runtime, args: &[String]) {
    let instance_id = args
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| "default".to_string());
    println!(
        "Checking Supabase profile for instance '{}'...",
        instance_id
    );
    match rt.block_on(supabase_sync::push_and_read_instance_email(&instance_id)) {
        Ok(message) => println!("[OK] [Notify] supabase: OK {}", message),
        Err(err) => {
            let trace = std::backtrace::Backtrace::force_capture();
            eprintln!("[Notify] supabase: FAIL {}\n{}", err, trace);
        }
    }
}

pub(crate) fn cmd_supabase_set_endpoint(args: &[String]) {
    if args.len() < 5 {
        println!("Usage: agm supabase set <id> <name> <url> <api_key> <role> [notes] [tags]");
        println!("Example:");
        println!("  agm supabase set ep-root-lovable-01 \"Root Lovable\" https://pezjuuddecbyfmqxytrv.supabase.co/rest/v1/ sb_publishable_... root \"Root by Lovable\" \"lovable,root\"");
        return;
    }

    let id = args[0].trim().to_string();
    let name = args[1].trim().to_string();
    let url = args[2].trim().to_string();
    let api_key = args[3].trim().to_string();
    let role = args[4].trim().to_lowercase();

    if role != "root" && role != "secondary" {
        eprintln!("[ERROR] Role must be 'root' or 'secondary', got '{}'", role);
        return;
    }

    let notes = args
        .get(5)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let tags: Vec<String> = args
        .get(6)
        .map(|s| {
            s.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let mut cfg = supabase_sync::load_config().unwrap_or_default();
    if let Some(existing) = cfg.endpoints.iter_mut().find(|e| e.id == id) {
        existing.name = name;
        existing.url = url;
        existing.api_key = api_key;
        existing.role = role;
        existing.notes = notes;
        existing.tags = tags;
        println!("✅ Updated existing endpoint '{}'", id);
    } else {
        let prune = if role == "root" { 400 } else { 200 };
        cfg.endpoints.push(supabase_client::SupabaseEndpoint {
            id: id.clone(),
            name,
            url,
            api_key,
            role,
            is_enabled: true,
            prune_threshold_mb: prune,
            priority: (cfg.endpoints.len() + 1) as u32,
            notes,
            tags,
        });
        println!("✅ Added new endpoint '{}'", id);
    }

    cfg.is_sync_enabled = true;
    if let Err(e) = supabase_sync::save_config(&cfg) {
        eprintln!("[ERROR] Failed to save config: {}", e);
    } else {
        println!("✅ Saved to supabase_config.json");
    }
}
