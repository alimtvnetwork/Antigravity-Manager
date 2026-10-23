//! Capsule Segmented Button Group Styler (`Imp-696`).
//!
//! Enforces AGENTS.md Minimalist & Contextual UI invariants:
//! "When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close),
//! always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle
//! divider lines, and dark-glass styling) rather than loose, disjointed circular buttons. Any
//! inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless
//! styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white
//! pill-in-pill visual defects."
//!
//! Provides styling verification, compliance reporting, and HTML generation for contiguous
//! dark-glass segmented button groups and window control capsules.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard dark-glass capsule container CSS classes for horizontal segmented button groups.
pub const CANONICAL_BUTTON_GROUP_CONTAINER_CLASSES: &str =
    "inline-flex items-center rounded-full border border-zinc-800 bg-zinc-950/80 p-0.5 backdrop-blur-md shadow-sm";

/// Standard dark-glass capsule container CSS classes for vertical segmented button groups.
pub const CANONICAL_BUTTON_GROUP_CONTAINER_VERTICAL_CLASSES: &str =
    "inline-flex flex-col items-center rounded-full border border-zinc-800 bg-zinc-950/80 p-0.5 backdrop-blur-md shadow-sm";

/// Standard subtle divider CSS classes between adjacent horizontal capsule buttons.
pub const CANONICAL_SEGMENT_DIVIDER_CLASSES: &str = "h-3.5 w-px bg-zinc-800/80";

/// Standard subtle divider CSS classes between adjacent vertical capsule buttons.
pub const CANONICAL_SEGMENT_DIVIDER_VERTICAL_CLASSES: &str = "w-3.5 h-px bg-zinc-800/80";

/// Standard button segment CSS classes inside the segmented capsule.
pub const CANONICAL_SEGMENT_BUTTON_CLASSES: &str =
    "relative flex h-6 items-center justify-center px-2.5 text-xs text-zinc-400 transition-colors hover:text-zinc-100 hover:bg-zinc-800/60 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-zinc-400";

/// Canonical active button styling modifier for active segments.
pub const CANONICAL_SEGMENT_BUTTON_ACTIVE_CLASSES: &str =
    "relative flex h-6 items-center justify-center px-2.5 text-xs text-zinc-100 bg-zinc-800/80 font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-zinc-400";

/// Canonical classes required for compliant inline `<select>` within capsules.
pub const MANDATORY_INLINE_SELECT_CLASSES: &[&str] = &[
    "bg-transparent",
    "border-0",
    "focus:ring-0",
    "focus:outline-none",
];

/// An individual button element rendered within a segmented capsule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentButton {
    /// Unique identifier for DOM and events.
    pub id: String,
    /// User-facing label or text.
    pub label: String,
    /// Optional icon glyph identifier.
    pub icon: Option<String>,
    /// Full accessibility text label.
    pub aria_label: String,
    /// True if currently selected/pressed.
    pub is_active: bool,
    /// True if currently disabled.
    pub is_disabled: bool,
    /// Optional additional CSS classes.
    pub custom_classes: Option<String>,
}

impl SegmentButton {
    /// Constructs a new segment button with defaults.
    pub fn new(id: impl Into<String>, label: impl Into<String>, aria_label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            aria_label: aria_label.into(),
            is_active: false,
            is_disabled: false,
            custom_classes: None,
        }
    }

    /// Builder method to specify an icon name.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Builder method to specify active state.
    pub fn with_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    /// Builder method to specify disabled state.
    pub fn with_disabled(mut self, is_disabled: bool) -> Self {
        self.is_disabled = is_disabled;
        self
    }

    /// Builder method to append custom CSS classes.
    pub fn with_custom_classes(mut self, classes: impl Into<String>) -> Self {
        self.custom_classes = Some(classes.into());
        self
    }
}

