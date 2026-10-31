//! Capsule Segmented Control Validator (`Imp-262`).
//!
//! Enforces AGENTS.md Minimalist & Contextual UI invariants:
//! "When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close),
//! always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle
//! divider lines, and dark-glass styling) rather than loose, disjointed circular buttons. Any
//! inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless
//! styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white
//! pill-in-pill visual defects."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Violations of capsule styling and segmented control invariants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapsuleStylingViolation {
    DisjointWindowControlsWithoutCapsule { line: usize, snippet: String },
    NestedSelectPillInPillDefect { line: usize, tag: String },
    MissingContiguousRoundedFull { line: usize, container: String },
}

impl CapsuleStylingViolation {
    /// Returns the line number where the violation occurred.
    pub fn line(&self) -> usize {
        match self {
            Self::DisjointWindowControlsWithoutCapsule { line, .. } => *line,
            Self::NestedSelectPillInPillDefect { line, .. } => *line,
            Self::MissingContiguousRoundedFull { line, .. } => *line,
        }
    }

    /// Returns true if this violation represents disjoint window controls.
    pub fn is_disjoint_controls(&self) -> bool {
        matches!(self, Self::DisjointWindowControlsWithoutCapsule { .. })
    }

    /// Returns true if this violation represents a nested select styling defect.
    pub fn is_nested_select_defect(&self) -> bool {
        matches!(self, Self::NestedSelectPillInPillDefect { .. })
    }

    /// Returns true if this violation represents a container missing rounded-full.
    pub fn is_missing_rounded_full(&self) -> bool {
        matches!(self, Self::MissingContiguousRoundedFull { .. })
    }
}

impl fmt::Display for CapsuleStylingViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DisjointWindowControlsWithoutCapsule { line, snippet } => {
                write!(
                    f,
                    "Line {}: Disjoint window control without capsule container: '{}'",
                    line, snippet
                )
            }
            Self::NestedSelectPillInPillDefect { line, tag } => {
                write!(
                    f,
                    "Line {}: Nested select pill-in-pill defect (missing bg-transparent border-0): '{}'",
                    line, tag
                )
            }
            Self::MissingContiguousRoundedFull { line, container } => {
                write!(
                    f,
                    "Line {}: Missing contiguous rounded-full on container: '{}'",
                    line, container
                )
            }
        }
    }
}

impl std::error::Error for CapsuleStylingViolation {}

/// Summary report produced by component markup validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleValidationReport {
    pub is_compliant: bool,
    pub capsules_checked: usize,
    pub violations: Vec<CapsuleStylingViolation>,
}

impl CapsuleValidationReport {
    /// Constructs a new capsule validation report.
    pub fn new(capsules_checked: usize, violations: Vec<CapsuleStylingViolation>) -> Self {
        let is_compliant = violations.is_empty();
        Self {
            is_compliant,
            capsules_checked,
            violations,
        }
    }

    /// Returns true if the markup is fully compliant with capsule styling invariants.
    pub fn is_compliant(&self) -> bool {
        self.is_compliant
    }

    /// Returns true if one or more styling violations were detected.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }
}

/// Returns true if the line declares a container element with rounded-full styling.
fn is_capsule_container_declaration(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("rounded-full")
        && (lower.contains("<div")
            || lower.contains("<nav")
            || lower.contains("<header")
            || lower.contains("<section")
            || lower.contains("classname")
            || lower.contains("class="))
}

/// Returns true if the line declares a controls/toolbar container without rounded-full.
fn is_control_container_missing_capsule(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let has_container_tag = lower.contains("<div")
        || lower.contains("<section")
        || lower.contains("<nav")
        || lower.contains("<header");
    let has_control_keyword = lower.contains("window-controls")
        || lower.contains("window_controls")
        || lower.contains("windowcontrols")
        || lower.contains("toolbar-actions")
        || lower.contains("toolbar_actions")
        || lower.contains("segmented-control")
        || lower.contains("segmented_control");
    has_container_tag && has_control_keyword && !lower.contains("rounded-full")
}

