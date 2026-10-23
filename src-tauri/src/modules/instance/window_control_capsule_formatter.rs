//! Window Control Capsule Formatter (`Imp-686`).
//!
//! Enforces AGENTS.md Minimalist & Contextual UI invariant:
//! "When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close),
//! always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border,
//! subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons."
//!
//! Formats desktop window action controls into contiguous, dark-glass segmented pill capsules
//! with responsive maximize/restore iconography and accessible keyboard trigger metadata.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard dark-glass capsule container CSS classes for window controls.
pub const CANONICAL_CAPSULE_CONTAINER_CLASSES: &str =
    "inline-flex items-center rounded-full border border-zinc-800 bg-zinc-950/80 p-0.5 backdrop-blur-md shadow-sm";

/// Standard button segment CSS classes inside the capsule.
pub const CANONICAL_CAPSULE_BUTTON_CLASSES: &str =
    "relative flex h-6 w-7 items-center justify-center text-xs text-zinc-400 transition-colors hover:text-zinc-100 hover:bg-zinc-800/60 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-zinc-400";

/// Standard divider CSS classes between adjacent capsule buttons.
pub const CANONICAL_CAPSULE_DIVIDER_CLASSES: &str = "h-3.5 w-px bg-zinc-800/80";

/// Individual window action represented by a capsule segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WindowControlAction {
    /// Minimize application window to system taskbar.
    Minimize,
    /// Toggle window between maximized and restored floating state.
    ToggleMaximize,
    /// Terminate / close application window.
    Close,
}

impl fmt::Display for WindowControlAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Minimize => write!(f, "Minimize"),
            Self::ToggleMaximize => write!(f, "ToggleMaximize"),
            Self::Close => write!(f, "Close"),
        }
    }
}

/// A structured segment within the window control capsule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleControlSegment {
    /// Action triggered on click.
    pub action: WindowControlAction,
    /// Accessibility text label.
    pub aria_label: String,
    /// Tooltip text.
    pub tooltip: String,
    /// Glyph or SVG path identifier.
    pub icon_glyph: String,
    /// Whether this is the terminal right-hand button (e.g. close).
    pub is_destructive: bool,
}

/// Complete formatted window control capsule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowControlCapsule {
    /// True if window is currently maximized (showing restore icon).
    pub is_maximized: bool,
    /// Container CSS classes.
    pub container_classes: String,
    /// Structured button segments.
    pub segments: Vec<CapsuleControlSegment>,
    /// Pre-rendered JSX/HTML template.
    pub rendered_markup: String,
}

/// Formats the contiguous 3-button window control capsule.
pub fn format_window_control_capsule(is_maximized: bool) -> WindowControlCapsule {
    let maximize_tooltip = if is_maximized {
        "Restore Down"
    } else {
        "Maximize"
    };

    let maximize_icon = if is_maximized {
        "restore"
    } else {
        "maximize"
    };

    let segments = vec![
        CapsuleControlSegment {
            action: WindowControlAction::Minimize,
            aria_label: "Minimize window".to_string(),
            tooltip: "Minimize".to_string(),
            icon_glyph: "minus".to_string(),
            is_destructive: false,
        },
        CapsuleControlSegment {
            action: WindowControlAction::ToggleMaximize,
            aria_label: format!("{maximize_tooltip} window"),
            tooltip: maximize_tooltip.to_string(),
            icon_glyph: maximize_icon.to_string(),
            is_destructive: false,
        },
        CapsuleControlSegment {
            action: WindowControlAction::Close,
            aria_label: "Close window".to_string(),
            tooltip: "Close".to_string(),
            icon_glyph: "x".to_string(),
            is_destructive: true,
        },
    ];

    let rendered_markup = format!(
        "<div role=\"toolbar\" aria-label=\"Window controls\" className=\"{CANONICAL_CAPSULE_CONTAINER_CLASSES}\">\
         <button aria-label=\"Minimize window\" title=\"Minimize\" className=\"{CANONICAL_CAPSULE_BUTTON_CLASSES} rounded-l-full\"><span className=\"icon-minus\" /></button>\
         <div className=\"{CANONICAL_CAPSULE_DIVIDER_CLASSES}\" />\
         <button aria-label=\"{maximize_tooltip} window\" title=\"{maximize_tooltip}\" className=\"{CANONICAL_CAPSULE_BUTTON_CLASSES}\"><span className=\"icon-{maximize_icon}\" /></button>\
         <div className=\"{CANONICAL_CAPSULE_DIVIDER_CLASSES}\" />\
         <button aria-label=\"Close window\" title=\"Close\" className=\"{CANONICAL_CAPSULE_BUTTON_CLASSES} rounded-r-full hover:!bg-red-500/20 hover:!text-red-300\"><span className=\"icon-x\" /></button>\
         </div>"
    );

    WindowControlCapsule {
        is_maximized,
        container_classes: CANONICAL_CAPSULE_CONTAINER_CLASSES.to_string(),
        segments,
        rendered_markup,
    }
}

/// Validates whether a container class string satisfies contiguous capsule styling.
pub fn validate_capsule_classes(classes: &str) -> bool {
    classes.contains("rounded-full") && classes.contains("border")
}
