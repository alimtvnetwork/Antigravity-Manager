//! Dynamic WAL Checkpoint Scheduler
//! Recommends checkpoint urgency and execution mode based on
//! WAL frame accumulation and concurrent write activity.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduledCheckpointAction {
    Skip,
    Passive,
    Restart,
    Truncate,
}

pub struct CheckpointScheduler;

impl CheckpointScheduler {
    pub const PASSIVE_THRESHOLD_FRAMES: i32 = 150;
    pub const RESTART_THRESHOLD_FRAMES: i32 = 1000;
    pub const TRUNCATE_THRESHOLD_FRAMES: i32 = 4000;

    /// Evaluates current uncheckpointed WAL log frames and determines recommended action.
    pub fn evaluate_action(
        uncheckpointed_frames: i32,
        is_idle_period: bool,
    ) -> ScheduledCheckpointAction {
        if uncheckpointed_frames >= Self::TRUNCATE_THRESHOLD_FRAMES {
            ScheduledCheckpointAction::Truncate
        } else if uncheckpointed_frames >= Self::RESTART_THRESHOLD_FRAMES {
            if is_idle_period {
                ScheduledCheckpointAction::Truncate
            } else {
                ScheduledCheckpointAction::Restart
            }
        } else if uncheckpointed_frames >= Self::PASSIVE_THRESHOLD_FRAMES {
            ScheduledCheckpointAction::Passive
        } else if is_idle_period && uncheckpointed_frames > 50 {
            ScheduledCheckpointAction::Passive
        } else {
            ScheduledCheckpointAction::Skip
        }
    }
}
