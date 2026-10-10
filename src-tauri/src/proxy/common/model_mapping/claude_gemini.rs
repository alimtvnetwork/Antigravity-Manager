// Claude <-> Gemini static model map (split from model_mapping.rs)
use once_cell::sync::Lazy;
use std::collections::HashMap;

static CLAUDE_TO_GEMINI: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // ── Claude 系列核心标准映射 ──
    m.insert("claude-sonnet-4-6", "claude-sonnet-4-6");
    m.insert("claude-sonnet-4-6-thinking", "claude-sonnet-4-6-thinking");
    m.insert("claude-opus-4-6", "claude-opus-4-6-thinking");
    m.insert("claude-opus-4-6-thinking", "claude-opus-4-6-thinking");
    m.insert("claude-sonnet-4-5", "claude-sonnet-4-6");
    m.insert("claude-sonnet-4-5-thinking", "claude-sonnet-4-6-thinking");
    m.insert("claude-opus-4-5-thinking", "claude-opus-4-6-thinking");
    m.insert("claude-haiku-4-5", "claude-sonnet-4-6");
    m.insert("claude-haiku-4", "claude-sonnet-4-6");

    // ── OpenAI 核心标准映射 ──
    m.insert("gpt-4o", "gemini-2.5-flash");
    m.insert("gpt-4o-mini", "gemini-2.5-flash");
    m.insert("gpt-4-turbo", "gemini-2.5-flash");
    m.insert("gpt-4", "gemini-2.5-flash");
    m.insert("gpt-3.5-turbo", "gemini-2.5-flash");

    // ── Gemini 核心标准映射 ──
    m.insert("gemini-3.8-flash", "gemini-3.8-flash-tiered");
    m.insert("gemini-3.7-flash", "gemini-3.7-flash-tiered");
    m.insert("gemini-3.7-flash-tiered", "gemini-3.7-flash-tiered");
    m.insert("gemini-3.7-flash-high", "gemini-3.7-flash-high");
    m.insert("gemini-3.7-flash-medium", "gemini-3.7-flash-medium");
    m.insert("gemini-3.7-flash-low", "gemini-3.7-flash-low");
    m.insert("gemini-3.5-flash", "gemini-3.5-flash");
    m.insert("gemini-3-flash", "gemini-3-flash");
    m.insert("gemini-3.1-pro-high", "gemini-pro-agent");
    m.insert("gemini-3.1-pro-low", "gemini-3.1-pro-low");
    m.insert("gemini-3.1-pro-preview", "gemini-3.1-pro-preview");
    m.insert("gemini-3.1-pro", "gemini-3.1-pro-preview");
    m.insert("gemini-3.1-flash-lite", "gemini-3.1-flash-lite");
    m.insert("gemini-3.1-flash-image", "gemini-3.1-flash-image");
    m.insert("gemini-3-pro-image", "gemini-3-pro-image");
    m.insert("gemini-2.5-pro", "gemini-2.5-pro");
    m.insert("gemini-2.5-flash", "gemini-2.5-flash");
    m.insert("gemini-2.5-flash-thinking", "gemini-2.5-flash-thinking");
    m.insert("gemini-2.5-flash-lite", "gemini-2.5-flash");

    m
});

/// Map Claude model names to Gemini model names
///
/// # 映射策略
/// 1. **精确匹配**: 检查 CLAUDE_TO_GEMINI 映射表
/// 2. **已知前缀透传**: gemini-* 和 *-thinking 模型直接透传
/// 3. **[NEW] 直接透传**: 未知模型 ID 直接传递给 Google API (支持体验未发布模型)
///
/// # 参数
/// - `input`: 原始模型名称
///
/// # 返回
/// 映射后的目标模型名称
///
/// # 示例
/// ```ignore
/// use antigravity_tools_lib::proxy::common::model_mapping::map_claude_model_to_gemini;
/// // 精确匹配
/// assert_eq!(map_claude_model_to_gemini("claude-opus-4"), "claude-opus-4-5-thinking");
///
/// // Gemini 模型透传
/// assert_eq!(map_claude_model_to_gemini("gemini-2.5-flash"), "gemini-2.5-flash");
///
/// // 直接透传未知模型 (NEW!)
/// assert_eq!(map_claude_model_to_gemini("claude-opus-4-6"), "claude-opus-4-6");
/// assert_eq!(map_claude_model_to_gemini("claude-sonnet-5"), "claude-sonnet-5");
/// ```
pub fn map_claude_model_to_gemini(input: &str) -> String {
    // 1. 精确匹配标准映射表
    if let Some(mapped) = CLAUDE_TO_GEMINI.get(input) {
        return mapped.to_string();
    }

    // 2. 兼容历史老版本 ID 重定向 (不泄漏到外部列表)
    match input {
        "claude-3-5-sonnet-20241022" | "claude-3-5-sonnet-20240620" | "claude-3-haiku-20240307" => {
            return "claude-sonnet-4-6".to_string()
        }
        "claude-opus-4" | "claude-opus-4-5-20251101" | "claude-opus-4-6-20260201" => {
            return "claude-opus-4-6-thinking".to_string()
        }
        "gemini-3-pro-high" => return "gemini-pro-agent".to_string(),
        "gemini-3-pro-low" => return "gemini-3-pro-low".to_string(),
        "gemini-3-pro" | "gemini-3-pro-preview" => return "gemini-3.1-pro-preview".to_string(),
        "internal-background-task" => return "gemini-2.5-flash".to_string(),
        _ => {}
    }

    // 3. Known prefixes (gemini-, -thinking) pass-through
    if input.starts_with("gemini-") || input.contains("thinking") {
        return input.to_string();
    }

    // 4. 直接透传未知模型 ID
    input.to_string()
}

