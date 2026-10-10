use super::*;

pub(crate) const OAUTH_CLIENTS_ENV: &str = "ANTIGRAVITY_OAUTH_CLIENTS";

pub(crate) const ACTIVE_OAUTH_CLIENT_ENV: &str = "ANTIGRAVITY_OAUTH_CLIENT_KEY";

pub(crate) const DEFAULT_OAUTH_CLIENT_KEY: &str = "antigravity_enterprise";

pub(crate) static OAUTH_CLIENT_REGISTRY: std::sync::OnceLock<
    std::sync::RwLock<OAuthClientRegistry>,
> = std::sync::OnceLock::new();

pub(crate) fn normalize_client_key(key: &str) -> String {
    key.trim().to_ascii_lowercase()
}

pub(crate) fn build_registry() -> OAuthClientRegistry {
    let mut clients: Vec<OAuthClientConfig> = vec![OAuthClientConfig {
        key: normalize_client_key(DEFAULT_OAUTH_CLIENT_KEY),
        label: "Antigravity Enterprise".to_string(),
        client_id: CLIENT_ID.to_string(),
        client_secret: CLIENT_SECRET.to_string(),
        is_builtin: true,
    }];

    if let Ok(raw_extra_clients) = std::env::var(OAUTH_CLIENTS_ENV) {
        for entry in raw_extra_clients.split(';') {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Expected format: key|client_id|client_secret|optional_label
            let parts: Vec<&str> = trimmed.split('|').map(|v| v.trim()).collect();
            if parts.len() < 3 {
                crate::modules::logger::log_warn(&format!(
                    "Ignored invalid OAuth client entry in {}: {}",
                    OAUTH_CLIENTS_ENV, trimmed
                ));
                continue;
            }

            let key = normalize_client_key(parts[0]);
            if key.is_empty() || parts[1].is_empty() || parts[2].is_empty() {
                crate::modules::logger::log_warn(&format!(
                    "Ignored incomplete OAuth client entry in {}: {}",
                    OAUTH_CLIENTS_ENV, trimmed
                ));
                continue;
            }

            let label = if parts.len() >= 4 && !parts[3].is_empty() {
                parts[3].to_string()
            } else {
                key.clone()
            };

            let custom_client = OAuthClientConfig {
                key: key.clone(),
                label,
                client_id: parts[1].to_string(),
                client_secret: parts[2].to_string(),
                is_builtin: false,
            };

            if let Some(existing_index) = clients.iter().position(|c| c.key == key) {
                clients[existing_index] = custom_client;
                crate::modules::logger::log_info(&format!(
                    "OAuth client '{}' overridden by {}",
                    key, OAUTH_CLIENTS_ENV
                ));
            } else {
                clients.push(custom_client);
                crate::modules::logger::log_info(&format!(
                    "OAuth client '{}' loaded from {}",
                    key, OAUTH_CLIENTS_ENV
                ));
            }
        }
    }

    let mut active_key = std::env::var(ACTIVE_OAUTH_CLIENT_ENV)
        .ok()
        .map(|v| normalize_client_key(&v))
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| normalize_client_key(DEFAULT_OAUTH_CLIENT_KEY));

    if !clients.iter().any(|c| c.key == active_key) {
        active_key = clients
            .first()
            .map(|c| c.key.clone())
            .unwrap_or_else(|| normalize_client_key(DEFAULT_OAUTH_CLIENT_KEY));
    }

    OAuthClientRegistry {
        clients,
        active_key,
    }
}

pub(crate) fn oauth_registry() -> &'static std::sync::RwLock<OAuthClientRegistry> {
    OAUTH_CLIENT_REGISTRY.get_or_init(|| std::sync::RwLock::new(build_registry()))
}

pub(crate) fn get_client_by_key<'a>(
    clients: &'a [OAuthClientConfig],
    client_key: &str,
) -> Option<&'a OAuthClientConfig> {
    let normalized = normalize_client_key(client_key);
    clients.iter().find(|c| c.key == normalized)
}

pub(crate) fn active_or_first_client(registry: &OAuthClientRegistry) -> Option<OAuthClientConfig> {
    if let Some(active) = get_client_by_key(&registry.clients, &registry.active_key) {
        return Some(active.clone());
    }
    registry.clients.first().cloned()
}

pub(crate) fn select_auth_client(client_key: Option<&str>) -> Result<OAuthClientConfig, String> {
    let registry_guard = oauth_registry().read().map_err(|e| e.to_string())?;
    let registry = &*registry_guard;

    if registry.clients.is_empty() {
        return Err("No OAuth clients configured".to_string());
    }

    if let Some(key) = client_key {
        if let Some(client) = get_client_by_key(&registry.clients, key) {
            return Ok(client.clone());
        }
        return Err(format!("Unknown OAuth client key: {}", key));
    }

    active_or_first_client(registry).ok_or_else(|| "No OAuth clients configured".to_string())
}

pub(crate) fn get_candidate_clients(preferred_client_key: Option<&str>) -> Vec<OAuthClientConfig> {
    let registry_guard = match oauth_registry().read() {
        Ok(guard) => guard,
        Err(_) => return vec![],
    };
    let registry = &*registry_guard;

    let mut candidates = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut push_candidate = |client: &OAuthClientConfig| {
        if seen.insert(client.key.clone()) {
            candidates.push(client.clone());
        }
    };

    if let Some(preferred_key) = preferred_client_key {
        if let Some(preferred) = get_client_by_key(&registry.clients, preferred_key) {
            push_candidate(preferred);
        } else {
            crate::modules::logger::log_warn(&format!(
                "Preferred OAuth client '{}' not found; fallback to active client list",
                preferred_key
            ));
        }
    }

    if let Some(active) = get_client_by_key(&registry.clients, &registry.active_key) {
        push_candidate(active);
    }

    for client in &registry.clients {
        push_candidate(client);
    }

    candidates
}
