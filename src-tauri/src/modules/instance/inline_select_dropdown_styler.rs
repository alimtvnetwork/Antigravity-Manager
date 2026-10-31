//! Inline Select Dropdown Styler (`Imp-691`).
//!
//! Enforces AGENTS.md Minimalist & Contextual UI invariant:
//! "Any inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless
//! styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white
//! pill-in-pill visual defects."
//!
//! Provides styling verification, forbidden class detection, class normalization, and
//! HTML generation for inline select controls embedded within dark-glass segmented capsules.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical CSS classes for inline `<select>` dropdowns mounted within segmented capsules.
pub const CANONICAL_INLINE_SELECT_CLASSES: &str =
    "bg-transparent border-0 text-xs text-zinc-300 focus:ring-0 focus:outline-none cursor-pointer py-0.5 px-1.5 pr-5 appearance-none";

/// Canonical CSS classes for `<option>` items inside inline select dropdowns.
pub const CANONICAL_INLINE_OPTION_CLASSES: &str = "bg-zinc-900 text-zinc-200 text-xs py-1 px-2";

/// Mandatory CSS classes that must all be present for capsule inline select compliance.
pub const MANDATORY_INLINE_SELECT_CLASSES: &[&str] = &[
    "bg-transparent",
    "border-0",
    "focus:ring-0",
    "focus:outline-none",
];

/// Evaluation report covering inline `<select>` dropdown styling within segmented capsules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DropdownStylingReport {
    /// True if the class list satisfies all transparent borderless requirements without forbidden classes.
    pub is_compliant: bool,
    /// True if `bg-transparent` is present.
    pub has_transparent_bg: bool,
    /// True if `border-0` or `border-none` is present.
    pub has_zero_border: bool,
    /// True if `focus:ring-0` is present.
    pub has_zero_focus_ring: bool,
    /// True if `focus:outline-none` or `outline-none` is present.
    pub has_zero_outline: bool,
    /// List of forbidden classes detected that cause nested pill-in-pill or opaque visual defects.
    pub forbidden_classes_found: Vec<String>,
    /// Human-readable list of styling violations.
    pub violations: Vec<String>,
}

impl DropdownStylingReport {
    /// Returns true if the styling is fully compliant.
    pub fn is_compliant(&self) -> bool {
        self.is_compliant
    }

    /// Summary string describing the compliance status.
    pub fn summary(&self) -> String {
        if self.is_compliant {
            "Compliant: inline select satisfies transparent borderless capsule requirements"
                .to_string()
        } else {
            format!(
                "Non-compliant: {} styling violation(s) detected",
                self.violations.len()
            )
        }
    }
}

impl fmt::Display for DropdownStylingReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// Checks whether an individual CSS class token violates transparent borderless capsule styling.
pub fn is_forbidden_select_class(class_token: &str) -> bool {
    let t = class_token.trim();
    if t.is_empty() {
        return false;
    }

    // Opaque or light background classes (causes nested white / solid pill defects)
    if t.starts_with("bg-") && t != "bg-transparent" {
        return true;
    }

    // Nested rounded/pill classes inside an already-rounded capsule
    if t.starts_with("rounded") {
        return true;
    }

    // Visible borders
    if (t == "border" || t.starts_with("border-")) && t != "border-0" && t != "border-none" {
        return true;
    }

    // Visible focus rings
    if (t == "ring" || t.starts_with("ring-") || t == "focus:ring" || t.starts_with("focus:ring-"))
        && t != "focus:ring-0"
        && t != "ring-0"
    {
        return true;
    }

    // Focus outlines other than outline-none
    if (t == "outline" || t == "focus:outline" || t.starts_with("focus:outline-"))
        && t != "focus:outline-none"
        && t != "outline-none"
    {
        return true;
    }

    // Outer or inner shadows
    if t.starts_with("shadow") {
        return true;
    }

    false
}

