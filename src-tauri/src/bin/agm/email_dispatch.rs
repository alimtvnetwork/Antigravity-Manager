//! email_dispatch — CLI command handlers, split from agm.rs.

pub(crate) fn cmd_email(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json");
    let is_help_flag = args.iter().any(|a| a == "-h" || a == "--help");
    let sub = if is_help_flag {
        "--help".to_string()
    } else {
        args.first()
            .filter(|s| !s.starts_with('-'))
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "status".to_string())
    };

    match sub.as_str() {
        "help" | "-h" | "--help" => {
            crate::email_arms_a::email_arm_help(args, is_json, sub);
        }
        "status" => {
            crate::email_arms_a::email_arm_status(args, is_json);
        }
        "ls" | "list" => {
            crate::email_arms_b::email_arm_ls(args, is_json);
        }
        "add" => {
            crate::email_arms_b::email_arm_add(args, is_json);
        }
        "rm" | "remove" | "delete" => {
            crate::email_arms_b::email_arm_rm(args, is_json);
        }
        "mv" | "default" | "set-default" => {
            crate::email_arms_b::email_arm_mv(args, is_json);
        }
        "export" => {
            crate::email_arms_b::email_arm_export(args, is_json);
        }
        "import" => {
            crate::email_arms_b::email_arm_import(args, is_json);
        }
        _ => {
            eprintln!("Unknown email subcommand: '{}'. Run 'agm email help'.", sub);
            std::process::exit(1);
        }
    }
}
