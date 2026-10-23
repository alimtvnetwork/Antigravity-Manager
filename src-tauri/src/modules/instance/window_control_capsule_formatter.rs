//! Window Control Capsule Formatter (`Imp-686`).
//!
//! Enforces AGENTS.md Minimalist & Contextual UI invariant:
//! "When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close),
//! always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border,
//! subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons."
//!
//! Formats desktop window control actions into contiguous, dark-glass segmented pill capsules
//! with state-aware Maximize / Restore toggles, accessible ARIA labels, and crisp dividers.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard dark-glass capsule container CSS classes for window controls.
pub const CANONICAL_CAPSULE_CONTAINER_CLASSES: &str =
    "inline-flex items-center rounded-full border border-zinc-800 bg-zinc-950/80 p-0.5 backdrop-blur-md shadow-sm";

/// Standard divider CSS classes between adjacent capsule buttons.
pub const CANONICAL_CAPSULE_DIVIDER_CLASSES: &str = "h-3.5 w-px bg-zinc-800/80";

/// Standard button segment CSS classes inside the capsule.
pub const CANONICAL_CAPSULE_BUTTON_CLASSES: &str =
    "relative flex h-6 w-7 items-center justify-center text-xs text-zinc-400 transition-colors hover:text-zinc-100 hover:bg-zinc-800/60 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-zinc-400";

/// Action performed by an individual window control capsule segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WindowControlAction {
    /// Minimize application window to system taskbar.
    Minimize,
    /// Expand application window to fill display workspace.
    Maximize,
    /// Restore maximized application window to floating dimensions.
    Restore,
    /// Terminate / close application window.
    Close,
}

impl fmt::Display for WindowControlAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Minimize => write!(f, "Minimize"),
            Self::Maximize => write!(f, "Maximize"),
            Self::Restore => write!(f, "Restore"),
            Self::Close => write!(f, "Close"),
        }
    }
}

/// Operational state of the desktop application window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WindowState {
    /// Floating, un-maximized window state.
    Normal,
    /// Window maximized across primary display.
    Maximized,
    /// Window minimized to system taskbar.
    Minimized,
}

impl fmt::Display for WindowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => write!(f, "Normal"),
            Self::Maximized => write!(f, "Maximized"),
            Self::Minimized => write!(f, "Minimized"),
        }
    }
}

impl WindowState {
    /// True if the window is currently in the maximized state.
    pub fn is_maximized(&self) -> bool {
        matches!(self, Self::Maximized)
    }
}

/// A structured segment within the contiguous window control capsule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowControlSegment {
    /// Action triggered on click.
    pub action: WindowControlAction,
    /// Short user-facing label or tooltip.
    pub label: String,
    /// Icon glyph identifier.
    pub icon_name: String,
    /// Full accessibility text label.
    pub aria_label: String,
}

impl WindowControlSegment {
    /// True if this segment triggers a destructive action (closing the window).
    pub fn is_destructive(&self) -> bool {
        self.action == WindowControlAction::Close
    }
}

/// Configuration specifying window state and active capabilities for capsule rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowCapsuleConfig {
    /// True if the window is currently maximized (toggles Maximize -> Restore).
    pub is_maximized: bool,
    /// Whether the minimize action button should be rendered.
    pub can_minimize: bool,
    /// Whether the maximize / restore action button should be rendered.
    pub can_maximize: bool,
    /// Whether the close action button should be rendered.
    pub can_close: bool,
}

impl Default for WindowCapsuleConfig {
    fn default() -> Self {
        Self {
            is_maximized: false,
            can_minimize: true,
            can_maximize: true,
            can_close: true,
        }
    }
}

impl WindowCapsuleConfig {
    /// Creates a standard configuration with all 3 actions enabled.
    pub fn new(is_maximized: bool) -> Self {
        Self {
            is_maximized,
            can_minimize: true,
            can_maximize: true,
            can_close: true,
        }
    }

