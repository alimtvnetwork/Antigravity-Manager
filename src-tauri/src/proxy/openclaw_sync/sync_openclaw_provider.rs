use super::*;

pub fn sync_openclaw_provider(
    proxy_url: String,
    api_key: String,
    target_version: String, // "v1" (<2026.8.1) or "v2" (>=2026.8.1)
    models: Vec<String>,
    activate: bool,
    default_model: Option<String>,
) -> Result<(), String> {
    let _lock = acquire_openclaw_config_lock();
    let normalized_url = normalize_base_url(&proxy_url);
    if normalized_url.trim().is_empty() || normalized_url == "/v1" || api_key.trim().is_empty() {
        return Err("OpenClaw base URL and API key are required".to_string());
    }

    let models: Vec<String> = models
        .into_iter()
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .collect();

    let path = get_config_path().ok_or("Failed to get OpenClaw config directory")?;
    let mut config = read_openclaw_config(&path)?;
    create_backup(&path)?;

    // 1. 构建 Provider 模型数组
    let model_items: Vec<Value> = models
        .iter()
        .map(|m| {
            let (context_window, max_tokens, is_reasoning, input_modalities) =
                resolve_model_specs(m);
            json!({
                "id": m,
                "name": format!("{m} (Antigravity)"),
                "contextWindow": context_window,
                "maxTokens": max_tokens,
                "reasoning": is_reasoning,
                "input": input_modalities
            })
        })
        .collect();

    // 2. 写入 models.providers.antigravity-manager
    let provider_entry = json!({
        "baseUrl": normalized_url,
        "apiKey": api_key.trim(),
        "api": "openai-completions",
        "models": model_items
    });

    if config.get("models").is_none() {
        config["models"] = json!({});
    }
    config["models"]["mode"] = json!("merge");
    if config["models"].get("providers").is_none() {
        config["models"]["providers"] = json!({});
    }
    config["models"]["providers"][PROVIDER_ID] = provider_entry;

    // 3. 针对不同版本进行模型与白名单匹配
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }

    let is_v1 = target_version.eq_ignore_ascii_case("v1");

    if is_v1 {
        // v1.0 规范 (< 2026.8.1): 使用 agents.defaults.models 字典注册可用模型
        if config["agents"]["defaults"].get("models").is_none() {
            config["agents"]["defaults"]["models"] = json!({});
        }
        if let Some(map) = config["agents"]["defaults"]["models"].as_object_mut() {
            // 清理旧的 antigravity-manager 记录
            let keys_to_remove: Vec<String> = map
                .keys()
                .filter(|k| k.starts_with(&format!("{PROVIDER_ID}/")))
                .cloned()
                .collect();
            for k in keys_to_remove {
                map.remove(&k);
            }
            // 批量添加新同步模型
            for m in &models {
                map.insert(format!("{PROVIDER_ID}/{m}"), json!({}));
            }
        }
    } else {
        // v2.0 规范 (>= 2026.8.1): 使用 agents.defaults.modelPolicy.allow 白名单
        if config["agents"]["defaults"].get("modelPolicy").is_none() {
            config["agents"]["defaults"]["modelPolicy"] = json!({});
        }
        let allow_entry = format!("{PROVIDER_ID}/*");
        let allow_arr = config["agents"]["defaults"]["modelPolicy"]
            .get("allow")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut next_allow: Vec<Value> = allow_arr
            .into_iter()
            .filter(|v| {
                v.as_str()
                    .is_some_and(|s| !s.starts_with(&format!("{PROVIDER_ID}/")) && s != allow_entry)
            })
            .collect();
        next_allow.push(json!(allow_entry));
        config["agents"]["defaults"]["modelPolicy"]["allow"] = json!(next_allow);
    }

    // 4. 激活默认模型 (如勾选)
    let selected_default = default_model
        .as_deref()
        .filter(|m| !m.trim().is_empty())
        .or_else(|| models.first().map(String::as_str));

    if activate {
        if let Some(model_name) = selected_default {
            if config["agents"]["defaults"].get("model").is_none() {
                config["agents"]["defaults"]["model"] = json!({});
            }
            config["agents"]["defaults"]["model"]["primary"] =
                json!(format!("{PROVIDER_ID}/{model_name}"));
        }
    }

    atomically_write_config(&path, &config)
}

pub fn restore_openclaw_config() -> Result<(), String> {
    let _lock = acquire_openclaw_config_lock();
    let path = get_config_path().ok_or("Failed to get OpenClaw config directory")?;
    let backup_path = get_backup_path().ok_or("Failed to get OpenClaw config directory")?;
    if !backup_path.exists() {
        return Err("No backup file found".to_string());
    }
    let restored = read_openclaw_config(&backup_path)?;
    atomically_write_config(&path, &restored)?;
    fs::remove_file(backup_path).map_err(|e| format!("Failed to remove backup: {e}"))
}

