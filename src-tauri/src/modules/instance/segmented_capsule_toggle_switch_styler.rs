//! Segmented Capsule Toggle Switch Styler & Linter (`Imp-711`).
//!
//! Enforces AGENTS.md Minimalist & Contextual UI invariants:
//! "When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close),
//! always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle
//! divider lines, and dark-glass styling) rather than loose, disjointed circular buttons. Any
//! inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless
//! styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white
//! pill-in-pill visual defects."
//!
//! Audits and generates JSX/HTML layout tokens for contiguous segmented toggle capsules. Enforces
//! `rounded-full`, shared dark-glass container borders, subtle divider lines, and transparent borderless
//! inline select dropdowns.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard dark-glass capsule container CSS classes for segmented toggle switches.
pub const CANONICAL_CAPSULE_CONTAINER_CLASSES: &str =
    "inline-flex items-center rounded-full border border-zinc-800 bg-zinc-950/80 p-0.5 backdrop-blur-md shadow-sm";

/// Standard subtle divider CSS classes between adjacent capsule segments.
pub const CANONICAL_CAPSULE_DIVIDER_CLASSES: &str = "h-3.5 w-px bg-zinc-800/80 my-auto";

/// Mandatory classes required for compliant inline `<select>` mounted inside capsules.
pub const MANDATORY_INLINE_SELECT_CLASSES: &[&str] = &[
    "bg-transparent",
    "border-0",
    "focus:ring-0",
    "focus:outline-none",
];

/// Forbidden classes that cause nested pill-in-pill or opaque visual defects on `<select>`.
pub const FORBIDDEN_SELECT_CLASSES: &[&str] = &[
    "bg-white",
    "bg-zinc-900",
    "bg-zinc-800",
    "border",
    "border-zinc-700",
    "rounded-md",
    "rounded-lg",
    "rounded-full",
    "rounded",
];

/// A single option within a segmented capsule toggle switch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleSegmentOption {
    /// Unique identifier for the option.
    pub id: String,
    /// User-facing label or text.
    pub label: String,
    /// True if currently active/selected.
    pub is_selected: bool,
    /// True if disabled.
    pub is_disabled: bool,
    /// Accessibility label.
    pub aria_label: String,
}

impl CapsuleSegmentOption {
    /// Constructs a new capsule segment option with defaults.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        let id_str = id.into();
        let label_str = label.into();
        let aria = if label_str.is_empty() {
            id_str.clone()
        } else {
            label_str.clone()
        };
        Self {
            id: id_str,
            label: label_str,
            is_selected: false,
            is_disabled: false,
            aria_label: aria,
        }
    }

    /// Builder to set selection state.
    pub fn with_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    /// Builder to set disabled state.
    pub fn with_disabled(mut self, is_disabled: bool) -> Self {
        self.is_disabled = is_disabled;
        self
    }

    /// Builder to set aria label.
    pub fn with_aria_label(mut self, aria_label: impl Into<String>) -> Self {
        self.aria_label = aria_label.into();
        self
    }
}

/// Styling configuration for a segmented capsule toggle switch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleStyleConfig {
    /// Size tier: "sm", "md", or "lg".
    pub size_tier: String,
    /// Whether to apply dark-glass styling (`bg-zinc-950/80 backdrop-blur-md border-zinc-800`).
    pub is_dark_glass: bool,
    /// Whether subtle divider lines are rendered between segments.
    pub has_dividers: bool,
}

impl Default for CapsuleStyleConfig {
    fn default() -> Self {
        Self {
            size_tier: "md".to_string(),
            is_dark_glass: true,
            has_dividers: true,
        }
    }
}

impl CapsuleStyleConfig {
    /// Constructs a custom capsule style configuration.
    pub fn new(size_tier: impl Into<String>, is_dark_glass: bool, has_dividers: bool) -> Self {
        Self {
            size_tier: size_tier.into(),
            is_dark_glass,
            has_dividers,
        }
    }

    /// Constructs a standard dark-glass style configuration for the given size tier.
    pub fn dark_glass(size_tier: impl Into<String>) -> Self {
        Self {
            size_tier: size_tier.into(),
            is_dark_glass: true,
            has_dividers: true,
        }
    }

    /// Constructs a minimal style configuration without dividers.
    pub fn minimal(size_tier: impl Into<String>) -> Self {
        Self {
            size_tier: size_tier.into(),
            is_dark_glass: true,
            has_dividers: false,
        }
    }
}

/// Formatted layout classes and rendered markup for a segmented capsule toggle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentedCapsuleFormatting {
    /// CSS classes applied to the outer capsule container.
    pub container_classes: String,
    /// CSS classes applied to each segment button.
    pub segment_classes: Vec<String>,
    /// CSS classes applied to divider elements between adjacent segments.
    pub divider_classes: String,
    /// Generated HTML / JSX markup for the segmented capsule switch.
    pub formatted_markup: String,
}

