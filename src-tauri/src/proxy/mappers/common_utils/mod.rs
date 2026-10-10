//! Shared request utilities (split from common_utils.rs).
//! Facade: implementation lives in submodules, each <= 500 lines.

// Common utilities for request mapping across all protocols
// Provides unified grounding/networking logic

use serde_json::{json, Value};

/// Request configuration after grounding resolution
#[derive(Debug, Clone)]

pub struct RequestConfig {
    /// The request type: "agent", "web_search", or "image_gen"
    pub request_type: String,
    /// Whether to inject the googleSearch tool
    pub inject_google_search: bool,
    /// The final model name (with suffixes stripped)
    pub final_model: String,
    /// Image generation configuration (if request_type is image_gen)
    pub image_config: Option<Value>,
}

pub fn resolve_request_config(
    original_model: &str,
    mapped_model: &str,
    tools: &Option<Vec<Value>>,
    size: Option<&str>,       // [NEW] Image size parameter
    quality: Option<&str>,    // [NEW] Image quality parameter
    image_size: Option<&str>, // [NEW] Direct imageSize parameter (e.g. "4K")
    body: Option<&Value>,     // [NEW] Request body for Gemini native imageConfig
) -> RequestConfig {
    // 1. Image Generation Check (Priority)
    // Detect via the original requested alias OR the account-resolved model name, because the
    // dynamic model rewrite may turn "gemini-3-pro-image" into e.g. "gemini-3.1-flash-image".
    if original_model.to_lowercase().contains("-image") || mapped_model.contains("-image") {
        // [RESOLVE #1694] Improved priority logic:
        // 1. First parse inferred config from model suffix and OpenAI parameters
        let (mut inferred_config, parsed_base_model) =
            parse_image_config_with_params(original_model, size, quality, image_size);

        // 2. Then merge with imageConfig from Gemini request body (if exists)
        if let Some(body_val) = body {
            if let Some(gen_config) = body_val.get("generationConfig") {
                if let Some(body_image_config) = gen_config.get("imageConfig") {
                    tracing::info!(
                        "[Common-Utils] Found imageConfig in body, merging with inferred config from suffix/params"
                    );

                    if let Some(inferred_obj) = inferred_config.as_object_mut() {
                        if let Some(body_obj) = body_image_config.as_object() {
                            // Merge body_obj into inferred_obj
                            for (key, value) in body_obj {
                                // CRITICAL: Only allow body to override if inferred doesn't already have a high-priority value
                                // Specifically, if we inferred imageSize from -4k, don't let body downgrade it if it's missing or standard.
                                let is_size_downgrade = key == "imageSize"
                                    && (value.as_str() == Some("1K") || value.is_null())
                                    && inferred_obj.contains_key("imageSize");

                                if !is_size_downgrade {
                                    inferred_obj.insert(key.clone(), value.clone());
                                } else {
                                    tracing::debug!("[Common-Utils] Shielding inferred imageSize from body downgrade");
                                }
                            }
                        }
                    }
                }
            }
        }

        tracing::info!(
            "[Common-Utils] Final Image Config for {}: {:?}",
            parsed_base_model,
            inferred_config
        );

        // Prefer the account-resolved concrete image model (mapped_model) for the upstream
        // call; fall back to the parsed base of the requested alias if it wasn't resolved.
        let upstream_model = if mapped_model.contains("-image") {
            mapped_model.to_string()
        } else {
            parsed_base_model
        };
        return RequestConfig {
            request_type: "image_gen".to_string(),
            inject_google_search: false,
            final_model: upstream_model,
            image_config: Some(inferred_config),
        };
    }

    // 检测是否有联网工具定义 (内置功能调用)
    let has_networking_tool = detects_networking_tool(tools);
    // 检测是否包含非联网工具 (如 MCP 本地工具)
    let _has_non_networking = contains_non_networking_tool(tools);

    // Strip -online suffix from original model if present (to detect networking intent)
    let is_online_suffix = original_model.ends_with("-online");

    // High-quality grounding allowlist (Only for models known to support search and be relatively 'safe')
    let _is_high_quality_model = mapped_model == "gemini-2.5-flash"
        || mapped_model == "gemini-1.5-pro"
        || mapped_model.starts_with("gemini-1.5-pro-")
        || mapped_model.starts_with("gemini-2.5-flash-")
        || mapped_model.starts_with("gemini-2.0-flash")
        || mapped_model.starts_with("gemini-3-")
        || mapped_model.starts_with("gemini-3.")
        || mapped_model.starts_with("gemini-3.5-")
        || mapped_model.starts_with("gemini-pro-")
        || mapped_model.starts_with("gemini-3-flash")
        || mapped_model.starts_with("gemini-3.5-flash")
        || mapped_model.starts_with("agent")
        || mapped_model.contains("claude-3-5-sonnet")
        || mapped_model.contains("claude-3-opus")
        || mapped_model.contains("claude-sonnet")
        || mapped_model.contains("claude-opus")
        || mapped_model.contains("claude-4")
        || crate::proxy::model_specs::is_gemini_v3_or_above(mapped_model);

    // Determine if we should enable networking
    // [FIX] 禁用基于模型的自动联网逻辑，防止图像请求被联网搜索结果覆盖。
    // 仅在用户显式请求联网时启用：1) -online 后缀 2) 携带联网工具定义
    let enable_networking = is_online_suffix || has_networking_tool;

    // The final model to send upstream should be the MAPPED model,
    // but if searching, we MUST ensure the model name is one the backend associates with search.
    // Force a stable search model for search requests.
    let mut final_model = mapped_model.trim_end_matches("-online").to_string();

    // Map explicit preview aliases that have stable physical counterparts.
    // Note: gemini-3-pro-preview / gemini-3.1-pro-preview are intentionally NOT forced
    // to *-high here; dynamic runtime rewrite is handled after account selection.
    final_model = match final_model.as_str() {
        "gemini-3-pro-image-preview" => "gemini-3-pro-image".to_string(),
        "gemini-3-flash-preview" => "gemini-3-flash".to_string(),
        _ => final_model,
    };

    // [FIX] 不再强行将模型降级为 gemini-2.5-flash，彻底杜绝静默降级
    if enable_networking && !_is_high_quality_model {
        tracing::debug!(
            "[Common-Utils] Request enables web search for model {}",
            final_model
        );
    }

    RequestConfig {
        request_type: if enable_networking {
            "web_search".to_string()
        } else {
            "agent".to_string()
        },
        inject_google_search: enable_networking,
        final_model,
        image_config: None,
    }
}