/// Geometric orientation of a segmented capsule group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapsuleOrientation {
    /// Horizontal row orientation (standard toolbar/window controls).
    Horizontal,
    /// Vertical column orientation.
    Vertical,
}

impl Default for CapsuleOrientation {
    fn default() -> Self {
        Self::Horizontal
    }
}

/// Configuration governing capsule segmented button group styling and layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleGroupConfig {
    /// Horizontal or Vertical orientation.
    pub orientation: CapsuleOrientation,
    /// Size tier ("sm", "md", "lg").
    pub size_tier: String,
    /// True for dark-glass styling (`bg-zinc-950/80 backdrop-blur-md`).
    pub is_dark_glass: bool,
}

impl Default for CapsuleGroupConfig {
    fn default() -> Self {
        Self {
            orientation: CapsuleOrientation::Horizontal,
            size_tier: "sm".to_string(),
            is_dark_glass: true,
        }
    }
}

impl CapsuleGroupConfig {
    /// Constructs a custom capsule group configuration.
    pub fn new(orientation: CapsuleOrientation, size_tier: impl Into<String>, is_dark_glass: bool) -> Self {
        Self {
            orientation,
            size_tier: size_tier.into(),
            is_dark_glass,
        }
    }

    /// Constructs a standard horizontal dark-glass configuration.
    pub fn horizontal_dark_glass(size_tier: impl Into<String>) -> Self {
        Self {
            orientation: CapsuleOrientation::Horizontal,
            size_tier: size_tier.into(),
            is_dark_glass: true,
        }
    }

    /// Constructs a vertical dark-glass configuration.
    pub fn vertical_dark_glass(size_tier: impl Into<String>) -> Self {
        Self {
            orientation: CapsuleOrientation::Vertical,
            size_tier: size_tier.into(),
            is_dark_glass: true,
        }
    }
}

/// Generated styling classes and formatted HTML for a segmented button capsule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleFormattingResult {
    /// Root container CSS classes.
    pub container_classes: String,
    /// Inter-button subtle divider CSS classes.
    pub divider_classes: String,
    /// Formatted CSS classes for each respective button.
    pub button_classes: Vec<String>,
    /// Complete accessible HTML/JSX markup.
    pub formatted_html: String,
}

/// Violation and compliance report produced by auditing button group markup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleViolationReport {
    /// True if the markup satisfies all capsule invariants.
    pub is_compliant: bool,
    /// List of detected styling violations.
    pub violations: Vec<String>,
    /// Concrete suggestions for remediation.
    pub suggestions: Vec<String>,
}

