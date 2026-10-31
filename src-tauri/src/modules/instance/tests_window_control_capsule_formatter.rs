//! Unit tests for Window Control Capsule Formatter (`Imp-686`).

use super::window_control_capsule_formatter::*;

#[test]
fn test_format_window_control_capsule_normal_state() {
    let config = WindowCapsuleConfig::default();
    let formatting = format_window_control_capsule(&config);

    assert_eq!(formatting.segment_count(), 3);
    assert!(!formatting.is_empty());
    assert_eq!(
        formatting.container_classes,
        CANONICAL_CAPSULE_CONTAINER_CLASSES
    );
    assert_eq!(
        formatting.divider_classes,
        CANONICAL_CAPSULE_DIVIDER_CLASSES
    );

    // Segment 1: Minimize
    assert_eq!(formatting.segments[0].action, WindowControlAction::Minimize);
    assert_eq!(formatting.segments[0].label, "Minimize");
    assert_eq!(formatting.segments[0].icon_name, "minus");
    assert_eq!(formatting.segments[0].aria_label, "Minimize window");
    assert!(!formatting.segments[0].is_destructive());

    // Segment 2: Maximize
    assert_eq!(formatting.segments[1].action, WindowControlAction::Maximize);
    assert_eq!(formatting.segments[1].label, "Maximize");
    assert_eq!(formatting.segments[1].icon_name, "maximize");
    assert_eq!(formatting.segments[1].aria_label, "Maximize window");
    assert!(!formatting.segments[1].is_destructive());

    // Segment 3: Close
    assert_eq!(formatting.segments[2].action, WindowControlAction::Close);
    assert_eq!(formatting.segments[2].label, "Close");
    assert_eq!(formatting.segments[2].icon_name, "x");
    assert_eq!(formatting.segments[2].aria_label, "Close window");
    assert!(formatting.segments[2].is_destructive());

    assert!(formatting.has_action(WindowControlAction::Minimize));
    assert!(formatting.has_action(WindowControlAction::Maximize));
    assert!(!formatting.has_action(WindowControlAction::Restore));
    assert!(formatting.has_action(WindowControlAction::Close));
}

#[test]
fn test_format_window_control_capsule_maximized_state() {
    let config = WindowCapsuleConfig::new(true);
    let formatting = format_window_control_capsule(&config);

    assert_eq!(formatting.segment_count(), 3);

    // Segment 2 should switch from Maximize to Restore
    assert_eq!(formatting.segments[1].action, WindowControlAction::Restore);
    assert_eq!(formatting.segments[1].label, "Restore");
    assert_eq!(formatting.segments[1].icon_name, "restore");
    assert_eq!(formatting.segments[1].aria_label, "Restore window");

    assert!(formatting.has_action(WindowControlAction::Restore));
    assert!(!formatting.has_action(WindowControlAction::Maximize));
}

#[test]
fn test_format_window_control_capsule_from_window_state() {
    let config_normal = WindowCapsuleConfig::from_state(WindowState::Normal);
    assert!(!config_normal.is_maximized);
    let fmt_normal = format_window_control_capsule(&config_normal);
    assert!(fmt_normal.has_action(WindowControlAction::Maximize));

    let config_max = WindowCapsuleConfig::from_state(WindowState::Maximized);
    assert!(config_max.is_maximized);
    let fmt_max = format_window_control_capsule(&config_max);
    assert!(fmt_max.has_action(WindowControlAction::Restore));

    let config_min = WindowCapsuleConfig::from_state(WindowState::Minimized);
    assert!(!config_min.is_maximized);
    let fmt_min = format_window_control_capsule(&config_min);
    assert!(fmt_min.has_action(WindowControlAction::Maximize));
}

#[test]
fn test_format_window_control_capsule_selective_toggles() {
    let config_no_min = WindowCapsuleConfig {
        is_maximized: false,
        can_minimize: false,
        can_maximize: true,
        can_close: true,
    };
    let fmt_no_min = format_window_control_capsule(&config_no_min);
    assert_eq!(fmt_no_min.segment_count(), 2);
    assert!(!fmt_no_min.has_action(WindowControlAction::Minimize));
    assert!(fmt_no_min.has_action(WindowControlAction::Maximize));
    assert!(fmt_no_min.has_action(WindowControlAction::Close));

    let config_no_close = WindowCapsuleConfig {
        is_maximized: false,
        can_minimize: true,
        can_maximize: true,
        can_close: false,
    };
    let fmt_no_close = format_window_control_capsule(&config_no_close);
    assert_eq!(fmt_no_close.segment_count(), 2);
    assert!(!fmt_no_close.has_action(WindowControlAction::Close));

    let config_empty = WindowCapsuleConfig {
        is_maximized: false,
        can_minimize: false,
        can_maximize: false,
        can_close: false,
    };
    let fmt_empty = format_window_control_capsule(&config_empty);
    assert_eq!(fmt_empty.segment_count(), 0);
    assert!(fmt_empty.is_empty());
}

#[test]
fn test_validate_capsule_classes() {
    assert!(validate_capsule_classes(
        CANONICAL_CAPSULE_CONTAINER_CLASSES
    ));
    assert!(validate_capsule_classes(
        "rounded-full border border-zinc-800"
    ));
    assert!(!validate_capsule_classes("rounded-md border"));
    assert!(!validate_capsule_classes("rounded-full bg-zinc-900"));
}

#[test]
fn test_render_markup() {
    let config = WindowCapsuleConfig::default();
    let formatting = format_window_control_capsule(&config);
    let markup = formatting.render_markup();

    assert!(markup.starts_with("<div role=\"toolbar\" aria-label=\"Window controls\""));
    assert!(markup.contains("rounded-full border border-zinc-800"));
    assert!(markup.contains("rounded-l-full"));
    assert!(markup.contains("rounded-r-full"));
    assert!(markup.contains("icon-minus"));
    assert!(markup.contains("icon-maximize"));
    assert!(markup.contains("icon-x"));
    assert!(markup.contains("h-3.5 w-px bg-zinc-800/80"));
    assert!(markup.contains("hover:!bg-red-500/20"));
    assert!(markup.ends_with("</div>"));
}

#[test]
fn test_serde_roundtrip() {
    let config = WindowCapsuleConfig::default();
    let formatting = format_window_control_capsule(&config);

    let serialized = serde_json::to_string(&formatting).expect("serialization failed");
    let deserialized: WindowCapsuleFormatting =
        serde_json::from_str(&serialized).expect("deserialization failed");

    assert_eq!(formatting, deserialized);
}

#[test]
fn test_action_and_state_displays() {
    assert_eq!(format!("{}", WindowControlAction::Minimize), "Minimize");
    assert_eq!(format!("{}", WindowControlAction::Maximize), "Maximize");
    assert_eq!(format!("{}", WindowControlAction::Restore), "Restore");
    assert_eq!(format!("{}", WindowControlAction::Close), "Close");

    assert_eq!(format!("{}", WindowState::Normal), "Normal");
    assert_eq!(format!("{}", WindowState::Maximized), "Maximized");
    assert_eq!(format!("{}", WindowState::Minimized), "Minimized");
}