    /// Derives configuration from a `WindowState` enum instance.
    pub fn from_state(state: WindowState) -> Self {
        Self {
            is_maximized: state.is_maximized(),
            can_minimize: true,
            can_maximize: true,
            can_close: true,
        }
    }
}

/// Complete formatted window control capsule specifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowCapsuleFormatting {
    /// Tailwind / CSS classes applied to the root segmented capsule container.
    pub container_classes: String,
    /// Ordered segments (buttons) rendered inside the capsule.
    pub segments: Vec<WindowControlSegment>,
    /// Tailwind / CSS classes applied to dividers placed between adjacent segments.
    pub divider_classes: String,
}

impl WindowCapsuleFormatting {
    /// Count of segments in the capsule.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// True if no segments are rendered.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Checks if a specific action is present in the capsule segments.
    pub fn has_action(&self, action: WindowControlAction) -> bool {
        self.segments.iter().any(|s| s.action == action)
    }

    /// Generates accessible HTML / JSX markup for the window control capsule toolbar.
    pub fn render_markup(&self) -> String {
        let mut markup = format!(
            "<div role=\"toolbar\" aria-label=\"Window controls\" className=\"{}\">",
            self.container_classes
        );

        let count = self.segments.len();
        for (idx, seg) in self.segments.iter().enumerate() {
            let corner_class = if count == 1 {
                "rounded-full"
            } else if idx == 0 {
                "rounded-l-full"
            } else if idx == count - 1 {
                "rounded-r-full"
            } else {
                ""
            };

            let destructive_class = if seg.is_destructive() {
                " hover:!bg-red-500/20 hover:!text-red-300"
            } else {
                ""
            };

            markup.push_str(&format!(
                "<button aria-label=\"{}\" title=\"{}\" className=\"{} {}{}\"><span className=\"icon-{}\" /></button>",
                seg.aria_label,
                seg.label,
                CANONICAL_CAPSULE_BUTTON_CLASSES,
                corner_class,
                destructive_class,
                seg.icon_name
            ));

            if idx + 1 < count {
                markup.push_str(&format!("<div className=\"{}\" />", self.divider_classes));
            }
        }

        markup.push_str("</div>");
        markup
    }
}

/// Formats desktop window control actions into a contiguous segmented pill capsule.
///
/// Generates 3-segment pill capsule definitions with `rounded-full`, shared dark-glass border,
/// subtle dividers, and state-aware action toggles (Maximize vs Restore).
pub fn format_window_control_capsule(config: &WindowCapsuleConfig) -> WindowCapsuleFormatting {
    let mut segments = Vec::new();

    if config.can_minimize {
        segments.push(WindowControlSegment {
            action: WindowControlAction::Minimize,
            label: "Minimize".to_string(),
            icon_name: "minus".to_string(),
            aria_label: "Minimize window".to_string(),
        });
    }

    if config.can_maximize {
        if config.is_maximized {
            segments.push(WindowControlSegment {
                action: WindowControlAction::Restore,
                label: "Restore".to_string(),
                icon_name: "restore".to_string(),
                aria_label: "Restore window".to_string(),
            });
        } else {
            segments.push(WindowControlSegment {
                action: WindowControlAction::Maximize,
                label: "Maximize".to_string(),
                icon_name: "maximize".to_string(),
                aria_label: "Maximize window".to_string(),
            });
        }
    }

    if config.can_close {
        segments.push(WindowControlSegment {
            action: WindowControlAction::Close,
            label: "Close".to_string(),
            icon_name: "x".to_string(),
            aria_label: "Close window".to_string(),
        });
    }

    WindowCapsuleFormatting {
        container_classes: CANONICAL_CAPSULE_CONTAINER_CLASSES.to_string(),
        segments,
        divider_classes: CANONICAL_CAPSULE_DIVIDER_CLASSES.to_string(),
    }
}

/// Validates whether a container class string satisfies contiguous capsule styling.
pub fn validate_capsule_classes(classes: &str) -> bool {
    classes.contains("rounded-full") && classes.contains("border")
}
