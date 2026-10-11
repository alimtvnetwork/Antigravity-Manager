// Image model config parsing (split from common_utils.rs).
// Common utilities for request mapping across all protocols
// Provides unified grounding/networking logic

use serde_json::{json, Value};

/// Request configuration after grounding resolution
pub fn parse_image_config(model_name: &str) -> (Value, String) {
    parse_image_config_with_params(model_name, None, None, None)
}

/// Parse image configuration while rejecting an explicit, unsupported `imageSize` value.
/// API handlers should use this variant so invalid client input becomes a boundary error.
pub fn try_parse_image_config_with_params(
    model_name: &str,
    size: Option<&str>,
    quality: Option<&str>,
    image_size: Option<&str>,
) -> Result<(Value, String), String> {
    let image_size = normalize_image_size(image_size)?;
    Ok(parse_image_config_with_normalized_params(
        model_name, size, quality, image_size,
    ))
}

/// Extended version that accepts OpenAI size and quality parameters
///
/// This function supports parsing image configuration from:
/// 1. Direct imageSize parameter - takes highest priority
/// 2. OpenAI API parameters (size, quality) - medium priority
/// 3. Model name suffixes (e.g., -16x9, -4k) - fallback
///
/// # Arguments
/// * `model_name` - The model name (may contain suffixes like -16x9-4k)
/// * `size` - Optional OpenAI size parameter (e.g., "1280x720", "1792x1024")
/// * `quality` - Optional OpenAI quality parameter ("standard", "hd", "medium")
/// * `image_size` - Optional direct Gemini imageSize parameter ("2K", "4K")
///
/// # Returns
/// (image_config, clean_model_name) where image_config contains aspectRatio and optionally imageSize
pub fn parse_image_config_with_params(
    model_name: &str,
    size: Option<&str>,
    quality: Option<&str>,
    image_size: Option<&str>,
) -> (Value, String) {
    // Legacy internal callers cannot return an HTTP boundary error. Invalid explicit values are
    // ignored here; public API handlers use `try_parse_image_config_with_params` instead.
    let image_size = normalize_image_size(image_size).ok().flatten();
    parse_image_config_with_normalized_params(model_name, size, quality, image_size)
}

fn parse_image_config_with_normalized_params(
    model_name: &str,
    size: Option<&str>,
    quality: Option<&str>,
    image_size: Option<&'static str>,
) -> (Value, String) {
    let mut aspect_ratio = "1:1";

    // 1. 优先从 size 参数解析宽高比
    if let Some(parsed_ratio) = size.and_then(image_aspect_ratio_from_size) {
        aspect_ratio = parsed_ratio;
    } else {
        // 2. 回退到模型后缀解析（保持向后兼容）
        if model_name.contains("-21x9") || model_name.contains("-21-9") {
            aspect_ratio = "21:9";
        } else if model_name.contains("-16x9") || model_name.contains("-16-9") {
            aspect_ratio = "16:9";
        } else if model_name.contains("-9x16") || model_name.contains("-9-16") {
            aspect_ratio = "9:16";
        } else if model_name.contains("-4x3") || model_name.contains("-4-3") {
            aspect_ratio = "4:3";
        } else if model_name.contains("-3x4") || model_name.contains("-3-4") {
            aspect_ratio = "3:4";
        } else if model_name.contains("-3x2") || model_name.contains("-3-2") {
            aspect_ratio = "3:2";
        } else if model_name.contains("-2x3") || model_name.contains("-2-3") {
            aspect_ratio = "2:3";
        } else if model_name.contains("-5x4") || model_name.contains("-5-4") {
            aspect_ratio = "5:4";
        } else if model_name.contains("-4x5") || model_name.contains("-4-5") {
            aspect_ratio = "4:5";
        } else if model_name.contains("-1x1") || model_name.contains("-1-1") {
            aspect_ratio = "1:1";
        }
    }

    let mut config = serde_json::Map::new();
    config.insert("aspectRatio".to_string(), json!(aspect_ratio));

    // [NEW] 0. 最高优先级：直接使用 image_size 参数
    if let Some(image_size) = image_size {
        config.insert("imageSize".to_string(), json!(image_size));
    } else {
        // 3. 优先从 quality 参数解析分辨率
        if let Some(image_size) = quality.and_then(image_size_from_quality) {
            config.insert("imageSize".to_string(), json!(image_size));
        } else {
            // 4. 回退到模型后缀解析（保持向后兼容）
            let is_hd = model_name.contains("-4k") || model_name.contains("-hd");
            let is_2k = model_name.contains("-2k");
            let is_1k = model_name.contains("-1k") || model_name.contains("-standard");

            if is_hd {
                config.insert("imageSize".to_string(), json!("4K"));
            } else if is_2k {
                config.insert("imageSize".to_string(), json!("2K"));
            } else if is_1k {
                config.insert("imageSize".to_string(), json!("1K"));
            }
        }
    }

    let clean_model_name = clean_image_model_name(model_name);

    (serde_json::Value::Object(config), clean_model_name)
}

