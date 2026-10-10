use super::*;

/// 反代服务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub port: u16,
    pub base_url: String,
    pub active_accounts: usize,
}

/// 反代服务全局状态
#[derive(Clone)]
pub struct ProxyServiceState {
    pub instance: Arc<RwLock<Option<ProxyServiceInstance>>>,
    pub monitor: Arc<RwLock<Option<Arc<ProxyMonitor>>>>,
    pub admin_server: Arc<RwLock<Option<AdminServerInstance>>>, // [NEW] 常驻管理服务器
    pub starting: Arc<AtomicBool>, // [NEW] 标识是否正在启动中，防止死锁
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

/// 反代服务实例
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
