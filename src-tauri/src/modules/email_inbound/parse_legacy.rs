use base64::prelude::*;

use super::*;

/// Parse legacy and flexible prefix command formats.
pub(crate) fn parse_legacy_prefix_command(
    clean_subj: &str,
    lower_subj: &str,
    body: &str,
    command_str: &str,
) -> InboundAction {
    // 2. Legacy and Flexible Prefix Support
    if lower_subj.starts_with("project:")
        || lower_subj.starts_with("project-prompt:")
        || lower_subj.starts_with("prompt:")
        || lower_subj.starts_with("prompt ")
        || lower_subj.starts_with("prompt injection:")
        || lower_subj.starts_with("prompt-injection:")
        || lower_subj.starts_with("ai:")
        || lower_subj.starts_with("ai-task:")
        || lower_subj.starts_with("instruction:")
        || lower_subj.starts_with("inject:")
    {
        let prefix_len = if lower_subj.starts_with("project-prompt:") {
            "project-prompt:".len()
        } else if lower_subj.starts_with("project:") {
            "project:".len()
        } else if lower_subj.starts_with("prompt injection:") {
            "prompt injection:".len()
        } else if lower_subj.starts_with("prompt-injection:") {
            "prompt-injection:".len()
        } else if lower_subj.starts_with("prompt:") {
            "prompt:".len()
        } else if lower_subj.starts_with("prompt ") {
            "prompt ".len()
        } else if lower_subj.starts_with("ai-task:") {
            "ai-task:".len()
        } else if lower_subj.starts_with("ai:") {
            "ai:".len()
        } else if lower_subj.starts_with("instruction:") {
            "instruction:".len()
        } else if lower_subj.starts_with("inject:") {
            "inject:".len()
        } else {
            0
        };
        let project_name = clean_subj[prefix_len..].trim().to_string();
        let (p_name, p_inst) = parse_prompt_body(body);
        let final_prompt = if p_inst.is_empty() {
            body.trim().to_string()
        } else {
            p_inst
        };
        return InboundAction::PromptInjection {
            project_name: if project_name.is_empty() {
                "Default".to_string()
            } else {
                project_name
            },
            prompt_name: p_name,
            prompt: final_prompt,
            instance_id: None,
        };
    }

    if lower_subj.starts_with("exec:") || lower_subj.starts_with("command:") {
        let prefix_len = if lower_subj.starts_with("exec:") {
            "exec:".len()
        } else {
            "command:".len()
        };
        let target_ip = clean_subj[prefix_len..].trim().to_string();
        return InboundAction::CliExecution {
            target_ip,
            command: body.trim().to_string(),
        };
    }

    if lower_subj.starts_with("instance: new") || lower_subj.starts_with("instance: create") {
        let profile = body
            .trim()
            .lines()
            .next()
            .unwrap_or("email-spawned")
            .trim()
            .to_string();
        return InboundAction::InstanceCreate {
            profile_name: if profile.is_empty() {
                "email-spawned".to_string()
            } else {
                profile
            },
        };
    }

    if lower_subj.starts_with("rotate: accounts")
        || lower_subj.starts_with("account: rotate")
        || lower_subj == "rotate"
        || lower_subj == "agm rotate"
    {
        return InboundAction::AccountRotate {
            target: "*".to_string(),
            instance_id: None,
        };
    }

    if lower_subj == "ff"
        || lower_subj.starts_with("ff:")
        || lower_subj.starts_with("fast-forward")
        || lower_subj.starts_with("fastforward")
        || lower_subj == "agm ff"
        || lower_subj == "agm smart-switch"
    {
        let prefix_len = if lower_subj.starts_with("ff:") {
            "ff:".len()
        } else if lower_subj.starts_with("fast-forward:") {
            "fast-forward:".len()
        } else {
            0
        };
        let target_node = if prefix_len > 0 {
            clean_subj[prefix_len..].trim().to_string()
        } else {
            "*".to_string()
        };
        return InboundAction::FastForward {
            target_node,
            instance_id: None,
        };
    }

    if lower_subj.contains("how many machines")
        || lower_subj.contains("nodes running")
        || lower_subj.contains("cluster")
    {
        return InboundAction::MultiNodeSnapshotQuery;
    }

    if lower_subj == "status" || lower_subj.starts_with("query") {
        return InboundAction::StatusQuery {
            target: "*".to_string(),
        };
    }

    // Check Named Prompt Execution
    let is_named = lower_subj.starts_with("named-prompt:");
    let is_run = lower_subj.starts_with("run-prompt:");
    let is_prompt = lower_subj.starts_with("prompt:");
    if is_named || is_run || is_prompt {
        let prefix_len = if is_named {
            "named-prompt:".len()
        } else if is_run {
            "run-prompt:".len()
        } else {
            "prompt:".len()
        };
        let query = clean_subj[prefix_len..].trim().to_string();
        let body_first = body.trim().lines().next().unwrap_or("").trim().to_string();
        let effective_query = if query.is_empty() { body_first } else { query };
        return InboundAction::NamedPromptExecution {
            prompt_query: effective_query,
        };
    }

    if lower_subj == "help"
        || lower_subj == "man"
        || lower_subj == "manual"
        || lower_subj == "agm help"
    {
        return InboundAction::HelpRequest {
            target: "*".to_string(),
            instance_id: None,
        };
    }

    if lower_subj == "accounts"
        || lower_subj == "acc"
        || lower_subj == "agm accounts"
        || lower_subj == "agm acc"
    {
        return InboundAction::ListAccounts {
            target: "*".to_string(),
        };
    }

    if lower_subj == "instances"
        || lower_subj == "ls"
        || lower_subj == "agm instances"
        || lower_subj == "agm ls"
    {
        return InboundAction::ListInstances {
            target: "*".to_string(),
        };
    }

    if lower_subj == "doctor"
        || lower_subj == "check"
        || lower_subj == "agm doctor"
        || lower_subj == "agm check"
    {
        return InboundAction::DoctorDiagnostic {
            target: "*".to_string(),
        };
    }

    if lower_subj == "clean"
        || lower_subj == "purge"
        || lower_subj == "agm clean"
        || lower_subj == "agm purge"
    {
        return InboundAction::SystemClean {
            target: "*".to_string(),
        };
    }

    if lower_subj == "sync" || lower_subj == "agm sync" {
        return InboundAction::SyncState {
            target: "*".to_string(),
        };
    }

    if lower_subj == "proxy" || lower_subj == "agm proxy" || lower_subj == "agm proxy status" {
        return InboundAction::ProxyStatus {
            target: "*".to_string(),
            is_test: false,
        };
    }

    if lower_subj == "proxy test" || lower_subj == "agm proxy test" {
        return InboundAction::ProxyStatus {
            target: "*".to_string(),
            is_test: true,
        };
    }

    if lower_subj == "prompts"
        || lower_subj == "agm prompts"
        || lower_subj == "agm prompts ls"
        || lower_subj == "agy prompts ls"
    {
        return InboundAction::ListPrompts {
            target: "*".to_string(),
            is_gitmap: false,
        };
    }

    if lower_subj.starts_with("gitmap ") || lower_subj == "gitmap" {
        let sub = if lower_subj.starts_with("gitmap ") {
            clean_subj["gitmap ".len()..].trim().to_string()
        } else {
            "status".to_string()
        };
        return InboundAction::GitMapExecution {
            target: "*".to_string(),
            command: sub,
        };
    }

    if lower_subj == "switch"
        || lower_subj == "agm switch"
        || lower_subj.starts_with("switch ")
        || lower_subj.starts_with("agm switch ")
    {
        let email_query = if lower_subj.starts_with("agm switch ") {
            clean_subj["agm switch ".len()..].trim().to_string()
        } else if lower_subj.starts_with("switch ") {
            clean_subj["switch ".len()..].trim().to_string()
        } else {
            body.trim().to_string()
        };
        return InboundAction::AccountSwitch {
            target: "*".to_string(),
            email_query,
            instance_id: None,
        };
    }

    InboundAction::Ignored {
        reason: format!(
            "Input '{}' does not match any recognized command pattern",
            command_str
        ),
    }
}

/// Parse single command line string and body into strongly typed InboundAction
pub fn parse_single_command_string(command_str: &str, body: &str) -> InboundAction {
    let clean_subj = strip_email_prefixes(command_str);
    let lower_subj = clean_subj.to_lowercase();

    // 1. Unified Pipe-Delimited Grammar
    if clean_subj.contains('|') {
        if let Some(action) = parse_pipe_delimited_command(&clean_subj, &lower_subj, body) {
            return action;
        }
    }

    // 2. Legacy and Flexible Prefix Support
    parse_legacy_prefix_command(&clean_subj, &lower_subj, body, command_str)
}
