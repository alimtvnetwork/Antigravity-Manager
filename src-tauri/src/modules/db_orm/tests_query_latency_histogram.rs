#[cfg(test)]
mod tests {
    use super::super::query_latency_histogram::{HistogramSnapshot, QueryLatencyHistogram};
    use std::time::Duration;

    #[test]
    fn test_histogram_initial_state() {
        let hist = QueryLatencyHistogram::new();
        let snap = hist.snapshot();
        assert_eq!(
            snap,
            HistogramSnapshot {
                under_1ms: 0,
                between_1_and_5ms: 0,
                between_5_and_20ms: 0,
                over_20ms: 0,
                total_queries: 0,
            }
        );
    }

    #[test]
    fn test_record_query_bucketization() {
        let hist = QueryLatencyHistogram::new();

        // < 1ms: 400 microseconds
        hist.record_query(Duration::from_micros(400));
        hist.record_query(Duration::from_micros(900));

        // 1-5ms: 2ms, 4ms
        hist.record_query(Duration::from_millis(2));
        hist.record_query(Duration::from_millis(4));

        // 5-20ms: 12ms
        hist.record_query(Duration::from_millis(12));

        // > 20ms: 25ms, 100ms
        hist.record_query(Duration::from_millis(25));
        hist.record_query(Duration::from_millis(100));

        let snap = hist.snapshot();
        assert_eq!(snap.under_1ms, 2);
        assert_eq!(snap.between_1_and_5ms, 2);
        assert_eq!(snap.between_5_and_20ms, 1);
        assert_eq!(snap.over_20ms, 2);
        assert_eq!(snap.total_queries, 7);
    }

    #[test]
    fn test_reset_clears_histogram() {
        let hist = QueryLatencyHistogram::new();
        hist.record_query(Duration::from_millis(1));
        hist.record_query(Duration::from_millis(50));
        assert_eq!(hist.snapshot().total_queries, 2);

        hist.reset();
        let snap = hist.snapshot();
        assert_eq!(snap.total_queries, 0);
        assert_eq!(snap.under_1ms, 0);
        assert_eq!(snap.between_1_and_5ms, 0);
        assert_eq!(snap.between_5_and_20ms, 0);
        assert_eq!(snap.over_20ms, 0);
    }
}
