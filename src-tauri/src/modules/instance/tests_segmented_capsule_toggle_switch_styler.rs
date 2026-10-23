//! Unit tests for Segmented Capsule Toggle Switch Styler & Linter (`Imp-711`).

use super::segmented_capsule_toggle_switch_styler::*;

#[test]
fn test_default_capsule_style_config() {
    let config = CapsuleStyleConfig::default();
    assert_eq!(config.size_tier, "md");
    assert!(config.is_dark_glass);
    assert!(config.has_dividers);
}

#[test]
fn test_format_segmented_capsule_toggle_three_options() {
    let options = vec![
        CapsuleSegmentOption::new("tab-chat", "Chat").with_selected(true),
        CapsuleSegmentOption::new("tab-code", "Code"),
        CapsuleSegmentOption::new("tab-docs", "Docs"),
    ];
    let config = CapsuleStyleConfig::dark_glass("sm");

    let formatting = format_segmented_capsule_toggle(&options, &config);

    // Verify container styling satisfies AGENTS.md dark-glass capsule rules
    assert!(formatting.container_classes.contains("rounded-full"));
    assert!(formatting.container_classes.contains("border-zinc-800"));
    assert!(formatting.container_classes.contains("bg-zinc-950/80"));
    assert!(formatting.container_classes.contains("backdrop-blur-md"));

    // Verify segment classes and boundary rounding
    assert_eq!(formatting.segment_classes.len(), 3);
    assert!(formatting.segment_classes[0].contains("rounded-l-full"));
    assert!(formatting.segment_classes[1].contains("rounded-none"));
    assert!(formatting.segment_classes[2].contains("rounded-r-full"));

    // Verify selected option has active styles
    assert!(formatting.segment_classes[0].contains("bg-zinc-800/90"));

    // Verify markup structure
    assert!(formatting.formatted_markup.contains("role=\"radiogroup\""));
    assert!(formatting.formatted_markup.contains("id=\"tab-chat\""));
    assert!(formatting.formatted_markup.contains("aria-checked=\"true\""));
    assert!(formatting.formatted_markup.contains("id=\"tab-code\""));
    assert!(formatting.formatted_markup.contains("aria-checked=\"false\""));
    assert!(formatting.formatted_markup.contains("id=\"tab-docs\""));

    // Verify 2 dividers between 3 options
    assert_eq!(
        formatting
            .formatted_markup
            .matches("aria-hidden=\"true\"")
            .count(),
        2
    );
}

#[test]
fn test_format_segmented_capsule_toggle_single_option() {
    let options = vec![CapsuleSegmentOption::new("tab-solo", "Solo View").with_selected(true)];
    let config = CapsuleStyleConfig::dark_glass("md");

    let formatting = format_segmented_capsule_toggle(&options, &config);

    assert_eq!(formatting.segment_classes.len(), 1);
    assert!(formatting.segment_classes[0].contains("rounded-full"));
    assert!(!formatting.formatted_markup.contains("aria-hidden=\"true\""));
}

#[test]
fn test_format_segmented_capsule_disabled_and_selected_states() {
    let options = vec![
        CapsuleSegmentOption::new("opt-active", "Active").with_selected(true),
        CapsuleSegmentOption::new("opt-disabled", "Locked").with_disabled(true),
    ];
    let config = CapsuleStyleConfig::default();

    let formatting = format_segmented_capsule_toggle(&options, &config);

    assert!(formatting.segment_classes[1].contains("opacity-40"));
    assert!(formatting.segment_classes[1].contains("cursor-not-allowed"));
    assert!(formatting.formatted_markup.contains("disabled"));
}

#[test]
fn test_format_segmented_capsule_minimal_without_dividers() {
    let options = vec![
        CapsuleSegmentOption::new("view-a", "A"),
        CapsuleSegmentOption::new("view-b", "B"),
    ];
    let config = CapsuleStyleConfig::minimal("sm");

    let formatting = format_segmented_capsule_toggle(&options, &config);

    assert_eq!(formatting.divider_classes, "hidden");
    assert!(!formatting.formatted_markup.contains("aria-hidden=\"true\""));
}

