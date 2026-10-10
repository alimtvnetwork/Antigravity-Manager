use super::*;

/// 导出网关全量核心模型（仅剔除带 -日期数字 的后缀版本）
pub fn get_core_gateway_models() -> &'static [CoreModel] {
    &[
        // Claude 系列 (256k 上下文)
        CoreModel {
            id: "claude-sonnet-4-6",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-sonnet-4-6-thinking",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-sonnet-4-5",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-sonnet-4-5-thinking",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-opus-4-6",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-opus-4-6-thinking",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-opus-4-5",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-opus-4-5-thinking",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-opus-4",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-3-7-sonnet",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-3-5-sonnet",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-haiku-4-5",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-haiku-4",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-3-5-haiku",
            context_window: 256_000,
        },
        CoreModel {
            id: "claude-3-haiku",
            context_window: 256_000,
        },
        // Gemini 3.x 系列 (1M 上下文)
        CoreModel {
            id: "gemini-3.8-flash",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.8-flash-tiered",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.8-flash-high",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.8-flash-medium",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.8-flash-low",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.7-flash",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.7-flash-tiered",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.7-flash-high",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.7-flash-medium",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.7-flash-low",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.6-flash",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.6-flash-tiered",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.6-flash-high",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.6-flash-medium",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.6-flash-low",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.5-flash",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.5-flash-high",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.5-flash-medium",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.5-flash-low",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3-flash",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3-flash-agent",
            context_window: 1_000_000,
        },
        CoreModel {
            id: "gemini-3.1-pro",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-3.1-pro-high",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-3.1-pro-low",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-3.1-pro-preview",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-3-pro",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-3-pro-high",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-3-pro-low",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-3-pro-preview",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-pro",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-pro-agent",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-3.1-flash-lite",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-3.1-flash-image",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-3-pro-image",
            context_window: 1_048_576,
        },
        // Gemini 2.x 系列 (1M 上下文)
        CoreModel {
            id: "gemini-2.5-pro",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-2.5-flash",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-2.5-flash-lite",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-2.5-flash-thinking",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-2.0-flash",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-2.0-flash-lite",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-2.0-flash-exp",
            context_window: 1_048_576,
        },
        CoreModel {
            id: "gemini-2.0-flash-thinking-exp",
            context_window: 1_048_576,
        },
        // 其他高频模型
        CoreModel {
            id: "grok-4.6",
            context_window: 500_000,
        },
        CoreModel {
            id: "gpt-oss-120b-medium",
            context_window: 128_000,
        },
    ]
}

