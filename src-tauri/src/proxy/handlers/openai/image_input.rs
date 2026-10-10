// Image input normalization helpers for OpenAI-compatible image endpoints.
use base64::Engine as _;
use serde_json::{json, Value};

const MAX_RETRY_ATTEMPTS: usize = 3;
pub(crate) const MAX_INPUT_IMAGES: usize = 16;
pub(crate) const MAX_INPUT_IMAGE_BYTES: usize = 20 * 1024 * 1024;
pub(crate) const MAX_TOTAL_INPUT_IMAGE_BYTES: usize = 32 * 1024 * 1024;
const CODEX_VISIBLE_THOUGHT_MESSAGE_PREFIX: &str = "msg_thought_";
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NormalizedInputImage {
    mime_type: String,
    base64_data: String,
    decoded_len: usize,
}

pub(crate) fn validate_input_image_limits(
    image_count: usize,
    image_bytes: usize,
    total_bytes: usize,
) -> Result<(), String> {
    if image_count > MAX_INPUT_IMAGES {
        return Err(format!(
            "Too many input images: maximum is {}",
            MAX_INPUT_IMAGES
        ));
    }
    if image_bytes > MAX_INPUT_IMAGE_BYTES {
        return Err(format!(
            "Input image is too large: maximum decoded size is {} bytes",
            MAX_INPUT_IMAGE_BYTES
        ));
    }
    if total_bytes > MAX_TOTAL_INPUT_IMAGE_BYTES {
        return Err(format!(
            "Total input image data is too large: maximum decoded size is {} bytes",
            MAX_TOTAL_INPUT_IMAGE_BYTES
        ));
    }
    Ok(())
}

pub(crate) fn normalized_image_from_bytes(
    bytes: &[u8],
    mime_type: &str,
    image_count: usize,
    total_bytes: usize,
) -> Result<NormalizedInputImage, String> {
    let next_total = total_bytes.saturating_add(bytes.len());
    validate_input_image_limits(image_count, bytes.len(), next_total)?;
    Ok(NormalizedInputImage {
        mime_type: mime_type.to_string(),
        base64_data: base64::engine::general_purpose::STANDARD.encode(bytes),
        decoded_len: bytes.len(),
    })
}

fn parse_image_data_url(
    data_url: &str,
    image_count: usize,
    total_bytes: usize,
) -> Result<NormalizedInputImage, String> {
    let (mime_type, encoded) = parse_image_data_url_parts(data_url)?;

    let max_encoded_len = MAX_INPUT_IMAGE_BYTES.div_ceil(3).saturating_mul(4);
    if encoded.len() > max_encoded_len {
        return Err(format!(
            "Input image is too large: maximum decoded size is {} bytes",
            MAX_INPUT_IMAGE_BYTES
        ));
    }
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| "Input image contains invalid base64 data".to_string())?;
    normalized_image_from_bytes(&decoded, mime_type, image_count, total_bytes)
}

pub(crate) fn parse_image_data_url_parts(data_url: &str) -> Result<(&str, &str), String> {
    let (metadata, encoded) = data_url
        .strip_prefix("data:")
        .and_then(|value| value.split_once(','))
        .ok_or_else(|| "Input image must be a base64 data:image URL".to_string())?;
    let mut metadata_parts = metadata.split(';');
    let mime_type = metadata_parts.next().unwrap_or_default();
    if !mime_type.starts_with("image/") || mime_type.len() <= "image/".len() {
        return Err("Input image data URL must use an image MIME type".to_string());
    }
    if !metadata_parts.any(|part| part.eq_ignore_ascii_case("base64")) {
        return Err("Input image data URL must be base64 encoded".to_string());
    }
    if encoded.is_empty() {
        return Err("Input image data URL is empty".to_string());
    }
    Ok((mime_type, encoded))
}

