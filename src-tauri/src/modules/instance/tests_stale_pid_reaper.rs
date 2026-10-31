// Unit Test Suite for Stale Instance PID Process Reaper (Imp-530)
// Validates cross-referencing instance PIDs against OS liveness probes,
// detecting dead instances, and ignoring healthy or zero PIDs.

#[cfg(test)]
mod tests {
    use super::super::stale_pid_reaper::{ReapedInstance, StalePidReaper};

    #[test]
    fn test_reap_dead_instances_with_mock_probe() {
        let tracked = vec![
            ("inst_alive_1".to_string(), 1234),
            ("inst_dead_1".to_string(), 5678),
            ("inst_alive_2".to_string(), 9999),
            ("inst_dead_2".to_string(), 1111),
            ("inst_zero_pid".to_string(), 0),
        ];

        // Mock probe: PIDs 1234 and 9999 are alive, others are dead
        let is_alive = |pid: u32| pid == 1234 || pid == 9999;

        let reaped = StalePidReaper::reap_dead_instances(&tracked, is_alive);

        assert_eq!(reaped.len(), 2);
        assert_eq!(reaped[0].instance_id, "inst_dead_1");
        assert_eq!(reaped[0].stale_pid, 5678);

        assert_eq!(reaped[1].instance_id, "inst_dead_2");
        assert_eq!(reaped[1].stale_pid, 1111);
    }

    #[test]
    fn test_reap_all_alive() {
        let tracked = vec![("inst_1".to_string(), 100), ("inst_2".to_string(), 200)];

        let reaped = StalePidReaper::reap_dead_instances(&tracked, |_| true);
        assert!(reaped.is_empty());
    }

    #[test]
    fn test_reap_empty_tracked_list() {
        let tracked: Vec<(String, u32)> = Vec::new();
        let reaped = StalePidReaper::reap_dead_instances(&tracked, |_| false);
        assert!(reaped.is_empty());
    }

    #[test]
    fn test_reap_ignores_zero_pids() {
        let tracked = vec![
            ("inst_stopped".to_string(), 0),
            ("inst_uninitialized".to_string(), 0),
        ];

        // Even though is_alive returns false, PID 0 must be skipped
        let reaped = StalePidReaper::reap_dead_instances(&tracked, |_| false);
        assert!(reaped.is_empty());
    }
}
