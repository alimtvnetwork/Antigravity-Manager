//! instance_routes_b — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};

pub(crate) fn refuse_if_caller(args: &[String], target_id: &str, is_json: bool) {
    let has_force = args.iter().any(|a| a == "--force" || a == "-f");
    if !has_force {
        let resolved =
            instance::resolve_instance_id(target_id).unwrap_or_else(|_| target_id.to_string());
        let mut matched_caller: Option<String> = None;
        if let Some(caller_id) = instance::close::detect_caller_instance_id() {
            if caller_id.eq_ignore_ascii_case(&resolved)
                || caller_id.eq_ignore_ascii_case(target_id)
            {
                matched_caller = Some(caller_id);
            }
        }
        if let Some(caller_id) = matched_caller {
            let msg = format!(
                "Refusing to stop, switch, restart, or delete instance '{caller_id}' because this command is running inside its IDE process. Target instance '{caller_id}' is your currently running IDE instance! Aborting to prevent terminating your own session. Pass --force to override."
            );
            if is_json {
                let err_res = serde_json::json!({
                    "success": false,
                    "error": msg,
                    "code": "REFUSE_CALLER_INSTANCE"
                });
                println!("{}", serde_json::to_string(&err_res).unwrap_or_default());
            }
            eprintln!("[ERROR] {msg}");
            std::process::exit(2);
        }
        if let Err(err) = instance::refuse_if_caller_instance(target_id) {
            let msg = format!("{err}. Pass --force to override.");
            if is_json {
                let err_res = serde_json::json!({
                    "success": false,
                    "error": msg,
                    "code": "REFUSE_CALLER_INSTANCE"
                });
                println!("{}", serde_json::to_string(&err_res).unwrap_or_default());
            }
            eprintln!("[ERROR] {msg}");
            std::process::exit(2);
        }
    } else {
        std::env::set_var("AGM_FORCE_CALLER_OVERRIDE", "1");
    }
}

pub(crate) fn route_instances_launch(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances launch <target> OR agm instances start <target> OR agm instances <target> launch
    if non_flag_args.len() >= 2
        && (non_flag_args[0].eq_ignore_ascii_case("launch")
            || non_flag_args[0].eq_ignore_ascii_case("start")
            || non_flag_args[1].eq_ignore_ascii_case("launch")
            || non_flag_args[1].eq_ignore_ascii_case("start"))
    {
        let target_spec = if non_flag_args[0].eq_ignore_ascii_case("launch")
            || non_flag_args[0].eq_ignore_ascii_case("start")
        {
            non_flag_args[1].clone()
        } else {
            non_flag_args[0].clone()
        };

        let resolved_id = match instance::resolve_instance_id(&target_spec) {
            Ok(id) => id,
            Err(e) => {
                eprintln!(
                    "[ERROR] Could not resolve instance '{}': {}",
                    target_spec, e
                );
                std::process::exit(1);
            }
        };

        println!(
            "[*] Launching instance '{}' (resolved from '{}')...",
            resolved_id, target_spec
        );
        match instance::launch_instance(&resolved_id) {
            Ok(_) => {
                println!(
                    "[SUCCESS] Launched instance '{}' successfully.",
                    resolved_id
                );
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to launch instance '{}': {}", resolved_id, e);
                std::process::exit(1);
            }
        }
        return true;
    }
    false
}

