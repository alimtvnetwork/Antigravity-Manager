//! Upstream Latency Benchmark Reporter.
//! Measures and compares roundtrip latency across upstream AI model endpoints.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamLatencyBenchmark {
    pub provider: String,
    pub endpoint_url: String,
    pub latency_ms: u64,
    pub is_reachable: bool,
    pub status_code: Option<u16>,
    pub last_checked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamBenchmarkReport {
    pub benchmarks: Vec<UpstreamLatencyBenchmark>,
    pub fastest_provider: Option<String>,
    pub generated_at: String,
}

/// Probes upstream endpoints with lightweight HEAD or GET checks to determine live latency.
pub async fn benchmark_upstream_endpoints() -> UpstreamBenchmarkReport {
    let endpoints = vec![
        (
            "Google Daily",
            "https://daily-generativelanguage.sandbox.googleapis.com",
        ),
        (
            "Google Sandbox",
            "https://sandbox-generativelanguage.googleapis.com",
        ),
        ("Google Prod", "https://generativelanguage.googleapis.com"),
        ("Anthropic Claude", "https://api.anthropic.com"),
    ];

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(4000))
        .build()
        .unwrap_or_default();

    let mut benchmarks = Vec::new();
    let mut fastest_provider = None;
    let mut min_latency = u64::MAX;

    for (name, url) in endpoints {
        let start = Instant::now();
        let (reachable, status) = match client.head(url).send().await {
            Ok(resp) => (true, Some(resp.status().as_u16())),
            Err(_) => (false, None),
        };
        let elapsed = start.elapsed().as_millis() as u64;

        if reachable && elapsed < min_latency {
            min_latency = elapsed;
            fastest_provider = Some(name.to_string());
        }

        benchmarks.push(UpstreamLatencyBenchmark {
            provider: name.to_string(),
            endpoint_url: url.to_string(),
            latency_ms: elapsed,
            is_reachable: reachable,
            status_code: status,
            last_checked: Utc::now().to_rfc3339(),
        });
    }

    UpstreamBenchmarkReport {
        benchmarks,
        fastest_provider,
        generated_at: Utc::now().to_rfc3339(),
    }
}
