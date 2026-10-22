// Post-Switch Settling Delay Guard (Imp-521)
// Enforces the mandatory 10-second settling delay following credential switches
// to allow the newly launched IDE process to initialize before dispatching prompt injections.

use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::info;

pub const DEFAULT_SETTLING_DELAY_SECS: u64 = 10;

#[derive(Debug, Clone)]
pub struct SwitchSettlingTimer {
    settling_duration: Duration,
}

impl Default for SwitchSettlingTimer {
    fn default() -> Self {
        Self::new(Duration::from_secs(DEFAULT_SETTLING_DELAY_SECS))
    }
}

impl SwitchSettlingTimer {
    pub fn new(settling_duration: Duration) -> Self {
        Self { settling_duration }
    }

    /// Asynchronously pauses execution to satisfy the post-switch settling barrier.
    pub async fn await_settling(&self, instance_id: &str) -> u64 {
        let start = Instant::now();
        info!(
            instance_id = %instance_id,
            duration_secs = self.settling_duration.as_secs(),
            "Entering post-switch settling barrier"
        );

        sleep(self.settling_duration).await;

        let elapsed_ms = start.elapsed().as_millis() as u64;
        info!(
            instance_id = %instance_id,
            elapsed_ms = elapsed_ms,
            "Exited post-switch settling barrier"
        );

        elapsed_ms
    }

    pub fn duration(&self) -> Duration {
        self.settling_duration
    }
}
