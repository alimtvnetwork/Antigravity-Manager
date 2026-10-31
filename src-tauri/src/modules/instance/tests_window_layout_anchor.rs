//! Unit tests for Window Layout Coordinates Anchor.

use super::window_layout_anchor::{calculate_staggered_coordinates, WindowCoordinates};

#[test]
fn test_default_base_coordinates() {
    let coords = calculate_staggered_coordinates(0, None, None);
    assert_eq!(
        coords,
        WindowCoordinates {
            x: 80,
            y: 60,
            width: 1280,
            height: 800,
        }
    );
}

#[test]
fn test_staggered_offsets() {
    let c0 = calculate_staggered_coordinates(0, Some(100), Some(100));
    let c1 = calculate_staggered_coordinates(1, Some(100), Some(100));
    let c2 = calculate_staggered_coordinates(2, Some(100), Some(100));

    assert_eq!(c0.x, 100);
    assert_eq!(c0.y, 100);

    assert_eq!(c1.x, 140);
    assert_eq!(c1.y, 140);

    assert_eq!(c2.x, 180);
    assert_eq!(c2.y, 180);
}

#[test]
fn test_modulo_eight_wrapping() {
    let c0 = calculate_staggered_coordinates(0, Some(50), Some(50));
    let c8 = calculate_staggered_coordinates(8, Some(50), Some(50));
    let c16 = calculate_staggered_coordinates(16, Some(50), Some(50));

    assert_eq!(c0.x, c8.x);
    assert_eq!(c0.y, c8.y);
    assert_eq!(c8.x, c16.x);
    assert_eq!(c8.y, c16.y);

    let c7 = calculate_staggered_coordinates(7, Some(50), Some(50));
    let c15 = calculate_staggered_coordinates(15, Some(50), Some(50));
    assert_eq!(c7.x, 50 + (7 * 40));
    assert_eq!(c7.x, c15.x);
}

#[test]
fn test_constant_dimensions() {
    for idx in 0..10 {
        let coords = calculate_staggered_coordinates(idx, None, None);
        assert_eq!(coords.width, 1280);
        assert_eq!(coords.height, 800);
    }
}