/// Returns true if the line represents a window control button or action.
fn is_window_control_element(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    if lower.contains("appwindow.minimize")
        || lower.contains("appwindow.maximize")
        || lower.contains("appwindow.close")
        || lower.contains("appwindow.togglemaximize")
    {
        return true;
    }
    if lower.contains("title=\"minimize\"")
        || lower.contains("title=\"maximize\"")
        || lower.contains("title=\"close\"")
        || lower.contains("aria-label=\"minimize\"")
        || lower.contains("aria-label=\"maximize\"")
        || lower.contains("aria-label=\"close\"")
    {
        return true;
    }
    lower.contains("<button")
        && (lower.contains("minimize") || lower.contains("maximize") || lower.contains("close"))
}

/// Returns true if the line contains a select dropdown opening element.
fn is_select_element(line: &str) -> bool {
    line.contains("<select") || line.contains("<Select")
}

/// Returns true if the select element features transparent borderless styling.
fn is_transparent_borderless_select(line: &str) -> bool {
    line.contains("bg-transparent") && line.contains("border-0")
}

/// Verifies that toolbar/window control containers declare contiguous rounded-full capsules.
fn check_container_compliance(
    line: &str,
    line_num: usize,
    violations: &mut Vec<CapsuleStylingViolation>,
) {
    if is_control_container_missing_capsule(line) {
        violations.push(CapsuleStylingViolation::MissingContiguousRoundedFull {
            line: line_num,
            container: line.to_string(),
        });
    }
}

/// Verifies that window control buttons are enclosed within a capsule container.
fn check_window_control_compliance(
    line: &str,
    line_num: usize,
    capsule_depth: usize,
    violations: &mut Vec<CapsuleStylingViolation>,
) {
    if is_window_control_element(line) && capsule_depth == 0 {
        violations.push(
            CapsuleStylingViolation::DisjointWindowControlsWithoutCapsule {
                line: line_num,
                snippet: line.to_string(),
            },
        );
    }
}

/// Verifies that select dropdowns within capsules use transparent borderless styling.
fn check_select_dropdown_compliance(
    line: &str,
    line_num: usize,
    capsule_depth: usize,
    violations: &mut Vec<CapsuleStylingViolation>,
) {
    if is_select_element(line) && capsule_depth > 0 && !is_transparent_borderless_select(line) {
        violations.push(CapsuleStylingViolation::NestedSelectPillInPillDefect {
            line: line_num,
            tag: line.to_string(),
        });
    }
}

/// Inspects an individual markup line for capsule declarations and styling defects.
fn inspect_markup_line(
    line: &str,
    line_num: usize,
    inside_capsule_depth: &mut usize,
    capsules_checked: &mut usize,
    violations: &mut Vec<CapsuleStylingViolation>,
) {
    if is_capsule_container_declaration(line) {
        *capsules_checked += 1;
        *inside_capsule_depth += 1;
    }

    check_container_compliance(line, line_num, violations);
    check_window_control_compliance(line, line_num, *inside_capsule_depth, violations);
    check_select_dropdown_compliance(line, line_num, *inside_capsule_depth, violations);

    if line.contains("</div>") || line.contains("</section>") || line.contains("</nav>") {
        *inside_capsule_depth = inside_capsule_depth.saturating_sub(1);
    }
}

/// Validates markup lines against AGENTS.md capsule and segmented control standards.
pub fn validate_component_markup(markup: &str) -> CapsuleValidationReport {
    let mut violations = Vec::new();
    let mut capsules_checked = 0;
    let mut inside_capsule_depth: usize = 0;

    for (idx, raw_line) in markup.lines().enumerate() {
        let line_num = idx + 1;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        inspect_markup_line(
            line,
            line_num,
            &mut inside_capsule_depth,
            &mut capsules_checked,
            &mut violations,
        );
    }

    CapsuleValidationReport::new(capsules_checked, violations)
}
