use crate::modules::ssh_manager::{
    self, ClusterInstanceBalanceItem, CsvExportResult, MeshDeploySummary,
    RemotePromptDispatchPayload, RemotePromptDispatchResult, SaveSshNodeRsaPayload,
    SshExecNodeResult, SshGitmapStatus, SshNodeRecord,
};

#[tauri::command]
pub async fn get_ssh_gitmap_status() -> Result<SshGitmapStatus, String> {
    tokio::task::spawn_blocking(ssh_manager::check_gitmap_installed_status)
        .await
        .map_err(|e| format!("Task join error: {}", e))
}

#[tauri::command]
pub async fn install_gitmap_powershell(open_window: Option<bool>) -> Result<String, String> {
    let should_open = open_window.unwrap_or(true);
    tokio::task::spawn_blocking(move || ssh_manager::install_gitmap_via_powershell(should_open))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn list_ssh_nodes() -> Result<Vec<SshNodeRecord>, String> {
    tokio::task::spawn_blocking(|| {
        // Justification: best-effort sync of ~/.ssh/config on node listing; failure is recorded, in-memory nodes remain authoritative
        crate::error::record_ignored(ssh_manager::update_ssh_config(false), "update_ssh_config");
        let conns = ssh_manager::load_ssh_connections()?;
        Ok(conns.iter().map(ssh_manager::to_ssh_node_record).collect())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn save_ssh_node_rsa(payload: SaveSshNodeRsaPayload) -> Result<SshNodeRecord, String> {
    tokio::task::spawn_blocking(move || ssh_manager::upsert_ssh_node_rsa(&payload))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn delete_ssh_node(id_or_alias: String) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || ssh_manager::delete_ssh_connection(&id_or_alias))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn test_ssh_node_connection(id_or_alias: String) -> Result<SshExecNodeResult, String> {
    tokio::task::spawn_blocking(move || {
        let clean = id_or_alias.trim();
        let cmd = vec!["echo AGM_SSH_OK".to_string()];
        let results = ssh_manager::exec_ssh_command(clean, &cmd, None, None, None)?;
        let first = results
            .into_iter()
            .next()
            .ok_or_else(|| format!("No SSH node matched '{}'", clean))?;

        if !first.success {
            let is_auth_err = first.exit_code == 255
                || first.stderr.to_lowercase().contains("permission denied")
                || first
                    .stderr
                    .to_lowercase()
                    .contains("authentication failed");

            if is_auth_err {
                // Attempt auto-repair via GitMap fix-auth
                if let Ok(fix_msg) = ssh_manager::fix_node_auth_via_gitmap(clean) {
                    tracing::info!(
                        target: "ssh_manager",
                        "Auto-repaired auth for {clean}: {fix_msg}"
                    );
                    if let Ok(retried) =
                        ssh_manager::exec_ssh_command(clean, &cmd, None, None, None)
                    {
                        if let Some(res) = retried.into_iter().next() {
                            if res.success {
                                return Ok(res);
                            }
                        }
                    }
                }
            }

            // Probe via GitMap check if port 22 is open or unreachable
            if let Ok((is_online, details)) = ssh_manager::check_node_via_gitmap(clean) {
                if is_online {
                    return Ok(SshExecNodeResult {
                        alias: first.alias,
                        target: first.target,
                        ip_address: first.ip_address,
                        username: first.username,
                        port: first.port,
                        exit_code: 0,
                        stdout: details,
                        stderr: String::new(),
                        duration_ms: first.duration_ms,
                        success: true,
                    });
                }
            }
        }

        Ok(first)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn import_ssh_nodes_from_gitmap() -> Result<Vec<SshNodeRecord>, String> {
    tokio::task::spawn_blocking(|| {
        let (_source, _stats) = ssh_manager::import_ssh_from_gitmap()?;
        let conns = ssh_manager::load_ssh_connections()?;
        Ok(conns.iter().map(ssh_manager::to_ssh_node_record).collect())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn deploy_encrypted_ssh_bundle(
    target_aliases: Option<Vec<String>>,
) -> Result<MeshDeploySummary, String> {
    tokio::task::spawn_blocking(move || ssh_manager::deploy_encrypted_ssh_to_nodes(target_aliases))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn get_cluster_instances_and_balances() -> Result<Vec<ClusterInstanceBalanceItem>, String>
{
    match tokio::task::spawn_blocking(ssh_manager::get_cluster_instances_and_balances).await {
        Ok(Ok(items)) => {
            for item in &items {
                if let Some(ref err) = item.error_message {
                    tracing::warn!(
                        machine = %item.machine_alias,
                        ip = %item.machine_ip,
                        error = %err,
                        "Cluster instance diagnostic warning"
                    );
                }
            }
            Ok(items)
        }
        Ok(Err(err)) => {
            tracing::warn!(error = %err, "Failed to retrieve cluster instances and balances");
            Err(err)
        }
        Err(join_err) => {
            let msg = format!("Task join error: {}", join_err);
            tracing::warn!(error = %msg, "Spawn blocking join failure in get_cluster_instances_and_balances");
            Err(msg)
        }
    }
}

#[tauri::command]
pub async fn send_remote_ssh_prompt(
    payload: RemotePromptDispatchPayload,
) -> Result<RemotePromptDispatchResult, String> {
    ssh_manager::send_remote_ssh_prompt(&payload).await
}

#[tauri::command]
pub async fn export_system_data_csv(
    target: String,
    format: Option<String>,
    send_to_telegram: Option<bool>,
) -> Result<CsvExportResult, String> {
    let fmt = format.unwrap_or_else(|| "csv".to_string());
    let doc = crate::modules::telegram_inbound::generate_export_document(&target, &fmt)?;
    let content_str = String::from_utf8_lossy(&doc.bytes).to_string();
    let row_count = content_str.lines().count().saturating_sub(1);

    if send_to_telegram.unwrap_or(false) {
        if let Ok(tg_cfg) = crate::modules::telegram_inbound::load_config() {
            if !tg_cfg.bot_token.trim().is_empty() {
                if let Some(chat_id) = tg_cfg.allowed_chat_id {
                    // Justification: best-effort optional upload to Telegram when requested from UI
                    crate::error::record_ignored(
                        crate::modules::telegram_inbound::send_telegram_document(
                            &tg_cfg.bot_token,
                            chat_id,
                            &doc.file_name,
                            &doc.bytes,
                            &doc.caption,
                            doc.markup,
                        )
                        .await,
                        "export_system_data_csv send_telegram_document",
                    );
                }
            }
        }
    }

    Ok(CsvExportResult {
        filename: doc.file_name,
        mime_type: doc.mime_type,
        content: content_str,
        row_count,
    })
}
