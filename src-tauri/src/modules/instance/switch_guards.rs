//! Switch preconditions: guards evaluated before an account switch proceeds.
use super::*;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Cross-machine distributed lease collision guard: refuse to switch to an
/// account that is actively leased by another machine in the cluster.
pub(crate) fn check_switch_lease_guard(account: &crate::models::Account) -> Result<(), String> {
    // Cross-Machine Distributed Lease Collision Guard
    if crate::modules::workspace_lease_manager::is_account_or_email_leased_by_other(
        &account.id,
        &account.email,
    ) {
        let holder_info = crate::modules::workspace_lease_manager::get_remote_lease_holder_info(
            &account.id,
            &account.email,
        );
        let detail = if let Some((alias, profile, remaining)) = holder_info {
            format!(
                "held by remote machine '{}' (Profile: '{}', expires in {}s)",
                alias, profile, remaining
            )
        } else {
            "currently leased by another active machine in the cluster".to_string()
        };
        let err = format!(
            "Cannot switch instance to account '{}': Account is {}",
            account.email, detail
        );
        crate::modules::logger::log_error(&format!("[INSTANCE_SWITCH:ERROR] {}", err));
        return Err(err);
    }
    Ok(())
}
