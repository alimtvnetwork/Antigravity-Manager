//! instance_routes_b — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};

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
        if matches.is_empty() {
            eprintln!("[ERROR] No account found matching '{}'.", acc_query);
            std::process::exit(1);
        }
        let target_acc = if matches.len() == 1 {
            matches[0]
        } else if let Some(exact) = matches.iter().find(|a| a.email.to_lowercase() == acc_query) {
            *exact
        } else {
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
                if is_json {
                    let res = serde_json::json!({
                        "success": true,
                        "instance": resolved_id,
                        "email": target_acc.email,
                        "previous_email": previous_email,
                        "account_id": target_acc.id
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
                    if is_json {
                        let active_acc = instance::load_registry()
                            .ok()
                            .and_then(|r| r.instances.into_iter().find(|i| i.id == resolved_id))
                            .and_then(|i| i.bound_email)
                            .unwrap_or_default();
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