/// Legacy wrapper for backward compatibility and simple usage
#[allow(dead_code)]
pub mod image_config;
pub mod multimodal;
pub mod request;
pub mod tools;

#[cfg(test)]
mod tests_config;
#[cfg(test)]
mod tests_defense;
#[cfg(test)]
mod tests_grounding;

pub use image_config::{
    clean_image_model_name, image_aspect_ratio_from_size, parse_image_config,
    parse_image_config_with_params, try_parse_image_config_with_params,
};
pub use multimodal::{
    create_gemini_inline_part, detect_mime_from_bytes, enhance_gemini_skills_prompt,
    extract_multimodal_from_tool_text, is_supported_tool_image_mime,
    parse_markdown_images_to_parts, sanitize_gemini_payload_inline_data,
    validate_and_sanitize_inline_data, SUPPORTED_TOOL_IMAGE_MIMES,
};
pub use request::{
    build_official_request_id, ensure_gemini_payload_ends_with_user, is_model_compatible,
    resolve_official_fingerprint, safe_truncate_chars, safe_truncate_str, wrap_in_system_reminder,
    TRANSIT_DEFENSE_FALLBACK_TEXT,
};
pub use tools::{
    contains_non_networking_tool, contents_has_tool_interactions, deep_clean_undefined,
    detects_networking_tool, has_any_tools, inject_google_search_tool,
};