pub(crate) fn route_instances_rm(args: &[String], non_flag_args: &[String], is_json: bool) -> bool {
    // Subcommand: agm instances rm <target> OR agm instances <target> rm
    if non_flag_args.len() >= 2
        && (non_flag_args[0].eq_ignore_ascii_case("rm")
            || non_flag_args[0].eq_ignore_ascii_case("remove")
            || non_flag_args[0].eq_ignore_ascii_case("delete")
            || non_flag_args[1].eq_ignore_ascii_case("rm")
            || non_flag_args[1].eq_ignore_ascii_case("remove")
            || non_flag_args[1].eq_ignore_ascii_case("delete"))
    {
        let target_spec = if non_flag_args[0].eq_ignore_ascii_case("rm")
            || non_flag_args[0].eq_ignore_ascii_case("remove")
            || non_flag_args[0].eq_ignore_ascii_case("delete")
        {
            non_flag_args[1].clone()
        } else {
            non_flag_args[0].clone()
        };

        let resolved_id = match instance::resolve_instance_id(&target_spec) {
            Ok(id) => id,
            Err(e) => {
                eprintln!(
                    "[ERROR] Could not resolve instance '{}': {}",
                    target_spec, e
                );
                std::process::exit(1);
            }
        };

        refuse_if_caller(args, &resolved_id, is_json);

        match instance::delete_instance(&resolved_id) {
            Ok(_) => println!("[SUCCESS] Deleted instance '{}'.", resolved_id),
            Err(e) => {
                eprintln!("[ERROR] Failed to delete instance '{}': {}", resolved_id, e);
                std::process::exit(1);
            }
        }
        return true;
    }
    false
}

