use crate::error::AppError;
use crate::modules::instance;
use crate::modules::supabase_client::{normalize_supabase_url, SupabaseClient, SupabaseEndpoint};
use chrono::Utc;
use serde_json::json;

use super::*;

/// Send single heartbeat to the Root DB endpoint
pub async fn send_heartbeat(endpoint: &SupabaseEndpoint, node_alias: &str) -> Result<(), AppError> {
    let client = SupabaseClient::new(endpoint)?;
    let node_id = get_local_node_id();
    let local_ip = get_local_ip();
    let uptime = get_uptime_seconds();
    let now = Utc::now().timestamp();

    // Calculate active project count from instance registry
    let mut project_count = 1;
    let mut instances_to_sync = Vec::new();
    if let Ok(reg) = instance::load_registry() {
        project_count = reg.instances.len() as i32;
        instances_to_sync = reg.instances;
    }

    // Compute live in-flight prompts count
    let mut running_prompts_count = 0;
    if let Ok(conn) = crate::modules::repo_db::connect_db() {
        if let Ok(mut stmt) =
            conn.prepare("SELECT count(*) FROM active_prompts WHERE status = 'running'")
        {
            if let Ok(cnt) = stmt.query_row([], |row| row.get::<_, usize>(0)) {
                running_prompts_count = cnt;
            }
        }
    }

    // 1. Upsert node record
    let node_payload = json!({
        "id": node_id,
        "alias": node_alias,
        "ip_address": local_ip,
        "uptime_seconds": uptime,
        "project_count": project_count,
        "running_prompts_count": running_prompts_count,
        "last_heartbeat_at": now,
        "status": "online"
    });

    let upsert_res = client.upsert("nodes", node_payload, "id").await;
    let should_fallback = match &upsert_res {
        Ok(val) => {
            if let Some(msg) = postgrest_error_message(val) {
                msg.contains("running_prompts_count")
            } else {
                false
            }
        }
        Err(e) => e.to_string().contains("running_prompts_count"),
    };

    if should_fallback {
        tracing::warn!(
            "[Heartbeat] Supabase nodes table missing 'running_prompts_count' column. Falling back without prompt count."
        );
        let base_payload = json!({
            "id": node_id,
            "alias": node_alias,
            "ip_address": local_ip,
            "uptime_seconds": uptime,
            "project_count": project_count,
            "last_heartbeat_at": now,
            "status": "online"
        });
        client.upsert("nodes", base_payload, "id").await?;
    } else {
        upsert_res?;
    }

    // 2. Upsert instance profiles (Child of this node)
    for inst in instances_to_sync {
        let is_active = inst.is_default;
        let profile_id = format!("{}_{}", node_id, inst.id);
        let active_acc_id = inst.bound_account_id.clone().unwrap_or_default();
        let active_acc_email = inst.bound_email.clone().unwrap_or_default();
        let running_prompts =
            crate::modules::repo_db::discover_running_prompts_from_antigravity(&inst.id);
        let running_prompts_count = running_prompts
            .iter()
            .filter(|p| p.status == "running" || p.status == "executing")
            .count();
        let profile_payload = json!({
            "id": profile_id,
            "node_id": node_id,
            "profile_name": inst.name,
            "active_account_id": active_acc_id,
            "active_account_email": active_acc_email,
            "is_active": is_active,
            "quota_percent": 100,
            "status": if is_active { "running" } else { "idle" },
            "running_prompts_count": running_prompts_count,
            "updated_at": now
        });
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            client
                .upsert("instance_profiles", profile_payload, "id")
                .await,
            "operation",
        );
    }

    Ok(())
}

pub(crate) fn resolve_sync_target(
    instance_id: &str,
) -> Result<(SupabaseEndpoint, instance::InstanceConfig, SupabaseConfig), AppError> {
    let config = load_config()?;
    let endpoint = config
        .endpoints
        .iter()
        .find(|ep| ep.is_enabled && ep.role == "root")
        .cloned()
        .ok_or_else(|| AppError::Config("No enabled root Supabase endpoint".to_string()))?;
    let registry = instance::load_registry().map_err(|e| AppError::Config(e.to_string()))?;
    let inst = registry
        .instances
        .into_iter()
        .find(|item| item.id == instance_id)
        .ok_or_else(|| AppError::Config(format!("Instance '{}' is not registered", instance_id)))?;
    Ok((endpoint, inst, config))
}

