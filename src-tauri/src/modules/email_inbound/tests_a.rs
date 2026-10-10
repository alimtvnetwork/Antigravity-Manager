#![cfg(test)]

use super::*;

use super::*;

#[test]
pub(crate) fn test_parse_project_command_with_re_prefix() {
    let action = parse_email_command("Re: Project: my-awesome-app", "Please fix bug #12");
    match action {
        InboundAction::PromptInjection {
            project_name,
            prompt,
            ..
        } => {
            assert_eq!(project_name, "my-awesome-app");
            assert_eq!(prompt, "Please fix bug #12");
        }
        _ => panic!("Expected PromptInjection"),
    }
}

#[test]
pub(crate) fn test_parse_pipe_prompt_with_instance_and_project() {
    let body = "prompt-name: fix-auth\nprompt instruction:\nPlease refactor auth middleware";
    let action = parse_email_command("sub: worker-1 | ins-default | prompt | proj-web", body);
    match action {
        InboundAction::PromptInjection {
            project_name,
            prompt_name,
            prompt,
            instance_id,
        } => {
            assert_eq!(project_name, "web");
            assert_eq!(prompt_name, "fix-auth");
            assert_eq!(prompt, "Please refactor auth middleware");
            assert_eq!(instance_id, Some("default".to_string()));
        }
        _ => panic!("Expected PromptInjection with pipe grammar"),
    }
}

#[test]
pub(crate) fn test_parse_pipe_ip_octet_prompt() {
    let action = parse_email_command("12 | prompt | proj-test", "Fix lint error");
    match action {
        InboundAction::PromptInjection {
            project_name,
            prompt,
            instance_id,
            ..
        } => {
            assert_eq!(project_name, "test");
            assert_eq!(prompt, "Fix lint error");
            assert_eq!(instance_id, None);
        }
        _ => panic!("Expected PromptInjection"),
    }
}

#[test]
pub(crate) fn test_parse_pipe_gitmap_status() {
    let action = parse_email_command("* | gitmap | status", "");
    match action {
        InboundAction::GitMapExecution { target, command } => {
            assert_eq!(target, "*");
            assert_eq!(command, "status");
        }
        _ => panic!("Expected GitMapExecution"),
    }
}

#[test]
pub(crate) fn test_parse_pipe_cmd() {
    let action = parse_email_command("local | cmd", "Get-Process");
    match action {
        InboundAction::CliExecution { target_ip, command } => {
            assert_eq!(target_ip, "local");
            assert_eq!(command, "Get-Process");
        }
        _ => panic!("Expected CliExecution"),
    }
}

#[test]
pub(crate) fn test_parse_pipe_agm_commands() {
    assert_eq!(
        parse_email_command("VM3 | agm update", ""),
        InboundAction::UpdateExecution {
            target: "VM3".to_string(),
            is_gitmap: false
        }
    );
    assert_eq!(
        parse_email_command("VM3 | gitmap update", ""),
        InboundAction::UpdateExecution {
            target: "VM3".to_string(),
            is_gitmap: true
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm status", ""),
        InboundAction::StatusQuery {
            target: "VM3".to_string()
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm doctor", ""),
        InboundAction::DoctorDiagnostic {
            target: "VM3".to_string()
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm check", ""),
        InboundAction::DoctorDiagnostic {
            target: "VM3".to_string()
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm accounts", ""),
        InboundAction::ListAccounts {
            target: "VM3".to_string()
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm acc", ""),
        InboundAction::ListAccounts {
            target: "VM3".to_string()
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm switch | abidul@example.com", ""),
        InboundAction::AccountSwitch {
            target: "VM3".to_string(),
            email_query: "abidul@example.com".to_string(),
            instance_id: None,
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm proxy", ""),
        InboundAction::ProxyStatus {
            target: "VM3".to_string(),
            is_test: false,
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm proxy test", ""),
        InboundAction::ProxyStatus {
            target: "VM3".to_string(),
            is_test: true,
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm clean", ""),
        InboundAction::SystemClean {
            target: "VM3".to_string(),
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm purge", ""),
        InboundAction::SystemClean {
            target: "VM3".to_string(),
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm sync", ""),
        InboundAction::SyncState {
            target: "VM3".to_string(),
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm prompts", ""),
        InboundAction::ListPrompts {
            target: "VM3".to_string(),
            is_gitmap: false,
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm instances", ""),
        InboundAction::ListInstances {
            target: "VM3".to_string(),
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm ls", ""),
        InboundAction::ListInstances {
            target: "VM3".to_string(),
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm ff", ""),
        InboundAction::FastForward {
            target_node: "VM3".to_string(),
            instance_id: None,
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agm smart-switch", ""),
        InboundAction::FastForward {
            target_node: "VM3".to_string(),
            instance_id: None,
        }
    );
    assert_eq!(
        parse_email_command("VM3 | agy prompts ls", ""),
        InboundAction::ListPrompts {
            target: "VM3".to_string(),
            is_gitmap: false
        }
    );
    assert_eq!(
        parse_email_command("VM3 | gitmap prompts ls", ""),
        InboundAction::ListPrompts {
            target: "VM3".to_string(),
            is_gitmap: true
        }
    );
}

#[test]
pub(crate) fn test_matches_target_node_or_ip() {
    let local_ip = "192.168.1.12";
    let local_name = "VM3";

    // Wildcards & local
    assert!(matches_target_node_or_ip("*", local_ip, local_name));
    assert!(matches_target_node_or_ip("all", local_ip, local_name));
    assert!(matches_target_node_or_ip("any", local_ip, local_name));
    assert!(matches_target_node_or_ip("local", local_ip, local_name));
    assert!(matches_target_node_or_ip("localhost", local_ip, local_name));

    // Node name match
    assert!(matches_target_node_or_ip("vm3", local_ip, local_name));
    assert!(matches_target_node_or_ip("VM3", local_ip, local_name));

    // Full IP match
    assert!(matches_target_node_or_ip(
        "192.168.1.12",
        local_ip,
        local_name
    ));

    // Octet match
    assert!(matches_target_node_or_ip("12", local_ip, local_name));

    // Mismatches
    assert!(!matches_target_node_or_ip("VM4", local_ip, local_name));
    assert!(!matches_target_node_or_ip(
        "192.168.1.99",
        local_ip,
        local_name
    ));
    assert!(!matches_target_node_or_ip("99", local_ip, local_name));
}

#[test]
pub(crate) fn test_extract_email_address() {
    assert_eq!(
        extract_email_address("John Doe <john@example.com>"),
        "john@example.com"
    );
    assert_eq!(
        extract_email_address("admin@example.com"),
        "admin@example.com"
    );
}
