//! Window Layout Coordinates Anchor
//!
//! Calculates staggered screen offset coordinates for newly launched IDE instances,
//! ensuring multiple concurrently running instances do not stack directly on top of each other.

use tracing::debug;

/// Calculated window position coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowCoordinates {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Computes staggered window coordinates based on instance sequence index.
pub fn calculate_staggered_coordinates(
    instance_index: usize,
    base_x: Option<i32>,
    base_y: Option<i32>,
) -> WindowCoordinates {
    let start_x = base_x.unwrap_or(80);
    let start_y = base_y.unwrap_or(60);

    // Stagger by 40 pixels per instance, wrapping every 8 instances
    let offset = ((instance_index % 8) * 40) as i32;

    let coords = WindowCoordinates {
        x: start_x + offset,
        y: start_y + offset,
        width: 1280,
        height: 800,
    };

    debug!(
        index = instance_index,
        x = coords.x,
        y = coords.y,
        "Calculated staggered window coordinates"
    );

    coords
}
