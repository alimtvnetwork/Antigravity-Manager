//! IDE Window Position & Monitor Restorer
//! Preserves and restores window coordinates, monitor bindings, and maximization
//! states across instance relaunches.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_maximized: bool,
}

impl Default for WindowBounds {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 1280,
            height: 800,
            is_maximized: false,
        }
    }
}

pub struct WindowStateRestorer;

impl WindowStateRestorer {
    /// Loads saved window bounds from instance configuration directory.
    pub fn load_bounds(instance_dir: &Path) -> WindowBounds {
        let state_file = instance_dir.join("window_state.json");
        if state_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&state_file) {
                if let Ok(bounds) = serde_json::from_str::<WindowBounds>(&content) {
                    return bounds;
                }
            }
        }
        WindowBounds::default()
    }

    /// Saves window bounds to instance configuration directory.
    pub fn save_bounds(instance_dir: &Path, bounds: &WindowBounds) -> std::io::Result<()> {
        let state_file = instance_dir.join("window_state.json");
        let serialized = serde_json::to_string_pretty(bounds)?;
        std::fs::write(state_file, serialized)
    }
}