pub(crate) fn parse_generation_input_images(
    image: Option<&Value>,
) -> Result<Vec<NormalizedInputImage>, String> {
    let Some(image) = image else {
        return Ok(Vec::new());
    };

    let urls: Vec<&str> = match image {
        Value::String(url) => vec![url.as_str()],
        Value::Array(urls) if urls.is_empty() => {
            return Err("Input image array must not be empty".to_string())
        }
        Value::Array(urls) => urls
            .iter()
            .map(|url| {
                url.as_str()
                    .ok_or_else(|| "Every input image must be a base64 data:image URL".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err("Input image must be a string or an array of strings".to_string()),
    };

    validate_input_image_limits(urls.len(), 0, 0)?;
    let mut images = Vec::with_capacity(urls.len());
    let mut total_bytes = 0;
    for url in urls {
        let image = parse_image_data_url(url, images.len() + 1, total_bytes)?;
        total_bytes = total_bytes.saturating_add(image.decoded_len);
        images.push(image);
    }
    Ok(images)
}

pub(crate) fn generation_image_size_param(body: &Value) -> Result<Option<&str>, String> {
    for key in ["image_size", "imageSize"] {
        let Some(value) = body.get(key) else {
            continue;
        };
        if value.is_null() {
            continue;
        }
        return value
            .as_str()
            .map(Some)
            .ok_or_else(|| "Invalid image_size: expected one of 1K, 2K, 4K, or auto".to_string());
    }
    Ok(None)
}

pub(crate) fn is_edit_image_field(name: &str) -> bool {
    name == "image"
        || name == "image[]"
        || name.strip_prefix("image").is_some_and(|suffix| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
        })
}

pub(crate) fn edit_size_input<'a>(
    aspect_ratio: Option<&'a str>,
    size: Option<&'a str>,
) -> Option<&'a str> {
    aspect_ratio
        .filter(|value| {
            crate::proxy::mappers::common_utils::image_aspect_ratio_from_size(value).is_some()
        })
        .or_else(|| {
            size.filter(|value| {
                crate::proxy::mappers::common_utils::image_aspect_ratio_from_size(value).is_some()
            })
        })
}

pub(crate) fn image_account_selection_target(model_to_use: &str) -> &str {
    model_to_use
}

fn image_inline_part(image: &NormalizedInputImage) -> Value {
    json!({
        "inlineData": {
            "mimeType": image.mime_type,
            "data": image.base64_data
        }
    })
}

pub(crate) fn build_image_contents(
    prompt: String,
    input_images: &[NormalizedInputImage],
    mask: Option<&NormalizedInputImage>,
) -> Vec<Value> {
    let mut parts = Vec::with_capacity(1 + input_images.len() + usize::from(mask.is_some()));
    parts.push(json!({ "text": prompt }));
    for (index, image) in input_images.iter().enumerate() {
        parts.push(image_inline_part(image));
        if index == 0 {
            if let Some(mask) = mask {
                parts.push(image_inline_part(mask));
            }
        }
    }
    if input_images.is_empty() {
        if let Some(mask) = mask {
            parts.push(image_inline_part(mask));
        }
    }
    parts
}

