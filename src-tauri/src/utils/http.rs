use crate::modules::config::load_app_config;
use once_cell::sync::Lazy;
use rquest::{Client, Proxy};
use rquest_util::Emulation;
use std::collections::HashMap;
use std::sync::RwLock;

/// Sharing the client's identity: Determines the timeout length and whether to enable JA3/TLS emulation.
///
/// Each combination is built only once and reused for a long time (`Client` comes with its own connection pool, clone is very light and shares the same pool).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ClientKey {
    timeout_secs: u64,
    emulated: bool,
}

impl ClientKey {
    const fn new(timeout_secs: u64, emulated: bool) -> Self {
        Self {
            timeout_secs,
            emulated,
        }
    }
}

struct SharedClients {
    built: HashMap<ClientKey, Client>,
}

impl SharedClients {
    fn new() -> Self {
        Self {
            built: HashMap::new(),
        }
    }

    fn get(&mut self, key: ClientKey) -> Client {
        if let Some(client) = self.built.get(&key) {
            return client.clone();
        }
        let client = if key.emulated {
            create_base_client(key.timeout_secs)
        } else {
            create_standard_client(key.timeout_secs)
        };
        self.built.insert(key, client.clone());
        client
    }
}

static SHARED_CLIENTS: Lazy<RwLock<SharedClients>> =
    Lazy::new(|| RwLock::new(SharedClients::new()));

fn acquire(key: ClientKey) -> Client {
    let mut guard = SHARED_CLIENTS
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.get(key)
}

/// Discard any built shared clients, causing them to be rebuilt with their current configuration the next time they are accessed.
///
/// **Why it must exist**: `create_base_client` / `create_standard_client` only at **build time**
/// Read once `load_app_config().proxy.upstream_proxy`. These clients were previously `Lazy` static values -
/// It is finalized when used for the first time and will never be rebuilt. Therefore, after the user changes the upstream agent during operation,
/// The main request path has been hot updated (`UpstreamClient::rebuild_default_client`),
/// However, these shared clients still continue to run with the **old proxy**: the performance is "After changing the proxy, the chat is normal.
/// But token refresh/quota refresh/project analysis (loadCodeAssist) keeps failing."
/// The app must be restarted to restore.
///
/// Call point: `AxumServer::update_proxy` (upstream proxy hot update).
/// Use lazy rebuilding - if no one uses it later, there is no construction cost.
pub fn invalidate_shared_clients() {
    let mut guard = SHARED_CLIENTS
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let dropped = guard.built.len();
    guard.built.clear();
    if dropped > 0 {
        tracing::info!(
            dropped,
            "Discarded shared HTTP clients after upstream proxy change; rebuild on next use"
        );
    }
}

/// For testing: List the currently built client keys.
#[cfg(test)]
fn shared_client_keys() -> Vec<ClientKey> {
    let guard = SHARED_CLIENTS
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.built.keys().copied().collect()
}

/// Unified HTTP client (15s timeout, with JA3 emulation)
pub fn get_client() -> Client {
    acquire(ClientKey::new(15, true))
}

/// Standard HTTP client (15s timeout, **without** JA3 emulation)
pub fn get_standard_client() -> Client {
    acquire(ClientKey::new(15, false))
}

/// Standard client with long timeout (60s, **without** JA3 emulation)
pub fn get_long_standard_client() -> Client {
    acquire(ClientKey::new(60, false))
}

/// Apply the current upstream proxy configuration to the builder.
fn apply_upstream_proxy(builder: rquest::ClientBuilder, label: &str) -> rquest::ClientBuilder {
    if let Ok(config) = load_app_config() {
        let proxy_config = config.proxy.upstream_proxy;
        if proxy_config.enabled && !proxy_config.url.is_empty() {
            match Proxy::all(&proxy_config.url) {
                Ok(proxy) => {
                    tracing::info!("{} enabled upstream proxy: {}", label, proxy_config.url);
                    return builder.proxy(proxy);
                }
                Err(e) => {
                    tracing::error!("invalid_proxy_url: {}, error: {}", proxy_config.url, e);
                }
            }
        }
    }
    builder
}

/// Base client creation logic with JA3 Emulation
fn create_base_client(timeout_secs: u64) -> Client {
    let builder = Client::builder()
        .emulation(Emulation::Chrome123)
        .timeout(std::time::Duration::from_secs(timeout_secs));
    let builder = apply_upstream_proxy(builder, "HTTP shared client");

    tracing::info!("Initialized JA3/TLS Impersonation (Chrome123)");
    builder.build().unwrap_or_else(|_| Client::new())
}

/// Base client creation logic strictly WITHOUT JA3 Emulation (Pure Native)
pub(crate) fn create_standard_client(timeout_secs: u64) -> Client {
    // No .emulation(Emulation::Chrome123) here!
    let builder = Client::builder().timeout(std::time::Duration::from_secs(timeout_secs));
    let builder = apply_upstream_proxy(builder, "HTTP standard client");

    tracing::info!("Initialized Pure Native Standard Client");
    builder.build().unwrap_or_else(|_| Client::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emulated_15_key() -> ClientKey {
        ClientKey::new(15, true)
    }

    #[test]
    fn shared_client_is_cached_until_invalidated() {
        let _guard = crate::proxy::config::TEST_CONFIG_LOCK
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        invalidate_shared_clients();

        // Justification: warms the shared-client cache; the returned client handle is intentionally unused.
        let _ = get_client();
        assert!(
            shared_client_keys().contains(&emulated_15_key()),
            "get_client should output the client corresponding to key"
        );

        // There should be only one copy of the same key (HashMap semantics), and no new instance can be created for repeated acquisition.
        // Justification: warms the shared-client cache; the returned client handle is intentionally unused.
        let _ = get_client();
        let count = shared_client_keys()
            .iter()
            .filter(|k| **k == emulated_15_key())
            .count();
        assert_eq!(
            count, 1,
            "Retrieving the same key repeatedly should not produce a second instance"
        );
    }

    /// This is the core semantic of this fix: after a hot update of the upstream proxy, the built shared client must be discarded,
    /// Otherwise they will continue to run with the **old agent** (token refresh/quota refresh/project resolution).
    #[test]
    fn invalidate_drops_clients_so_they_rebuild_with_current_proxy() {
        let _guard = crate::proxy::config::TEST_CONFIG_LOCK
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        // Justification: warms the shared-client cache; the returned client handle is intentionally unused.
        let _ = get_client();
        assert!(shared_client_keys().contains(&emulated_15_key()));

        invalidate_shared_clients();
        assert!(
            !shared_client_keys().contains(&emulated_15_key()),
            "invalidate must discard the built client, otherwise after changing the proxy, the old proxy will still be used until restarted"
        );

        // Rebuild according to current configuration on next visit
        // Justification: warms the shared-client cache; the returned client handle is intentionally unused.
        let _ = get_client();
        assert!(
            shared_client_keys().contains(&emulated_15_key()),
            "Should be rebuilt after failure"
        );

        invalidate_shared_clients();
    }
}
