//! Model Alias Translation Dictionary.
//! Maps generic, shorthand, or legacy model identifiers to exact upstream provider models.

use std::collections::HashMap;

/// Resolves a requested model name or alias to its canonical upstream model string.
pub fn resolve_model_alias(input_model: &str) -> String {
    let lower = input_model.trim().to_ascii_lowercase();

    let aliases: HashMap<&str, &str> = [
        ("flash", "gemini-2.5-flash"),
        ("gemini-flash", "gemini-2.5-flash"),
        ("gemini-flash-thinking", "gemini-2.5-flash-thinking"),
        ("pro", "gemini-2.5-pro"),
        ("gemini-pro", "gemini-2.5-pro"),
        ("claude-sonnet", "claude-3-5-sonnet-20241022"),
        ("claude-3-5-sonnet", "claude-3-5-sonnet-20241022"),
        ("claude-3-7-sonnet", "claude-3-7-sonnet-20250219"),
        ("claude-haiku", "claude-3-5-haiku-20241022"),
        ("claude-opus", "claude-3-opus-20240229"),
        ("gpt-4o", "gemini-2.5-flash"),
        ("gpt-4o-mini", "gemini-2.5-flash"),
        ("o1", "gemini-2.5-pro"),
        ("o3-mini", "gemini-2.5-flash-thinking"),
    ]
    .into_iter()
    .collect();

    if let Some(canonical) = aliases.get(lower.as_str()) {
        canonical.to_string()
    } else {
        input_model.trim().to_string()
    }
}