/// Findings emitted when auditing capsule markup compliance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapsuleAuditFinding {
    /// Disjoint loose circular buttons or window controls detected without capsule grouping.
    DisjointButtonsDetected,
    /// Segmented container missing `rounded-full` pill geometry.
    MissingRoundedFull,
    /// Inline `<select>` dropdown missing transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`).
    BorderlessSelectMissing,
    /// Markup satisfies all AGENTS.md Minimalist & Contextual UI invariants.
    Compliant,
}

impl CapsuleAuditFinding {
    /// Returns true if this finding represents full compliance.
    pub fn is_compliant(&self) -> bool {
        matches!(self, Self::Compliant)
    }
}

impl fmt::Display for CapsuleAuditFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DisjointButtonsDetected => write!(
                f,
                "DisjointButtonsDetected: loose circular buttons detected without contiguous capsule"
            ),
            Self::MissingRoundedFull => {
                write!(
                    f,
                    "MissingRoundedFull: capsule container missing 'rounded-full' pill styling"
                )
            }
            Self::BorderlessSelectMissing => write!(
                f,
                "BorderlessSelectMissing: inline select missing transparent borderless styling"
            ),
            Self::Compliant => write!(f, "Compliant: markup satisfies capsule UI guidelines"),
        }
    }
}

/// Compliance report generated from auditing component markup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleAuditReport {
    /// True if all capsule styling invariants are satisfied.
    pub is_compliant: bool,
    /// Catalog of findings discovered during markup inspection.
    pub findings: Vec<CapsuleAuditFinding>,
    /// Recommended corrective actions for any violations.
    pub recommendations: Vec<String>,
}

impl CapsuleAuditReport {
    /// Constructs a new report.
    pub fn new(
        is_compliant: bool,
        findings: Vec<CapsuleAuditFinding>,
        recommendations: Vec<String>,
    ) -> Self {
        Self {
            is_compliant,
            findings,
            recommendations,
        }
    }

    /// Returns true if compliant.
    pub fn is_compliant(&self) -> bool {
        self.is_compliant
    }

    /// Checks if a specific finding is present in the report.
    pub fn has_finding(&self, finding: &CapsuleAuditFinding) -> bool {
        self.findings.contains(finding)
    }

    /// Summary text of the audit report.
    pub fn summary(&self) -> String {
        if self.is_compliant {
            "Compliant: all segmented capsule styling invariants satisfied".to_string()
        } else {
            format!(
                "Non-compliant: {} styling finding(s) detected",
                self.findings.len()
            )
        }
    }
}

impl fmt::Display for CapsuleAuditReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// Formats and generates class tokens and markup for a segmented capsule toggle switch.
pub fn format_segmented_capsule_toggle(
    options: &[CapsuleSegmentOption],
    config: &CapsuleStyleConfig,
) -> SegmentedCapsuleFormatting {
    let container_classes = if config.is_dark_glass {
        CANONICAL_CAPSULE_CONTAINER_CLASSES.to_string()
    } else {
        "inline-flex items-center rounded-full border border-zinc-300 bg-zinc-100 p-0.5 shadow-sm"
            .to_string()
    };

    let divider_classes = if config.has_dividers {
        CANONICAL_CAPSULE_DIVIDER_CLASSES.to_string()
    } else {
        "hidden".to_string()
    };

    let size_padding = match config.size_tier.as_str() {
        "sm" => "px-2 py-0.5 text-xs",
        "lg" => "px-4 py-1.5 text-base",
        _ => "px-3 py-1 text-sm",
    };

    let total = options.len();
    let mut segment_classes = Vec::with_capacity(total);
    let mut markup_buttons = Vec::with_capacity(total);

    for (i, opt) in options.iter().enumerate() {
        let rounding = if total <= 1 {
            "rounded-full"
        } else if i == 0 {
            "rounded-l-full"
        } else if i == total - 1 {
            "rounded-r-full"
        } else {
            "rounded-none"
        };

        let state_classes = if opt.is_disabled {
            "opacity-40 cursor-not-allowed text-zinc-500"
        } else if opt.is_selected {
            if config.is_dark_glass {
                "bg-zinc-800/90 text-zinc-100 shadow-sm font-medium"
            } else {
                "bg-white text-zinc-900 shadow-sm font-medium"
            }
        } else if config.is_dark_glass {
            "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/40 cursor-pointer"
        } else {
            "text-zinc-600 hover:text-zinc-900 hover:bg-zinc-200/60 cursor-pointer"
        };

        let classes = format!(
            "relative inline-flex items-center justify-center font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-zinc-400 {} {} {}",
            size_padding, rounding, state_classes
        );

        let disabled_attr = if opt.is_disabled { " disabled" } else { "" };
        let aria_checked = if opt.is_selected { "true" } else { "false" };

        let btn_html = format!(
            "<button id=\"{}\" class=\"{}\" role=\"radio\" aria-checked=\"{}\" aria-label=\"{}\"{}>{}</button>",
            opt.id, classes, aria_checked, opt.aria_label, disabled_attr, opt.label
        );

        segment_classes.push(classes);
        markup_buttons.push(btn_html);
    }

    let mut markup_body = String::new();
    for (i, btn) in markup_buttons.iter().enumerate() {
        markup_body.push_str(btn);
        if config.has_dividers && i + 1 < total {
            markup_body.push_str(&format!(
                "<div class=\"{}\" aria-hidden=\"true\"></div>",
                divider_classes
            ));
        }
    }

    let formatted_markup = format!(
        "<div class=\"{}\" role=\"radiogroup\">{}</div>",
        container_classes, markup_body
    );

    SegmentedCapsuleFormatting {
        container_classes,
        segment_classes,
        divider_classes,
        formatted_markup,
    }
}

