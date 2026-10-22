#[cfg(test)]
mod tests {
    use super::h2_ping_scheduler::H2PingScheduler;

    #[test]
    fn test_initial_state_prompts_immediate_ping() {
        let scheduler = H2PingScheduler::default();
        assert!(scheduler.should_send_ping(1000));
        assert!(!scheduler.is_connection_stale(1000));
    }

    #[test]
    fn test_ping_interval_throttling() {
        let mut scheduler = H2PingScheduler::new(Some(30), Some(10), Some(3));
        scheduler.record_ping_sent(1000);

        // Before 30s elapsed, should not send another ping
        assert!(!scheduler.should_send_ping(1020));

        // After 30s elapsed, should send ping
        assert!(scheduler.should_send_ping(1030));
        assert!(scheduler.should_send_ping(1050));
    }

    #[test]
    fn test_pong_received_clears_unacknowledged() {
        let mut scheduler = H2PingScheduler::default();
        scheduler.record_ping_sent(1000);
        assert_eq!(scheduler.unacknowledged_pings, 1);

        scheduler.record_pong_received(1001, Some(45));
        assert_eq!(scheduler.unacknowledged_pings, 0);
        assert_eq!(scheduler.last_rtt_ms, Some(45));
        assert!(!scheduler.is_connection_stale(1005));
    }

    #[test]
    fn test_timeout_marks_stale() {
        let mut scheduler = H2PingScheduler::new(Some(30), Some(10), Some(3));
        scheduler.record_ping_sent(1000);

        // 5s elapsed: still within 10s timeout
        assert!(!scheduler.is_connection_stale(1005));

        // 10s elapsed: exceeds timeout, connection is stale
        assert!(scheduler.is_connection_stale(1010));
        assert!(scheduler.is_connection_stale(1020));
    }

    #[test]
    fn test_max_unacknowledged_marks_stale() {
        let mut scheduler = H2PingScheduler::new(Some(30), Some(60), Some(3));
        scheduler.record_ping_sent(1000);
        scheduler.record_ping_sent(1030);
        assert!(!scheduler.is_connection_stale(1035));

        scheduler.record_ping_sent(1060);
        // 3 unacknowledged pings reaches max_unacknowledged
        assert!(scheduler.is_connection_stale(1061));
    }

    #[test]
    fn test_reset_clears_state() {
        let mut scheduler = H2PingScheduler::default();
        scheduler.record_ping_sent(1000);
        scheduler.record_pong_received(1002, Some(50));
        scheduler.reset();

        assert_eq!(scheduler.last_ping_sec, 0);
        assert_eq!(scheduler.last_pong_sec, 0);
        assert_eq!(scheduler.unacknowledged_pings, 0);
        assert_eq!(scheduler.last_rtt_ms, None);
        assert!(scheduler.should_send_ping(2000));
    }
}