pub(crate) fn build_sync_payloads(
    config: &SupabaseConfig,
    inst: &instance::InstanceConfig,
    profile_id: &str,
    email: &str,
) -> (serde_json::Value, serde_json::Value) {
    let now = Utc::now().timestamp();
    let node_id = get_local_node_id();
    let running_prompts =
        crate::modules::repo_db::discover_running_prompts_from_antigravity(&inst.id);
    let running_prompts_count = running_prompts
        .iter()
        .filter(|p| p.status == "running" || p.status == "executing")
        .count();
    let node_payload = json!({
        "id": node_id,
        "alias": config.node_alias,
        "ip_address": get_local_ip(),
        "uptime_seconds": get_uptime_seconds(),
        "project_count": 1,
        "last_heartbeat_at": now,
        "status": "online"
    });
    let profile_payload = json!({
        "id": profile_id,
        "node_id": node_id,
        "profile_name": inst.name,
        "active_account_id": inst.bound_account_id.clone().unwrap_or_default(),
        "active_account_email": email,
        "is_active": inst.is_default,
        "quota_percent": 100,
        "status": if inst.is_default { "running" } else { "idle" },
        "running_prompts_count": running_prompts_count,
        "updated_at": now
    });
    (node_payload, profile_payload)
}

async fn sync_node_and_profile(
    client: &SupabaseClient,
    config: &SupabaseConfig,
    inst: &instance::InstanceConfig,
    profile_id: &str,
    email: &str,
) -> Result<(), AppError> {
    let (node_payload, profile_payload) = build_sync_payloads(config, inst, profile_id, email);
    let node_written = client.upsert("nodes", node_payload, "id").await?;
    if let Some(message) = postgrest_error_message(&node_written) {
        return Err(AppError::Network(
            format!("Supabase node upsert failed: {}", message),
            None,
        ));
    }
    let written = client
        .upsert("instance_profiles", profile_payload, "id")
        .await
        .map_err(|e| AppError::Network(format!("Supabase upsert failed: {}", e), None))?;
    if let Some(message) = postgrest_error_message(&written) {
        return Err(AppError::Network(
            format!("Supabase profile upsert failed: {}", message),
            None,
        ));
    }
    Ok(())
}

async fn read_back_profile_email(
    client: &SupabaseClient,
    profile_id: &str,
) -> Result<String, AppError> {
    let rows = client
        .select(
            "instance_profiles",
            &format!("id=eq.{}&select=active_account_email", profile_id),
        )
        .await?;
    if let Some(message) = postgrest_error_message(&rows) {
        return Err(AppError::Network(
            format!("Supabase profile read failed: {}", message),
            None,
        ));
    }
    email_from_postgrest(&rows).ok_or_else(|| {
        AppError::Network(
            format!(
                "Supabase profile read-back returned no record for '{}'",
                profile_id
            ),
            None,
        )
    })
}

/// Upsert one instance profile to the root Supabase endpoint and read the stored email back.
pub async fn push_and_read_instance_email(instance_id: &str) -> Result<String, AppError> {
    let (endpoint, inst, config) = resolve_sync_target(instance_id)?;
    let client = SupabaseClient::new(&endpoint)?;
    let profile_id = format!("{}_{}", get_local_node_id(), inst.id);
    let email = inst.bound_email.clone().unwrap_or_default();

    sync_node_and_profile(&client, &config, &inst, &profile_id, &email).await?;
    let stored = read_back_profile_email(&client, &profile_id).await?;

    if !stored.eq_ignore_ascii_case(&email) {
        return Err(AppError::Config(format!(
            "Supabase stored '{}' for profile '{}' but the local instance is bound to '{}'",
            stored, profile_id, email
        )));
    }
    Ok(format!("profile {} email {} confirmed", profile_id, stored))
}

pub(crate) fn postgrest_error_message(value: &serde_json::Value) -> Option<String> {
    let code = value
        .get("code")
        .or_else(|| value.get("error_code"))
        .and_then(|item| item.as_str())
        .unwrap_or("");
    let message_opt = value
        .get("message")
        .or_else(|| value.get("error"))
        .or_else(|| value.get("msg"))
        .and_then(|item| item.as_str());

    match (message_opt, code.is_empty()) {
        (Some(msg), false) => Some(format!("{} ({})", msg, code)),
        (Some(msg), true) => Some(msg.to_string()),
        (None, false) => Some(format!("PostgREST error ({})", code)),
        (None, true) => None,
    }
}

pub(crate) fn email_from_postgrest(value: &serde_json::Value) -> Option<String> {
    let row = value
        .as_array()
        .and_then(|items| items.first())
        .unwrap_or(value);
    let email = row
        .get("active_account_email")
        .and_then(|item| item.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if email.is_empty() {
        None
    } else {
        Some(email)
    }
}

/// Force an immediate heartbeat and instance registry sync to all root endpoints
pub async fn sync_local_node_now() -> Result<(), AppError> {
    let config = load_config()?;
    for ep in &config.endpoints {
        if ep.is_enabled && ep.role == "root" {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                send_heartbeat(ep, &config.node_alias).await,
                "send_heartbeat",
            );
        }
    }
    Ok(())
}

/// Trigger manual synchronization of local node and profiles to Supabase
pub async fn trigger_manual_sync() -> Result<(), AppError> {
    sync_local_node_now().await
}
