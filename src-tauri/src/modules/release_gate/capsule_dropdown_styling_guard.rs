//! Capsule & Dropdown Styling Guard.
//!
//! Enforces UI styling and layout invariants per AGENTS.md:
//! - When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close),
//!   always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border,
//!   subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons.
//! - Any inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless
//!   styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white
//!   pill-in-pill visual defects.

use serde::{Deserialize, Serialize};

/// Violation types for segmented pill capsules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapsuleStylingViolation {
    MissingRoundedFull { context: String },
    MissingSharedBorder { context: String },
    MissingSubtleDividers { context: String },
    MissingDarkGlassStyling { context: String },
    DisjointedCircularButtons { context: String },
}

/// Violation types for inline `<select>` dropdowns mounted inside capsules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropdownStylingViolation {
    MissingTransparentBackground {
        snippet: String,
    },
    MissingBorderZero {
        snippet: String,
    },
    MissingFocusRingZero {
        snippet: String,
    },
    MissingFocusOutlineNone {
        snippet: String,
    },
    NestedPillVisualDefect {
        snippet: String,
        offending_class: String,
    },
}

/// Evaluation report covering capsule and dropdown styling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleDropdownReport {
    pub is_clean: bool,
    pub capsule_violations: Vec<CapsuleStylingViolation>,
    pub dropdown_violations: Vec<DropdownStylingViolation>,
    pub summary: String,
}

/// Mandatory CSS classes for inline `<select>` dropdowns inside capsules.
pub const MANDATORY_DROPDOWN_CLASSES: &[&str] = &[
    "bg-transparent",
    "border-0",
    "focus:ring-0",
    "focus:outline-none",
];

/// Returns a conforming class string for segmented pill capsules.
pub fn conforming_segmented_capsule_classes() -> &'static str {
    "inline-flex items-center rounded-full border border-white/10 bg-zinc-900/60 backdrop-blur-md divide-x divide-white/10 shadow-sm"
}

/// Returns a conforming class string for desktop window controls capsules.
pub fn conforming_window_controls_capsule_classes() -> &'static str {
    "inline-flex items-center rounded-full border border-white/10 bg-zinc-900/70 backdrop-blur-md divide-x divide-white/10 overflow-hidden"
}

/// Returns a conforming class string for inline `<select>` dropdowns mounted inside capsules.
pub fn conforming_inline_select_classes() -> &'static str {
    "bg-transparent border-0 focus:ring-0 focus:outline-none text-xs text-zinc-300 py-1 px-2.5 cursor-pointer appearance-none"
}

/// Checks whether a class string contains `rounded-full`.
pub fn has_rounded_full(classes: &str) -> bool {
    classes.split_whitespace().any(|c| c == "rounded-full")
}

/// Checks whether a class string contains a shared border definition.
pub fn has_shared_border(classes: &str) -> bool {
    classes.split_whitespace().any(|c| {
        c == "border"
            || (c.starts_with("border-")
                && c != "border-0"
                && c != "border-none"
                && !c.starts_with("border-t-")
                && !c.starts_with("border-b-")
                && !c.starts_with("border-l-")
                && !c.starts_with("border-r-"))
    })
}

/// Checks whether a class string contains subtle divider lines between segments.
pub fn has_subtle_dividers(classes: &str) -> bool {
    classes.split_whitespace().any(|c| {
        c == "divide-x"
            || c.starts_with("divide-x-")
            || c.starts_with("divide-white/")
            || c.starts_with("divide-zinc-")
            || c.starts_with("divide-neutral-")
            || c.starts_with("divide-slate-")
            || c.starts_with("divide-gray-")
    })
}

/// Checks whether a class string contains dark-glass styling.
pub fn has_dark_glass_styling(classes: &str) -> bool {
    let tokens: Vec<&str> = classes.split_whitespace().collect();
    let has_blur = tokens.iter().any(|c| c.starts_with("backdrop-blur"));
    let has_dark_bg = tokens.iter().any(|c| {
        c.starts_with("bg-zinc-9")
            || c.starts_with("bg-black")
            || c.starts_with("bg-neutral-9")
            || c.starts_with("bg-slate-9")
            || *c == "dark-glass"
            || *c == "glass"
    });
    has_blur && has_dark_bg
}

/// Validates segmented pill capsule classes.
pub fn validate_segmented_capsule_classes(
    classes: &str,
    context: &str,
) -> Vec<CapsuleStylingViolation> {
    let mut violations = Vec::new();

    if !has_rounded_full(classes) {
        violations.push(CapsuleStylingViolation::MissingRoundedFull {
            context: context.to_string(),
        });
    }

    if !has_shared_border(classes) {
        violations.push(CapsuleStylingViolation::MissingSharedBorder {
            context: context.to_string(),
        });
    }

    if !has_subtle_dividers(classes) {
        violations.push(CapsuleStylingViolation::MissingSubtleDividers {
            context: context.to_string(),
        });
    }

    if !has_dark_glass_styling(classes) {
        violations.push(CapsuleStylingViolation::MissingDarkGlassStyling {
            context: context.to_string(),
        });
    }

    violations
}