#[test]
fn test_audit_compliant_markup() {
    let options = vec![
        CapsuleSegmentOption::new("view-1", "One").with_selected(true),
        CapsuleSegmentOption::new("view-2", "Two"),
    ];
    let config = CapsuleStyleConfig::dark_glass("md");
    let formatting = format_segmented_capsule_toggle(&options, &config);

    let report = audit_capsule_markup_compliance(&formatting.formatted_markup);

    assert!(report.is_compliant());
    assert!(report.has_finding(&CapsuleAuditFinding::Compliant));
    assert_eq!(report.findings.len(), 1);
}

#[test]
fn test_audit_compliant_markup_with_borderless_select() {
    let markup = r#"
        <div class="inline-flex items-center rounded-full border border-zinc-800 bg-zinc-950/80 p-0.5" role="radiogroup">
            <button class="px-3 py-1 rounded-l-full text-zinc-100">Option 1</button>
            <select class="bg-transparent border-0 focus:ring-0 focus:outline-none text-xs text-zinc-300">
                <option value="1">Sub-option 1</option>
            </select>
        </div>
    "#;

    let report = audit_capsule_markup_compliance(markup);

    assert!(report.is_compliant());
    assert!(report.has_finding(&CapsuleAuditFinding::Compliant));
}

#[test]
fn test_audit_disjoint_buttons_detected() {
    let disjoint_markup = r#"
        <div class="flex gap-2">
            <button class="w-8 h-8 rounded-full bg-zinc-800 text-zinc-100">1</button>
            <button class="w-8 h-8 rounded-full bg-zinc-800 text-zinc-100">2</button>
        </div>
    "#;

    let report = audit_capsule_markup_compliance(disjoint_markup);

    assert!(!report.is_compliant());
    assert!(report.has_finding(&CapsuleAuditFinding::DisjointButtonsDetected));
}

#[test]
fn test_audit_missing_rounded_full() {
    let non_pill_markup = r#"
        <div class="inline-flex border border-zinc-800 rounded-md bg-zinc-950 p-1" role="radiogroup">
            <button class="px-2 py-1 text-sm">Mode A</button>
            <button class="px-2 py-1 text-sm">Mode B</button>
        </div>
    "#;

    let report = audit_capsule_markup_compliance(non_pill_markup);

    assert!(!report.is_compliant());
    assert!(report.has_finding(&CapsuleAuditFinding::MissingRoundedFull));
}

#[test]
fn test_audit_borderless_select_missing() {
    let nested_select_markup = r#"
        <div class="inline-flex items-center rounded-full border border-zinc-800 bg-zinc-950/80 p-0.5" role="radiogroup">
            <button class="px-3 py-1 rounded-l-full">Action</button>
            <select class="bg-white border rounded-md text-xs text-zinc-900">
                <option value="item">Nested Pill Defect</option>
            </select>
        </div>
    "#;

    let report = audit_capsule_markup_compliance(nested_select_markup);

    assert!(!report.is_compliant());
    assert!(report.has_finding(&CapsuleAuditFinding::BorderlessSelectMissing));
}

#[test]
fn test_audit_multiple_violations() {
    let multi_violation_markup = r#"
        <div class="inline-flex border border-zinc-800 rounded-lg p-1" role="radiogroup">
            <select class="bg-zinc-800 border rounded-full text-xs">
                <option>Invalid</option>
            </select>
        </div>
    "#;

    let report = audit_capsule_markup_compliance(multi_violation_markup);

    assert!(!report.is_compliant());
    assert!(report.has_finding(&CapsuleAuditFinding::MissingRoundedFull));
    assert!(report.has_finding(&CapsuleAuditFinding::BorderlessSelectMissing));
    assert_eq!(report.findings.len(), 2);
    assert_eq!(report.recommendations.len(), 2);
}

#[test]
fn test_finding_and_report_display() {
    let finding = CapsuleAuditFinding::DisjointButtonsDetected;
    assert!(format!("{finding}").contains("loose circular buttons detected"));

    let report = CapsuleAuditReport::new(
        false,
        vec![CapsuleAuditFinding::MissingRoundedFull],
        vec!["Apply 'rounded-full'".to_string()],
    );
    assert!(format!("{report}").contains("Non-compliant: 1 styling finding(s) detected"));
}