/// Audits HTML or JSX markup for segmented capsule and inline select compliance.
pub fn audit_capsule_markup_compliance(html_or_jsx: &str) -> CapsuleAuditReport {
    let mut findings = Vec::new();
    let mut recommendations = Vec::new();

    // 1. Audit for loose / disjoint circular buttons or window controls
    let has_disjoint_buttons = {
        // Look for buttons styled with individual circular classes outside a segmented capsule
        let has_loose_circular_button = (html_or_jsx.contains("rounded-full w-8 h-8")
            || html_or_jsx.contains("w-8 h-8 rounded-full")
            || html_or_jsx.contains("w-6 h-6 rounded-full")
            || html_or_jsx.contains("rounded-full p-2")
            || html_or_jsx.contains("rounded-full p-1"))
            && !html_or_jsx.contains(CANONICAL_CAPSULE_CONTAINER_CLASSES);

        // Look for window control buttons (minimize, maximize, close) in loose flex containers
        let has_loose_window_controls = (html_or_jsx.contains("title=\"Minimize\"")
            || html_or_jsx.contains("title=\"Maximize\"")
            || html_or_jsx.contains("title=\"Close\"")
            || html_or_jsx.contains("win-min")
            || html_or_jsx.contains("win-max")
            || html_or_jsx.contains("win-close"))
            && !html_or_jsx.contains("rounded-full");

        // Multiple buttons in loose flex container with gap
        let has_loose_flex_button_group = (html_or_jsx.contains("flex gap-2")
            || html_or_jsx.contains("flex gap-1")
            || html_or_jsx.contains("flex space-x-"))
            && html_or_jsx.matches("<button").count() > 1
            && !html_or_jsx.contains("rounded-full");

        has_loose_circular_button || has_loose_window_controls || has_loose_flex_button_group
    };

    if has_disjoint_buttons {
        findings.push(CapsuleAuditFinding::DisjointButtonsDetected);
        recommendations.push(
            "Wrap adjacent toolbar actions or desktop window controls into a contiguous segmented pill capsule (`rounded-full`, shared border, subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons.".to_string(),
        );
    }

    // 2. Audit for segmented container missing `rounded-full`
    let is_segmented_group = html_or_jsx.contains("role=\"radiogroup\"")
        || html_or_jsx.contains("role=\"group\"")
        || html_or_jsx.contains("segmented")
        || html_or_jsx.contains("capsule")
        || (html_or_jsx.contains("inline-flex") && html_or_jsx.contains("border"));

    if is_segmented_group && !html_or_jsx.contains("rounded-full") {
        findings.push(CapsuleAuditFinding::MissingRoundedFull);
        recommendations.push(
            "Apply 'rounded-full' to the segmented capsule container to enforce contiguous pill capsule geometry.".to_string(),
        );
    }

    // 3. Audit for inline `<select>` missing transparent borderless styling
    if html_or_jsx.contains("<select") {
        let mut select_is_compliant = true;

        for mandatory in MANDATORY_INLINE_SELECT_CLASSES {
            if !html_or_jsx.contains(mandatory) {
                select_is_compliant = false;
                break;
            }
        }

        if select_is_compliant {
            for forbidden in FORBIDDEN_SELECT_CLASSES {
                if html_or_jsx.contains(forbidden) {
                    select_is_compliant = false;
                    break;
                }
            }
        }

        if !select_is_compliant {
            findings.push(CapsuleAuditFinding::BorderlessSelectMissing);
            recommendations.push(
                "Apply transparent borderless styling ('bg-transparent border-0 focus:ring-0 focus:outline-none') to inline <select> dropdowns mounted within segmented capsules to eliminate nested white pill-in-pill visual defects.".to_string(),
            );
        }
    }

    if findings.is_empty() {
        findings.push(CapsuleAuditFinding::Compliant);
        recommendations.push(
            "Segmented capsule markup complies with AGENTS.md Minimalist & Contextual UI guidelines."
                .to_string(),
        );
    }

    let is_compliant = findings
        .iter()
        .all(|f| matches!(f, CapsuleAuditFinding::Compliant));

    CapsuleAuditReport::new(is_compliant, findings, recommendations)
}
