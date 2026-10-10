use super::*;

#[tauri::command]
pub async fn get_email_watcher_status() -> AppResult<WatcherStatus> {
    Ok(email_watcher::get_watcher_status().await)
}

fn get_active_recipient_emails() -> Result<Vec<String>, AppError> {
    let recipients = email_vault_db::list_notify_recipients().map_err(AppError::Email)?;
    let active_emails: Vec<String> = recipients
        .into_iter()
        .filter(|r| r.is_active)
        .map(|r| r.email)
        .collect();
    if active_emails.is_empty() {
        return Err(AppError::Email(
            "No active notification recipients configured".to_string(),
        ));
    }
    Ok(active_emails)
}

#[tauri::command]
pub async fn trigger_manual_email_check() -> AppResult<String> {
    let active_emails = get_active_recipient_emails()?;
    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();
    let (subj, body) = email_sender::render_help_email(&m_name, &m_ip);

    email_sender::dispatch_email_with_failover(&subj, &body, &active_emails)
        .map(|res| format!("Dispatched alert via account '{}'", res.used_account_email))
        .map_err(AppError::Email)
}

#[tauri::command]
pub async fn dispatch_email_test_ping(project_name: Option<String>) -> AppResult<String> {
    let active_emails = get_active_recipient_emails()?;
    let proj = project_name.unwrap_or_else(|| "Antigravity-Workspace".to_string());
    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();
    let now = chrono::Utc::now().timestamp();
    let (subj, body) = email_sender::render_test_ping_email(&proj, &m_name, &m_ip, now);

    let res = email_sender::dispatch_email_with_failover(&subj, &body, &active_emails)
        .map_err(AppError::Email)?;
    email_watcher::activate_awaiting_reply(300);

    Ok(format!(
        "Dispatched test ping for '{}' via '{}'. Fast polling active (5m).",
        proj, res.used_account_email
    ))
}

#[tauri::command]
pub async fn dispatch_custom_email_task(
    recipient_email: Option<String>,
    task_type: String,
    task_payload: String,
    subject: Option<String>,
) -> AppResult<String> {
    let target_recipients = if let Some(ref email) = recipient_email {
        let trimmed = email.trim();
        if !trimmed.is_empty() {
            vec![trimmed.to_string()]
        } else {
            get_active_recipient_emails()?
        }
    } else {
        get_active_recipient_emails()?
    };

    if target_recipients.is_empty() {
        return Err(AppError::Email(
            "No recipients configured. Please add a recipient or specify an email address."
                .to_string(),
        ));
    }

    let m_name = email_watcher::detect_machine_name();
    let _m_ip = email_watcher::detect_local_ip();

    // Clean any legacy prefixes (powershell:, ps:, prompt:, cmd:) from payload so Body has ONLY the prompt/command
    let trimmed_payload = task_payload.trim();
    let lower_payload = trimmed_payload.to_lowercase();
    let clean_payload = if lower_payload.starts_with("powershell:") {
        trimmed_payload["powershell:".len()..].trim()
    } else if lower_payload.starts_with("ps:") {
        trimmed_payload["ps:".len()..].trim()
    } else if lower_payload.starts_with("cmd:") {
        trimmed_payload["cmd:".len()..].trim()
    } else if lower_payload.starts_with("prompt:") {
        trimmed_payload["prompt:".len()..].trim()
    } else {
        trimmed_payload
    };

    let full_subject = subject.unwrap_or_else(|| {
        let lower_type = task_type.trim().to_lowercase();
        if lower_type.contains("help") {
            "* | help".to_string()
        } else if lower_type.contains("ps")
            || lower_type.contains("powershell")
            || lower_type.contains("cli")
        {
            format!(
                "ps | {}",
                clean_payload.lines().next().unwrap_or("Get-Process")
            )
        } else if lower_type.contains("status") {
            "* | status".to_string()
        } else {
            "prompt | proj-Antigravity-Manager".to_string()
        }
    });

    // Body contains ONLY the prompt/payload in 100% plaintext (zero HTML tags)
    let full_body = format!(
        "{}\n\n---\nAGM Plaintext Interactive Mail (Node: {})\nReply directly to this email with your next prompt in the body.\nSubject format: * | prompt | proj-Antigravity-Manager  OR  * | ps | Get-Process",
        clean_payload, m_name
    );

    let res =
        email_sender::dispatch_email_with_failover(&full_subject, &full_body, &target_recipients)
            .map_err(AppError::Email)?;

    // Activate fast adaptive awaiting poll (5–10s quick-poll)
    email_watcher::activate_awaiting_reply(300);

    Ok(format!(
        "Task '{}' dispatched to {} recipient(s) via account '{}'. Quick-poll activated (5m).",
        task_type,
        target_recipients.len(),
        res.used_account_email
    ))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CliExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub machine_name: String,
    pub machine_ip: String,
}
