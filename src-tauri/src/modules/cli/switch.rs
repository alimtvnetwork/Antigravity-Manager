use crate::modules::{account, auto_switcher, config, instance, repo_db};

use super::*;

// -----------------------------------------------------------------------------
// Switch Account Command Handler
// -----------------------------------------------------------------------------

pub(crate) fn handle_switch_command(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_switch_cli_help();
        std::process::exit(0);
    }

    let is_json = is_flag_present(args, &["--json", "-j"]);
    let raw_non_flags: Vec<&String> = args
        .iter()
        .filter(|a| {
            !a.starts_with('-')
                && !a.eq_ignore_ascii_case("switch")
                && !a.eq_ignore_ascii_case("swtich")
                && !a.eq_ignore_ascii_case("use")
        })
        .collect();

    // Strip semantic filler keywords like "account", "to", "for"
    let mut non_flag_args: Vec<&String> = Vec::new();
    for (idx, arg) in raw_non_flags.iter().enumerate() {
        let lower = arg.to_lowercase();
        if (lower == "account" || lower == "acc" || lower == "to" || lower == "for")
            && (idx == 0 || idx + 1 < raw_non_flags.len())
            && raw_non_flags.len() > 1
        {
            continue;
        }
        non_flag_args.push(arg);
    }

    let index = match account::load_account_index() {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("[ERROR] Failed to load accounts index: {}", e);
            std::process::exit(1);
        }
    };

    if non_flag_args.is_empty() {
        let curr = account::get_current_account().ok().flatten();
        let active_inst =
            instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());
        println!("AGM Profile Switch:");
        println!("  Current active instance: {}", active_inst);
        println!(
            "  Current active account:  {}",
            curr.as_ref().map(|a| a.email.as_str()).unwrap_or("(none)")
        );
        println!("\nUsage: antigravity-manager switch <email|id|#seq> [--instance <id>]");
        println!("       antigravity-manager switch <instance> <account>");
        println!("       antigravity-manager switch account <email>");
        println!("Run 'antigravity-manager switch --help' for full guide with examples.");
        std::process::exit(1);
    }

    let explicit_inst_opt = args
        .iter()
        .position(|a| a == "--instance" || a == "-i")
        .and_then(|pos| args.get(pos + 1).map(|s| s.as_str()));

    let (target_inst_spec, acc_query) = if non_flag_args.len() >= 2 {
        let first_is_instance = instance::resolve_instance_id(non_flag_args[0]).is_ok();
        let second_is_instance = instance::resolve_instance_id(non_flag_args[1]).is_ok();

        if first_is_instance && !second_is_instance {
            (Some(non_flag_args[0].as_str()), non_flag_args[1].as_str())
        } else if second_is_instance && !first_is_instance {
            (Some(non_flag_args[1].as_str()), non_flag_args[0].as_str())
        } else if explicit_inst_opt.is_some() {
            (explicit_inst_opt, non_flag_args[0].as_str())
        } else {
            (Some(non_flag_args[0].as_str()), non_flag_args[1].as_str())
        }
    } else {
        (explicit_inst_opt, non_flag_args[0].as_str())
    };

    let query_lower = acc_query.trim().to_lowercase();
    let index_match =
        if query_lower.starts_with('#') || query_lower.chars().all(|c| c.is_ascii_digit()) {
            let num_str = query_lower.trim_start_matches('#');
            if let Ok(num) = num_str.parse::<usize>() {
                if num >= 1 && num <= index.accounts.len() {
                    Some(&index.accounts[num - 1])
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

    let target_account = if let Some(acc) = index_match {
        acc
    } else {
        let matches: Vec<_> = index
            .accounts
            .iter()
            .filter(|a| {
                let email_l = a.email.to_lowercase();
                let id_l = a.id.to_lowercase();
                email_l == query_lower
                    || id_l == query_lower
                    || email_l.starts_with(&query_lower)
                    || email_l.contains(&query_lower)
                    || id_l.contains(&query_lower)
            })
            .collect();

        if matches.is_empty() {
            eprintln!("[ERROR] No account found matching '{}'.", acc_query);
            std::process::exit(1);
        } else if matches.len() == 1 {
            matches[0]
        } else if let Some(exact) = matches
            .iter()
            .find(|a| a.email.to_lowercase() == query_lower)
        {
            *exact
        } else {
            eprintln!(
                "[ERROR] Ambiguous query '{}' matched multiple accounts:",
                acc_query
            );
            for m in &matches {
                eprintln!("  - {} ({})", m.email, m.id);
            }
            std::process::exit(1);
        }
    };

    let resolved_inst_id = match target_inst_spec {
        Some(s) => match instance::resolve_instance_id(s) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("[ERROR] Failed to resolve target instance '{}': {}", s, e);
                std::process::exit(1);
            }
        },
        None => instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string()),
    };

    println!(
        "[*] Switching instance '{}' to account '{}' ({})...",
        resolved_inst_id, target_account.email, target_account.id
    );

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    if let Err(e) = rt.block_on(instance::switch_account_to_instance(
        &target_account.id,
        Some(&resolved_inst_id),
    )) {
        eprintln!("[ERROR] Account switch failed: {}", e);
        std::process::exit(1);
    }

    if is_json {
        let res = serde_json::json!({
            "status": "success",
            "instance_id": resolved_inst_id,
            "account_id": target_account.id,
            "account_email": target_account.email,
        });
        println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
    } else {
        println!(
            "[SUCCESS] Instance '{}' successfully switched to '{}'.",
            resolved_inst_id, target_account.email
        );
        println!("  [OK] Credentials injected into instance state.vscdb.");
        println!("  [OK] Running prompts backed up and preserved.");
    }
    std::process::exit(0);
}
