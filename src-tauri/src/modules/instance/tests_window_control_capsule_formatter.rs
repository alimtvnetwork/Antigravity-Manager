//! Unit tests for Window Control Capsule Formatter (`Imp-686`).

use super::window_control_capsule_formatter::*;

#[test]
fn test_format_capsule_normal_floating_state() {
    let capsule = format_window_control_capsule(false);
    assert!(!capsule.is_maximized);
    assert_eq!(capsule.segments.len(), 3);

    // Segment 1: Minimize
    assert_eq!(capsule.segments[0].action, WindowControlAction::Minimize);
    assert_eq!(capsule.segments[0].tooltip, "Minimize");

    // Segment 2: Maximize
    assert_eq!(capsule.segments[1].action, WindowControlAction::ToggleMaximize);
    assert_eq!(capsule.segments[1].tooltip, "Maximize");
    assert_eq!(capsule.segments[1].icon_glyph, "maximize");

    // Segment 3: Close
    assert_eq!(capsule.segments[2].action, WindowControlAction::Close);
    assert!(capsule.segments[2].is_destructive);

    assert!(capsule.rendered_markup.contains("rounded-full"));
    assert!(capsule.rendered_markup.contains("role=\"toolbar\""));
}

#[test]
fn test_format_capsule_maximized_state() {
    let capsule = format_window_control_capsule(true);
    assert!(capsule.is_maximized);

    // Segment 2 should switch to Restore Down
    assert_eq!(capsule.segments[1].tooltip, "Restore Down");
    assert_eq!(capsule.segments[1].icon_glyph, "restore");
    assert!(capsule.rendered_markup.contains("Restore Down"));
}

#[test]
fn test_validate_capsule_classes() {
    assert!(validate_capsule_classes(CANONICAL_CAPSULE_CONTAINER_CLASSES));
    assert!(validate_capsule_classes("rounded-full border border-zinc-800"));
    assert!(!validate_capsule_classes("rounded-md border")); // Missing rounded-full
    assert!(!validate_capsule_classes("rounded-full bg-zinc-900")); // Missing border
}