impl CapsuleViolationReport {
    /// Constructs a compliant report.
    pub fn compliant() -> Self {
        Self {
            is_compliant: true,
            violations: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Adds a violation and remediation suggestion.
    pub fn with_violation(mut self, violation: impl Into<String>, suggestion: impl Into<String>) -> Self {
        self.is_compliant = false;
        self.violations.push(violation.into());
        self.suggestions.push(suggestion.into());
        self
    }

    /// Returns true if violations exist.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }
}

/// Formats a list of segment buttons into a contiguous segmented pill capsule per AGENTS.md rules.
pub fn format_segmented_button_group(
    buttons: &[SegmentButton],
    config: &CapsuleGroupConfig,
) -> CapsuleFormattingResult {
    let container_classes = match (config.orientation, config.is_dark_glass) {
        (CapsuleOrientation::Horizontal, true) => {
            CANONICAL_BUTTON_GROUP_CONTAINER_CLASSES.to_string()
        }
        (CapsuleOrientation::Vertical, true) => {
            CANONICAL_BUTTON_GROUP_CONTAINER_VERTICAL_CLASSES.to_string()
        }
        (CapsuleOrientation::Horizontal, false) => {
            "inline-flex items-center rounded-full border border-zinc-300 bg-zinc-100 p-0.5 shadow-sm".to_string()
        }
        (CapsuleOrientation::Vertical, false) => {
            "inline-flex flex-col items-center rounded-full border border-zinc-300 bg-zinc-100 p-0.5 shadow-sm".to_string()
        }
    };

    let divider_classes = match (config.orientation, config.is_dark_glass) {
        (CapsuleOrientation::Horizontal, true) => CANONICAL_SEGMENT_DIVIDER_CLASSES.to_string(),
        (CapsuleOrientation::Vertical, true) => CANONICAL_SEGMENT_DIVIDER_VERTICAL_CLASSES.to_string(),
        (CapsuleOrientation::Horizontal, false) => "h-3.5 w-px bg-zinc-300".to_string(),
        (CapsuleOrientation::Vertical, false) => "w-3.5 h-px bg-zinc-300".to_string(),
    };

    let num_buttons = buttons.len();
    let mut button_classes = Vec::with_capacity(num_buttons);

    for (idx, btn) in buttons.iter().enumerate() {
        let is_icon_only = btn.label.is_empty() && btn.icon.is_some();

        let size_classes = match (config.size_tier.as_str(), is_icon_only) {
            ("lg", true) => "h-8 w-9 text-sm",
            ("lg", false) => "h-8 px-4 text-sm",
            ("md", true) => "h-7 w-8 text-xs",
            ("md", false) => "h-7 px-3 text-xs",
            ("sm", true) => "h-6 w-7 text-xs",
            _ => "h-6 px-2.5 text-xs",
        };

        let corner_class = if num_buttons == 1 {
            "rounded-full"
        } else if config.orientation == CapsuleOrientation::Horizontal {
            if idx == 0 {
                "rounded-l-full"
            } else if idx == num_buttons - 1 {
                "rounded-r-full"
            } else {
                ""
            }
        } else if idx == 0 {
            "rounded-t-full"
        } else if idx == num_buttons - 1 {
            "rounded-b-full"
        } else {
            ""
        };

        let state_classes = if btn.is_disabled {
            "opacity-50 cursor-not-allowed pointer-events-none"
        } else if btn.is_active {
            if config.is_dark_glass {
                "bg-zinc-800 text-zinc-100 shadow-xs font-medium"
            } else {
                "bg-white text-zinc-900 shadow-xs font-medium"
            }
        } else if config.is_dark_glass {
            "text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800/60"
        } else {
            "text-zinc-600 hover:text-zinc-900 hover:bg-zinc-200"
        };

        let base_cls = "relative flex items-center justify-center transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-zinc-400";

        let mut classes = format!("{base_cls} {size_classes}");
        if !corner_class.is_empty() {
            classes.push(' ');
            classes.push_str(corner_class);
        }
        classes.push(' ');
        classes.push_str(state_classes);

        if let Some(custom) = &btn.custom_classes {
            classes.push(' ');
            classes.push_str(custom);
        }

        button_classes.push(classes);
    }

    let mut formatted_html = format!(
        "<div role=\"group\" aria-label=\"Segmented button group\" class=\"{}\">\n",
        escape_html(&container_classes)
    );

    for (idx, btn) in buttons.iter().enumerate() {
        if idx > 0 {
            let orientation_attr = if config.orientation == CapsuleOrientation::Vertical {
                "horizontal"
            } else {
                "vertical"
            };
            formatted_html.push_str(&format!(
                "  <div class=\"{}\" role=\"separator\" aria-orientation=\"{}\" aria-hidden=\"true\"></div>\n",
                escape_html(&divider_classes),
                orientation_attr
            ));
        }

        let pressed_attr = if btn.is_active {
            " aria-pressed=\"true\""
        } else {
            " aria-pressed=\"false\""
        };
        let disabled_attr = if btn.is_disabled { " disabled" } else { "" };

        let content = match (&btn.icon, btn.label.as_str()) {
            (Some(icon), "") => format!("<span class=\"icon-{}\" aria-hidden=\"true\"></span>", escape_html(icon)),
            (Some(icon), lbl) => format!(
                "<span class=\"icon-{}\" aria-hidden=\"true\"></span><span>{}</span>",
                escape_html(icon),
                escape_html(lbl)
            ),
            (None, lbl) => escape_html(lbl),
        };

        formatted_html.push_str(&format!(
            "  <button type=\"button\" id=\"{}\" aria-label=\"{}\"{}{} class=\"{}\">{}</button>\n",
            escape_html(&btn.id),
            escape_html(&btn.aria_label),
            pressed_attr,
            disabled_attr,
            escape_html(&button_classes[idx]),
            content
        ));
    }

    formatted_html.push_str("</div>");

    CapsuleFormattingResult {
        container_classes,
        divider_classes,
        button_classes,
        formatted_html,
    }
}

/// Audits HTML or JSX markup for compliance with AGENTS.md capsule styling invariants.
///
/// Flags loose circular buttons (`rounded-full p-2` not wrapped in a shared container),
/// missing divider classes, missing `rounded-full` on container, or non-transparent inline `<select>` controls.
pub fn audit_button_group_markup(html_or_jsx: &str) -> CapsuleViolationReport {
    let mut violations = Vec::new();
    let mut suggestions = Vec::new();

    let normalized = html_or_jsx.replace("><", ">\n<");
    let lines: Vec<&str> = normalized.lines().collect();

    let mut inside_capsule_depth: usize = 0;
    let mut capsule_button_count: usize = 0;
    let mut capsule_has_divider: bool = false;

    for raw_line in lines {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let lower = line.to_ascii_lowercase();

        // 1. Container open check
        let is_container_open = lower.contains("<div")
            || lower.contains("<nav")
            || lower.contains("<header")
            || lower.contains("<section")
            || lower.contains("<toolbar");

        if is_container_open {
            let is_control_container = lower.contains("window-controls")
                || lower.contains("windowcontrols")
                || lower.contains("window_controls")
                || lower.contains("toolbar-actions")
                || lower.contains("toolbar_actions")
                || lower.contains("segmented-control")
                || lower.contains("segmented-button")
                || lower.contains("role=\"toolbar\"")
                || lower.contains("role=\"group\"");

            if is_control_container && !lower.contains("rounded-full") {
                violations.push(format!(
                    "Button group or window control container is missing mandatory 'rounded-full' pill styling: '{line}'"
                ));
                suggestions.push(
                    "Wrap adjacent controls in a contiguous segmented pill capsule with 'rounded-full border' styling"
                        .to_string(),
                );
            }

            let is_capsule = lower.contains("rounded-full")
                && (lower.contains("border") || lower.contains("bg-zinc") || lower.contains("inline-flex") || lower.contains("flex"));

            if is_capsule {
                inside_capsule_depth += 1;
                capsule_button_count = 0;
                capsule_has_divider = false;
            }
        }

        // 2. Button checks
        if lower.contains("<button") {
            if inside_capsule_depth > 0 {
                capsule_button_count += 1;
            } else {
                let has_circular_padding = lower.contains("p-2")
                    || lower.contains("p-1.5")
                    || lower.contains("p-1")
                    || lower.contains("p-2.5")
                    || lower.contains("p-3")
                    || lower.contains("w-8 h-8")
                    || lower.contains("w-7 h-7")
                    || lower.contains("aspect-square");

                if lower.contains("rounded-full") && has_circular_padding {
                    violations.push(format!(
                        "Loose disjoint circular button detected outside segmented capsule container: '{line}'"
                    ));
                    suggestions.push(
                        "Wrap adjacent action buttons inside a contiguous segmented capsule container with 'rounded-full border' and subtle dividers"
                            .to_string(),
                    );
                }
            }
        }

        // 3. Divider checks
        if inside_capsule_depth > 0 {
            let is_divider_tag = lower.contains("<div") || lower.contains("<span");
            let has_divider_class = lower.contains("w-px")
                || lower.contains("h-px")
                || lower.contains("divider")
                || lower.contains("role=\"separator\"");

            if is_divider_tag && has_divider_class {
                capsule_has_divider = true;
            }
        }

        // 4. Inline select checks
        if lower.contains("<select") && inside_capsule_depth > 0 {
            let has_transparent = lower.contains("bg-transparent");
            let has_border_0 = lower.contains("border-0") || lower.contains("border-none");
            let has_focus_ring = lower.contains("focus:ring-0");
            let has_focus_outline = lower.contains("focus:outline-none") || lower.contains("outline-none");

            if !(has_transparent && has_border_0 && has_focus_ring && has_focus_outline) {
                violations.push(format!(
                    "Inline <select> dropdown mounted within segmented capsule is missing transparent borderless styling ('bg-transparent border-0 focus:ring-0 focus:outline-none'): '{line}'"
                ));
                suggestions.push(
                    "Apply 'bg-transparent border-0 focus:ring-0 focus:outline-none' to eliminate nested white pill-in-pill visual defects"
                        .to_string(),
                );
            }
        }

        // 5. Container close check
        let is_closing_container = lower.contains("</div>")
            || lower.contains("</nav>")
            || lower.contains("</header>")
            || lower.contains("</section>");

        if is_closing_container && !lower.contains("<div") && inside_capsule_depth > 0 {
            if capsule_button_count > 1 && !capsule_has_divider {
                violations.push(
                    "Segmented capsule container with multiple buttons is missing subtle divider elements between segments"
                        .to_string(),
                );
                suggestions.push(
                    "Insert subtle divider elements ('h-3.5 w-px bg-zinc-800/80') between adjacent capsule buttons"
                        .to_string(),
                );
            }
            inside_capsule_depth = inside_capsule_depth.saturating_sub(1);
        }
    }

    let is_compliant = violations.is_empty();
    CapsuleViolationReport {
        is_compliant,
        violations,
        suggestions,
    }
}

// =========================================================================
// Backward Compatibility Layer (SegmentedGroupItem, validate_segmented_group_classes)
// =========================================================================

/// An individual button item within a segmented capsule button group (backward compatibility).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentedGroupItem {
    pub id: String,
    pub label: String,
    pub is_active: bool,
    pub is_disabled: bool,
}

impl SegmentedGroupItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            is_active: false,
            is_disabled: false,
        }
    }

    pub fn with_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    pub fn with_disabled(mut self, is_disabled: bool) -> Self {
        self.is_disabled = is_disabled;
        self
    }
}