/// Validates inline `<select>` class names against AGENTS.md requirements.
pub fn validate_select_classes(classes: &str) -> DropdownStylingReport {
    let tokens: Vec<&str> = classes.split_whitespace().collect();

    let has_transparent_bg = tokens.contains(&"bg-transparent");
    let has_zero_border = tokens.contains(&"border-0") || tokens.contains(&"border-none");
    let has_zero_focus_ring = tokens.contains(&"focus:ring-0");
    let has_zero_outline =
        tokens.contains(&"focus:outline-none") || tokens.contains(&"outline-none");

    let mut forbidden_classes_found = Vec::new();
    for &token in &tokens {
        if is_forbidden_select_class(token) && !forbidden_classes_found.contains(&token.to_string())
        {
            forbidden_classes_found.push(token.to_string());
        }
    }

    let mut violations = Vec::new();
    if !has_transparent_bg {
        violations.push("Missing mandatory transparent background: 'bg-transparent'".to_string());
    }
    if !has_zero_border {
        violations.push("Missing mandatory zero border: 'border-0'".to_string());
    }
    if !has_zero_focus_ring {
        violations.push("Missing mandatory zero focus ring: 'focus:ring-0'".to_string());
    }
    if !has_zero_outline {
        violations.push("Missing mandatory zero focus outline: 'focus:outline-none'".to_string());
    }
    for forbidden in &forbidden_classes_found {
        violations.push(format!(
            "Forbidden class causing nested pill defect: '{forbidden}'"
        ));
    }

    let is_compliant = has_transparent_bg
        && has_zero_border
        && has_zero_focus_ring
        && has_zero_outline
        && forbidden_classes_found.is_empty();

    DropdownStylingReport {
        is_compliant,
        has_transparent_bg,
        has_zero_border,
        has_zero_focus_ring,
        has_zero_outline,
        forbidden_classes_found,
        violations,
    }
}

/// Normalizes an existing class string by removing forbidden classes and ensuring mandatory classes.
pub fn normalize_select_classes(existing_classes: &str) -> String {
    let trimmed = existing_classes.trim();
    if trimmed.is_empty() {
        return CANONICAL_INLINE_SELECT_CLASSES.to_string();
    }

    let mut retained_tokens = Vec::new();

    for token in trimmed.split_whitespace() {
        if is_forbidden_select_class(token) {
            continue;
        }
        // Skip redundant variations of mandatory classes
        if token == "border-none" || token == "outline-none" {
            continue;
        }
        if MANDATORY_INLINE_SELECT_CLASSES.contains(&token) {
            continue;
        }
        if !retained_tokens.contains(&token) {
            retained_tokens.push(token);
        }
    }

    let mut result_tokens =
        Vec::with_capacity(MANDATORY_INLINE_SELECT_CLASSES.len() + retained_tokens.len());
    for &mandatory in MANDATORY_INLINE_SELECT_CLASSES {
        result_tokens.push(mandatory);
    }
    for retained in retained_tokens {
        result_tokens.push(retained);
    }

    result_tokens.join(" ")
}

/// Generates compliant HTML for an inline `<select>` dropdown inside a segmented capsule.
pub fn format_capsule_select_html(
    name: &str,
    options: &[(&str, &str)],
    selected_value: &str,
) -> String {
    format_capsule_select_html_with_classes(
        name,
        options,
        selected_value,
        CANONICAL_INLINE_SELECT_CLASSES,
        CANONICAL_INLINE_OPTION_CLASSES,
    )
}

/// Generates HTML for an inline `<select>` dropdown with custom classes.
pub fn format_capsule_select_html_with_classes(
    name: &str,
    options: &[(&str, &str)],
    selected_value: &str,
    select_classes: &str,
    option_classes: &str,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "<select name=\"{}\" class=\"{}\">\n",
        escape_html(name),
        escape_html(select_classes)
    ));

    for (value, label) in options {
        let is_selected = *value == selected_value;
        let selected_attr = if is_selected { " selected" } else { "" };
        out.push_str(&format!(
            "  <option value=\"{}\" class=\"{}\"{}>{}</option>\n",
            escape_html(value),
            escape_html(option_classes),
            selected_attr,
            escape_html(label)
        ));
    }

    out.push_str("</select>");
    out
}

/// Escapes HTML entities for safe attribute and text embedding.
fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
