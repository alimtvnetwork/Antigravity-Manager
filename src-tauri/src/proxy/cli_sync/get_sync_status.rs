use super::*;

/// 读取当前配置并检测同步状态
pub fn get_sync_status(app: &CliApp, proxy_url: &str) -> (bool, bool, Option<String>) {
    let files = app.config_files();
    if files.is_empty() {
        return (false, false, None);
    }

    let mut all_synced = true;
    let mut has_backup = false;
    let mut current_base_url = None;

    for file in &files {
        // 使用更简单的命名规则: original_name + .antigravity.bak
        let backup_path = file
            .path
            .with_file_name(format!("{}.antigravity.bak", file.name));

        if backup_path.exists() {
            has_backup = true;
        }

        // 如果物理文件不存在
        // 如果物理文件不存在
        if !file.path.exists() {
            // Gemini 的 settings.json/config.json 只要有一个存在即可，或者都不存在（视为未同步）
            if app == &CliApp::Gemini
                && (file.name == "settings.json" || file.name == "config.json")
            {
                continue;
            }
            all_synced = false;
            continue;
        }

        let content = match fs::read_to_string(&file.path) {
            Ok(c) => c,
            Err(_) => {
                all_synced = false;
                continue;
            }
        };

        match app {
            CliApp::Claude => {
                if file.name == "settings.json" {
                    let json: Value = serde_json::from_str(&content).unwrap_or_default();
                    let url = json
                        .get("env")
                        .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
                        .and_then(|v| v.as_str());
                    if let Some(u) = url {
                        current_base_url = Some(u.to_string());
                        if u.trim_end_matches('/') != proxy_url.trim_end_matches('/') {
                            all_synced = false;
                        }
                    } else {
                        all_synced = false;
                    }
                } else if file.name == ".claude.json" {
                    let json: Value = serde_json::from_str(&content).unwrap_or_default();
                    if json.get("hasCompletedOnboarding") != Some(&Value::Bool(true)) {
                        all_synced = false;
                    }
                }
            }
            CliApp::Codex => {
                if file.name == "config.toml" {
                    // 正则匹配 base_url
                    let re =
                        regex::Regex::new(r#"(?m)^\s*base_url\s*=\s*['"]([^'"]+)['"]"#).unwrap();
                    if let Some(caps) = re.captures(&content) {
                        let url = &caps[1];
                        current_base_url = Some(url.to_string());
                        if url.trim_end_matches('/') != proxy_url.trim_end_matches('/') {
                            all_synced = false;
                        }
                    } else {
                        all_synced = false;
                    }
                }
            }
            CliApp::Gemini => {
                if file.name == ".env" {
                    let re = regex::Regex::new(r#"(?m)^GOOGLE_GEMINI_BASE_URL=(.*)$"#).unwrap();
                    if let Some(caps) = re.captures(&content) {
                        let url = caps[1].trim();
                        current_base_url = Some(url.to_string());
                        if url.trim_end_matches('/') != proxy_url.trim_end_matches('/') {
                            all_synced = false;
                        }
                    } else {
                        all_synced = false;
                    }
                }
            }
            CliApp::OpenCode => {
                if file.name == "config.json" {
                    let json: Value = serde_json::from_str(&content).unwrap_or_default();
                    let url = json
                        .get("providers")
                        .and_then(|p| p.get("openai"))
                        .and_then(|o| o.get("baseURL"))
                        .and_then(|v| v.as_str());
                    if let Some(u) = url {
                        current_base_url = Some(u.to_string());
                        if u.trim_end_matches('/') != proxy_url.trim_end_matches('/') {
                            all_synced = false;
                        }
                    } else {
                        all_synced = false;
                    }
                }
            }
            CliApp::JeikCode => {
                if file.name == "config.toml" {
                    use toml_edit::DocumentMut;
                    if let Ok(doc) = content.parse::<DocumentMut>() {
                        let normalized_proxy = proxy_url
                            .trim_end_matches('/')
                            .trim_end_matches("/v1")
                            .trim_end_matches('/');

                        let mut is_ag_synced = false;
                        if let Some(accounts) =
                            doc.get("provider_accounts").and_then(|i| i.as_table())
                        {
                            // 优先检查 antigravity-manager 账号
                            if let Some(ag_item) = accounts.get("antigravity-manager") {
                                if let Some(base_url) =
                                    ag_item.get("base_url").and_then(|v| v.as_str())
                                {
                                    current_base_url = Some(base_url.to_string());
                                    let norm = base_url
                                        .trim_end_matches('/')
                                        .trim_end_matches("/v1")
                                        .trim_end_matches('/');
                                    let provider_type = ag_item
                                        .get("provider")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    if norm == normalized_proxy && provider_type == "anthropic" {
                                        is_ag_synced = true;
                                    }
                                }
                            }

                            // 若尚未找到，但有其他同 URL 的账号，记录其 URL
                            if current_base_url.is_none() {
                                for (_, acc_item) in accounts.iter() {
                                    if let Some(base_url) =
                                        acc_item.get("base_url").and_then(|v| v.as_str())
                                    {
                                        let norm = base_url
                                            .trim_end_matches('/')
                                            .trim_end_matches("/v1")
                                            .trim_end_matches('/');
                                        if norm == normalized_proxy {
                                            current_base_url = Some(base_url.to_string());
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        // 验证是否已同步最核心模型
                        let has_core_models =
                            if let Some(models) = doc.get("models").and_then(|i| i.as_table()) {
                                models.contains_key("claude-sonnet-4-6")
                            } else {
                                false
                            };

                        if is_ag_synced && has_core_models {
                            // 同步完全正常
                        } else {
                            all_synced = false;
                        }
                    } else {
                        all_synced = false;
                    }
                }
            }
            CliApp::GrokBuild => {
                if file.name == "config.toml" {
                    use toml_edit::DocumentMut;
                    if let Ok(doc) = content.parse::<DocumentMut>() {
                        let normalized_proxy = proxy_url
                            .trim_end_matches('/')
                            .trim_end_matches("/v1")
                            .trim_end_matches('/');

                        let mut matched = false;
                        if let Some(models) = doc.get("model").and_then(|i| i.as_table()) {
                            for (_, m_item) in models.iter() {
                                if let Some(b_url) = m_item.get("base_url").and_then(|v| v.as_str())
                                {
                                    current_base_url = Some(b_url.to_string());
                                    let norm = b_url
                                        .trim_end_matches('/')
                                        .trim_end_matches("/v1")
                                        .trim_end_matches('/');
                                    if norm == normalized_proxy {
                                        matched = true;
                                        break;
                                    }
                                }
                            }
                        }

                        if !matched {
                            all_synced = false;
                        }
                    } else {
                        all_synced = false;
                    }
                }
            }
        }
    }

    (all_synced, has_backup, current_base_url)
}
