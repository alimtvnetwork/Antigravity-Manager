use super::*;

pub(crate) fn is_client_mismatch_error(status: reqwest::StatusCode, error_text: &str) -> bool {
    let text = error_text.to_ascii_lowercase();
    status == reqwest::StatusCode::BAD_REQUEST
        || status == reqwest::StatusCode::UNAUTHORIZED
        || status == reqwest::StatusCode::FORBIDDEN
        || text.contains("unauthorized_client")
        || text.contains("invalid_client")
}

pub(crate) fn normalize_refreshed_oauth_client_key(
    current_token: &crate::models::TokenData,
    refreshed_client_key: Option<String>,
) -> Option<String> {
    let resolved = refreshed_client_key.or_else(|| current_token.oauth_client_key.clone());
    let project_missing = current_token
        .project_id
        .as_deref()
        .map(str::trim)
        .map(|value| value.is_empty())
        .unwrap_or(true);

    if current_token.oauth_client_key.is_none()
        && project_missing
        && matches!(resolved.as_deref(), Some("antigravity_enterprise"))
    {
        crate::modules::logger::log_warn(
            "Refreshed token via enterprise client for a legacy account without project_id; keep oauth_client_key unset to avoid accidental enterprise lock",
        );
        return None;
    }

    resolved
}

pub fn list_oauth_clients() -> Result<Vec<OAuthClientDescriptor>, String> {
    let registry_guard = oauth_registry().read().map_err(|e| e.to_string())?;
    let registry = &*registry_guard;

    Ok(registry
        .clients
        .iter()
        .map(|client| OAuthClientDescriptor {
            key: client.key.clone(),
            label: client.label.clone(),
            client_id: client.client_id.clone(),
            is_active: client.key == registry.active_key,
            is_builtin: client.is_builtin,
        })
        .collect())
}

pub fn get_active_oauth_client_key() -> Result<String, String> {
    let registry_guard = oauth_registry().read().map_err(|e| e.to_string())?;
    Ok(registry_guard.active_key.clone())
}

pub fn set_active_oauth_client_key(client_key: &str) -> Result<(), String> {
    let mut registry_guard = oauth_registry().write().map_err(|e| e.to_string())?;
    let normalized = normalize_client_key(client_key);

    if get_client_by_key(&registry_guard.clients, &normalized).is_none() {
        let available = registry_guard
            .clients
            .iter()
            .map(|c| c.key.clone())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "Unknown OAuth client key '{}'. Available: {}",
            client_key, available
        ));
    }

    registry_guard.active_key = normalized.clone();
    crate::modules::logger::log_info(&format!("Active OAuth client switched to '{}'", normalized));
    Ok(())
}

/// Generate OAuth authorization URL with optional client selection.
/// Returns (auth_url, resolved_client_key).
pub fn get_auth_url_with_client(
    redirect_uri: &str,
    state: &str,
    client_key: Option<&str>,
) -> Result<(String, String), String> {
    let client = select_auth_client(client_key)?;

    let scopes = vec![
        "openid",
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/userinfo.email",
        "https://www.googleapis.com/auth/userinfo.profile",
        "https://www.googleapis.com/auth/cclog",
        "https://www.googleapis.com/auth/experimentsandconfigs",
    ]
    .join(" ");

    let params = vec![
        ("client_id", client.client_id.as_str()),
        ("redirect_uri", redirect_uri),
        ("response_type", "code"),
        ("scope", &scopes),
        ("access_type", "offline"),
        ("prompt", "consent"),
        ("include_granted_scopes", "true"),
        ("state", state),
    ];

    let url = url::Url::parse_with_params(AUTH_URL, &params)
        .map_err(|e| format!("Invalid Auth URL: {}", e))?;
    Ok((url.to_string(), client.key))
}

/// Generate OAuth authorization URL using current active client.
pub fn get_auth_url(redirect_uri: &str, state: &str) -> String {
    get_auth_url_with_client(redirect_uri, state, None)
        .map(|(url, _)| url)
        .expect("Failed to build OAuth URL")
}
