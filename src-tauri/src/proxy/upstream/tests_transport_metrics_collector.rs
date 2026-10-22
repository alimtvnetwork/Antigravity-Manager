//! Unit tests for Upstream Transport Metrics Collector (Imp-553).

use super::transport_metrics_collector::{TransportMetricsCollector, TransportTimings};

#[test]
fn test_transport_metrics_collector_empty() {
    let collector = TransportMetricsCollector::new();
    assert_eq!(collector.average_ttfb("https://api.anthropic.com"), None);
}

#[test]
fn test_transport_metrics_record_initial_sample() {
    let collector = TransportMetricsCollector::new();
    let sample = TransportTimings {
        endpoint_url: "https://api.openai.com/v1".to_string(),
        dns_ms: 12.5,
        tcp_ms: 25.0,
        tls_ms: 30.0,
        ttfb_ms: 150.0,
        total_ms: 217.5,
    };

    collector.record(&sample);
    let avg = collector.average_ttfb("https://api.openai.com/v1");
    assert!(avg.is_some());
    // Initial entry inserts 150.0, then immediately * 0.8 + 150.0 * 0.2 = 150.0
    let val = avg.unwrap();
    assert!((val - 150.0).abs() < 1e-4);
}

#[test]
fn test_transport_metrics_moving_average() {
    let collector = TransportMetricsCollector::new();
    let url = "https://generativelanguage.googleapis.com";

    let sample1 = TransportTimings {
        endpoint_url: url.to_string(),
        dns_ms: 10.0,
        tcp_ms: 20.0,
        tls_ms: 30.0,
        ttfb_ms: 100.0,
        total_ms: 160.0,
    };
    collector.record(&sample1);

    let sample2 = TransportTimings {
        endpoint_url: url.to_string(),
        dns_ms: 10.0,
        tcp_ms: 20.0,
        tls_ms: 30.0,
        ttfb_ms: 200.0,
        total_ms: 260.0,
    };
    collector.record(&sample2);

    // Expected: 100.0 * 0.8 + 200.0 * 0.2 = 80 + 40 = 120.0
    let avg = collector.average_ttfb(url).expect("average should exist");
    assert!((avg - 120.0).abs() < 1e-4);
}

#[test]
fn test_transport_metrics_distinct_endpoints() {
    let collector = TransportMetricsCollector::new();
    let sample_a = TransportTimings {
        endpoint_url: "https://ep1.example.com".to_string(),
        dns_ms: 5.0,
        tcp_ms: 10.0,
        tls_ms: 15.0,
        ttfb_ms: 50.0,
        total_ms: 80.0,
    };
    let sample_b = TransportTimings {
        endpoint_url: "https://ep2.example.com".to_string(),
        dns_ms: 5.0,
        tcp_ms: 10.0,
        tls_ms: 15.0,
        ttfb_ms: 300.0,
        total_ms: 330.0,
    };

    collector.record(&sample_a);
    collector.record(&sample_b);

    let avg_a = collector.average_ttfb("https://ep1.example.com").unwrap();
    let avg_b = collector.average_ttfb("https://ep2.example.com").unwrap();

    assert!((avg_a - 50.0).abs() < 1e-4);
    assert!((avg_b - 300.0).abs() < 1e-4);
}
