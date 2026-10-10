use super::*;

#[tauri::command]
pub async fn get_antigravity_path(
    bypass_config: Option<bool>,
) -> Result<String, crate::error::AppError> {
    // 1. 优先从配置查询 (除非明确要求绕过)
    if bypass_config != Some(true) {
        if let Ok(config) = crate::modules::config::load_app_config() {
            if let Some(path) = config.antigravity_executable {
                if std::path::Path::new(&path).exists() {
                    return Ok(path);
                }
            }
        }
    }

    // 2. 执行实时探测 (附带完整诊断追踪与调用栈)
    let path = crate::modules::process::detect_antigravity_with_diagnostics(None)?;
    Ok(path.to_string_lossy().to_string())
}

/// 获取 Antigravity CLI (agy) 可执行文件路径
#[tauri::command]
pub async fn get_antigravity_cli_path(bypass_config: Option<bool>) -> Result<String, String> {
    // 1. 优先从配置查询 (除非明确要求绕过)
    if bypass_config != Some(true) {
        if let Ok(config) = crate::modules::config::load_app_config() {
            if let Some(path) = config.antigravity_cli_executable {
                if std::path::Path::new(&path).exists() {
                    return Ok(path);
                }
            }
        }
    }

    // 2. 执行实时探测
    match crate::modules::process::get_antigravity_cli_executable_path() {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("未找到 Antigravity CLI (agy) 安装路径".to_string()),
    }
}

/// 获取 Antigravity 启动参数
#[tauri::command]
pub async fn get_antigravity_args() -> Result<Vec<String>, String> {
    match crate::modules::process::get_args_from_running_process(None) {
        Some(args) => Ok(args),
        None => Err("未找到正在运行的 Antigravity 进程".to_string()),
    }
}