/// Evaluation report assessing segmented button group CSS class compliance (backward compatibility).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentedGroupStylingReport {
    pub is_compliant: bool,
    pub has_rounded_full: bool,
    pub has_shared_border: bool,
    pub has_subtle_divider: bool,
    pub has_dark_glass_bg: bool,
    pub violations: Vec<String>,
}

impl SegmentedGroupStylingReport {
    pub fn is_compliant(&self) -> bool {
        self.is_compliant
    }

    pub fn summary(&self) -> String {
        if self.is_compliant {
            "Compliant: segmented button group satisfies all AGENTS.md capsule invariants".to_string()
        } else {
            format!(
                "Non-compliant: {} styling violation(s) detected",
                self.violations.len()
            )
        }
    }
}

impl fmt::Display for SegmentedGroupStylingReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// Validates segmented button group container CSS classes against AGENTS.md capsule styling invariants.
pub fn validate_segmented_group_classes(classes: &str) -> SegmentedGroupStylingReport {
    let tokens: Vec<&str> = classes.split_whitespace().collect();

    let has_rounded_full = tokens.contains(&"rounded-full");
    let has_shared_border = tokens.contains(&"border")
        || tokens.iter().any(|t| t.starts_with("border-") && *t != "border-none" && *t != "border-0");
    let has_dark_glass_bg = tokens.iter().any(|t| {
        t.starts_with("bg-zinc-950")
            || t.starts_with("bg-zinc-900")
            || t.starts_with("bg-zinc-800")
            || t.starts_with("bg-black")
    }) && tokens.iter().any(|t| t.starts_with("backdrop-blur"));

    let has_subtle_divider = tokens.iter().any(|t| t.starts_with("divide-") || *t == "h-3.5" || t.contains("divider"))
        || classes.contains("border-zinc-800")
        || classes.contains("bg-zinc-800");

    let mut violations = Vec::new();

    if !has_rounded_full {
        violations.push("Missing mandatory 'rounded-full' pill capsule contour".to_string());
    }
    if !has_shared_border {
        violations.push("Missing shared border enclosure ('border' or 'border-zinc-*')".to_string());
    }
    if !has_dark_glass_bg {
        violations.push("Missing dark-glass background styling ('bg-zinc-*' and 'backdrop-blur-*')".to_string());
    }
    if !has_subtle_divider {
        violations.push("Missing subtle divider configuration ('divide-zinc-*' or discrete divider)".to_string());
    }

    if tokens.contains(&"space-x-2")
        || tokens.contains(&"space-x-4")
        || tokens.contains(&"gap-2")
        || tokens.contains(&"gap-4")
    {
        violations.push(
            "Detected loose button spacing ('space-x-*' or 'gap-*') instead of contiguous segmented capsule"
                .to_string(),
        );
    }

    if tokens.contains(&"bg-white") || tokens.iter().any(|t| t.starts_with("bg-zinc-100") || t.starts_with("bg-gray-100")) {
        violations.push("Detected forbidden light background in dark-glass capsule".to_string());
    }

    let is_compliant = has_rounded_full
        && has_shared_border
        && has_dark_glass_bg
        && has_subtle_divider
        && violations.is_empty();

    SegmentedGroupStylingReport {
        is_compliant,
        has_rounded_full,
        has_shared_border,
        has_subtle_divider,
        has_dark_glass_bg,
        violations,
    }
}