/// 核心逻辑：为 JeikCode 生成/更新 config.toml 内容
pub fn sync_jeikcode_toml_content(
    content: &str,
    proxy_url: &str,
    api_key: &str,
    model: Option<&str>,
) -> Result<String, String> {
    use toml_edit::{value, Array, DocumentMut, Item, Table};
    let mut doc = content
        .parse::<DocumentMut>()
        .unwrap_or_else(|_| DocumentMut::new());

    let target_provider_id = "antigravity-manager";
    let normalized_proxy_url = {
        let trimmed = proxy_url.trim().trim_end_matches('/');
        if trimmed.ends_with("/v1") {
            trimmed.to_string()
        } else {
            format!("{}/v1", trimmed)
        }
    };
    let raw_base_url = proxy_url
        .trim()
        .trim_end_matches('/')
        .trim_end_matches("/v1")
        .trim_end_matches('/');

    // 1. 查找并清理已经存在的相同提供商 URL 的账号 (以 anthropic 协议最兼容接入)
    let mut accounts_to_remove = Vec::new();
    if let Some(accounts_table) = doc.get("provider_accounts").and_then(|i| i.as_table()) {
        for (acc_name, acc_item) in accounts_table.iter() {
            if acc_name == target_provider_id {
                continue;
            }
            if let Some(b_url) = acc_item.get("base_url").and_then(|v| v.as_str()) {
                let norm = b_url
                    .trim_end_matches('/')
                    .trim_end_matches("/v1")
                    .trim_end_matches('/');
                if norm == raw_base_url {
                    accounts_to_remove.push(acc_name.to_string());
                }
            }
        }
    }

    // 从 provider_accounts 移除旧同 URL 账号
    if let Some(accounts_table) = doc
        .get_mut("provider_accounts")
        .and_then(|i| i.as_table_mut())
    {
        for acc in &accounts_to_remove {
            accounts_table.remove(acc);
        }
    }

    // 检查旧版 providers 表
    if let Some(providers_table) = doc.get_mut("providers").and_then(|i| i.as_table_mut()) {
        let mut old_p_remove = Vec::new();
        for (p_name, p_item) in providers_table.iter() {
            if let Some(b_url) = p_item.get("base_url").and_then(|v| v.as_str()) {
                let norm = b_url
                    .trim_end_matches('/')
                    .trim_end_matches("/v1")
                    .trim_end_matches('/');
                if norm == raw_base_url {
                    old_p_remove.push(p_name.to_string());
                }
            }
        }
        for p in old_p_remove {
            providers_table.remove(&p);
        }
    }

    // 2. 清理属于被删除账号的模型，以及之前属于 antigravity-manager 的旧模型（以便完全重新生成）
    if let Some(models_table) = doc.get_mut("models").and_then(|i| i.as_table_mut()) {
        let mut models_to_remove = Vec::new();
        for (m_key, m_item) in models_table.iter() {
            if let Some(acc) = m_item.get("account").and_then(|v| v.as_str()) {
                if accounts_to_remove.iter().any(|a| a == acc) || acc == target_provider_id {
                    models_to_remove.push(m_key.to_string());
                }
            }
        }
        for m in models_to_remove {
            models_table.remove(&m);
        }
    }

    // 3. 添加 / 更新 [provider_accounts.antigravity-manager]，以 anthropic 协议接入
    let accounts = doc
        .entry("provider_accounts")
        .or_insert(Item::Table(Table::new()));
    if let Some(acc_table) = accounts.as_table_mut() {
        let ag = acc_table
            .entry(target_provider_id)
            .or_insert(Item::Table(Table::new()));
        if let Some(ag_table) = ag.as_table_mut() {
            ag_table.insert("provider", value("anthropic"));
            ag_table.insert("base_url", value(&normalized_proxy_url));
            ag_table.insert("api_key", value(api_key));
        }
    }

    // 4. 添加最核心的所有 Claude 和 Gemini 模型
    let core_models = get_core_gateway_models();

    let models_entry = doc.entry("models").or_insert(Item::Table(Table::new()));
    if let Some(m_table) = models_entry.as_table_mut() {
        for m in core_models {
            let mut model_table = Table::new();
            model_table.insert("account", value(target_provider_id));
            model_table.insert("model", value(m.id));
            model_table.insert("context_window", value(m.context_window));
            model_table.insert("image_input", value(true));
            model_table.insert("reasoning_model", value(true));
            model_table.insert("reasoning_history", value("exclude"));
            model_table.insert("reasoning_effort", value("high"));

            let mut levels = Array::new();
            levels.push("low");
            levels.push("medium");
            levels.push("high");
            model_table.insert(
                "reasoning_levels",
                Item::Value(toml_edit::Value::Array(levels)),
            );

            m_table.insert(m.id, Item::Table(model_table));
        }
    }

    // 5. 设置默认模型
    let existing_default_model = doc
        .get("default_model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let chosen_model = match model {
        Some(m) if !m.is_empty() => m.to_string(),
        _ => {
            if let Some(existing) = existing_default_model {
                if core_models.iter().any(|cm| cm.id == existing) {
                    existing
                } else {
                    "gemini-3.8-flash-high".to_string()
                }
            } else {
                "gemini-3.8-flash-high".to_string()
            }
        }
    };
    doc.insert("default_model", value(&chosen_model));
    doc.insert("default_provider", value(&chosen_model));

    Ok(doc.to_string())
}
