//! Unit tests for SSE Heartbeat Generator (`Imp-616`)

#[cfg(test)]
mod tests {
    use super::super::sse_heartbeat_generator::{
        evaluate_heartbeat_tick, format_sse_comment, is_valid_sse_comment_frame, HeartbeatConfig,
        HeartbeatFrameAction, HeartbeatGeneratorState,
    };

    #[test]
    fn test_heartbeat_config_defaults() {
        let config = HeartbeatConfig::default();
        assert_eq!(config.interval_ms, 15_000);
        assert_eq!(config.ping_payload, ": ping\n\n");
        assert_eq!(config.min_interval_ms, 5_000);
        assert_eq!(config.effective_interval_ms(), 15_000);
    }

    #[test]
    fn test_heartbeat_config_min_interval_clamping() {
        let aggressive_config = HeartbeatConfig::new(1_000, ": ping\n\n");
        assert_eq!(aggressive_config.interval_ms, 1_000);
        assert_eq!(aggressive_config.effective_interval_ms(), 5_000);
    }

    #[test]
    fn test_heartbeat_config_minimal_comment() {
        let config = HeartbeatConfig::minimal_comment();
        assert_eq!(config.ping_payload, ":\n\n");
        assert!(is_valid_sse_comment_frame(&config.ping_payload));
    }

    #[test]
    fn test_generator_state_initialization() {
        let state = HeartbeatGeneratorState::new(100_000);
        assert_eq!(state.last_event_epoch_ms, 100_000);
        assert_eq!(state.last_ping_epoch_ms, 0);
        assert_eq!(state.total_pings_sent, 0);
        assert_eq!(state.last_activity_epoch_ms(), 100_000);
    }

    #[test]
    fn test_tick_returns_idle_before_interval_elapses() {
        let config = HeartbeatConfig::default();
        let mut state = HeartbeatGeneratorState::new(10_000);

        let action = evaluate_heartbeat_tick(&config, &mut state, 20_000);
        assert_eq!(action, HeartbeatFrameAction::Idle);
        assert_eq!(state.total_pings_sent, 0);
        assert_eq!(state.last_ping_epoch_ms, 0);
    }

    #[test]
    fn test_tick_emits_ping_at_interval_threshold() {
        let config = HeartbeatConfig::default();
        let mut state = HeartbeatGeneratorState::new(10_000);

        let action = evaluate_heartbeat_tick(&config, &mut state, 25_000);
        assert_eq!(
            action,
            HeartbeatFrameAction::EmitPing(": ping\n\n".to_string())
        );
        assert_eq!(state.total_pings_sent, 1);
        assert_eq!(state.last_ping_epoch_ms, 25_000);

        let action_next = evaluate_heartbeat_tick(&config, &mut state, 25_100);
        assert_eq!(action_next, HeartbeatFrameAction::Idle);
        assert_eq!(state.total_pings_sent, 1);
    }

    #[test]
    fn test_tick_emits_successive_pings_in_long_thinking_turn() {
        let config = HeartbeatConfig::default();
        let mut state = HeartbeatGeneratorState::new(0);

        let a1 = evaluate_heartbeat_tick(&config, &mut state, 15_000);
        assert!(a1.is_emit());
        assert_eq!(state.total_pings_sent, 1);

        let a2 = evaluate_heartbeat_tick(&config, &mut state, 30_000);
        assert!(a2.is_emit());
        assert_eq!(state.total_pings_sent, 2);

        let a3 = evaluate_heartbeat_tick(&config, &mut state, 45_000);
        assert!(a3.is_emit());
        assert_eq!(state.total_pings_sent, 3);
        assert_eq!(state.last_ping_epoch_ms, 45_000);
    }

    #[test]
    fn test_record_activity_resets_countdown() {
        let config = HeartbeatConfig::default();
        let mut state = HeartbeatGeneratorState::new(0);

        assert_eq!(
            evaluate_heartbeat_tick(&config, &mut state, 10_000),
            HeartbeatFrameAction::Idle
        );

        state.record_activity(12_000);
        assert_eq!(state.last_activity_epoch_ms(), 12_000);

        assert_eq!(
            evaluate_heartbeat_tick(&config, &mut state, 15_000),
            HeartbeatFrameAction::Idle
        );

        let action = evaluate_heartbeat_tick(&config, &mut state, 27_000);
        assert!(action.is_emit());
        assert_eq!(state.total_pings_sent, 1);
    }

    #[test]
    fn test_clock_skew_backwards_jump_handled_safely() {
        let config = HeartbeatConfig::default();
        let mut state = HeartbeatGeneratorState::new(50_000);

        let action = evaluate_heartbeat_tick(&config, &mut state, 40_000);
        assert_eq!(action, HeartbeatFrameAction::Idle);
        assert_eq!(state.total_pings_sent, 0);
    }

    #[test]
    fn test_time_until_next_tick_ms() {
        let config = HeartbeatConfig::default();
        let state = HeartbeatGeneratorState::new(10_000);

        assert_eq!(state.time_until_next_tick_ms(&config, 10_000), 15_000);
        assert_eq!(state.time_until_next_tick_ms(&config, 15_000), 10_000);
        assert_eq!(state.time_until_next_tick_ms(&config, 25_000), 0);
        assert_eq!(state.time_until_next_tick_ms(&config, 30_000), 0);
    }

    #[test]
    fn test_format_and_validate_sse_comment() {
        assert_eq!(format_sse_comment("ping"), ": ping\n\n");
        assert_eq!(
            format_sse_comment("keepalive - deep thinking"),
            ": keepalive - deep thinking\n\n"
        );
        assert_eq!(format_sse_comment(""), ":\n\n");

        assert!(is_valid_sse_comment_frame(": ping\n\n"));
        assert!(is_valid_sse_comment_frame(":\n\n"));
        assert!(is_valid_sse_comment_frame(": keepalive\r\n\n"));
        assert!(!is_valid_sse_comment_frame("data: {\"done\":true}\n\n"));
        assert!(!is_valid_sse_comment_frame(": incomplete"));
    }
}
