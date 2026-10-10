use base64::prelude::*;

use super::*;

/// Parse single command line string and body into strongly typed InboundAction
/// Parse the unified pipe-delimited command grammar.
/// Returns `Some(action)` when the grammar matches, `None` to fall through to legacy parsing.
pub(crate) fn parse_pipe_delimited_command(
    clean_subj: &str,
    lower_subj: &str,
    body: &str,
) -> Option<InboundAction> {
    let parts: Vec<&str> = clean_subj
        .split('|')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if !parts.is_empty() {
        let target = parts[0];
        let mut instance_id: Option<String> = None;
        let mut cmd_str = "";
        let mut proj_str = "";

        let mut arg_str = "";
        if parts.len() >= 3 {
            if is_instance_specifier(parts[1])
                || (!is_known_command(parts[1]) && is_known_command(parts[2]))
            {
                instance_id = Some(extract_clean_instance_id(parts[1]));
                cmd_str = parts[2];
                if parts.len() >= 4 {
                    arg_str = parts[3];
                    proj_str = parts[3];
                }
            } else {
                cmd_str = parts[1];
                arg_str = parts[2];
                if parts.len() >= 4 {
                    proj_str = parts[3];
                } else if parts[2].to_lowercase().starts_with("proj-")
                    || parts[2].to_lowercase().starts_with("project:")
                {
                    proj_str = parts[2];
                }
            }
        } else if parts.len() >= 2 {
            if is_instance_specifier(parts[1]) {
                instance_id = Some(extract_clean_instance_id(parts[1]));
                cmd_str = "status";
            } else {
                cmd_str = parts[1];
            }
        } else {
            cmd_str = "help";
        }

        let project_name = if !proj_str.is_empty() {
            let lower_p = proj_str.to_lowercase();
            if lower_p.starts_with("proj-") {
                proj_str["proj-".len()..].trim().to_string()
            } else if lower_p.starts_with("project:") {
                proj_str["project:".len()..].trim().to_string()
            } else {
                proj_str.to_string()
            }
        } else {
            String::new()
        };

        let lower_cmd = cmd_str.to_lowercase();

        if lower_cmd == "prompt" {
            let (p_name, p_inst) = parse_prompt_body(body);
            return Some(InboundAction::PromptInjection {
                project_name,
                prompt_name: p_name,
                prompt: p_inst,
                instance_id,
            });
        }

        if lower_cmd == "gitmap update" {
            return Some(InboundAction::UpdateExecution {
                target: target.to_string(),
                is_gitmap: true,
            });
        }

        if lower_cmd == "gitmap prompts ls" {
            return Some(InboundAction::ListPrompts {
                target: target.to_string(),
                is_gitmap: true,
            });
        }

        if lower_cmd.starts_with("gitmap macro") {
            let macro_name = if lower_cmd.len() > "gitmap macro".len() {
                cmd_str["gitmap macro".len()..].trim()
            } else {
                body.trim()
            };
            return Some(InboundAction::GitMapExecution {
                target: target.to_string(),
                command: format!("macro {}", macro_name).trim().to_string(),
            });
        }

        if lower_cmd == "gitmap" || lower_cmd.starts_with("gitmap ") {
            let cmd_arg = if lower_cmd.starts_with("gitmap ") {
                cmd_str["gitmap ".len()..].trim().to_string()
            } else if parts.len() >= 3 {
                let p2 = parts[2].to_lowercase();
                if p2.starts_with("proj-") {
                    let raw = body.trim();
                    if raw.to_lowercase().starts_with("gitmap ") {
                        raw["gitmap ".len()..].trim().to_string()
                    } else if raw.is_empty() {
                        "status".to_string()
                    } else {
                        raw.to_string()
                    }
                } else {
                    parts[2].trim().to_string()
                }
            } else {
                let raw = body.trim();
                if raw.to_lowercase().starts_with("gitmap ") {
                    raw["gitmap ".len()..].trim().to_string()
                } else if raw.is_empty() {
                    "status".to_string()
                } else {
                    raw.to_string()
                }
            };

            let clean_arg = if cmd_arg.to_lowercase().starts_with("gitmap ") {
                cmd_arg["gitmap ".len()..].trim().to_string()
            } else {
                cmd_arg
            };

            return Some(InboundAction::GitMapExecution {
                target: target.to_string(),
                command: clean_arg,
            });
        }

        if lower_cmd == "cmd" {
            return Some(InboundAction::CliExecution {
                target_ip: target.to_string(),
                command: body.trim().to_string(),
            });
        }

        if lower_cmd == "update" || lower_cmd == "agm update" {
            return Some(InboundAction::UpdateExecution {
                target: target.to_string(),
                is_gitmap: false,
            });
        }

        if lower_cmd == "ls" || lower_cmd == "agm ls" || lower_cmd == "agm instances" {
            return Some(InboundAction::ListInstances {
                target: target.to_string(),
            });
        }

        if lower_cmd == "help" {
            return Some(InboundAction::HelpRequest {
                target: target.to_string(),
                instance_id: instance_id.clone(),
            });
        }

        if lower_cmd == "status" || lower_cmd == "agm status" {
            return Some(InboundAction::StatusQuery {
                target: target.to_string(),
            });
        }

        if lower_cmd == "agm ff"
            || lower_cmd == "agm smart-switch"
            || lower_cmd == "agm ff/smart-switch"
            || lower_cmd == "ff"
        {
            return Some(InboundAction::FastForward {
                target_node: target.to_string(),
                instance_id,
            });
        }

        if lower_cmd == "rotate" || lower_cmd == "agm rotate" {
            return Some(InboundAction::AccountRotate {
                target: target.to_string(),
                instance_id,
            });
        }

        if lower_cmd == "agy prompts ls"
            || lower_cmd == "agm prompts"
            || lower_cmd == "agm prompts ls"
            || lower_cmd == "prompts"
        {
            return Some(InboundAction::ListPrompts {
                target: target.to_string(),
                is_gitmap: false,
            });
        }

        if lower_cmd == "doctor"
            || lower_cmd == "agm doctor"
            || lower_cmd == "check"
            || lower_cmd == "agm check"
        {
            return Some(InboundAction::DoctorDiagnostic {
                target: target.to_string(),
            });
        }

        if lower_cmd == "accounts"
            || lower_cmd == "agm accounts"
            || lower_cmd == "agm acc"
            || lower_cmd == "acc"
        {
            return Some(InboundAction::ListAccounts {
                target: target.to_string(),
            });
        }

        if lower_cmd.starts_with("agm switch") || lower_cmd.starts_with("switch") {
            let inline_query = if lower_cmd.starts_with("agm switch") {
                cmd_str["agm switch".len()..].trim().to_string()
            } else if lower_cmd.starts_with("switch") {
                cmd_str["switch".len()..].trim().to_string()
            } else {
                String::new()
            };
            let query = if !inline_query.is_empty() {
                inline_query
            } else if !arg_str.is_empty() {
                arg_str.trim().to_string()
            } else {
                body.trim().to_string()
            };
            return Some(InboundAction::AccountSwitch {
                target: target.to_string(),
                email_query: query,
                instance_id,
            });
        }

        if lower_cmd == "proxy" || lower_cmd == "agm proxy" || lower_cmd == "agm proxy status" {
            return Some(InboundAction::ProxyStatus {
                target: target.to_string(),
                is_test: false,
            });
        }

        if lower_cmd == "agm proxy test" || lower_cmd == "proxy test" {
            return Some(InboundAction::ProxyStatus {
                target: target.to_string(),
                is_test: true,
            });
        }

        if lower_cmd == "clean"
            || lower_cmd == "agm clean"
            || lower_cmd == "purge"
            || lower_cmd == "agm purge"
        {
            return Some(InboundAction::SystemClean {
                target: target.to_string(),
            });
        }

        if lower_cmd == "sync" || lower_cmd == "agm sync" {
            return Some(InboundAction::SyncState {
                target: target.to_string(),
            });
        }
    }
    None
}
