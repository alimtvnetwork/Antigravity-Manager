//! Unit tests for Capsule Segmented Button Group Styler (`Imp-696`).

use super::capsule_segmented_button_group_styler::*;

#[test]
fn test_format_window_control_segmented_capsule() {
    let buttons = vec![
        SegmentButton::new("btn-min", "", "Minimize window")
            .with_icon("minus"),
        SegmentButton::new("btn-max", "", "Maximize window")
            .with_icon("maximize"),
        SegmentButton::new("btn-close", "", "Close window")
            .with_icon("x"),
    ];

    let config = CapsuleGroupConfig::horizontal_dark_glass("sm");
    let result = format_segmented_button_group(&buttons, &config);

    assert!(result.container_classes.contains("rounded-full"));
    assert!(result.container_classes.contains("border-zinc-800"));
    assert!(result.container_classes.contains("bg-zinc-950/80"));
    assert!(result.container_classes.contains("backdrop-blur-md"));

    assert_eq!(result.divider_classes, CANONICAL_SEGMENT_DIVIDER_CLASSES);
    assert_eq!(result.button_classes.len(), 3);

    // Assert corner rounding on contiguous horizontal segments
    assert!(result.button_classes[0].contains("rounded-l-full"));
    assert!(!result.button_classes[1].contains("rounded-l-full"));
    assert!(!result.button_classes[1].contains("rounded-r-full"));
    assert!(result.button_classes[2].contains("rounded-r-full"));

    // Assert HTML output contains all buttons and dividers
    assert!(result.formatted_html.contains("id=\"btn-min\""));
    assert!(result.formatted_html.contains("id=\"btn-max\""));
    assert!(result.formatted_html.contains("id=\"btn-close\""));
    assert_eq!(result.formatted_html.matches("role=\"separator\"").count(), 2);
}

#[test]
fn test_format_html_aria_roles_and_borderless_styling() {
    let buttons = vec![
        SegmentButton::new("opt-1", "Day", "Select day view").with_active(true),
        SegmentButton::new("opt-2", "Week", "Select week view"),
    ];

    let config = CapsuleGroupConfig::default();
    let result = format_segmented_button_group(&buttons, &config);

    assert!(result.formatted_html.contains("role=\"group\""));
    assert!(result.formatted_html.contains("aria-label=\"Segmented button group\""));
    assert!(result.formatted_html.contains("aria-pressed=\"true\""));
    assert!(result.formatted_html.contains("aria-pressed=\"false\""));
}

#[test]
fn test_format_vertical_segmented_capsule() {
    let buttons = vec![
        SegmentButton::new("up", "Up", "Move up"),
        SegmentButton::new("down", "Down", "Move down"),
    ];

    let config = CapsuleGroupConfig::vertical_dark_glass("md");
    let result = format_segmented_button_group(&buttons, &config);

    assert!(result.container_classes.contains("flex-col"));
    assert!(result.button_classes[0].contains("rounded-t-full"));
    assert!(result.button_classes[1].contains("rounded-b-full"));
    assert_eq!(result.divider_classes, CANONICAL_SEGMENT_DIVIDER_VERTICAL_CLASSES);
}

#[test]
fn test_audit_flags_disjoint_circular_buttons() {
    let markup = r#"
        <div class="flex gap-2">
            <button class="rounded-full p-2 bg-zinc-800 text-white" aria-label="Minimize"></button>
            <button class="rounded-full p-2 bg-zinc-800 text-white" aria-label="Maximize"></button>
            <button class="rounded-full p-2 bg-zinc-800 text-white" aria-label="Close"></button>
        </div>
    "#;

    let report = audit_button_group_markup(markup);
    assert!(!report.is_compliant);
    assert!(report.has_violations());
    assert!(report
        .violations
        .iter()
        .any(|v| v.contains("Loose disjoint circular button detected")));
}