pub(crate) fn route_instances_switch(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances switch <target> [account] OR agm instances <target> switch [account]
    let is_switch_order1 = !non_flag_args.is_empty()
        && (non_flag_args[0].eq_ignore_ascii_case("switch")
            || non_flag_args[0].eq_ignore_ascii_case("swtich")
            || non_flag_args[0].eq_ignore_ascii_case("use"));
    let is_switch_order2 = non_flag_args.len() >= 2
        && (non_flag_args[1].eq_ignore_ascii_case("switch")
            || non_flag_args[1].eq_ignore_ascii_case("swtich")
            || non_flag_args[1].eq_ignore_ascii_case("use"));
    if is_switch_order1 || is_switch_order2 {
        if non_flag_args.len() == 1 {
            eprintln!("Usage: agm instances switch <instance> [account]");
            std::process::exit(1);
        }

        // Single-argument switch: agm instances switch <target>
        if non_flag_args.len() == 2 {
            let target_spec = if is_switch_order1 {
                non_flag_args[1].as_str()
            } else {
                non_flag_args[0].as_str()
            };

            // Case A: target resolves to an instance profile -> switch active instance!
            if let Ok(resolved_id) = instance::resolve_instance_id(&target_spec) {
                let reg = instance::load_registry().ok();
                let inst_name = reg
                    .as_ref()
                    .and_then(|r| r.instances.iter().find(|i| i.id == resolved_id))
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| resolved_id.clone());
                let inst_email = reg
                    .as_ref()
                    .and_then(|r| r.instances.iter().find(|i| i.id == resolved_id))
                    .and_then(|i| i.bound_email.clone());

                if let Err(e) = instance::set_active_instance_id(&resolved_id) {
                    eprintln!("[ERROR] Failed to switch active instance: {}", e);
                    std::process::exit(1);
                }

                if is_json {
                    let res = serde_json::json!({
                        "success": true,
                        "active_instance": resolved_id,
                        "name": inst_name,
                        "bound_email": inst_email
                    });
                    println!("{}", serde_json::to_string(&res).unwrap_or_default());
                } else {
                    println!(
                        "[SUCCESS] Switched active instance to '{}' (ID: {}).",
                        inst_name, resolved_id
                    );
                }
                return true;
            }

            // Case B: target is an account query -> switch account for currently active instance!
            let acc_query = target_spec.trim().to_lowercase();
            let index = match account::load_account_index() {
                Ok(idx) => idx,
                Err(e) => {
                    eprintln!("[ERROR] Failed to load accounts: {}", e);
                    std::process::exit(1);
                }
            };
            let matches: Vec<_> = index
                .accounts
                .iter()
                .filter(|a| {
                    let email_l = a.email.to_lowercase();
                    let id_l = a.id.to_lowercase();
                    email_l.contains(&acc_query) || id_l.contains(&acc_query)
                })
                .collect();

            if !matches.is_empty() {
                let target_acc = if matches.len() == 1 {
                    matches[0]
                } else if let Some(exact) =
                    matches.iter().find(|a| a.email.to_lowercase() == acc_query)
                {
                    *exact
                } else {
                    matches[0]
                };

                let active_id =
                    instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());
                refuse_if_caller(args, &active_id, is_json);
                let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                match rt.block_on(instance::switch_account_to_instance(
                    &target_acc.id,
                    Some(&active_id),
                )) {
                    Ok(_) => {
                        let _ = instance::set_active_instance_id(&active_id);
                        if is_json {
                            let res = serde_json::json!({
                                "success": true,
                                "instance": active_id,
                                "email": target_acc.email,
                                "account_id": target_acc.id
                            });
                            println!("{}", serde_json::to_string(&res).unwrap_or_default());
                        } else {
                            println!(
                                "[SUCCESS] Active instance '{}' successfully switched to '{}'.",
                                active_id, target_acc.email
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Instance switch failed: {}", e);
                        std::process::exit(1);
                    }
                }
                return true;
            }

            eprintln!(
                "[ERROR] Could not resolve '{}' as a valid instance profile or account query.",
                target_spec
            );
            std::process::exit(1);
        }

        // Two-or-more arguments switch: agm instances switch <instance> <account>
        let (target_spec, acc_query) = if is_switch_order1 {
            // Check if non_flag_args[1] is an instance
            if instance::resolve_instance_id(&non_flag_args[1]).is_ok() {
                (
                    non_flag_args[1].clone(),
                    non_flag_args[2..].join(" ").trim().to_lowercase(),
                )
            } else if let Some(last) = non_flag_args.last() {
                if instance::resolve_instance_id(last).is_ok() {
                    (
                        (*last).clone(),
                        non_flag_args[1..non_flag_args.len() - 1]
                            .join(" ")
                            .trim()
                            .to_lowercase(),
                    )
                } else {
                    (
                        non_flag_args[1].clone(),
                        non_flag_args[2..].join(" ").trim().to_lowercase(),
                    )
                }
            } else {
                (
                    non_flag_args[1].clone(),
                    non_flag_args[2..].join(" ").trim().to_lowercase(),
                )
            }
        } else {
            (
                non_flag_args[0].clone(),
                non_flag_args[2..].join(" ").trim().to_lowercase(),
            )
        };

        let resolved_id = match instance::resolve_instance_id(&target_spec) {
            Ok(id) => id,
            Err(e) => {
                eprintln!(
                    "[ERROR] Could not resolve instance '{}': {}",
                    target_spec, e
                );
                std::process::exit(1);
            }
        };

        refuse_if_caller(args, &resolved_id, is_json);

        let index = match account::load_account_index() {
            Ok(idx) => idx,
            Err(e) => {
                if is_json {
                    let err_res = serde_json::json!({
                        "success": false,
                        "error": format!("Failed to load accounts: {}", e)
                    });
                    println!("{}", serde_json::to_string(&err_res).unwrap_or_default());
                }
                eprintln!("[ERROR] Failed to load accounts: {}", e);
                std::process::exit(1);
            }
        };
        let matches: Vec<_> = index
            .accounts
            .iter()
            .filter(|a| {
                let email_l = a.email.to_lowercase();
                let id_l = a.id.to_lowercase();
                email_l.contains(&acc_query) || id_l.contains(&acc_query)
            })
            .collect();
        let target_acc = if matches.is_empty() && (acc_query == "default" || acc_query.is_empty()) {
            if let Some(first) = index.accounts.first() {
                first
            } else {
                if is_json {
                    let err_res = serde_json::json!({
                        "success": false,
                        "error": "No accounts registered to switch to."
                    });
                    println!("{}", serde_json::to_string(&err_res).unwrap_or_default());
                }
                eprintln!("[ERROR] No accounts registered to switch to.");
                std::process::exit(1);
            }
        } else if matches.is_empty() {
            if is_json {
                let err_res = serde_json::json!({
                    "success": false,
                    "error": format!("No account found matching '{}'.", acc_query)
                });
                println!("{}", serde_json::to_string(&err_res).unwrap_or_default());
            }
            eprintln!("[ERROR] No account found matching '{}'.", acc_query);
            std::process::exit(1);
        } else if matches.len() == 1 {
            matches[0]
        } else if let Some(exact) = matches.iter().find(|a| a.email.to_lowercase() == acc_query) {
            *exact
        } else {
            if is_json {
                let err_res = serde_json::json!({
                    "success": false,
                    "error": format!("Query '{}' matched multiple accounts.", acc_query)
                });
                println!("{}", serde_json::to_string(&err_res).unwrap_or_default());
            }
            eprintln!("[ERROR] Query '{}' matched multiple accounts:", acc_query);
            for m in &matches {
                eprintln!("  - {} (ID: {})", m.email, m.id);
            }
            std::process::exit(1);
        };

        let previous_email = instance::load_registry()
            .ok()
            .and_then(|r| r.instances.into_iter().find(|i| i.id == resolved_id))
            .and_then(|i| i.bound_email)
            .unwrap_or_default();

        if !is_json {
            println!(
                "[*] Switching instance '{}' to account '{}' (ID: {})...",
                resolved_id, target_acc.email, target_acc.id
            );
        }
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        match rt.block_on(instance::switch_account_to_instance(
            &target_acc.id,
            Some(&resolved_id),
        )) {
            Ok(_) => {
                let _ = instance::set_active_instance_id(&resolved_id);
                antigravity_tools_lib::modules::audit::record_if_active(
                    "cli",
                    "cli",
                    "switch_account_to_instance",
                    &format!("instance:{resolved_id}"),
                    serde_json::json!({
                        "instanceId": resolved_id,
                        "accountId": target_acc.id,
                        "email": target_acc.email,
                        "previousEmail": previous_email,
                    }),
                    "ok",
                    &format!("Switched instance {resolved_id} to {}", target_acc.email),
                );
                if is_json {
                    let res = serde_json::json!({
                        "success": true,
                        "instance": resolved_id,
                        "email": target_acc.email,
                        "previous_email": previous_email,
                        "account_id": target_acc.id,
                        "data": {
                            "instance_id": resolved_id,
                            "email": target_acc.email,
                            "previous_email": previous_email,
                            "account_id": target_acc.id,
                            "status": "switched"
                        }
                    });
                    println!("{}", serde_json::to_string(&res).unwrap_or_default());
                } else {
                    println!(
                        "[SUCCESS] Instance '{}' successfully switched to '{}'.",
                        resolved_id, target_acc.email
                    );
                }
            }
            Err(e) => {
                if is_json {
                    let err = serde_json::json!({
                        "success": false,
                        "error": e
                    });
                    println!("{}", serde_json::to_string(&err).unwrap_or_default());
                } else {
                    eprintln!("[ERROR] Instance switch failed: {}", e);
                }
                std::process::exit(1);
            }
        }
        return true;
    }
    false
}

