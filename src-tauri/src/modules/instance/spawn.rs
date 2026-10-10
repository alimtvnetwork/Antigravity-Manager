//! Post-spawn bookkeeping for instance launches.
use super::*;
use crate::error::AppError;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Record a freshly spawned instance process and run the post-launch prompt
/// restoration pipeline when requested.
pub(crate) fn record_spawned_instance_process(
    instance_id: &str,
    data_dir: &str,
    reinject_prompts: bool,
    child_pid: u32,
) -> Result<(), crate::error::AppError> {
    // Justification: PID cache bookkeeping; staleness is tolerated via the OS-level re-detect fallback
    crate::error::record_ignored(
        record_instance_pid(instance_id, child_pid, data_dir),
        "record instance pid",
    );
    if reinject_prompts {
        wait_for_instance_prompt_channel(instance_id);
        // Justification: background restore; prompts are re-restored on the next launch
        crate::error::record_ignored(
            crate::modules::backup_prompts_db::restore_running_prompts(
                Some(instance_id),
                false,
                None,
            ),
            "restore running prompts",
        );
        // Justification: background resend; the scheduler re-attempts on its next tick
        crate::error::record_ignored(
            crate::modules::repo_db::resend_running_commands_for_instance(Some(instance_id), 20),
            "resend running commands",
        );
        // Justification: background dispatch; the scheduler re-attempts on its next tick
        crate::error::record_ignored(
            crate::modules::repo_db::dispatch_running_prompts(instance_id),
            "dispatch running prompts",
        );
        // Justification: intentionally discards the ensured-goals count; the function is infallible and the scheduler re-verifies on its next tick
        let _ = crate::modules::repo_db::ensure_prompt_goals_running_for_instance(instance_id);
    }
    crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));
    Ok(())
}
