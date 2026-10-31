use super::*;

/// Anti-generation service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub port: u16,
    pub base_url: String,
    pub active_accounts: usize,
}

/// Anti-generation service global status
#[derive(Clone)]
pub struct ProxyServiceState {
    pub instance: Arc<RwLock<Option<ProxyServiceInstance>>>,
    pub monitor: Arc<RwLock<Option<Arc<ProxyMonitor>>>>,
    pub admin_server: Arc<RwLock<Option<AdminServerInstance>>>, // [NEW] Resident management server
    pub starting: Arc<AtomicBool>, // [NEW] Identifies whether it is starting to prevent deadlock
}

pub struct AdminServerInstance {
    pub axum_server: crate::proxy::AxumServer,
    pub server_handle: tokio::task::JoinHandle<()>,
}

impl AdminServerInstance {
    /// Gracefully stop admin server and wait for listening tasks to release port
    pub async fn stop(mut self) {
        self.axum_server.stop();
        // Justification: best-effort guarded wait; a timeout or inner failure is logged
        crate::error::record_ignored(
            tokio::time::timeout(
                std::time::Duration::from_millis(1000),
                &mut self.server_handle,
            )
            .await,
            "timeout wait",
        );
        if !self.server_handle.is_finished() {
            self.server_handle.abort();
        }
    }
}

/// Anti-generation service instance
pub struct ProxyServiceInstance {
    pub config: ProxyConfig,
    pub token_manager: Arc<TokenManager>,
    pub axum_server: crate::proxy::AxumServer,
}

impl ProxyServiceState {
    pub fn new() -> Self {
        Self {
            instance: Arc::new(RwLock::new(None)),
            monitor: Arc::new(RwLock::new(None)),
            admin_server: Arc::new(RwLock::new(None)),
            starting: Arc::new(AtomicBool::new(false)),
        }
    }
}