/// 获取所有内置支持的标准公开模型列表 (已清理过期实验模型、重复笛卡尔积及旧快照)
pub fn get_supported_models() -> Vec<String> {
    vec![
        // Gemini 3.x 主力系列
        "gemini-3.8-flash",
        "gemini-3.7-flash",
        "gemini-3.7-flash-high",
        "gemini-3.7-flash-medium",
        "gemini-3.7-flash-low",
        "gemini-3.7-flash-tiered",
        "gemini-3.5-flash",
        "gemini-3-flash",
        "gemini-3.1-pro",
        "gemini-3.1-pro-high",
        "gemini-3.1-pro-low",
        "gemini-3.1-pro-preview",
        "gemini-3.1-flash-lite",
        // Gemini 图像生成主力模型 (统一保留规范名称，彻底消除 21 种宽高比/分辨率组合噪点)
        "gemini-3.1-flash-image",
        "gemini-3-pro-image",
        // Gemini 2.5 经典系列
        "gemini-2.5-pro",
        "gemini-2.5-flash",
        "gemini-2.5-flash-thinking",
        "gemini-2.5-flash-lite",
        // Claude 系列
        "claude-sonnet-4-6",
        "claude-sonnet-4-6-thinking",
        "claude-opus-4-6",
        "claude-opus-4-6-thinking",
        "claude-sonnet-4-5",
        "claude-sonnet-4-5-thinking",
        "claude-haiku-4-5",
        "claude-haiku-4",
        // OpenAI GPT 核心兼容系列
        "gpt-4o",
        "gpt-4o-mini",
        "gpt-4-turbo",
        "gpt-4",
        "gpt-3.5-turbo",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

/// 动态获取所有可用模型列表 (包含内置与用户自定义与官方端点动态下发)
pub async fn get_all_dynamic_models(
    custom_mapping: &tokio::sync::RwLock<std::collections::HashMap<String, String>>,
    token_manager: Option<&crate::proxy::token_manager::TokenManager>,
    only_raw_quota_models: bool,
) -> Vec<String> {
    use std::collections::HashSet;
    let mut model_ids = HashSet::new();

    // 1. 获取所有账号从官方接口汇聚而来的动态模型 (Quota Models)
    if let Some(tm) = token_manager {
        for dynamic_model in tm.get_all_collected_models() {
            model_ids.insert(dynamic_model);
        }
    }

    // 如果未开启 only_raw_quota_models，则追加 custom_mapping 与内置标准公开模型
    if !only_raw_quota_models {
        // 2. 获取所有自定义映射模型 (Custom)
        {
            let mapping = custom_mapping.read().await;
            for key in mapping.keys() {
                model_ids.insert(key.clone());
            }
        }

        // 3. 获取所有内置标准模型
        for m in get_supported_models() {
            model_ids.insert(m);
        }

        // 4. 确保包含常用的画画与配额模型 ID (去重)
        model_ids.insert("gemini-3.1-flash-image".to_string());
        model_ids.insert("gemini-3-pro-image".to_string());
        model_ids.insert("gemini-3.1-pro-high".to_string());
        model_ids.insert("gemini-3.1-pro-low".to_string());
    }

    // 5. 过滤掉内部虚拟 ID 和已下线的过期实验模型
    model_ids.remove("internal-background-task");
    model_ids.remove("gemini-2.0-flash-exp");

    let mut sorted_ids: Vec<_> = model_ids.into_iter().collect();
    sorted_ids.sort();
    sorted_ids
}