fn normalize_image_size(image_size: Option<&str>) -> Result<Option<&'static str>, String> {
    let Some(image_size) = image_size.map(str::trim) else {
        return Ok(None);
    };

    if image_size.is_empty() || image_size.eq_ignore_ascii_case("auto") {
        return Ok(None);
    }

    match image_size.to_ascii_lowercase().as_str() {
        "1k" => Ok(Some("1K")),
        "2k" => Ok(Some("2K")),
        "4k" => Ok(Some("4K")),
        _ => Err("Invalid image_size: expected one of 1K, 2K, 4K, or auto".to_string()),
    }
}

fn image_size_from_quality(quality: &str) -> Option<&'static str> {
    match quality.trim().to_ascii_lowercase().as_str() {
        "low" | "standard" | "1k" => Some("1K"),
        "medium" | "2k" => Some("2K"),
        "high" | "hd" | "4k" => Some("4K"),
        "auto" | "" => None,
        _ => None,
    }
}

/// Helper function to clean image model names by removing resolution/aspect-ratio suffixes.
/// E.g., "gemini-3.1-flash-image-16x9-4k" -> "gemini-3.1-flash-image"
pub fn clean_image_model_name(model_name: &str) -> String {
    let mut clean_name = model_name.to_lowercase();

    // Ordered list of known suffixes to strip
    let suffixes = [
        "-4k",
        "-2k",
        "-1k",
        "-hd",
        "-standard",
        "-medium",
        "-21x9",
        "-21-9",
        "-16x9",
        "-16-9",
        "-9x16",
        "-9-16",
        "-4x3",
        "-4-3",
        "-3x4",
        "-3-4",
        "-3x2",
        "-3-2",
        "-2x3",
        "-2-3",
        "-5x4",
        "-5-4",
        "-4x5",
        "-4-5",
        "-1x1",
        "-1-1",
    ];

    // Repeatedly strip suffixes until no more are found
    let mut changed = true;
    while changed {
        changed = false;
        for suffix in &suffixes {
            if clean_name.ends_with(suffix) {
                clean_name.truncate(clean_name.len() - suffix.len());
                changed = true;
            }
        }
    }

    clean_name
}

/// 动态计算宽高比（解决硬编码问题）
///
/// 从 "WIDTHxHEIGHT" 格式的字符串解析并计算宽高比，
/// 使用容差匹配常见的标准比例。
///
/// # Arguments
/// * `size` - 尺寸字符串，格式为 "WIDTHxHEIGHT" (e.g., "1280x720", "1792x1024")
///
/// # Returns
/// 标准宽高比字符串 ("1:1", "16:9", "9:16", "4:3", "3:4", "21:9")
pub fn image_aspect_ratio_from_size(size: &str) -> Option<&'static str> {
    let size = size.trim();
    if size.is_empty() || size.eq_ignore_ascii_case("auto") {
        return None;
    }

    // 0. Explicitly check known aspect ratios first
    match size {
        "21:9" => return Some("21:9"),
        "16:9" => return Some("16:9"),
        "9:16" => return Some("9:16"),
        "4:3" => return Some("4:3"),
        "3:4" => return Some("3:4"),
        "3:2" => return Some("3:2"),
        "2:3" => return Some("2:3"),
        "5:4" => return Some("5:4"),
        "4:5" => return Some("4:5"),
        "1:1" => return Some("1:1"),
        _ => {}
    }

    if let Some((w_str, h_str)) = size.split_once('x') {
        if let (Ok(width), Ok(height)) = (w_str.parse::<f64>(), h_str.parse::<f64>()) {
            if width > 0.0 && height > 0.0 {
                let ratio = width / height;

                // 容差匹配常见比例（容差 0.05，避免 3:4 和 2:3 重叠）
                if (ratio - 21.0 / 9.0).abs() < 0.05 {
                    return Some("21:9");
                }
                if (ratio - 16.0 / 9.0).abs() < 0.05 {
                    return Some("16:9");
                }
                if (ratio - 4.0 / 3.0).abs() < 0.05 {
                    return Some("4:3");
                }
                if (ratio - 3.0 / 4.0).abs() < 0.05 {
                    return Some("3:4");
                }
                if (ratio - 9.0 / 16.0).abs() < 0.05 {
                    return Some("9:16");
                }
                if (ratio - 3.0 / 2.0).abs() < 0.05 {
                    return Some("3:2");
                }
                if (ratio - 2.0 / 3.0).abs() < 0.05 {
                    return Some("2:3");
                }
                if (ratio - 5.0 / 4.0).abs() < 0.05 {
                    return Some("5:4");
                }
                if (ratio - 4.0 / 5.0).abs() < 0.05 {
                    return Some("4:5");
                }
                if (ratio - 1.0).abs() < 0.05 {
                    return Some("1:1");
                }
            }
        }
    }

    None
}

pub fn calculate_aspect_ratio_from_size(size: &str) -> &'static str {
    image_aspect_ratio_from_size(size).unwrap_or("1:1")
}