/// Formats a list of segmented group items into canonical contiguous dark-glass HTML markup.
pub fn format_segmented_group_html(items: &[SegmentedGroupItem]) -> String {
    format_segmented_group_html_with_classes(
        items,
        CANONICAL_BUTTON_GROUP_CONTAINER_CLASSES,
        CANONICAL_SEGMENT_BUTTON_CLASSES,
        CANONICAL_SEGMENT_DIVIDER_CLASSES,
    )
}

/// Formats a list of segmented group items with customizable CSS classes.
pub fn format_segmented_group_html_with_classes(
    items: &[SegmentedGroupItem],
    container_classes: &str,
    button_classes: &str,
    divider_classes: &str,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "<div class=\"{}\" role=\"group\">\n",
        escape_html(container_classes)
    ));

    for (idx, item) in items.iter().enumerate() {
        if idx > 0 {
            out.push_str(&format!(
                "  <div class=\"{}\" role=\"separator\" aria-orientation=\"vertical\"></div>\n",
                escape_html(divider_classes)
            ));
        }

        let active_attr = if item.is_active {
            " aria-pressed=\"true\""
        } else {
            " aria-pressed=\"false\""
        };
        let disabled_attr = if item.is_disabled { " disabled" } else { "" };
        let state_classes = if item.is_active {
            " text-zinc-100 bg-zinc-800/80 font-medium"
        } else {
            ""
        };
        let disabled_classes = if item.is_disabled {
            " opacity-50 cursor-not-allowed"
        } else {
            ""
        };

        out.push_str(&format!(
            "  <button type=\"button\" id=\"{}\" class=\"{}{}{}\"{}{}>{}</button>\n",
            escape_html(&item.id),
            escape_html(button_classes),
            state_classes,
            disabled_classes,
            active_attr,
            disabled_attr,
            escape_html(&item.label)
        ));
    }

    out.push_str("</div>");
    out
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
