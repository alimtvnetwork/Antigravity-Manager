use super::*;

pub use crate::modules::update_checker::UpdateInfo;

#[derive(serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NativeUpdateMetadata {
    pub rid: tauri::ResourceId,
    pub current_version: String,
    pub version: String,
    pub date: Option<String>,
    pub body: Option<String>,
    pub raw_json: serde_json::Value,
}

/// 检测 GitHub releases 更新
#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    modules::logger::log_info("收到前端触发的更新检查请求");
    crate::modules::update_checker::check_for_updates().await
}

/// 基于动态通道和端点进行原生更新检查，支持预发布版 (Beta) 与正式版自动更新分离
#[tauri::command]
pub async fn check_native_update<R: tauri::Runtime>(
    webview: tauri::Webview<R>,
    endpoint: Option<String>,
    proxy: Option<String>,
) -> Result<Option<NativeUpdateMetadata>, String> {
    use tauri_plugin_updater::UpdaterExt;
    use url::Url;

    let target_endpoint = if let Some(ep) = endpoint.filter(|s| !s.trim().is_empty()) {
        ep
    } else {
        let settings = crate::modules::update_checker::load_update_settings().unwrap_or_default();
        match settings.update_channel {
            crate::modules::update_checker::UpdateChannel::Beta => {
                crate::modules::update_checker::PREVIEW_UPDATER_JSON_URL.to_string()
            }
            crate::modules::update_checker::UpdateChannel::Stable => {
                crate::modules::update_checker::STABLE_UPDATER_JSON_URL.to_string()
            }
        }
    };

    crate::modules::logger::log_info(&format!("原生更新器准备检查目标地址: {}", target_endpoint));

    let mut builder = webview.updater_builder();

    let url = Url::parse(&target_endpoint)
        .map_err(|e| format!("无效的更新地址 '{}': {}", target_endpoint, e))?;
    builder = builder.endpoints(vec![url]).map_err(|e| e.to_string())?;

    let proxy_url = proxy.or_else(crate::modules::update_checker::get_upstream_proxy_url);
    if let Some(proxy_str) = proxy_url {
        if let Ok(proxy_parsed) = Url::parse(&proxy_str) {
            crate::modules::logger::log_info(&format!("原生更新器应用代理配置: {}", proxy_str));
            builder = builder.proxy(proxy_parsed);
        }
    }

    builder = builder.version_comparator(|current, release| {
        crate::modules::update_checker::compare_versions(
            &release.version.to_string(),
            &current.to_string(),
        )
    });

    let updater = builder.build().map_err(|e| {
        let msg = format!("构建原生更新器失败: {}", e);
        crate::modules::logger::log_error(&msg);
        msg
    })?;

    let update = updater.check().await.map_err(|e| {
        let msg = format!("原生更新器检查失败: {}", e);
        crate::modules::logger::log_error(&msg);
        msg
    })?;

    if let Some(update) = update {
        crate::modules::logger::log_info(&format!(
            "原生更新器发现可用更新: {} (当前版本: {})",
            update.version, update.current_version
        ));
        let current_version = update.current_version.clone();
        let version = update.version.clone();
        let body = update.body.clone();
        let raw_json = update.raw_json.clone();
        let formatted_date = update.date.and_then(|date| {
            date.format(&time::format_description::well_known::Rfc3339)
                .ok()
        });
        let rid = webview.resources_table().add(update);
        let metadata = NativeUpdateMetadata {
            rid,
            current_version,
            version,
            date: formatted_date,
            body,
            raw_json,
        };
        Ok(Some(metadata))
    } else {
        crate::modules::logger::log_info("原生更新器未检测到更新");
        Ok(None)
    }
}

#[tauri::command]
pub async fn should_check_updates() -> Result<bool, String> {
    let settings = crate::modules::update_checker::load_update_settings()?;
    Ok(crate::modules::update_checker::should_check_for_updates(
        &settings,
    ))
}

#[tauri::command]
pub async fn should_check_updates_on_startup() -> Result<bool, String> {
    let settings = crate::modules::update_checker::load_update_settings()?;
    Ok(settings.auto_check)
}

#[tauri::command]
pub async fn check_update_via_script() -> Result<UpdateInfo, String> {
    crate::modules::update_checker::check_update_via_script().await
}

#[tauri::command]
pub async fn run_installer_update() -> Result<String, String> {
    crate::modules::update_checker::run_installer_update().await
}

#[tauri::command]
pub async fn update_last_check_time() -> Result<(), String> {
    crate::modules::update_checker::update_last_check_time()
}

/// 检测是否通过 Homebrew Cask 安装
#[tauri::command]
pub async fn check_homebrew_installation() -> Result<bool, String> {
    Ok(crate::modules::update_checker::is_homebrew_installed())
}

/// 检测是否以 AppImage 方式运行（Linux 专用）
/// Tauri 的原生更新器在 Linux 上只支持 AppImage，
/// RPM/DEB 安装的用户不应触发原生自动更新以避免 ENOEXEC 错误。
#[tauri::command]
pub async fn check_appimage_installation() -> Result<bool, String> {
    Ok(crate::modules::update_checker::is_appimage_running())
}

/// 通过 Homebrew Cask 升级应用
#[tauri::command]
pub async fn brew_upgrade_cask() -> Result<String, String> {
    modules::logger::log_info("收到前端触发的 Homebrew 升级请求");
    crate::modules::update_checker::brew_upgrade_cask().await
}

/// 获取更新设置
#[tauri::command]
pub async fn get_update_settings() -> Result<crate::modules::update_checker::UpdateSettings, String>
{
    crate::modules::update_checker::load_update_settings()
}

/// 保存更新设置
#[tauri::command]
pub async fn save_update_settings(
    settings: crate::modules::update_checker::UpdateSettings,
) -> Result<(), String> {
    crate::modules::update_checker::save_update_settings(&settings)
}
