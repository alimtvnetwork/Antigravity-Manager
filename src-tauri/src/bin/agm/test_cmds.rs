//! test_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::process::{Command, Stdio};
use uuid::Uuid;

pub(crate) fn cmd_test_auto_switch(args: &[String]) {
    println!("[*] Testing Auto-Switcher on this machine...");
    let threshold: f64 = args.first().and_then(|s| s.parse().ok()).unwrap_or(90.0);

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to initialize async runtime: {}", e);
            std::process::exit(1);
        }
    };

    let mut app_cfg = match config::load_app_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    let orig_enabled = app_cfg.auto_profile_switcher.is_enabled;
    let orig_low = app_cfg.auto_profile_switcher.low_quota_threshold_percent;
    let orig_crit = app_cfg.auto_profile_switcher.critical_threshold_percent;

    println!(
        "    Target Model: {}",
        app_cfg.auto_profile_switcher.target_model
    );
    println!("    Test Low-Quota Threshold: {:.1}%", threshold);

    // Apply test threshold and enable switcher
    app_cfg.auto_profile_switcher.is_enabled = true;
    app_cfg.auto_profile_switcher.low_quota_threshold_percent = threshold;
    app_cfg.auto_profile_switcher.critical_threshold_percent = threshold.min(15.0);
    let _ = config::save_app_config(&app_cfg);

    let status_before = auto_switcher::get_status();
    println!(
        "    Monitored Instance: {}",
        status_before.active_instance_id
    );
    println!(
        "    Current Bound Account: {}",
        status_before
            .active_account_email
            .as_deref()
            .unwrap_or("none")
    );
    println!(
        "    Current Quota: {:.1}%",
        status_before.current_quota_percent.unwrap_or(100.0)
    );

    let interval = auto_switcher::calculate_next_interval_seconds(
        status_before.current_quota_percent,
        &app_cfg.auto_profile_switcher,
    );
    println!("    Calculated Polling Interval: {}s", interval);

    println!("[*] Triggering check_and_rotate_if_needed()...");
    let rotate_res = rt.block_on(auto_switcher::check_and_rotate_if_needed());

    // Restore original config
    app_cfg.auto_profile_switcher.is_enabled = orig_enabled;
    app_cfg.auto_profile_switcher.low_quota_threshold_percent = orig_low;
    app_cfg.auto_profile_switcher.critical_threshold_percent = orig_crit;
    let _ = config::save_app_config(&app_cfg);

    match rotate_res {
        Ok(Some(reason)) => {
            println!("[SUCCESS] Auto-switcher rotated successfully!");
            println!("          Reason: {}", reason);
            if let Ok(Some(current)) = account::get_current_account() {
                println!("          New Active Account: {}", current.email);
            }
        }
        Ok(None) => {
            println!(
                "[INFO] Check cycle complete: No rotation needed (quota was above {:.1}% or candidate optimal).",
                threshold
            );
        }
        Err(e) => {
            eprintln!("[ERROR] Auto-switcher check failed: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn cmd_test_email(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Inbound Email & SMTP Diagnostic:");
        println!("  agm test-email [--help]");
        println!("\nDescription:");
        println!("  Performs full diagnostic checks of SMTP dispatch, IMAP inbound watchers,");
        println!("  credentials validity, and recipient notification queues.");
        println!("\nAliases: agm test-email, agm email-test, agm check-email");
        println!("\nExamples:");
        println!("  agm test-email                      # Run full email subsystem diagnostics");
        return;
    }

    println!("============================================================");
    println!("  AGM INBOUND EMAIL & SMTP DIAGNOSTIC SUITE");
    println!("============================================================");

    // 1. Settings
    let settings = match email_vault_db::get_notification_settings() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[ERROR] Failed to load email settings: {}", e);
            return;
        }
    };
    println!("[*] Notification Settings:");
    println!("    Enabled:                    {}", settings.is_enabled);
    println!(
        "    Local Node Name:            {}",
        settings.local_machine_name
    );
    println!(
        "    Polling Interval (min):     {}",
        settings.polling_interval_minutes
    );
    println!(
        "    Inbox Check Interval (min): {}",
        settings.inbox_check_interval_minutes
    );

    // 2. Recipients
    let recipients = email_vault_db::list_notify_recipients().unwrap_or_default();
    println!(
        "[*] Authorized Notifier Recipients ({} found):",
        recipients.len()
    );
    for r in &recipients {
        println!("    - {} (active: {})", r.email, r.is_active);
    }

    // 3. Accounts
    let accounts = email_vault_db::list_email_accounts().unwrap_or_default();
    println!(
        "[*] Configured Mailbox Accounts ({} found):",
        accounts.len()
    );
    for a in &accounts {
        println!(
            "    - [{}] {} | IMAP: {}:{} | SMTP: {}:{} | default: {} | active: {}",
            a.id,
            a.email,
            a.imap_host,
            a.imap_port,
            a.smtp_host,
            a.smtp_port,
            a.is_default,
            a.is_active
        );
    }

    let default_acc = match accounts.into_iter().find(|a| a.is_default && a.is_active) {
        Some(a) => a,
        None => {
            eprintln!("[ERROR] No active default email account found in vault!");
            return;
        }
    };

    // 4. Test IMAP poll
    println!(
        "[*] Connecting to IMAP server '{}:{}' for '{}'...",
        default_acc.imap_host, default_acc.imap_port, default_acc.email
    );
    match email_inbound::poll_unread_messages(&default_acc, 5) {
        Ok(msgs) => {
            println!("[SUCCESS] IMAP connection and authentication succeeded!");
            println!("          Found {} unread message(s):", msgs.len());
            for (i, m) in msgs.iter().enumerate() {
                println!("          [{}] From:    {}", i + 1, m.from);
                println!("              Subject: {}", m.subject);
                println!("              Msg-ID:  {}", m.message_id);
                let parsed = email_inbound::parse_email_command(&m.subject, &m.body);
                println!("              Parsed:  {:?}", parsed);
                let is_auth = email_inbound::is_authorized_notifier(&m.from);
                println!("              Authorized Sender: {}", is_auth);
            }
        }
        Err(e) => {
            eprintln!("[ERROR] IMAP poll failed: {}", e);
        }
    }

    // 5. Test Subject Parser with various inputs
    println!("[*] Testing Subject Command Parser:");
    let test_subjects = vec![
        "VM3 | 1 | help",
        "VM3 | help",
        "VM3 | default | help",
        "VM3 | #1 | help",
        "VM3 | ins-1 | help",
        "VM3 | 1 | cmd",
        "VM3 | 1 | agm status",
        "* | gitmap | status",
    ];
    for subj in test_subjects {
        let action = email_inbound::parse_email_command(subj, "");
        println!("    '{}' => {:?}", subj, action);
    }

    // 6. Optional SMTP test if requested: agm test-email send [recipient]
    if args.first().map(|s| s.as_str()) == Some("send") {
        let target_rcpt = args
            .get(1)
            .map(|s| s.as_str())
            .unwrap_or("alim.karim@riseup-asia.com");
        println!("[*] Sending test SMTP dispatch to '{}'...", target_rcpt);
        let (subj, body) = email_sender::render_help_email(
            &settings.local_machine_name,
            &email_watcher::detect_local_ip(),
        );
        match email_sender::dispatch_email_with_failover(&subj, &body, &[target_rcpt.to_string()]) {
            Ok(res) => {
                println!(
                    "[SUCCESS] SMTP test email delivered successfully via '{}'!",
                    res.used_account_email
                );
            }
            Err(e) => {
                eprintln!("[ERROR] SMTP test email failed: {}", e);
            }
        }
    }

    // 7. Optional end-to-end command execution test: agm test-email execute [subject]
    if args.first().map(|s| s.as_str()) == Some("execute") {
        let test_subject = args.get(1).map(|s| s.as_str()).unwrap_or("VM3 | 1 | help");
        let test_from = args
            .get(2)
            .map(|s| s.as_str())
            .unwrap_or("Alim Ul Karim <alim.karim@riseup-asia.com>");
        let test_body = args.get(3).map(|s| s.as_str()).unwrap_or("");
        println!("[*] Executing live end-to-end simulated inbound message:");
        println!("    From:    {}", test_from);
        println!("    Subject: {}", test_subject);
        if !test_body.is_empty() {
            println!("    Body:    {}", test_body);
        }
        let mock_msg = email_inbound::RawEmailMessage {
            message_id: format!("<test-{}@agm>", uuid::Uuid::new_v4()),
            from: test_from.to_string(),
            subject: test_subject.to_string(),
            body: test_body.to_string(),
        };
        let action = email_inbound::parse_email_command(&mock_msg.subject, &mock_msg.body);
        println!("    Parsed Action: {:?}", action);
        let local_ip = email_watcher::detect_local_ip();
        let local_name = email_watcher::detect_machine_name();
        match email_inbound::execute_inbound_action(&mock_msg, action, &local_ip, &local_name) {
            Ok(summary) => {
                println!("[SUCCESS] Inbound action executed and receipts dispatched!");
                println!("          Summary: {}", summary);
            }
            Err(e) => {
                eprintln!("[ERROR] Inbound execution failed: {}", e);
            }
        }
    }

    // 8. Optional process live unread messages: agm test-email poll
    if args.first().map(|s| s.as_str()) == Some("poll") {
        println!("[*] Polling and executing real unread messages from IMAP...");
        match email_inbound::poll_unread_messages(&default_acc, 5) {
            Ok(msgs) => {
                println!("    Found {} unread message(s)", msgs.len());
                let local_ip = email_watcher::detect_local_ip();
                let local_name = email_watcher::detect_machine_name();
                for (i, m) in msgs.iter().enumerate() {
                    println!("    [{}] Message-ID: {}", i + 1, m.message_id);
                    println!("        From: {}", m.from);
                    println!("        Subject: {}", m.subject);
                    let action = email_inbound::parse_email_command(&m.subject, &m.body);
                    println!("        Action: {:?}", action);
                    match email_inbound::execute_inbound_action(m, action, &local_ip, &local_name) {
                        Ok(res) => println!("        [SUCCESS] {}", res),
                        Err(e) => eprintln!("        [ERROR] {}", e),
                    }
                }
            }
            Err(e) => eprintln!("[ERROR] Polling failed: {}", e),
        }
    }
}