pub fn clear_openclaw_config() -> Result<(), String> {
    let _lock = acquire_openclaw_config_lock();
    let path = get_config_path().ok_or("Failed to get OpenClaw config directory")?;
    if !path.exists() {
        return Ok(());
    }
    let mut config = read_openclaw_config(&path)?;
    create_backup(&path)?;

    // 1. 移除 models.providers.antigravity-manager
    if let Some(providers) = config
        .get_mut("models")
        .and_then(|m| m.get_mut("providers"))
        .and_then(|p| p.as_object_mut())
    {
        providers.remove(PROVIDER_ID);
    }

    // 2. 清理 agents.defaults.models 里面的 antigravity-manager/*
    if let Some(agent_models) = config
        .get_mut("agents")
        .and_then(|a| a.get_mut("defaults"))
        .and_then(|d| d.get_mut("models"))
        .and_then(|m| m.as_object_mut())
    {
        let to_remove: Vec<String> = agent_models
            .keys()
            .filter(|k| is_managed_provider(k))
            .cloned()
            .collect();
        for k in to_remove {
            agent_models.remove(&k);
        }
    }

    // 3. 清理 agents.defaults.modelPolicy.allow
    if let Some(allow) = config
        .get_mut("agents")
        .and_then(|a| a.get_mut("defaults"))
        .and_then(|d| d.get_mut("modelPolicy"))
        .and_then(|p| p.get_mut("allow"))
        .and_then(|v| v.as_array_mut())
    {
        allow.retain(|v| {
            v.as_str()
                .is_some_and(|s| s != format!("{PROVIDER_ID}/*") && !is_managed_provider(s))
        });
    }

    // 4. 若主模型指向 antigravity-manager，则清理
    if let Some(primary) = config
        .get("agents")
        .and_then(|a| a.get("defaults"))
        .and_then(|d| d.get("model"))
        .and_then(|m| m.get("primary"))
        .and_then(|p| p.as_str())
    {
        if is_managed_provider(primary) {
            if let Some(model_obj) = config["agents"]["defaults"]["model"].as_object_mut() {
                model_obj.remove("primary");
            }
        }
    }

    atomically_write_config(&path, &config)
}

pub fn read_openclaw_config_content() -> Result<String, String> {
    let _lock = acquire_openclaw_config_lock();
    let path = get_config_path().ok_or("Failed to get OpenClaw config directory")?;
    if !path.exists() {
        return Err(format!("Config file does not exist: {path:?}"));
    }
    let mut config = read_openclaw_config(&path)?;
    redact_json_value(&mut config);
    serde_json::to_string_pretty(&config).map_err(|e| format!("Failed to format config: {e}"))
}

#[tauri::command]
pub async fn get_openclaw_sync_status(proxy_url: Option<String>) -> Result<OpenClawStatus, String> {
    let (installed, version, detected_version_target) = check_openclaw_installed().await;
    tokio::task::spawn_blocking(move || {
        let _lock = acquire_openclaw_config_lock();
        let path = get_config_path();
        let has_backup = get_backup_path().is_some_and(|p| p.exists());

        let mut status = OpenClawStatus {
            installed,
            version,
            detected_version_target,
            is_synced: false,
            has_backup,
            current_base_url: None,
            files: vec![OPENCLAW_CONFIG_FILE.to_string()],
            configured_models: Vec::new(),
            is_active: false,
            default_model: None,
            synced_version: None,
        };

        let Some(path) = path else {
            return Ok(status);
        };
        let Ok(config) = read_openclaw_config(&path) else {
            return Ok(status);
        };

        // 读取 provider 配置
        if let Some(provider) = config
            .get("models")
            .and_then(|m| m.get("providers"))
            .and_then(|p| p.get(PROVIDER_ID))
        {
            if let Some(url) = provider.get("baseUrl").and_then(|u| u.as_str()) {
                status.current_base_url = Some(url.to_string());
                if let Some(expected) = &proxy_url {
                    status.is_synced = normalize_base_url(url) == normalize_base_url(expected);
                } else {
                    status.is_synced = true;
                }
            }

            if let Some(models) = provider.get("models").and_then(|m| m.as_array()) {
                status.configured_models = models
                    .iter()
                    .filter_map(|item| {
                        item.get("id")
                            .and_then(|id| id.as_str())
                            .map(str::to_string)
                    })
                    .collect();
            }
        }

        // 读取 active 模型与版本特征
        if let Some(primary) = config
            .get("agents")
            .and_then(|a| a.get("defaults"))
            .and_then(|d| d.get("model"))
            .and_then(|m| m.get("primary"))
            .and_then(|p| p.as_str())
        {
            if is_managed_provider(primary) {
                status.is_active = true;
                status.default_model = primary
                    .strip_prefix(&format!("{PROVIDER_ID}/"))
                    .map(str::to_string);
            }
        }

        // 判断当前配置更偏向 v1 还是 v2
        let has_v1_models = config
            .get("agents")
            .and_then(|a| a.get("defaults"))
            .and_then(|d| d.get("models"))
            .and_then(|m| m.as_object())
            .is_some_and(|map| map.keys().any(|k| is_managed_provider(k)));

        let has_v2_allow = config
            .get("agents")
            .and_then(|a| a.get("defaults"))
            .and_then(|d| d.get("modelPolicy"))
            .and_then(|p| p.get("allow"))
            .and_then(|v| v.as_array())
            .is_some_and(|arr| {
                arr.iter().any(|item| {
                    item.as_str()
                        .is_some_and(|s| s == format!("{PROVIDER_ID}/*") || is_managed_provider(s))
                })
            });

        if has_v2_allow {
            status.synced_version = Some("v2".to_string());
        } else if has_v1_models {
            status.synced_version = Some("v1".to_string());
        }

        Ok(status)
    })
    .await
    .unwrap_or_else(|_| Err("Failed to execute check".to_string()))
}

