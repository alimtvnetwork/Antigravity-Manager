//! cmd_ui_trace — Tauri command to record fine-grained UI trace telemetry and audit entries.

#[tauri::command]
pub async fn record_ui_trace(
    step: String,
    action: String,
    target: String,
    metadata: Option<serde_json::Value>,
) -> Result<(), String> {
    tracing::info!(
        target: "ui_trace",
        "[UI_STEP] [{}] Action: '{}' Target: '{}' Meta: {:?}",
        step,
        action,
        target,
        metadata
    );

    let target_str = format!("{}:{}", step, target);
    let params_str = metadata.map(|m| m.to_string()).unwrap_or_default();

    // Justification: UI telemetry trace recording is best-effort and must not fail the caller.
    crate::error::record_ignored(
        crate::modules::audit::append_entry(
            chrono::Utc::now().timestamp_millis(),
            "user",
            &action,
            &target_str,
            "ui_interaction",
            &params_str,
            "ok",
            "UI trace recorded",
            None,
        ),
        "record_ui_trace audit append",
    );

    Ok(())
}
