use super::*;

/// 执行同步逻辑
pub fn sync_config(
    app: &CliApp,
    proxy_url: &str,
    api_key: &str,
    model: Option<&str>,
) -> Result<(), String> {
    let files = app.config_files();

    for file in &files {
        // Gemini 兼容性逻辑：优先使用 settings.json
        if app == &CliApp::Gemini && file.name == "config.json" && !file.path.exists() {
            let settings_path = file.path.with_file_name("settings.json");
            if settings_path.exists() {
                continue;
            }
        }

        if let Some(parent) = file.path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("无法创建目录: {}", e))?;
        }

        // [New Feature] 自动备份：如果文件存在且没有备份，创建 .antigravity.bak 备份
        // 这样可以保留用户最初的配置，后续多次同步不会覆盖这个备份
        if file.path.exists() {
            let backup_path = file
                .path
                .with_file_name(format!("{}.antigravity.bak", file.name));
            if !backup_path.exists() {
                if let Err(e) = fs::copy(&file.path, &backup_path) {
                    tracing::warn!("Failed to create backup for {}: {}", file.name, e);
                } else {
                    tracing::info!("Created backup for {}: {:?}", file.name, backup_path);
                }
            }
        }

        let mut content = if file.path.exists() {
            fs::read_to_string(&file.path).unwrap_or_default()
        } else {
            String::new()
        };

        match app {
            CliApp::Claude => {
                if file.name == ".claude.json" {
                    let mut json: Value =
                        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
                    if let Some(obj) = json.as_object_mut() {
                        obj.insert("hasCompletedOnboarding".to_string(), Value::Bool(true));
                    }
                    content = serde_json::to_string_pretty(&json).unwrap();
                } else if file.name == "settings.json" {
                    let mut json: serde_json::Value =
                        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
                    if json.as_object().is_none() {
                        json = serde_json::json!({});
                    }
                    let env = json
                        .as_object_mut()
                        .unwrap()
                        .entry("env")
                        .or_insert(serde_json::json!({}));
                    if let Some(env_obj) = env.as_object_mut() {
                        env_obj.insert(
                            "ANTHROPIC_BASE_URL".to_string(),
                            Value::String(proxy_url.to_string()),
                        );
                        if !api_key.is_empty() {
                            if proxy_url.contains("apikey.fun") || proxy_url.contains("apikey.fan")
                            {
                                env_obj.insert(
                                    "ANTHROPIC_AUTH_TOKEN".to_string(),
                                    Value::String(api_key.to_string()),
                                );
                                env_obj.insert(
                                    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                                    Value::String("1".to_string()),
                                );
                                env_obj.insert(
                                    "CLAUDE_CODE_ATTRIBUTION_HEADER".to_string(),
                                    Value::String("0".to_string()),
                                );
                                env_obj.remove("ANTHROPIC_API_KEY");
                            } else {
                                env_obj.insert(
                                    "ANTHROPIC_API_KEY".to_string(),
                                    Value::String(api_key.to_string()),
                                );
                                // [FIX] 避免冲突：如果存在则移除 ANTHROPIC_AUTH_TOKEN
                                env_obj.remove("ANTHROPIC_AUTH_TOKEN");
                            }

                            // [FIX] 清理可能来自其他 Provider 的模型覆盖设置
                            env_obj.remove("ANTHROPIC_MODEL");
                            env_obj.remove("ANTHROPIC_DEFAULT_HAIKU_MODEL");
                            env_obj.remove("ANTHROPIC_DEFAULT_OPUS_MODEL");
                            env_obj.remove("ANTHROPIC_DEFAULT_SONNET_MODEL");
                        } else {
                            // 如果 API Key 为空，则移除该键，避免设置为空字符串
                            env_obj.remove("ANTHROPIC_API_KEY");
                            env_obj.remove("ANTHROPIC_AUTH_TOKEN");
                        }
                    }

                    if let Some(m) = model {
                        // 注意：Claude Code 的官方配置中，当前选定模型放在根节点的 model 字段
                        json.as_object_mut()
                            .unwrap()
                            .insert("model".to_string(), Value::String(m.to_string()));
                    }
                    content = serde_json::to_string_pretty(&json).unwrap();
                }
            }
            CliApp::Codex => {
                if file.name == "auth.json" {
                    let mut json: Value =
                        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
                    if let Some(obj) = json.as_object_mut() {
                        obj.insert(
                            "OPENAI_API_KEY".to_string(),
                            Value::String(api_key.to_string()),
                        );
                        if proxy_url.contains("apikey.fun") || proxy_url.contains("apikey.fan") {
                            obj.remove("OPENAI_BASE_URL");
                        } else {
                            // Codex 的 auth.json 似乎也支持 OPENAI_BASE_URL，但 ccs 没写，我们也同步写一下
                            obj.insert(
                                "OPENAI_BASE_URL".to_string(),
                                Value::String(proxy_url.to_string()),
                            );
                        }
                    }
                    content = serde_json::to_string_pretty(&json).unwrap();
                } else if file.name == "config.toml" {
                    use toml_edit::{value, DocumentMut};
                    let mut doc = content
                        .parse::<DocumentMut>()
                        .unwrap_or_else(|_| DocumentMut::new());

                    // 必须使用 custom 提供商，Codex 不支持原生的 codex provider
                    let provider_key = "custom";
                    let is_apikey_fun =
                        proxy_url.contains("apikey.fun") || proxy_url.contains("apikey.fan");
                    let display_name = if is_apikey_fun {
                        "APIKEY.FUN"
                    } else {
                        "Custom Node"
                    };

                    // 优先设置 Root Keys 确保位于顶部
                    doc.insert("model_provider", value(provider_key));

                    if is_apikey_fun {
                        doc.insert("model", value("gpt-5.5"));
                        doc.insert("review_model", value("gpt-5.5"));
                        doc.insert("model_reasoning_effort", value("high"));
                        doc.insert("disable_response_storage", value(true));
                        doc.insert("network_access", value("enabled"));
                        doc.insert("windows_wsl_setup_acknowledged", value(true));
                        doc.insert("model_context_window", value(270000));
                        doc.insert("model_auto_compact_token_limit", value(270000));
                        doc.insert("effective_context_window_percent", value(95));
                    } else {
                        if let Some(m) = model {
                            doc.insert("model", value(m));
                            let cw = if m.contains("gemini") {
                                1_024_000
                            } else if m.contains("claude") {
                                256_000
                            } else {
                                128_000
                            };
                            doc.insert("model_context_window", value(cw));
                            doc.insert("model_auto_compact_token_limit", value(cw));
                        }
                    }

                    // 移除可能的根级别旧配置
                    doc.remove("openai_api_key");
                    doc.remove("openai_base_url");

                    // 设置层级 [model_providers.custom]
                    let providers = doc
                        .entry("model_providers")
                        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
                    if let Some(p_table) = providers.as_table_mut() {
                        let custom = p_table
                            .entry(provider_key)
                            .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
                        if let Some(c_table) = custom.as_table_mut() {
                            c_table.insert("name", value(display_name));
                            c_table.insert("wire_api", value("responses"));
                            c_table.insert("requires_openai_auth", value(true));
                            c_table.insert("base_url", value(proxy_url.to_string()));
                            if let Some(m) = model {
                                c_table.insert("model", value(m));
                            }
                        }
                    }

                    if is_apikey_fun {
                        let features = doc
                            .entry("features")
                            .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
                        if let Some(f_table) = features.as_table_mut() {
                            f_table.insert("goals", value(true));
                        }
                    }
                    content = doc.to_string();
                }
            }
            CliApp::Gemini => {
                if file.name == ".env" {
                    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                    let mut found_url = false;
                    let mut found_key = false;
                    for line in lines.iter_mut() {
                        if line.starts_with("GOOGLE_GEMINI_BASE_URL=") {
                            *line = format!("GOOGLE_GEMINI_BASE_URL={}", proxy_url);
                            found_url = true;
                        } else if line.trim().starts_with("GEMINI_API_KEY=") {
                            *line = format!("GEMINI_API_KEY={}", api_key);
                            found_key = true;
                        }
                    }
                    if !found_url {
                        lines.push(format!("GOOGLE_GEMINI_BASE_URL={}", proxy_url));
                    }
                    if !found_key {
                        lines.push(format!("GEMINI_API_KEY={}", api_key));
                    }
                    if let Some(m) = model {
                        let mut found_model = false;
                        for line in lines.iter_mut() {
                            if line.starts_with("GOOGLE_GEMINI_MODEL=") {
                                *line = format!("GOOGLE_GEMINI_MODEL={}", m);
                                found_model = true;
                            }
                        }
                        if !found_model {
                            lines.push(format!("GOOGLE_GEMINI_MODEL={}", m));
                        }
                    }
                    content = lines.join("\n");
                } else if file.name == "settings.json" || file.name == "config.json" {
                    let mut json: Value =
                        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
                    if json.as_object().is_none() {
                        json = serde_json::json!({});
                    }
                    let sec = json
                        .as_object_mut()
                        .unwrap()
                        .entry("security")
                        .or_insert(serde_json::json!({}));
                    let auth = sec
                        .as_object_mut()
                        .unwrap()
                        .entry("auth")
                        .or_insert(serde_json::json!({}));
                    if let Some(auth_obj) = auth.as_object_mut() {
                        auth_obj.insert(
                            "selectedType".to_string(),
                            Value::String("gemini-api-key".to_string()),
                        );
                    }
                    content = serde_json::to_string_pretty(&json).unwrap();
                }
            }
            CliApp::OpenCode => {
                if file.name == "config.json" {
                    let mut json: Value =
                        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
                    if json.as_object().is_none() {
                        json = serde_json::json!({});
                    }
                    let providers = json
                        .as_object_mut()
                        .unwrap()
                        .entry("providers")
                        .or_insert(serde_json::json!({}));
                    let openai = providers
                        .as_object_mut()
                        .unwrap()
                        .entry("openai")
                        .or_insert(serde_json::json!({}));
                    if let Some(openai_obj) = openai.as_object_mut() {
                        openai_obj
                            .insert("baseURL".to_string(), Value::String(proxy_url.to_string()));
                        if !api_key.is_empty() {
                            openai_obj
                                .insert("apiKey".to_string(), Value::String(api_key.to_string()));
                        }
                    }
                    content = serde_json::to_string_pretty(&json).unwrap();
                }
            }
            CliApp::JeikCode => {
                if file.name == "config.toml" {
                    content = sync_jeikcode_toml_content(&content, proxy_url, api_key, model)?;
                }
            }
            CliApp::GrokBuild => {
                if file.name == "config.toml" {
                    use toml_edit::{value, DocumentMut, Item, Table};
                    let mut doc = content
                        .parse::<DocumentMut>()
                        .unwrap_or_else(|_| DocumentMut::new());

                    let normalized_proxy_url = {
                        let trimmed = proxy_url.trim().trim_end_matches('/');
                        if trimmed.ends_with("/v1") {
                            trimmed.to_string()
                        } else {
                            format!("{}/v1", trimmed)
                        }
                    };

                    let core_models = get_core_gateway_models();

                    let default_model_id = match model {
                        Some(m) if !m.is_empty() => m,
                        _ => "gemini-3.8-flash-high",
                    };

                    // 1. 设置 [models] default 为真实选定的网关模型
                    let models_sec = doc.entry("models").or_insert(Item::Table(Table::new()));
                    if let Some(m_table) = models_sec.as_table_mut() {
                        m_table.insert("default", value(default_model_id));
                    }

                    // 2. 注入所有网关真实模型到 [model."<id>"]
                    let model_sec = doc.entry("model").or_insert(Item::Table(Table::new()));
                    if let Some(m_table) = model_sec.as_table_mut() {
                        for m in core_models {
                            let entry = m_table.entry(m.id).or_insert(Item::Table(Table::new()));
                            if let Some(t) = entry.as_table_mut() {
                                t.insert("model", value(m.id));
                                t.insert("base_url", value(&normalized_proxy_url));
                                t.insert("name", value(m.id));
                                t.insert("api_backend", value("responses"));
                                t.insert("context_window", value(m.context_window));
                                t.insert("image_input", value(true));
                                if !api_key.is_empty() {
                                    t.insert("api_key", value(api_key));
                                }
                            }
                        }
                    }

                    content = doc.to_string();
                }
            }
        }

        // 使用临时文件原子写入
        let tmp_path = file.path.with_extension("tmp");
        fs::write(&tmp_path, &content).map_err(|e| format!("写入临时文件失败: {}", e))?;
        fs::rename(&tmp_path, &file.path).map_err(|e| format!("重命名配置文件失败: {}", e))?;
    }

    Ok(())
}