pub(crate) fn build_image_edit_body(
    project_id: String,
    resolved_model: &str,
    contents_parts: Vec<Value>,
    image_config: Value,
) -> Value {
    json!({
        "project": project_id,
        "requestId": format!("img-edit-{}", uuid::Uuid::new_v4()),
        "model": resolved_model,
        "userAgent": "antigravity",
        "requestType": "image_gen",
        "request": {
            "contents": [{
                "role": "user",
                "parts": contents_parts
            }],
            "generationConfig": {
                "candidateCount": 1,
                "imageConfig": image_config,
                "maxOutputTokens": 8192,
                "stopSequences": [],
                "temperature": 1.0,
                "topP": 0.95,
                "topK": 40
            },
            "safetySettings": [
                { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "OFF" },
            ]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn generation_image_extension_adds_single_and_array_images_in_order() {
        let single = json!("data:image/png;base64,AQ==");
        let single_images = parse_generation_input_images(Some(&single)).expect("single data URL");
        let single_parts = build_image_contents("prompt".to_string(), &single_images, None);
        assert_eq!(single_parts.len(), 2);
        assert_eq!(single_parts[1]["inlineData"]["data"], "AQ==");

        let multiple = json!(["data:image/png;base64,AQ==", "data:image/jpeg;base64,Ag=="]);
        let multiple_images =
            parse_generation_input_images(Some(&multiple)).expect("ordered data URL array");
        let multiple_parts = build_image_contents("prompt".to_string(), &multiple_images, None);
        assert_eq!(multiple_parts.len(), 3);
        assert_eq!(multiple_parts[1]["inlineData"]["data"], "AQ==");
        assert_eq!(multiple_parts[2]["inlineData"]["data"], "Ag==");
    }

    fn generation_without_image_remains_text_only() {
        let images =
            parse_generation_input_images(None).expect("absent image is standard text-to-image");
        let parts = build_image_contents("prompt".to_string(), &images, None);
        assert_eq!(parts, vec![json!({"text": "prompt"})]);
    }

    fn generation_image_extension_rejects_invalid_or_remote_inputs() {
        let invalid_inputs = [
            json!("https://example.invalid/image.png"),
            json!("data:text/plain;base64,AQ=="),
            json!("data:image/png;base64,not-base64"),
            json!([]),
            json!(["data:image/png;base64,AQ==", 2]),
        ];
        for input in invalid_inputs {
            assert!(parse_generation_input_images(Some(&input)).is_err());
        }

        let too_many = Value::Array(
            (0..=MAX_INPUT_IMAGES)
                .map(|_| json!("data:image/png;base64,AQ=="))
                .collect(),
        );
        assert!(parse_generation_input_images(Some(&too_many)).is_err());
        assert!(validate_input_image_limits(1, MAX_INPUT_IMAGE_BYTES + 1, 0).is_err());
        assert!(validate_input_image_limits(1, 1, MAX_TOTAL_INPUT_IMAGE_BYTES + 1).is_err());

        assert!(generation_image_size_param(&json!({"imageSize": 4})).is_err());
    }

    fn edit_image_fields_preserve_all_supported_forms_in_arrival_order() {
        let field_names = [
            "image",
            "image",
            "image[]",
            "image[]",
            "image1",
            "image2",
            "imageSize",
            "image_size",
            "imageReference",
        ];
        let accepted: Vec<(usize, &str)> = field_names
            .iter()
            .enumerate()
            .filter(|(_, name)| is_edit_image_field(name))
            .map(|(index, name)| (index, *name))
            .collect();
        assert_eq!(
            accepted,
            vec![
                (0, "image"),
                (1, "image"),
                (2, "image[]"),
                (3, "image[]"),
                (4, "image1"),
                (5, "image2"),
            ]
        );
    }

    fn edit_aspect_ratio_priority_preserves_suffix_without_explicit_size() {
        use crate::proxy::mappers::common_utils::try_parse_image_config_with_params;

        let suffix_input = edit_size_input(None, None);
        let (suffix_config, _) = try_parse_image_config_with_params(
            "gemini-3.1-flash-image-16x9",
            suffix_input,
            None,
            None,
        )
        .expect("model suffix config");
        assert_eq!(suffix_config["aspectRatio"], "16:9");

        let explicit_input = edit_size_input(Some("4:3"), Some("1280x720"));
        let (explicit_config, _) = try_parse_image_config_with_params(
            "gemini-3.1-flash-image-16x9",
            explicit_input,
            None,
            None,
        )
        .expect("explicit aspect ratio config");
        assert_eq!(explicit_config["aspectRatio"], "4:3");
    }

    fn edit_flash_model_is_used_for_account_selection_and_resolved_upstream_body() {
        use crate::proxy::mappers::common_utils::try_parse_image_config_with_params;

        let (image_config, model_to_use) =
            try_parse_image_config_with_params("gemini-3.1-flash-image", None, None, None)
                .expect("flash edit model config");
        assert_eq!(
            image_account_selection_target(&model_to_use),
            "gemini-3.1-flash-image"
        );

        let body = build_image_edit_body(
            "project".to_string(),
            "account-resolved-image-model",
            json!([{"text": "prompt"}])
                .as_array()
                .cloned()
                .expect("parts"),
            image_config,
        );
        assert_eq!(body["model"], "account-resolved-image-model");
    }
}