pub(crate) fn route_instances_ff(args: &[String], non_flag_args: &[String], is_json: bool) -> bool {
    // Subcommand: agm instances ff [target] OR agm instances <target> ff
    if !non_flag_args.is_empty() {
        let is_ff_first = non_flag_args[0].eq_ignore_ascii_case("ff")
            || non_flag_args[0].eq_ignore_ascii_case("fast-forward")
            || non_flag_args[0].eq_ignore_ascii_case("rotate");
        let is_ff_second = non_flag_args.len() >= 2
            && (non_flag_args[1].eq_ignore_ascii_case("ff")
                || non_flag_args[1].eq_ignore_ascii_case("fast-forward")
                || non_flag_args[1].eq_ignore_ascii_case("rotate"));

        if is_ff_first || is_ff_second {
            let target_spec = if is_ff_first {
                non_flag_args.get(1).map(|s| s.as_str()).unwrap_or("active")
            } else {
                non_flag_args[0].as_str()
            };

            let resolved_id = match instance::resolve_instance_id(target_spec) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!(
                        "[ERROR] Could not resolve instance '{}': {}",
                        target_spec, e
                    );
                    std::process::exit(1);
                }
            };

            refuse_if_caller(args, &resolved_id, is_json);
            let prev_email = instance::load_registry()
                .ok()
                .and_then(|r| r.instances.into_iter().find(|i| i.id == resolved_id))
                .and_then(|i| i.bound_email)
                .unwrap_or_default();
            if !is_json {
                println!(
                    "[*] Fast-forward rotating account for instance '{}' (resolved from '{}')...",
                    resolved_id, target_spec
                );
            }
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            let _ = repo_db::backup_running_prompts(&resolved_id);
            match rt.block_on(auto_switcher::trigger_manual_rotation_for_instance(Some(
                &resolved_id,
            ))) {
                Ok(msg) => {
                    let _ = repo_db::resend_running_commands_for_instance(Some(&resolved_id), 20);
                    let active_acc = instance::load_registry()
                        .ok()
                        .and_then(|r| r.instances.into_iter().find(|i| i.id == resolved_id))
                        .and_then(|i| i.bound_email)
                        .unwrap_or_default();
                    antigravity_tools_lib::modules::audit::record_if_active(
                        "cli",
                        "cli",
                        "fast_forward_instance",
                        &format!("instance:{resolved_id}"),
                        serde_json::json!({
                            "instanceId": resolved_id,
                            "selectedEmail": active_acc,
                            "previousEmail": prev_email,
                        }),
                        "ok",
                        &msg,
                    );
                    if is_json {
                        let res = serde_json::json!({
                            "success": true,
                            "instance": resolved_id,
                            "selected_email": active_acc,
                            "previous_email": prev_email,
                            "message": msg
                        });
                        println!("{}", serde_json::to_string(&res).unwrap_or_default());
                    } else {
                        println!("[SUCCESS] {}", msg);
                    }
                }
                Err(e) => {
                    if is_json {
                        let err_res = serde_json::json!({
                            "success": false,
                            "error": e
                        });
                        println!("{}", serde_json::to_string(&err_res).unwrap_or_default());
                    } else {
                        eprintln!("[ERROR] Instance fast-forward failed: {}", e);
                    }
                    std::process::exit(1);
                }
            }
            return true;
        }
    }
    false
}