// Tauri Commands

#[tauri::command]
pub async fn get_cli_sync_status(app_type: CliApp, proxy_url: String) -> Result<CliStatus, String> {
    tokio::task::spawn_blocking(move || {
        let (installed, version) = check_cli_installed(&app_type);
        let (is_synced, has_backup, current_base_url) = if installed {
            get_sync_status(&app_type, &proxy_url)
        } else {
            (false, false, None)
        };

        Ok(CliStatus {
            installed,
            version,
            is_synced,
            has_backup,
            current_base_url,
            files: app_type
                .config_files()
                .into_iter()
                .map(|f| f.name)
                .collect(),
        })
    })
    .await
    .unwrap_or_else(|_| Err("Task panicked".to_string()))
}

#[tauri::command]
pub async fn execute_cli_sync(
    app_type: CliApp,
    proxy_url: String,
    api_key: String,
    model: Option<String>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        sync_config(&app_type, &proxy_url, &api_key, model.as_deref())
    })
    .await
    .unwrap_or_else(|_| Err("Task panicked".to_string()))
}

#[tauri::command]
pub async fn execute_cli_restore(app_type: CliApp) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let files = app_type.config_files();
        let mut restored_count = 0;

        // 尝试从备份恢复
        for file in &files {
            let backup_path = file
                .path
                .with_file_name(format!("{}.antigravity.bak", file.name));
            if backup_path.exists() {
                // 还原：覆盖原文件
                if let Err(e) = fs::rename(&backup_path, &file.path) {
                    return Err(format!("恢复备份失败 {}: {}", file.name, e));
                }
                restored_count += 1;
            }
        }

        if restored_count > 0 {
            // 如果成功恢复了至少一个备份，就认为是恢复成功
            return Ok(());
        }

        // 如果没有备份，则执行原来的逻辑：恢复为默认配置
        let default_url = app_type.default_url();
        // 恢复默认时清空 API Key，让用户重新授权或使用官方 Key
        sync_config(&app_type, default_url, "", None)
    })
    .await
    .unwrap_or_else(|_| Err("Task panicked".to_string()))
}

#[tauri::command]
pub async fn get_cli_config_content(
    app_type: CliApp,
    file_name: Option<String>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let files = app_type.config_files();
        let file = if let Some(name) = file_name {
            files
                .into_iter()
                .find(|f| f.name == name)
                .ok_or("找不到指定的文件".to_string())?
        } else {
            files
                .into_iter()
                .next()
                .ok_or("找不到配置文件".to_string())?
        };

        if !file.path.exists() {
            return Err("配置文件不存在".to_string());
        }
        fs::read_to_string(&file.path).map_err(|e| format!("读取配置文件失败: {}", e))
    })
    .await
    .unwrap_or_else(|_| Err("Task panicked".to_string()))
}

#[derive(Debug, Clone, Copy)]
pub struct CoreModel {
    pub id: &'static str,
    pub context_window: i64,
}
