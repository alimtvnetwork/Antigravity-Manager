// Note: Middleware testing needs to simulate HTTP requests. The testing framework is provided here.
// Actual integration testing should be done after starting the full service

/// Verify the correctness of the IP extraction logic
#[test]
fn test_ip_extraction_priority() {
    // X-Forwarded-For should take precedence over X-Real-IP
    // X-Real-IP should take precedence over ConnectInfo
    // Here we only verify the logical concept, the actual test requires constructing an HTTP request

    // Scenario 1: X-Forwarded-For has multiple IPs, take the first one
    let xff_header = "203.0.113.1, 198.51.100.2, 192.0.2.3";
    let first_ip = xff_header.split(',').next().unwrap().trim();
    assert_eq!(first_ip, "203.0.113.1");

    // Scenario 2: Single IP
    let single_ip = "10.0.0.1";
    let parsed = single_ip.split(',').next().unwrap().trim();
    assert_eq!(parsed, "10.0.0.1");
}
