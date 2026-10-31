// Claude <-> Gemini static model map (split from model_mapping.rs)
use once_cell::sync::Lazy;
use std::collections::HashMap;

static CLAUDE_TO_GEMINI: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // ── Claude series core standard mapping ──
    m.insert("claude-sonnet-4-6", "claude-sonnet-4-6");
    m.insert("claude-sonnet-4-6-thinking", "claude-sonnet-4-6-thinking");
    m.insert("claude-opus-4-6", "claude-opus-4-6-thinking");
    m.insert("claude-opus-4-6-thinking", "claude-opus-4-6-thinking");
    m.insert("claude-sonnet-4-5", "claude-sonnet-4-6");
    m.insert("claude-sonnet-4-5-thinking", "claude-sonnet-4-6-thinking");
    m.insert("claude-opus-4-5-thinking", "claude-opus-4-6-thinking");
    m.insert("claude-haiku-4-5", "claude-sonnet-4-6");
    m.insert("claude-haiku-4", "claude-sonnet-4-6");

    // ──OpenAI Core Standard Mapping──
    m.insert("gpt-4o", "gemini-2.5-flash");
    m.insert("gpt-4o-mini", "gemini-2.5-flash");
    m.insert("gpt-4-turbo", "gemini-2.5-flash");
    m.insert("gpt-4", "gemini-2.5-flash");
    m.insert("gpt-3.5-turbo", "gemini-2.5-flash");

    // ── Gemini core standard mapping ──
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
/// # Mapping strategy
/// 1. **Exact Match**: Check CLAUDE_TO_GEMINI mapping table
/// 2. **Known prefix transparent transmission**: gemini-* and *-thinking models are directly transparently transmitted
/// 3. **[NEW] Direct transparent transmission**: Unknown model ID is directly passed to Google API (supports experiencing unreleased models)
///
/// # Parameters
/// - `input`: original model name
///
/// # Return
/// The mapped target model name
///
/// # Example
/// ```ignore
/// use antigravity_tools_lib::proxy::common::model_mapping::map_claude_model_to_gemini;
/// //Exact match
/// assert_eq!(map_claude_model_to_gemini("claude-opus-4"), "claude-opus-4-5-thinking");
///
/// // Gemini model transparent transmission
/// assert_eq!(map_claude_model_to_gemini("gemini-2.5-flash"), "gemini-2.5-flash");
///
/// // Directly transparently transmit the unknown model (NEW!)
/// assert_eq!(map_claude_model_to_gemini("claude-opus-4-6"), "claude-opus-4-6");
/// assert_eq!(map_claude_model_to_gemini("claude-sonnet-5"), "claude-sonnet-5");
/// ```
pub fn map_claude_model_to_gemini(input: &str) -> String {
    // 1. Exactly match the standard mapping table
    if let Some(mapped) = CLAUDE_TO_GEMINI.get(input) {
        return mapped.to_string();
    }

    // 2. Compatible with historical old version ID redirection (not leaked to external lists)
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

    // 4. Directly and transparently transmit the unknown model ID
    input.to_string()
}

/// Get a list of all built-in supported standard public models (expired experimental models, repeated Cartesian products and old snapshots have been cleaned)
pub fn get_supported_models() -> Vec<String> {
    vec![
        // Gemini 3.x main series
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
        // Gemini image generation main model (unified to retain standard names, completely eliminate noise in 21 aspect ratio/resolution combinations)
        "gemini-3.1-flash-image",
        "gemini-3-pro-image",
        // Gemini 2.5 Classic Series
        "gemini-2.5-pro",
        "gemini-2.5-flash",
        "gemini-2.5-flash-thinking",
        "gemini-2.5-flash-lite",
        // Claude series
        "claude-sonnet-4-6",
        "claude-sonnet-4-6-thinking",
        "claude-opus-4-6",
        "claude-opus-4-6-thinking",
        "claude-sonnet-4-5",
        "claude-sonnet-4-5-thinking",
        "claude-haiku-4-5",
        "claude-haiku-4",
        // OpenAI GPT core compatible series
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

/// Dynamically obtain a list of all available models (including built-in and user-defined and official endpoint dynamic distribution)
pub async fn get_all_dynamic_models(
    custom_mapping: &tokio::sync::RwLock<std::collections::HashMap<String, String>>,
    token_manager: Option<&crate::proxy::token_manager::TokenManager>,
    only_raw_quota_models: bool,
) -> Vec<String> {
    use std::collections::HashSet;
    let mut model_ids = HashSet::new();

    // 1. Get the dynamic models (Quota Models) of all accounts gathered from the official interface
    if let Some(tm) = token_manager {
        for dynamic_model in tm.get_all_collected_models() {
            model_ids.insert(dynamic_model);
        }
    }

    // If only_raw_quota_models is not turned on, add custom_mapping and built-in standard public models
    if !only_raw_quota_models {
        // 2. Get all custom mapping models (Custom)
        {
            let mapping = custom_mapping.read().await;
            for key in mapping.keys() {
                model_ids.insert(key.clone());
            }
        }

        // 3. Get all built-in standard models
        for m in get_supported_models() {
            model_ids.insert(m);
        }

        // 4. Make sure to include commonly used drawing and quota model IDs (remove duplicates)
        model_ids.insert("gemini-3.1-flash-image".to_string());
        model_ids.insert("gemini-3-pro-image".to_string());
        model_ids.insert("gemini-3.1-pro-high".to_string());
        model_ids.insert("gemini-3.1-pro-low".to_string());
    }

    // 5. Filter out internal virtual IDs and expired experimental models that have been offline
    model_ids.remove("internal-background-task");
    model_ids.remove("gemini-2.0-flash-exp");

    let mut sorted_ids: Vec<_> = model_ids.into_iter().collect();
    sorted_ids.sort();
    sorted_ids
}
