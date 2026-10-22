#[cfg(test)]
mod tests {
    use super::super::commit_latency_observer::CommitLatencyObserver;
    use std::time::Duration;

    #[test]
    fn test_observer_initial_state() {
        let observer = CommitLatencyObserver::default();
        let stats = observer.get_stats();

        assert_eq!(stats.total_commits_tracked, 0);
        assert_eq!(stats.avg_commit_duration_ms, 0.0);
        assert_eq!(stats.max_commit_duration_ms, 0.0);
        assert_eq!(stats.slow_commits_gt_50ms, 0);
    }

    #[test]
    fn test_record_commit_durations_aggregation() {
        let observer = CommitLatencyObserver::default();

        observer.record_commit_duration(Duration::from_millis(10));
        observer.record_commit_duration(Duration::from_millis(20));
        observer.record_commit_duration(Duration::from_millis(30));

        let stats = observer.get_stats();
        assert_eq!(stats.total_commits_tracked, 3);
        assert!((stats.avg_commit_duration_ms - 20.0).abs() < 1e-4);
        assert!((stats.max_commit_duration_ms - 30.0).abs() < 1e-4);
        assert_eq!(stats.slow_commits_gt_50ms, 0);
    }

    #[test]
    fn test_slow_commits_detection() {
        let observer = CommitLatencyObserver::default();

        // Under 50ms threshold
        observer.record_commit_duration(Duration::from_millis(25));
        // Over 50ms threshold
        observer.record_commit_duration(Duration::from_millis(75));
        observer.record_commit_duration(Duration::from_millis(150));

        let stats = observer.get_stats();
        assert_eq!(stats.total_commits_tracked, 3);
        assert_eq!(stats.slow_commits_gt_50ms, 2);
        assert!((stats.max_commit_duration_ms - 150.0).abs() < 1e-4);
    }

    #[test]
    fn test_custom_slow_threshold() {
        let custom_observer = CommitLatencyObserver::new(Some(Duration::from_millis(15)));

        custom_observer.record_commit_duration(Duration::from_millis(10));
        custom_observer.record_commit_duration(Duration::from_millis(20));

        let stats = custom_observer.get_stats();
        assert_eq!(stats.total_commits_tracked, 2);
        assert_eq!(stats.slow_commits_gt_50ms, 1); // 20ms is > 15ms threshold
    }
}
