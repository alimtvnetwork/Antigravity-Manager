//! Unit tests for Upstream Endpoint Weight Matrix (Imp-562).

use super::endpoint_weight_matrix::EndpointWeightMatrix;

#[test]
fn test_endpoint_weight_matrix_empty() {
    let matrix = EndpointWeightMatrix::new();
    assert_eq!(matrix.best_endpoint(), None);
}

#[test]
fn test_endpoint_weight_matrix_single_endpoint() {
    let mut matrix = EndpointWeightMatrix::new();
    matrix.register_endpoint("https://ep1.com", 10.0);
    assert_eq!(matrix.best_endpoint(), Some("https://ep1.com".to_string()));
}

#[test]
fn test_endpoint_weight_matrix_failure_degrades_selection() {
    let mut matrix = EndpointWeightMatrix::new();
    matrix.register_endpoint("https://primary.com", 10.0);
    matrix.register_endpoint("https://secondary.com", 8.0);

    // Primary is initially best
    assert_eq!(
        matrix.best_endpoint(),
        Some("https://primary.com".to_string())
    );

    // Degrade primary: 10.0 * 0.6 = 6.0
    matrix.record_failure("https://primary.com");

    // Secondary (8.0) should now be preferred over degraded primary (6.0)
    assert_eq!(
        matrix.best_endpoint(),
        Some("https://secondary.com".to_string())
    );
}

#[test]
fn test_endpoint_weight_matrix_success_restores_weight() {
    let mut matrix = EndpointWeightMatrix::new();
    matrix.register_endpoint("https://recovering.com", 10.0);

    matrix.record_failure("https://recovering.com"); // down to 6.0
    matrix.record_success("https://recovering.com"); // 6.0 * 1.1 = 6.6

    assert_eq!(
        matrix.best_endpoint(),
        Some("https://recovering.com".to_string())
    );
}