pub(crate) fn route_instances_self(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    let _ = args;
    if non_flag_args.is_empty() {
        return false;
    }
    let sub = non_flag_args[0].to_lowercase();
    if sub != "self" && sub != "current" && sub != "whoami" && sub != "which-self" {
        return false;
    }

    let detected = instance::close::detect_caller_instance_id();
    match detected {
        Some(caller_id) => {
            let (name, pid, data_dir, bound_email) =
                if let Ok(instances) = instance::list_instances() {
                    if let Some(inst) = instances.iter().find(|i| i.config.id == caller_id) {
                        (
                            inst.config.name.clone(),
                            inst.pid.or(inst.config.pid),
                            inst.config.data_dir.clone(),
                            inst.config.bound_email.clone(),
                        )
                    } else {
                        (caller_id.clone(), None, String::new(), None)
                    }
                } else {
                    (caller_id.clone(), None, String::new(), None)
                };

            if is_json {
                let data_payload = serde_json::json!({
                    "detected": true,
                    "id": caller_id,
                    "instance_id": caller_id,
                    "name": name,
                    "pid": pid,
                    "data_dir": data_dir,
                    "bound_email": bound_email,
                });
                let out = serde_json::json!({
                    "success": true,
                    "detected": true,
                    "id": caller_id,
                    "instance_id": caller_id,
                    "name": name,
                    "pid": pid,
                    "data_dir": data_dir,
                    "bound_email": bound_email,
                    "data": data_payload,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                println!("[INFO] Caller Instance Profile:");
                println!("  ID:        {}", caller_id);
                println!("  Name:      {}", name);
                if let Some(p) = pid {
                    println!("  PID:       {}", p);
                } else {
                    println!("  PID:       -");
                }
                println!("  Data Dir:  {}", data_dir);
                if let Some(email) = bound_email {
                    println!("  Account:   {}", email);
                }
            }
        }
        None => {
            if is_json {
                let data_payload = serde_json::json!({
                    "detected": false,
                    "instance_id": null,
                    "id": null,
                    "message": "Running in standalone terminal (no active IDE instance detected)"
                });
                let out = serde_json::json!({
                    "success": true,
                    "detected": false,
                    "instance_id": null,
                    "id": null,
                    "message": "Running in standalone terminal (no active IDE instance detected)",
                    "data": data_payload,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                println!("Running in standalone terminal (no active IDE instance detected)");
            }
        }
    }
    true
}

pub(crate) fn route_instances_test_lifecycle(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    if non_flag_args.is_empty() {
        return false;
    }

    let is_test_create = non_flag_args[0].eq_ignore_ascii_case("test-create")
        || non_flag_args[0].eq_ignore_ascii_case("create-test")
        || (non_flag_args.len() >= 2
            && non_flag_args[0].eq_ignore_ascii_case("test")
            && (non_flag_args[1].eq_ignore_ascii_case("create")
                || non_flag_args[1].eq_ignore_ascii_case("new")
                || non_flag_args[1].eq_ignore_ascii_case("add")));

    if is_test_create {
        let mut custom_name: Option<String> = None;
        let mut idx = 0;
        while idx < args.len() {
            if (args[idx] == "--name" || args[idx] == "-n") && idx + 1 < args.len() {
                custom_name = Some(args[idx + 1].clone());
                break;
            } else if let Some(val) = args[idx].strip_prefix("--name=") {
                custom_name = Some(val.to_string());
                break;
            }
            idx += 1;
        }

        if custom_name.is_none() {
            if non_flag_args[0].eq_ignore_ascii_case("test") {
                if non_flag_args.len() > 2 {
                    custom_name = Some(non_flag_args[2].clone());
                }
            } else if non_flag_args.len() > 1 {
                custom_name = Some(non_flag_args[1].clone());
            }
        }

        let timestamp = chrono::Utc::now().timestamp();
        let gen_name = match custom_name {
            Some(ref n) => {
                let clean = n.trim().trim_start_matches("test-");
                format!("test-{}-{}", clean, timestamp)
            }
            None => format!("test-{}", timestamp),
        };

        match instance::create_instance_with_account(gen_name, None) {
            Ok(cfg) => {
                if is_json {
                    let res = serde_json::json!({
                        "success": true,
                        "id": cfg.id,
                        "name": cfg.name,
                        "data_dir": cfg.data_dir,
                        "config": cfg.clone(),
                        "data": {
                            "id": cfg.id,
                            "name": cfg.name,
                            "data_dir": cfg.data_dir,
                            "config": cfg,
                        }
                    });
                    println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
                } else {
                    println!(
                        "[SUCCESS] Created test instance '{}' (ID: {}).",
                        cfg.name, cfg.id
                    );
                    println!("  Data directory: {}", cfg.data_dir);
                }
            }
            Err(e) => {
                if is_json {
                    let err_res = serde_json::json!({
                        "success": false,
                        "error": format!("Failed to create test instance: {}", e)
                    });
                    println!("{}", serde_json::to_string(&err_res).unwrap_or_default());
                }
                eprintln!("[ERROR] Failed to create test instance: {}", e);
                std::process::exit(1);
            }
        }
        return true;
    }

    let is_test_clean = non_flag_args[0].eq_ignore_ascii_case("test-clean")
        || non_flag_args[0].eq_ignore_ascii_case("test-destroy")
        || non_flag_args[0].eq_ignore_ascii_case("clean-test")
        || non_flag_args[0].eq_ignore_ascii_case("destroy-test")
        || non_flag_args[0].eq_ignore_ascii_case("rm-test")
        || (non_flag_args.len() >= 2
            && non_flag_args[0].eq_ignore_ascii_case("test")
            && (non_flag_args[1].eq_ignore_ascii_case("clean")
                || non_flag_args[1].eq_ignore_ascii_case("destroy")
                || non_flag_args[1].eq_ignore_ascii_case("rm")
                || non_flag_args[1].eq_ignore_ascii_case("delete")
                || non_flag_args[1].eq_ignore_ascii_case("purge")));

    if is_test_clean {
        let target_spec: Option<String> = if non_flag_args[0].eq_ignore_ascii_case("test") {
            if non_flag_args.len() > 2 {
                Some(non_flag_args[2].clone())
            } else {
                None
            }
        } else if non_flag_args.len() > 1 {
            Some(non_flag_args[1].clone())
        } else {
            None
        };

        let registry = match instance::load_registry() {
            Ok(reg) => reg,
            Err(e) => {
                if is_json {
                    let err_res = serde_json::json!({
                        "success": false,
                        "error": format!("Failed to load registry: {}", e)
                    });
                    println!("{}", serde_json::to_string(&err_res).unwrap_or_default());
                }
                eprintln!("[ERROR] Failed to load registry: {}", e);
                std::process::exit(1);
            }
        };

        let targets_to_clean: Vec<instance::InstanceConfig> = if let Some(ref target) = target_spec
        {
            let resolved = instance::resolve_instance_id(target).unwrap_or_else(|_| target.clone());
            registry
                .instances
                .iter()
                .filter(|inst| {
                    !inst.is_default
                        && (inst.id.eq_ignore_ascii_case(&resolved)
                            || inst.id.eq_ignore_ascii_case(target)
                            || inst.name.eq_ignore_ascii_case(target))
                })
                .cloned()
                .collect()
        } else {
            registry
                .instances
                .iter()
                .filter(|inst| {
                    !inst.is_default
                        && (inst.id.to_lowercase().starts_with("test-")
                            || inst.name.to_lowercase().starts_with("test-"))
                })
                .cloned()
                .collect()
        };

        let mut cleaned_count = 0usize;
        let mut cleaned_ids = Vec::new();

        for inst in targets_to_clean {
            refuse_if_caller(args, &inst.id, is_json);
            antigravity_tools_lib::error::record_ignored(
                instance::stop_instance(&inst.id),
                "stop test instance before deletion",
            ); // Justification: best-effort stop before deleting instance directory
            match instance::delete_instance(&inst.id) {
                Ok(_) => {
                    if !is_json {
                        println!("  [✓] Purged test instance '{}' ({})", inst.name, inst.id);
                    }
                    cleaned_count += 1;
                    cleaned_ids.push(inst.id);
                }
                Err(e) => {
                    eprintln!("  [!] Failed to delete test instance '{}': {}", inst.id, e);
                }
            }
        }

        // Clean any orphaned test directories on disk
        if target_spec.is_none() {
            if let Ok(instances_root) = instance::get_instances_dir() {
                if let Ok(entries) = std::fs::read_dir(&instances_root) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                if file_name.to_lowercase().starts_with("test-")
                                    && !cleaned_ids.iter().any(|id| id == file_name)
                                {
                                    antigravity_tools_lib::error::record_ignored(
                                        std::fs::remove_dir_all(&path),
                                        "remove orphaned test instance directory",
                                    ); // Justification: test cleanup; absence is safe
                                }
                            }
                        }
                    }
                }
            }
        }

        if is_json {
            let res = serde_json::json!({
                "success": true,
                "cleaned_count": cleaned_count,
                "cleaned_ids": cleaned_ids.clone(),
                "data": {
                    "cleaned_count": cleaned_count,
                    "cleaned_ids": cleaned_ids,
                }
            });
            println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
        } else {
            println!("[SUCCESS] Purged {} test instance(s).", cleaned_count);
        }
        return true;
    }

    false
}