#[tauri::command]
pub async fn execute_openclaw_sync(
    proxy_url: String,
    api_key: String,
    target_version: String, // "v1" or "v2"
    models: Vec<String>,
    activate: bool,
    default_model: Option<String>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        sync_openclaw_provider(
            proxy_url,
            api_key,
            target_version,
            models,
            activate,
            default_model,
        )
    })
    .await
    .unwrap_or_else(|_| Err("Failed to execute sync".to_string()))
}

#[tauri::command]
pub async fn execute_openclaw_restore() -> Result<(), String> {
    tokio::task::spawn_blocking(restore_openclaw_config)
        .await
        .unwrap_or_else(|_| Err("Failed to execute restore".to_string()))
}

#[tauri::command]
pub async fn execute_openclaw_clear() -> Result<(), String> {
    tokio::task::spawn_blocking(clear_openclaw_config)
        .await
        .unwrap_or_else(|_| Err("Failed to execute clear".to_string()))
}

#[tauri::command]
pub async fn get_openclaw_config_content() -> Result<String, String> {
    tokio::task::spawn_blocking(read_openclaw_config_content)
        .await
        .unwrap_or_else(|_| Err("Failed to read config".to_string()))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
use crate::proxy::openclaw_sync::acquire_openclaw_config_lock::PROVIDER_ID;

    #[test]
    fn test_classify_openclaw_version() {
        assert_eq!(classify_openclaw_version("2026.8.1"), "v2");
        assert_eq!(classify_openclaw_version("2026.9.2"), "v2");
        assert_eq!(classify_openclaw_version("2.0.0"), "v2");
        assert_eq!(classify_openclaw_version("2026.7.749"), "v1");
        assert_eq!(classify_openclaw_version("2026.7.1"), "v1");
        assert_eq!(classify_openclaw_version("1.0.0"), "v1");
    }

    #[test]
    fn test_extract_version() {
        assert_eq!(extract_version("openclaw 2026.8.1"), "2026.8.1");
        assert_eq!(extract_version("v2026.7.749 (local)"), "2026.7.749");
    }

    #[test]
    fn test_resolve_model_specs() {
        // Gemini 3.8 / 3.0 / 2.5 系列 (1024000 上下文, 65536 输出)
        let (cw_gemini, max_gemini, _, input_gemini) = resolve_model_specs("gemini-3.8-flash-high");
        assert_eq!(cw_gemini, 1024000);
        assert_eq!(max_gemini, 65536);
        assert_eq!(input_gemini, vec!["text", "image"]);

        // Claude 系列 (256000 上下文, 65536 输出)
        let (cw_claude, max_claude, _, input_claude) =
            resolve_model_specs("claude-3-5-sonnet-latest");
        assert_eq!(cw_claude, 256000);
        assert_eq!(max_claude, 65536);
        assert_eq!(input_claude, vec!["text", "image"]);

        let (cw_claude_37, max_claude_37, r_claude_37, _) =
            resolve_model_specs("claude-3-7-sonnet-thinking");
        assert_eq!(cw_claude_37, 256000);
        assert_eq!(max_claude_37, 65536);
        assert!(r_claude_37);
    }
}