pub(crate) fn cmd_test_training(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Machine Training & Telemetry REST API Test:");
        println!("  agm test-training [--help]");
        println!("\nDescription:");
        println!("  Runs diagnostic checks against the machine telemetry and REST training API endpoints.");
        println!("\nAliases: agm test-training, agm training, agm train");
        println!("\nExamples:");
        println!("  agm test-training                   # Verify telemetry and training API");
        return;
    }

    println!("============================================================");
    println!("  AGM MACHINE TRAINING & TELEMETRY REST API TEST SUITE");
    println!("============================================================");

    // 1. Telemetry Gathering
    println!("[*] Gathering machine telemetry via training_api::gather_telemetry()...");
    match training_api::gather_telemetry() {
        Ok(t) => {
            println!("[SUCCESS] Telemetry retrieved:");
            println!("          Node Name:           {}", t.node_name);
            println!("          Local IP:            {}", t.local_ip);
            println!("          CLI/Lib Version:     v{}", t.version);
            println!("          Platform OS/Arch:    {}/{}", t.os, t.arch);
            println!("          API Enabled:         {}", t.training_api_enabled);
            println!(
                "          Active Prompts:      {} running ({} total)",
                t.active_prompts_running, t.active_prompts_total
            );
            if let Some(acc) = t.active_account {
                println!(
                    "          Active Account:      {} ({:.1}% quota, {})",
                    acc.email, acc.quota_percent, acc.tier
                );
            }
            println!("          Accounts Summary:    {}", t.accounts_summary);
            println!("          Instances Monitored: {}", t.instances.len());
        }
        Err(e) => {
            eprintln!("[ERROR] Telemetry gathering failed: {}", e);
            std::process::exit(1);
        }
    }

    // 2. Training Learning Feedback Ingestion
    println!("[*] Ingesting test learning signal into SQLite training_vault.db...");
    let req = training_api::LearnRequest {
        session_id: Some("agm-cli-test-session".to_string()),
        model: Some("gemini-flash".to_string()),
        prompt_type: Some("e2e-verification".to_string()),
        input_tokens: Some(256),
        output_tokens: Some(512),
        latency_ms: Some(180),
        success: Some(true),
        score: Some(0.99),
        feedback: Some("Live machine training test successfully ingested".to_string()),
        adjust_routing: Some(false),
    };
    match training_api::ingest_learning(req) {
        Ok(res) => {
            println!("[SUCCESS] Learning feedback ingested:");
            println!("          Log ID:    {}", res.log_id);
            println!("          Timestamp: {}", res.timestamp);
            println!("          Message:   {}", res.message);
        }
        Err(e) => {
            eprintln!("[ERROR] Learning feedback ingestion failed: {}", e);
            std::process::exit(1);
        }
    }

    // 3. Machine Remote Modification
    println!("[*] Testing machine remote modification (adjust_threshold action)...");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mod_req = training_api::MachineModifyRequest {
        action: "adjust_threshold".to_string(),
        account_email_or_id: None,
        instance_id: None,
        target_model: None,
        low_quota_threshold: Some(85.0),
        critical_quota_threshold: Some(15.0),
    };
    match rt.block_on(training_api::execute_machine_modify(mod_req)) {
        Ok(res) => {
            println!("[SUCCESS] Machine modification completed:");
            println!("          Action:  {}", res.action);
            println!("          Message: {}", res.message);
        }
        Err(e) => {
            eprintln!("[ERROR] Machine modification failed: {}", e);
            std::process::exit(1);
        }
    }

    // 4. Settings Toggle Test
    println!("[*] Verifying training_api_enabled settings toggle...");
    let orig = training_api::is_training_api_enabled();
    let _ = training_api::set_training_api_enabled(!orig);
    assert_eq!(training_api::is_training_api_enabled(), !orig);
    let _ = training_api::set_training_api_enabled(orig);
    println!(
        "[SUCCESS] Settings toggle successfully verified (state restored to {}).",
        orig
    );

    println!("============================================================");
    println!("[SUCCESS] All Training REST API engine tests passed!");
    println!("============================================================");
}