/// Validates inline `<select>` classes against AGENTS.md requirements.
pub fn validate_inline_select_classes(
    classes: &str,
    snippet: &str,
) -> Vec<DropdownStylingViolation> {
    let tokens: Vec<&str> = classes.split_whitespace().collect();
    let mut violations = Vec::new();

    if !tokens.contains(&"bg-transparent") {
        violations.push(DropdownStylingViolation::MissingTransparentBackground {
            snippet: snippet.to_string(),
        });
    }

    if !tokens.contains(&"border-0") {
        violations.push(DropdownStylingViolation::MissingBorderZero {
            snippet: snippet.to_string(),
        });
    }

    if !tokens.contains(&"focus:ring-0") {
        violations.push(DropdownStylingViolation::MissingFocusRingZero {
            snippet: snippet.to_string(),
        });
    }

    if !tokens.contains(&"focus:outline-none") {
        violations.push(DropdownStylingViolation::MissingFocusOutlineNone {
            snippet: snippet.to_string(),
        });
    }

    // Check for nested pill visual defects
    for token in &tokens {
        let is_offending = *token == "rounded-full"
            || *token == "rounded-md"
            || *token == "rounded-lg"
            || *token == "rounded-xl"
            || *token == "bg-white"
            || token.starts_with("bg-white/")
            || *token == "bg-zinc-800"
            || *token == "bg-zinc-700"
            || *token == "bg-gray-100"
            || *token == "bg-gray-200"
            || (*token != "border-0" && token.starts_with("border-2"))
            || *token == "ring-1"
            || *token == "ring-2";

        if is_offending {
            violations.push(DropdownStylingViolation::NestedPillVisualDefect {
                snippet: snippet.to_string(),
                offending_class: token.to_string(),
            });
        }
    }

    violations
}

/// Sanitizes a dropdown class string by removing nested pill visual defect tokens
/// and ensuring all mandatory transparent borderless classes are present.
pub fn sanitize_dropdown_classes(current_classes: &str) -> String {
    let tokens: Vec<&str> = current_classes.split_whitespace().collect();
    let mut retained: Vec<String> = Vec::new();

    for token in tokens {
        let is_offending = token.starts_with("rounded")
            || (token.starts_with("border") && token != "border-0")
            || (token.starts_with("bg-") && token != "bg-transparent")
            || (token.starts_with("ring-") && token != "focus:ring-0" && token != "ring-0")
            || token == "ring"
            || token == "outline"
            || (token.starts_with("outline-") && token != "focus:outline-none");

        if !is_offending
            && token != "bg-transparent"
            && token != "border-0"
            && token != "focus:ring-0"
            && token != "focus:outline-none"
        {
            retained.push(token.to_string());
        }
    }

    let mut result = vec![
        "bg-transparent".to_string(),
        "border-0".to_string(),
        "focus:ring-0".to_string(),
        "focus:outline-none".to_string(),
    ];
    result.extend(retained);
    result.join(" ")
}

/// Evaluates both capsule and dropdown classes together.
pub fn validate_capsule_with_dropdown(
    capsule_classes: &str,
    select_classes: &str,
) -> Result<(), CapsuleDropdownReport> {
    let capsule_violations =
        validate_segmented_capsule_classes(capsule_classes, "capsule_container");
    let dropdown_violations = validate_inline_select_classes(select_classes, "<select ...>");

    let is_clean = capsule_violations.is_empty() && dropdown_violations.is_empty();
    let summary = if is_clean {
        "Capsule and dropdown styling fully conform to AGENTS.md".to_string()
    } else {
        format!(
            "Styling violations detected: {} capsule issues, {} dropdown issues",
            capsule_violations.len(),
            dropdown_violations.len()
        )
    };

    let report = CapsuleDropdownReport {
        is_clean,
        capsule_violations,
        dropdown_violations,
        summary,
    };

    if is_clean {
        Ok(())
    } else {
        Err(report)
    }
}

/// Scans UI markup (JSX/HTML) for capsule styling compliance and inline dropdown defects.
pub fn lint_ui_markup_for_capsules_and_dropdowns(content: &str) -> CapsuleDropdownReport {
    let mut capsule_violations = Vec::new();
    let mut dropdown_violations = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        // Detect disjointed circular buttons for window controls
        let is_window_control_btn =
            (line.contains("minimize") || line.contains("maximize") || line.contains("close"))
                && line.contains("<button");
        if is_window_control_btn
            && line.contains("rounded-full")
            && !line.contains("divide-x")
            && !line.contains("capsule")
        {
            // Check if it's rendered as loose disjointed buttons without a shared segmented wrapper
            if line.contains("w-") && line.contains("h-") {
                capsule_violations.push(CapsuleStylingViolation::DisjointedCircularButtons {
                    context: format!(
                        "Line {line_num}: Loose circular window control button detected"
                    ),
                });
            }
        }

        // Detect <select> tags
        if let Some(select_start) = line.find("<select") {
            let select_snippet = &line[select_start..];
            // Extract class or className attribute
            let class_str = extract_class_attribute(select_snippet);
            let select_issues = validate_inline_select_classes(
                &class_str,
                &format!("Line {line_num}: {}", line.trim()),
            );
            dropdown_violations.extend(select_issues);
        }
    }

    let is_clean = capsule_violations.is_empty() && dropdown_violations.is_empty();
    let summary = if is_clean {
        "UI markup fully satisfies capsule and dropdown styling invariants".to_string()
    } else {
        format!(
            "Markup violations: {} capsule, {} dropdown",
            capsule_violations.len(),
            dropdown_violations.len()
        )
    };

    CapsuleDropdownReport {
        is_clean,
        capsule_violations,
        dropdown_violations,
        summary,
    }
}

/// Helper to extract class or className value from a tag snippet.
fn extract_class_attribute(snippet: &str) -> String {
    let search_keys = ["className=\"", "class=\"", "className='", "class='"];
    for key in search_keys {
        if let Some(pos) = snippet.find(key) {
            let start = pos + key.len();
            let quote_char = if key.ends_with('\'') { '\'' } else { '"' };
            if let Some(end) = snippet[start..].find(quote_char) {
                return snippet[start..start + end].to_string();
            }
        }
    }
    String::new()
}
