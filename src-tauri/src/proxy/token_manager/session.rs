//! 会话与调度配置：粘性会话绑定、调度模式、偏好账号。

use super::TokenManager;
use crate::proxy::sticky_config::SchedulingMode;
use crate::proxy::sticky_config::StickySessionConfig;

impl TokenManager {
    /// 获取当前调度配置
    pub async fn get_sticky_config(&self) -> StickySessionConfig {
        self.sticky_config.read().await.clone()
    }

    /// 更新调度配置
    pub async fn update_sticky_config(&self, new_config: StickySessionConfig) {
        let mut config = self.sticky_config.write().await;
        *config = new_config;
        tracing::debug!("Scheduling configuration updated: {:?}", *config);
    }

    /// [NEW] 更新熔断器配置
    pub async fn update_circuit_breaker_config(&self, config: crate::models::CircuitBreakerConfig) {
        let mut lock = self.circuit_breaker_config.write().await;
        *lock = config;
        tracing::debug!("Circuit breaker configuration updated");
    }

    /// [NEW] 获取熔断器配置
    pub async fn get_circuit_breaker_config(&self) -> crate::models::CircuitBreakerConfig {
        self.circuit_breaker_config.read().await.clone()
    }

    /// 清除特定会话的粘性映射
    #[allow(dead_code)]
    pub fn clear_session_binding(&self, session_id: &str) {
        self.session_accounts.remove(session_id);
    }

    /// [FIX] 遭遇 429/529 等限流或过载时解绑会话并清空最近使用记录，打破粘性死锁
    pub async fn unbind_session_and_clear_last_used(&self, session_id: Option<&str>) {
        if let Some(sid) = session_id {
            self.session_accounts.remove(sid);
        }
        let mut last_used = self.last_used_account.lock().await;
        *last_used = None;
    }

    /// 获取当前生效的负载均衡调度模式（从内存中安全读取，无需触碰磁盘）
    pub async fn get_scheduling_mode(&self) -> crate::proxy::sticky_config::SchedulingMode {
        self.sticky_config.read().await.mode
    }

    /// 清除所有会话的粘性映射
    pub fn clear_all_sessions(&self) {
        self.session_accounts.clear();
    }

    // ===== [FIX #820] 固定账号模式相关方法 =====

    /// 设置优先使用的账号ID（固定账号模式）
    /// 传入 Some(account_id) 启用固定账号模式，传入 None 恢复轮询模式
    pub async fn set_preferred_account(&self, account_id: Option<String>) {
        let mut preferred = self.preferred_account_id.write().await;
        if let Some(ref id) = account_id {
            tracing::info!("🔒 [FIX #820] Fixed account mode enabled: {}", id);
        } else {
            tracing::info!("🔄 [FIX #820] Round-robin mode enabled (no preferred account)");
        }
        *preferred = account_id;
    }

    /// 获取当前优先使用的账号ID
    pub async fn get_preferred_account(&self) -> Option<String> {
        self.preferred_account_id.read().await.clone()
    }
}