#[test]
fn test_audit_passes_compliant_dark_glass_capsule() {
    let markup = r#"
        <div role="toolbar" aria-label="Window controls" class="inline-flex items-center rounded-full border border-zinc-800 bg-zinc-950/80 p-0.5 backdrop-blur-md">
            <button aria-label="Minimize" class="relative flex h-6 w-7 items-center justify-center rounded-l-full"></button>
            <div class="h-3.5 w-px bg-zinc-800/80" role="separator"></div>
            <button aria-label="Maximize" class="relative flex h-6 w-7 items-center justify-center"></button>
            <div class="h-3.5 w-px bg-zinc-800/80" role="separator"></div>
            <button aria-label="Close" class="relative flex h-6 w-7 items-center justify-center rounded-r-full"></button>
        </div>
    "#;

    let report = audit_button_group_markup(markup);
    assert!(report.is_compliant);
    assert!(!report.has_violations());
}

#[test]
fn test_audit_flags_missing_rounded_full_on_container() {
    let markup = r#"
        <div role="toolbar" class="flex border border-zinc-800 bg-zinc-900 p-1">
            <button class="px-2" aria-label="Item 1"></button>
            <div class="h-3.5 w-px bg-zinc-800"></div>
            <button class="px-2" aria-label="Item 2"></button>
        </div>
    "#;

    let report = audit_button_group_markup(markup);
    assert!(!report.is_compliant);
    assert!(report
        .violations
        .iter()
        .any(|v| v.contains("missing mandatory 'rounded-full' pill styling")));
}

#[test]
fn test_audit_flags_missing_dividers() {
    let markup = r#"
        <div class="inline-flex items-center rounded-full border border-zinc-800 bg-zinc-950/80 p-0.5">
            <button aria-label="One"></button>
            <button aria-label="Two"></button>
        </div>
    "#;

    let report = audit_button_group_markup(markup);
    assert!(!report.is_compliant);
    assert!(report
        .violations
        .iter()
        .any(|v| v.contains("missing subtle divider elements between segments")));
}

#[test]
fn test_audit_flags_non_transparent_inline_select() {
    let markup = r#"
        <div class="inline-flex items-center rounded-full border border-zinc-800 bg-zinc-950/80 p-0.5">
            <button aria-label="Action"></button>
            <div class="h-3.5 w-px bg-zinc-800" role="separator"></div>
            <select class="bg-white border rounded text-xs">
                <option value="1">Model 1</option>
            </select>
        </div>
    "#;

    let report = audit_button_group_markup(markup);
    assert!(!report.is_compliant);
    assert!(report
        .violations
        .iter()
        .any(|v| v.contains("missing transparent borderless styling")));
}

#[test]
fn test_audit_passes_transparent_borderless_inline_select() {
    let markup = r#"
        <div class="inline-flex items-center rounded-full border border-zinc-800 bg-zinc-950/80 p-0.5">
            <button aria-label="Action"></button>
            <div class="h-3.5 w-px bg-zinc-800" role="separator"></div>
            <select class="bg-transparent border-0 focus:ring-0 focus:outline-none text-xs">
                <option value="1">Model 1</option>
            </select>
        </div>
    "#;

    let report = audit_button_group_markup(markup);
    assert!(report.is_compliant);
}

#[test]
fn test_legacy_validate_segmented_group_classes() {
    let report = validate_segmented_group_classes(CANONICAL_BUTTON_GROUP_CONTAINER_CLASSES);
    assert!(report.is_compliant);
    assert!(report.has_rounded_full);
    assert!(report.has_shared_border);
    assert!(report.has_subtle_divider);
    assert!(report.has_dark_glass_bg);
}

#[test]
fn test_legacy_format_segmented_group_html() {
    let items = vec![
        SegmentedGroupItem::new("item-1", "Tab 1").with_active(true),
        SegmentedGroupItem::new("item-2", "Tab 2"),
    ];
    let html = format_segmented_group_html(&items);
    assert!(html.contains("id=\"item-1\""));
    assert!(html.contains("id=\"item-2\""));
    assert!(html.contains("role=\"separator\""));
}
