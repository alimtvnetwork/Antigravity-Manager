use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn switch_payload_keeps_from_to_reason_and_reinject() {
        let raw = switch_payload(&SwitchFacts {
            from_email: "alpha@gmail.com".to_string(),
            to_email: "beta@gmail.com".to_string(),
            reason: "Manual account switch".to_string(),
            how: "Closed the IDE, wrote the account, opened the IDE.".to_string(),
            prompt_id: "p1".to_string(),
            prompt_text: "keep going".to_string(),
            conversation_id: "conv-same".to_string(),
            prompt_reinjected: true,
            switch_ok: true,
            instance_id: "default".to_string(),
            ide_type: "antigravity".to_string(),
            idc_machine_alias: "node-1".to_string(),
            ide_path: "/usr/bin/antigravity".to_string(),
            switch_reason: "Manual account switch".to_string(),
            steps: None,
        });
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(value["from_email"], "alpha@gmail.com");
        assert_eq!(value["to_email"], "beta@gmail.com");
        assert_eq!(value["reason"], "Manual account switch");
        assert_eq!(value["prompt_text"], "keep going");
        assert_eq!(value["conversation_id"], "conv-same");
        assert_eq!(value["prompt_reinjected"], true);
        assert_eq!(value["instance_id"], "default");
        assert_eq!(value["ide_type"], "antigravity");
        assert_eq!(value["idc_machine_alias"], "node-1");
        assert_eq!(value["ide_path"], "/usr/bin/antigravity");
        assert_eq!(value["switch_reason"], "Manual account switch");
        assert_eq!(value.get("steps"), None);
        let (from_email, to_email) = payload_emails(Some(&raw));
        assert_eq!(from_email, "alpha@gmail.com");
        assert_eq!(to_email, "beta@gmail.com");
    }

    #[test]
    pub(crate) fn switch_payload_serializes_structured_audit_steps() {
        let raw = switch_payload(&SwitchFacts {
            from_email: "a@gmail.com".to_string(),
            to_email: "b@gmail.com".to_string(),
            reason: "Switch".to_string(),
            how: "Clean restart".to_string(),
            prompt_id: "".to_string(),
            prompt_text: "".to_string(),
            conversation_id: "".to_string(),
            prompt_reinjected: true,
            switch_ok: true,
            instance_id: "inst-1".to_string(),
            ide_type: "antigravity".to_string(),
            idc_machine_alias: "node-1".to_string(),
            ide_path: "/bin/antigravity".to_string(),
            switch_reason: "Switch".to_string(),
            steps: Some(SwitchAuditSteps {
                backup: Some(SwitchBackupStep {
                    prompt_count: 3,
                    project_names: vec!["proj-1".to_string()],
                    project_paths: vec!["/path/proj-1".to_string()],
                    backup_batch_id: "batch-123".to_string(),
                    success: true,
                }),
                reset: Some(SwitchResetStep {
                    terminated_pids: vec![1234, 5678],
                    auth_swapped: true,
                    credentials_injected: true,
                    success: true,
                }),
                restore: Some(SwitchRestoreStep {
                    method: "resume_task_json + prompt_channel_restore".to_string(),
                    restored_count: 3,
                    dispatched_count: 3,
                    prompt_channel_waited: true,
                    success: true,
                }),
                verification: Some(SwitchVerificationStep {
                    verified: true,
                    message: "Verified prompts restored and session active".to_string(),
                }),
            }),
        });
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert!(value.get("steps").is_some());
        assert_eq!(value["steps"]["backup"]["prompt_count"], 3);
        assert_eq!(value["steps"]["reset"]["terminated_pids"][0], 1234);
        assert_eq!(value["steps"]["restore"]["restored_count"], 3);
        assert_eq!(value["steps"]["verification"]["verified"], true);
    }
}
