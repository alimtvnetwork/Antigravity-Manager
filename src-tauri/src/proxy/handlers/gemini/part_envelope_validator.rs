//! Gemini Part Envelope Validator.
//! Inspects and normalizes Gemini response payload parts (`candidates[0].content.parts`),
//! filtering out corrupted empty text fragments and validating thought envelope structures.

use serde_json::{json, Value};

/// Result of validating a Gemini content part envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartValidationOutcome {
    ValidText(String),
    ValidThought(String),
    ValidFunctionCall { name: String, args: Value },
    ValidFunctionResponse { name: String, response: Value },
    EmptyOrRedundant,
    InvalidStructure(String),
}

/// Validates an individual part object in a Gemini content structure.
pub fn validate_part_envelope(part: &Value) -> PartValidationOutcome {
    let part_obj = match part.as_object() {
        Some(obj) => obj,
        None => return PartValidationOutcome::InvalidStructure("Part is not a JSON object".to_string()),
    };

    // Check thought block
    if let Some(thought) = part_obj.get("thought").and_then(|v| v.as_str()) {
        if thought.trim().is_empty() {
            return PartValidationOutcome::EmptyOrRedundant;
        }
        return PartValidationOutcome::ValidThought(thought.to_string());
    }

    // Check text block
    if let Some(text) = part_obj.get("text").and_then(|v| v.as_str()) {
        if text.trim().is_empty() {
            return PartValidationOutcome::EmptyOrRedundant;
        }
        return PartValidationOutcome::ValidText(text.to_string());
    }

    // Check function call
    if let Some(func_call) = part_obj.get("functionCall") {
        if let Some(name) = func_call.get("name").and_then(|v| v.as_str()) {
            let args = func_call.get("args").cloned().unwrap_or(json!({}));
            return PartValidationOutcome::ValidFunctionCall {
                name: name.to_string(),
                args,
            };
        }
    }

    // Check function response
    if let Some(func_resp) = part_obj.get("functionResponse") {
        if let Some(name) = func_resp.get("name").and_then(|v| v.as_str()) {
            let response = func_resp.get("response").cloned().unwrap_or(json!({}));
            return PartValidationOutcome::ValidFunctionResponse {
                name: name.to_string(),
                response,
            };
        }
    }

    PartValidationOutcome::InvalidStructure("Unrecognized part signature".to_string())
}

/// Sanitizes a candidates array by stripping corrupted or empty parts.
pub fn sanitize_gemini_candidates(candidates: &mut Value) -> usize {
    let mut stripped_count = 0;

    if let Some(cand_list) = candidates.as_array_mut() {
        for candidate in cand_list {
            if let Some(content) = candidate.get_mut("content") {
                if let Some(parts) = content.get_mut("parts").and_then(|p| p.as_array_mut()) {
                    let initial_len = parts.len();
                    parts.retain(|part| {
                        match validate_part_envelope(part) {
                            PartValidationOutcome::ValidText(_)
                            | PartValidationOutcome::ValidThought(_)
                            | PartValidationOutcome::ValidFunctionCall { .. }
                            | PartValidationOutcome::ValidFunctionResponse { .. } => true,
                            PartValidationOutcome::EmptyOrRedundant
                            | PartValidationOutcome::InvalidStructure(_) => false,
                        }
                    });
                    stripped_count += initial_len.saturating_sub(parts.len());
                }
            }
        }
    }

    stripped_count
}
