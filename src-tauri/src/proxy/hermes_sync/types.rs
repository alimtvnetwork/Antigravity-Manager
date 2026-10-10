use super::*;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;
const VERSION_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

const HERMES_DIR: &str = ".hermes";
const HERMES_CONFIG_FILE: &str = "config.yaml";
const BACKUP_SUFFIX: &str = ".antigravity-manager.bak";
const PROVIDER_ID: &str = "antigravity-manager";
const PROVIDER_DISPLAY_NAME: &str = "Antigravity Manager";
const PROVIDER_REF: &str = "custom:antigravity-manager";
const EMPTY_CONFIG: &str = "{}\n";

pub(crate) static HERMES_CONFIG_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn acquire_hermes_config_lock() -> std::sync::MutexGuard<'static, ()> {
    HERMES_CONFIG_MUTEX.lock().unwrap_or_else(|poisoned| {
        tracing::warn!("HERMES_CONFIG_MUTEX was poisoned, recovering lock");
        poisoned.into_inner()
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HermesStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub is_synced: bool,
    pub has_backup: bool,
    pub current_base_url: Option<String>,
    pub files: Vec<String>,
    pub discover_models: bool,
    pub configured_models: Vec<String>,
    pub is_active: bool,
    pub default_model: Option<String>,
}
