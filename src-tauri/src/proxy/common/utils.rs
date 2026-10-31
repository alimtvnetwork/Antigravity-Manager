// Utility function

pub fn generate_random_id() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}

/// Infer function type based on model name
// NOTE: This function is deprecated, use mappers::common_utils::resolve_request_config instead
pub fn _deprecated_infer_quota_group(model: &str) -> String {
    if model.to_lowercase().starts_with("claude") {
        "claude".to_string()
    } else {
        "gemini".to_string()
    }
}

/// Tool call IDs are unified and normalized.
///
/// The standard format is unified to start with `call_` (such as `call_573077`).
/// Some clients (such as Antigravity IDE, VS Code plug-in, or some intermediary proxies) lose underscores on postback
/// Produces a form similar to `call573077`, resulting in an inability to accurately match the gateway signature cache and thinking storage.
///
/// This function normalizes `call<digits>` or call IDs missing underscores into a unified `call_<digits>` format.
pub fn normalize_tool_id(id: &str) -> std::borrow::Cow<'_, str> {
    let trimmed = id.trim();
    if trimmed.starts_with("call") && !trimmed.starts_with("call_") && trimmed.len() > 4 {
        let rest = &trimmed[4..];
        if rest.chars().next().map_or(false, |c| c.is_ascii_digit())
            || (rest.len() >= 6 && rest.chars().all(|c| c.is_ascii_hexdigit()))
        {
            return std::borrow::Cow::Owned(format!("call_{}", rest));
        }
    }
    std::borrow::Cow::Borrowed(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_tool_id() {
        // Client numbers/hash IDs that are missing underscores are normalized to call_
        assert_eq!(normalize_tool_id("call573077").as_ref(), "call_573077");
        assert_eq!(normalize_tool_id("call1599720").as_ref(), "call_1599720");
        assert_eq!(normalize_tool_id("call932369").as_ref(), "call_932369");
        assert_eq!(normalize_tool_id("call389691").as_ref(), "call_389691");
        assert_eq!(normalize_tool_id("call0123456").as_ref(), "call_0123456");
        assert_eq!(normalize_tool_id("calld4e5f6a1").as_ref(), "call_d4e5f6a1");

        // Already in compliance with specifications, remain as is
        assert_eq!(normalize_tool_id("call_573077").as_ref(), "call_573077");
        assert_eq!(
            normalize_tool_id("call_read_root_ada28549fe0e_0").as_ref(),
            "call_read_root_ada28549fe0e_0"
        );
        assert_eq!(normalize_tool_id("toolu_01Abc").as_ref(), "toolu_01Abc");

        // Common words or short IDs will not be misjudged
        assert_eq!(normalize_tool_id("call").as_ref(), "call");
        assert_eq!(normalize_tool_id("calling").as_ref(), "calling");
        assert_eq!(normalize_tool_id("callback").as_ref(), "callback");
        assert_eq!(normalize_tool_id("").as_ref(), "");
    }
}
