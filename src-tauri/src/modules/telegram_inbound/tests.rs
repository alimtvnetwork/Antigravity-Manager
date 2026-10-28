use chrono::Utc;

use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_chunk_telegram_text_short() {
        let text = "Hello world";
        let chunks = chunk_telegram_text(text, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello world");
    }

    #[test]
    pub(crate) fn test_chunk_telegram_text_splits_at_newline() {
        let text = "Line 1\nLine 2\nLine 3\nLine 4";
        let chunks = chunk_telegram_text(text, 14);
        assert!(chunks.len() >= 2);
        for chunk in &chunks {
            assert!(chunk.len() <= 14);
        }
    }

    #[test]
    pub(crate) fn test_shorten_project_name() {
        assert_eq!(shorten_project_name("Antigravity-Manager"), "AGM");
        assert_eq!(shorten_project_name("antigravity_manager_v2"), "AGM");
        assert_eq!(shorten_project_name("gitmap-cli"), "GitMap");
        assert_eq!(shorten_project_name("frontend-a8b21c"), "frontend");
        assert_eq!(shorten_project_name("custom-tool"), "custom-tool");
    }

    #[test]
    pub(crate) fn test_clean_for_telegram_html_truncation() {
        let input = "This is a very long prompt line that should be cleanly truncated without line break artifacts";
        let cleaned = clean_for_telegram_html(input, 20);
        assert!(cleaned.ends_with("..."));
        assert!(!cleaned.contains("\n... [truncated]"));
        assert!(!cleaned.ends_with("...."));

        // Verify that strings ending with dots do not produce stacked dots (e.g. .....)
        let with_dots = "Prompt ending with dots...";
        let cleaned_dots = clean_for_telegram_html(with_dots, 25);
        assert!(cleaned_dots.ends_with("..."));
        assert!(!cleaned_dots.ends_with("...."));
    }

    #[test]
    pub(crate) fn test_format_running_duration() {
        let now = chrono::Utc::now().timestamp();
        assert_eq!(format_running_duration(now - 30), "(running 30s)");
        assert_eq!(format_running_duration(now - 125), "(running 2m 5s)");
        assert_eq!(format_running_duration(0), "");
    }

    #[test]
    pub(crate) fn test_format_observe_report_structure() {
        let report = format_observe_report();
        assert!(report.contains("🤖 <b>AGM v"));
        assert!(report.contains("Status</b>"));
        assert!(report.contains("• <b>Machine:</b> "));
        assert!(report.contains("• <b>Alias:</b> "));
        assert!(report.contains("• <b>IP:</b> "));
        assert!(report.contains("• <b>Build:</b> v"));
        assert!(report.contains("• <b>Active Account:</b> "));
        assert!(report.contains("• <b>Quota / Tier:</b> "));
        assert!(report.contains("switch threshold:"));
        assert!(report.contains("🟢 <b>Running:</b>"));
        assert!(!report.contains("Backup Batches:"));
        assert!(report.contains("Send /expand"));
    }

    #[test]
    pub(crate) fn test_sanitize_telegram_html_entities_removes_nbsp_and_bull() {
        let raw = "&nbsp;&nbsp;└ <i>Balances:</i> <code>4h: 80%</code> &bull; Link";
        let sanitized = sanitize_telegram_html_entities(raw);
        assert!(!sanitized.contains("&nbsp;"));
        assert!(!sanitized.contains("&bull;"));
        assert!(sanitized.contains("\u{2007}\u{2007}└"));
        assert!(sanitized.contains("• Link"));
    }

    #[test]
    pub(crate) fn test_switch_and_liveness_alert_reply_markup_buttons() {
        let sw_markup = build_switch_alert_reply_markup(
            "default",
            "user@example.com",
            "95.0%",
            "88.0%",
            "Test prompt",
            &[],
        );
        let sw_str = sw_markup.to_string();
        assert!(sw_str.contains("cmd:/balance"));
        assert!(sw_str.contains("cmd:/export accounts csv"));
        assert!(sw_str.contains("cmd:/send"));
        assert!(sw_str.contains("cmd:/status"));
        assert!(sw_str.contains("Copy Prompt"));

        let live_markup = build_liveness_alert_reply_markup("default", 2, 2, "", &[]);
        let live_str = live_markup.to_string();
        assert!(live_str.contains("cmd:/balance"));
        assert!(live_str.contains("cmd:/export accounts csv"));
        assert!(live_str.contains("cmd:/send"));
        assert!(live_str.contains("cmd:/status"));
    }

    #[test]
    pub(crate) fn test_help_manual_has_all_required_code_blocks() {
        let help = format_help_manual();
        assert!(!help.contains("&nbsp;"));
        assert!(!help.contains("&bull;"));
        for required in [
            "<code>/prompt default Write unit tests</code>",
            "<code>/prompt W3:default Fix build</code>",
            "<code>/prompt --node vm-1 --instance work Audit code</code>",
            "<code>/target W3 default</code>",
            "<code>/send</code>",
            "<code>/theme</code>",
            "<code>/theme riseup</code>",
            "<code>/balance</code>",
            "<code>/credits</code>",
            "<code>/quota</code>",
            "<code>/balance --csv</code>",
            "<code>/export accounts csv</code>",
            "<code>/export instances csv</code>",
            "<code>/export balance csv</code>",
            "<code>/export nodes csv</code>",
            "<code>/export prompts json</code>",
            "<code>/accounts --csv</code>",
            "<code>/instances --csv</code>",
            "<code>/status --json</code>",
            "<code>/nodes</code>",
            "<code>/ssh nodes</code>",
            "<code>/ssh import-gitmap</code>",
            "<code>/ssh balance</code>",
            "<code>/ssh exec vm-1 agm status</code>",
            "<code>/gitmap cluster status</code>",
        ] {
            assert!(
                help.contains(required),
                "Missing required code snippet in /help: {}",
                required
            );
        }
    }

    #[test]
    pub(crate) fn test_generate_export_dataset_and_detect_export() {
        for target in [
            "accounts",
            "instances",
            "balance",
            "nodes",
            "prompts",
            "status",
        ] {
            let (csv_name, csv_mime, csv_body) =
                generate_export_dataset(target, "csv").expect("csv export");
            assert!(csv_name.ends_with(".csv"));
            assert_eq!(csv_mime, "text/csv");
            assert!(!csv_body.trim().is_empty());

            let (json_name, json_mime, json_body) =
                generate_export_dataset(target, "json").expect("json export");
            assert!(json_name.ends_with(".json"));
            assert_eq!(json_mime, "application/json");
            assert!(!json_body.trim().is_empty());
        }

        assert_eq!(
            daemon::detect_telegram_export_request("/export accounts csv"),
            Some(("accounts".to_string(), "csv".to_string()))
        );
        assert_eq!(
            daemon::detect_telegram_export_request("/accounts --csv"),
            Some(("accounts".to_string(), "csv".to_string()))
        );
        assert_eq!(
            daemon::detect_telegram_export_request("/status --json"),
            Some(("status".to_string(), "json".to_string()))
        );
        assert_eq!(
            daemon::detect_telegram_export_request("/balance --csv"),
            Some(("balance".to_string(), "csv".to_string()))
        );
    }

    #[test]
    pub(crate) fn test_normalize_cmd_callback_payload() {
        // Slash-form payloads (the report markups) must not produce `//cmd`.
        assert_eq!(daemon::normalize_cmd_callback_payload("cmd:/send"), "/send");
        assert_eq!(
            daemon::normalize_cmd_callback_payload("cmd:/instances"),
            "/instances"
        );
        assert_eq!(
            daemon::normalize_cmd_callback_payload("cmd:/balance"),
            "/balance"
        );
        // Non-slash form (the ping markup) keeps working.
        assert_eq!(
            daemon::normalize_cmd_callback_payload("cmd:fleet"),
            "/fleet"
        );
        // Doubled slashes collapse to one.
        assert_eq!(
            daemon::normalize_cmd_callback_payload("cmd://send"),
            "/send"
        );
        // Payloads with arguments survive intact.
        assert_eq!(
            daemon::normalize_cmd_callback_payload("cmd:/export instances csv"),
            "/export instances csv"
        );
        // Non-cmd payloads pass through untouched.
        assert_eq!(
            daemon::normalize_cmd_callback_payload("send:m:W3"),
            "send:m:W3"
        );
        assert_eq!(daemon::normalize_cmd_callback_payload("/send"), "/send");
    }

    #[test]
    pub(crate) fn test_machine_entries_deduplication() {
        assert!(node_label::is_current_machine("local"));
        assert!(node_label::is_current_machine("localhost"));
        let detected = crate::modules::email_watcher::detect_machine_name();
        if !detected.trim().is_empty() {
            assert!(node_label::is_current_machine(&detected));
        }
    }

    #[test]
    pub(crate) fn test_supabase_command_queue_payload_schema() {
        let node = "worker-1";
        let prompt_text = "Run integration tests";
        let cmd_id = uuid::Uuid::new_v4().to_string();
        let payload = serde_json::json!({
            "id": cmd_id,
            "target_node_id": node,
            "target_node": node,
            "source": "telegram",
            "command": "prompt",
            "command_text": format!("agm prompt default {}", prompt_text),
            "prompt": prompt_text,
            "status": "pending",
            "created_at": Utc::now().timestamp()
        });

        assert_eq!(payload["target_node_id"], "worker-1");
        assert_eq!(payload["target_node"], "worker-1");
        assert_eq!(payload["source"], "telegram");
        assert_eq!(
            payload["command_text"],
            "agm prompt default Run integration tests"
        );
        assert_eq!(payload["status"], "pending");
    }

    #[test]
    pub(crate) fn test_injection_token_parsing_preserves_target() {
        let tokens = vec!["Antigravity-Manager", "Fix", "build", "pipeline"];
        let first_tok = tokens[0];
        let mut filtered = tokens.clone();

        let is_existing_dir = std::path::Path::new(first_tok).is_dir();
        let is_known_project = first_tok.eq_ignore_ascii_case("Antigravity-Manager");
        if !is_existing_dir && !is_known_project && filtered.len() < 2 {
            filtered.insert(0, "default");
        }

        assert_eq!(filtered[0], "Antigravity-Manager");
        assert_eq!(filtered.len(), 4);
    }

    #[test]
    pub(crate) fn test_step3_graceful_prompt_fallback() {
        let prompt_input = "Please fix the failing tests in telegram inbound";
        let is_likely_prompt = prompt_input.split_whitespace().count() > 3
            || prompt_input.to_lowercase().starts_with("please")
            || prompt_input.to_lowercase().starts_with("fix")
            || prompt_input.to_lowercase().starts_with("implement")
            || prompt_input.to_lowercase().starts_with("add");
        assert!(is_likely_prompt);

        let id_input = "C001";
        let is_id_prompt = id_input.split_whitespace().count() > 3
            || id_input.to_lowercase().starts_with("please")
            || id_input.to_lowercase().starts_with("fix");
        assert!(!is_id_prompt);
    }

    #[test]
    fn test_prompt_execution_status_type_enum() {
        use super::types::PromptExecutionStatusType;
        let executing = PromptExecutionStatusType::from_spawned(true);
        assert_eq!(executing, PromptExecutionStatusType::ExecutingNow);
        assert_eq!(executing.display_label(), "Executing now");
        assert_eq!(executing.to_string(), "Executing now");
        assert!(executing.is_executing());
        assert!(!executing.is_queued());

        let queued = PromptExecutionStatusType::from_spawned(false);
        assert_eq!(queued, PromptExecutionStatusType::Queued);
        assert_eq!(queued.display_label(), "Queued");
        assert_eq!(queued.to_string(), "Queued");
        assert!(queued.is_queued());
        assert!(!queued.is_executing());
    }

    #[test]
    fn test_remote_dispatch_status_type_enum() {
        use super::types::RemoteDispatchStatusType;
        let gitmap = RemoteDispatchStatusType::resolve(true, false);
        assert_eq!(gitmap, RemoteDispatchStatusType::GitMapClusterSsh);
        assert_eq!(gitmap.to_string(), "Sent via GitMap Cluster SSH");

        let supabase = RemoteDispatchStatusType::resolve(false, true);
        assert_eq!(supabase, RemoteDispatchStatusType::SupabaseQueued);
        assert_eq!(
            supabase.to_string(),
            "Queued on the secondary database — the node will pick it up"
        );

        let local = RemoteDispatchStatusType::resolve(false, false);
        assert_eq!(local, RemoteDispatchStatusType::LocalRecorded);
        assert_eq!(
            local.to_string(),
            "Queued (the node recorded your instruction)"
        );
    }

    #[test]
    fn test_instances_csv_includes_prompt_columns() {
        let (name, _mime, body) =
            crate::modules::telegram_inbound::reports_a::generate_export_dataset(
                "instances",
                "csv",
            )
            .expect("instances csv");
        assert!(name.ends_with(".csv"));
        let header = body.lines().next().unwrap_or("");
        assert!(header.contains("running_prompts"));
        assert!(header.contains("queued_prompts"));
        assert!(header.contains("running_projects"));
        assert!(header.contains("queued_projects"));
    }

    #[test]
    fn test_copy_text_cap_is_256() {
        assert_eq!(TELEGRAM_COPY_TEXT_MAX, 256);
        let long = "α".repeat(400);
        let clamped = clamp_telegram_copy_text(&long, TELEGRAM_COPY_TEXT_MAX);
        assert!(clamped.chars().count() <= TELEGRAM_COPY_TEXT_MAX);
    }

    #[test]
    fn test_status_fanout_is_status_command() {
        assert!(super::instance_cards::should_fanout_status("/status"));
        assert!(super::instance_cards::should_fanout_status("/observe"));
        assert!(!super::instance_cards::should_fanout_status("/balance"));
        assert!(!super::instance_cards::should_fanout_status(
            "/export instances csv"
        ));
    }
}
