use base64::prelude::*;

use super::*;

/// Helper extracting prefix length for prompt injection variants.
pub(crate) fn extract_prompt_injection_prefix_len(lower_subj: &str) -> Option<usize> {
    const PREFIXES: &[&str] = &[
        "project-prompt:",
        "project:",
        "prompt injection:",
        "prompt-injection:",
        "prompt:",
        "prompt ",
        "ai-task:",
        "ai:",
        "instruction:",
        "inject:",
    ];
    for &p in PREFIXES {
        if lower_subj.starts_with(p) {
            return Some(p.len());
        }
    }
    None
}

/// Helper extracting target node for fast-forward commands.
pub(crate) fn extract_fast_forward_target(clean_subj: &str, lower_subj: &str) -> Option<String> {
    if lower_subj == "ff"
        || lower_subj == "agm ff"
        || lower_subj == "agm smart-switch"
        || lower_subj == "fastforward"
        || lower_subj == "fast-forward"
    {
        return Some("*".to_string());
    }

    if lower_subj.starts_with("ff:") {
        let node = clean_subj["ff:".len()..].trim();
        return Some(if node.is_empty() {
            "*".to_string()
        } else {
            node.to_string()
        });
    }

    if lower_subj.starts_with("fast-forward:") {
        let node = clean_subj["fast-forward:".len()..].trim();
        return Some(if node.is_empty() {
            "*".to_string()
        } else {
            node.to_string()
        });
    }

    None
}

/// Helper extracting target IP for exec / command directives.
pub(crate) fn extract_exec_target(clean_subj: &str, lower_subj: &str) -> Option<String> {
    if lower_subj.starts_with("exec:") {
        return Some(clean_subj["exec:".len()..].trim().to_string());
    }
    if lower_subj.starts_with("command:") {
        return Some(clean_subj["command:".len()..].trim().to_string());
    }
    None
}

/// Helper extracting account switch query.
pub(crate) fn extract_account_switch_query(
    clean_subj: &str,
    lower_subj: &str,
    body: &str,
) -> Option<String> {
    if lower_subj.starts_with("agm switch ") {
        return Some(clean_subj["agm switch ".len()..].trim().to_string());
    }
    if lower_subj.starts_with("switch ") {
        return Some(clean_subj["switch ".len()..].trim().to_string());
    }
    if lower_subj == "switch" || lower_subj == "agm switch" {
        return Some(body.trim().to_string());
    }
    None
}

/// Parse legacy and flexible prefix command formats.
pub(crate) fn parse_legacy_prefix_command(
    clean_subj: &str,
    lower_subj: &str,
    body: &str,
    command_str: &str,
) -> InboundAction {
    // 1. Legacy and Flexible Prefix Support for Prompt Injection
    if let Some(prefix_len) = extract_prompt_injection_prefix_len(lower_subj) {
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

    // 2. Exec / Command
    if let Some(target_ip) = extract_exec_target(clean_subj, lower_subj) {
        return InboundAction::CliExecution {
            target_ip,
            command: body.trim().to_string(),
        };
    }

    // 3. Instance Create
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

    // 4. Account Rotate
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

    // 5. Fast Forward
    if let Some(target_node) = extract_fast_forward_target(clean_subj, lower_subj) {
        return InboundAction::FastForward {
            target_node,
            instance_id: None,
        };
    }

    // 6. Multi-node cluster query
    if lower_subj.contains("how many machines")
        || lower_subj.contains("nodes running")
        || lower_subj.contains("cluster")
    {
        return InboundAction::MultiNodeSnapshotQuery;
    }

    // 7. Status Query
    if lower_subj == "status"
        || lower_subj.starts_with("query")
        || lower_subj == "fleet"
        || lower_subj == "whoami"
        || lower_subj == "machines"
        || lower_subj == "agm fleet"
        || lower_subj == "agm whoami"
    {
        return InboundAction::StatusQuery {
            target: "*".to_string(),
        };
    }

    // 8. Check Named Prompt Execution
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
        let effective_query = if query.is_empty() {
            body_first
        } else {
            query
        };
        return InboundAction::NamedPromptExecution {
            prompt_query: effective_query,
        };
    }

    // 9. Help
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

    // 10. Accounts
    if lower_subj == "accounts"
        || lower_subj == "acc"
        || lower_subj == "agm accounts"
        || lower_subj == "agm acc"
    {
        return InboundAction::ListAccounts {
            target: "*".to_string(),
        };
    }

    // 11. Instances
    if lower_subj == "instances"
        || lower_subj == "ls"
        || lower_subj == "agm instances"
        || lower_subj == "agm ls"
    {
        return InboundAction::ListInstances {
            target: "*".to_string(),
        };
    }

    // 12. Doctor
    if lower_subj == "doctor"
        || lower_subj == "check"
        || lower_subj == "agm doctor"
        || lower_subj == "agm check"
    {
        return InboundAction::DoctorDiagnostic {
            target: "*".to_string(),
        };
    }

    // 13. Clean
    if lower_subj == "clean"
        || lower_subj == "purge"
        || lower_subj == "agm clean"
        || lower_subj == "agm purge"
    {
        return InboundAction::SystemClean {
            target: "*".to_string(),
        };
    }

    // 14. Sync
    if lower_subj == "sync" || lower_subj == "agm sync" {
        return InboundAction::SyncState {
            target: "*".to_string(),
        };
    }

    // 15. Proxy
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

    // 16. Prompts
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

    // 17. GitMap
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

    // 18. Switch
    if let Some(email_query) = extract_account_switch_query(clean_subj, lower_subj, body) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_prompt_injection_prefix_len() {
        assert_eq!(
            extract_prompt_injection_prefix_len("project: my-proj"),
            Some("project:".len())
        );
        assert_eq!(
            extract_prompt_injection_prefix_len("ai-task: refactor"),
            Some("ai-task:".len())
        );
        assert_eq!(
            extract_prompt_injection_prefix_len("prompt write code"),
            Some("prompt ".len())
        );
        assert_eq!(extract_prompt_injection_prefix_len("unknown command"), None);
    }

    #[test]
    fn test_extract_fast_forward_target() {
        assert_eq!(
            extract_fast_forward_target("ff", "ff"),
            Some("*".to_string())
        );
        assert_eq!(
            extract_fast_forward_target("ff: node-1", "ff: node-1"),
            Some("node-1".to_string())
        );
        assert_eq!(
            extract_fast_forward_target("fast-forward: worker-2", "fast-forward: worker-2"),
            Some("worker-2".to_string())
        );
        assert_eq!(
            extract_fast_forward_target("status", "status"),
            None
        );
    }

    #[test]
    fn test_extract_exec_target() {
        assert_eq!(
            extract_exec_target("exec: 192.168.1.5", "exec: 192.168.1.5"),
            Some("192.168.1.5".to_string())
        );
        assert_eq!(
            extract_exec_target("command: 10.0.0.1", "command: 10.0.0.1"),
            Some("10.0.0.1".to_string())
        );
        assert_eq!(extract_exec_target("status", "status"), None);
    }

    #[test]
    fn test_extract_account_switch_query() {
        assert_eq!(
            extract_account_switch_query("agm switch user@example.com", "agm switch user@example.com", ""),
            Some("user@example.com".to_string())
        );
        assert_eq!(
            extract_account_switch_query("switch", "switch", "alt@domain.org"),
            Some("alt@domain.org".to_string())
        );
    }
}
