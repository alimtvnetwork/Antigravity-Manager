//! Account preservation and integrity verification for database migrations.

use crate::error::AppError;
use crate::modules::account;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Verification snapshot representing accounts integrity state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct AccountsVerification {
    pub account_count: usize,
    pub emails: Vec<String>,
    pub sha256: String,
}

/// Resolves path to `accounts.json` with an optional data directory override.
pub fn get_accounts_json_path_in(data_dir_override: Option<&Path>) -> Result<PathBuf, AppError> {
    let data_dir = match data_dir_override {
        Some(p) => p.to_path_buf(),
        None => account::get_data_dir().map_err(AppError::Config)?,
    };
    Ok(data_dir.join("accounts.json"))
}

/// Resolves path to `accounts.json` using the default data directory.
pub fn get_accounts_json_path() -> Result<PathBuf, AppError> {
    get_accounts_json_path_in(None)
}

/// Resolves path to temporary migration backup for `accounts.json` with optional data directory override.
pub fn get_accounts_backup_path_in(data_dir_override: Option<&Path>) -> Result<PathBuf, AppError> {
    let data_dir = match data_dir_override {
        Some(p) => p.to_path_buf(),
        None => account::get_data_dir().map_err(AppError::Config)?,
    };
    Ok(data_dir.join("accounts.json.migration_bak"))
}

/// Resolves path to temporary migration backup for `accounts.json`
pub fn get_accounts_backup_path() -> Result<PathBuf, AppError> {
    get_accounts_backup_path_in(None)
}

/// Captures a point-in-time cryptographic verification snapshot of `accounts.json` with optional data dir override.
pub fn capture_accounts_snapshot_in(
    data_dir_override: Option<&Path>,
) -> Result<AccountsVerification, AppError> {
    let path = get_accounts_json_path_in(data_dir_override)?;
    if !path.exists() {
        return Ok(AccountsVerification {
            account_count: 0,
            emails: Vec::new(),
            sha256: String::new(),
        });
    }

    let bytes = fs::read(&path).map_err(AppError::Io)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash_hex = format!("{:x}", hasher.finalize());

    let text = String::from_utf8_lossy(&bytes);
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);

    let mut emails = Vec::new();
    if let Some(arr) = parsed.as_array() {
        for item in arr {
            if let Some(email) = item.get("email").and_then(|e| e.as_str()) {
                emails.push(email.to_string());
            }
        }
    } else if let Some(accounts_val) = parsed.get("accounts").and_then(|v| v.as_array()) {
        for item in accounts_val {
            if let Some(email) = item.get("email").and_then(|e| e.as_str()) {
                emails.push(email.to_string());
            }
        }
    }

    emails.sort();

    Ok(AccountsVerification {
        account_count: emails.len(),
        emails,
        sha256: hash_hex,
    })
}

/// Captures a point-in-time cryptographic verification snapshot of `accounts.json`.
pub fn capture_accounts_snapshot() -> Result<AccountsVerification, AppError> {
    capture_accounts_snapshot_in(None)
}

/// Creates a safe isolated file backup of `accounts.json` prior to any migration actions.
pub fn backup_accounts_json_in(
    data_dir_override: Option<&Path>,
) -> Result<Option<PathBuf>, AppError> {
    let src = get_accounts_json_path_in(data_dir_override)?;
    if !src.exists() {
        return Ok(None);
    }

    let dst = get_accounts_backup_path_in(data_dir_override)?;
    fs::copy(&src, &dst).map_err(AppError::Io)?;
    Ok(Some(dst))
}

/// Creates a safe isolated file backup of `accounts.json` prior to any migration actions.
pub fn backup_accounts_json() -> Result<Option<PathBuf>, AppError> {
    backup_accounts_json_in(None)
}

/// Restores `accounts.json` from the migration backup upon detecting any integrity fault.
pub fn rollback_accounts_json_in(
    backup_path: &Path,
    data_dir_override: Option<&Path>,
) -> Result<(), AppError> {
    if !backup_path.exists() {
        return Ok(());
    }

    let target = get_accounts_json_path_in(data_dir_override)?;
    fs::copy(backup_path, &target).map_err(AppError::Io)?;

    crate::modules::logger::log_warn(&format!(
        "[AccountGuard] Executed rollback: restored accounts.json from {:?}",
        backup_path
    ));

    Ok(())
}

/// Restores `accounts.json` from the migration backup upon detecting any integrity fault.
pub fn rollback_accounts_json(backup_path: &Path) -> Result<(), AppError> {
    rollback_accounts_json_in(backup_path, None)
}

/// Asserts 100% hash, count, and email equivalence between two verification snapshots.
pub fn verify_accounts_equivalence(
    before: &AccountsVerification,
    after: &AccountsVerification,
) -> Result<(), AppError> {
    // If before had 0 accounts and after has 0, perfectly equivalent
    if before.account_count == 0 && after.account_count == 0 {
        return Ok(());
    }

    // 1. Account count assertion
    if after.account_count != before.account_count {
        let msg = format!(
            "Zero account loss violated! Expected {} accounts, but found {}.",
            before.account_count, after.account_count
        );
        crate::modules::logger::log_error(&msg);
        return Err(AppError::Account(msg));
    }

    // 2. Email list equivalence assertion
    if after.emails != before.emails {
        let msg = format!(
            "Zero account loss violated! Account email discrepancy. Expected {:?}, got {:?}",
            before.emails, after.emails
        );
        crate::modules::logger::log_error(&msg);
        return Err(AppError::Account(msg));
    }

    // 3. Cryptographic SHA-256 equivalence assertion
    if !before.sha256.is_empty() && !after.sha256.is_empty() && before.sha256 != after.sha256 {
        let msg = format!(
            "Zero account loss violated! SHA-256 hash mismatch. Expected {}, got {}",
            before.sha256, after.sha256
        );
        crate::modules::logger::log_error(&msg);
        return Err(AppError::Account(msg));
    }

    Ok(())
}

/// Verifies that no accounts have been lost between pre-migration and post-migration states with optional dir override.
pub fn verify_zero_account_loss_in(
    before: &AccountsVerification,
    data_dir_override: Option<&Path>,
) -> Result<(), AppError> {
    let after = capture_accounts_snapshot_in(data_dir_override)?;
    verify_accounts_equivalence(before, &after)?;

    crate::modules::logger::log_info(&format!(
        "[AccountGuard] Successfully verified zero account loss: {} accounts preserved.",
        after.account_count
    ));

    Ok(())
}

/// Verifies that no accounts have been lost between pre-migration and post-migration states.
pub fn verify_zero_account_loss(before: &AccountsVerification) -> Result<(), AppError> {
    verify_zero_account_loss_in(before, None)
}

/// RAII and explicit lifecycle guard ensuring automated rollback if any discrepancy is detected.
pub struct AccountGuard {
    pre_flight: AccountsVerification,
    backup_path: Option<PathBuf>,
    data_dir_override: Option<PathBuf>,
    committed: bool,
}

impl AccountGuard {
    /// Captures pre-flight verification snapshot and creates a file backup of accounts.json.
    pub fn acquire() -> Result<Self, AppError> {
        Self::acquire_in(None)
    }

    /// Captures pre-flight verification snapshot and creates a file backup with optional directory override.
    pub fn acquire_in(data_dir_override: Option<&Path>) -> Result<Self, AppError> {
        let pre_flight = capture_accounts_snapshot_in(data_dir_override)?;
        let backup_path = backup_accounts_json_in(data_dir_override)?;

        crate::modules::logger::log_info(&format!(
            "[AccountGuard] Pre-flight verification captured: count={}, SHA256={}",
            pre_flight.account_count, pre_flight.sha256
        ));

        Ok(Self {
            pre_flight,
            backup_path,
            data_dir_override: data_dir_override.map(|p| p.to_path_buf()),
            committed: false,
        })
    }

    /// Accessor for pre-flight verification metadata.
    pub fn pre_flight(&self) -> &AccountsVerification {
        &self.pre_flight
    }

    /// Asserts 100% equivalence post-migration and commits the state, removing temporary backup.
    /// If discrepancy is detected, triggers automated rollback and returns an error.
    pub fn verify_and_commit(mut self) -> Result<AccountsVerification, AppError> {
        let post_flight = capture_accounts_snapshot_in(self.data_dir_override.as_deref())?;
        if let Err(err) = verify_accounts_equivalence(&self.pre_flight, &post_flight) {
            crate::modules::logger::log_error(&format!(
                "[AccountGuard] Discrepancy detected during post-migration verification: {}. Triggering automated rollback!",
                err
            ));
            if let Err(rb_err) = self.trigger_rollback() {
                crate::modules::logger::log_error(&format!(
                    "[AccountGuard] Critical: Automated rollback failed: {}",
                    rb_err
                ));
            }
            return Err(err);
        }

        self.committed = true;
        self.cleanup_backup();

        crate::modules::logger::log_info(&format!(
            "[AccountGuard] Post-migration verification confirmed 100% equivalence: {} accounts preserved.",
            post_flight.account_count
        ));

        Ok(post_flight)
    }

    /// Explicitly restores accounts.json from backup if available.
    pub fn trigger_rollback(&self) -> Result<(), AppError> {
        if let Some(ref bp) = self.backup_path {
            rollback_accounts_json_in(bp, self.data_dir_override.as_deref())?;
        }
        Ok(())
    }

    fn cleanup_backup(&self) {
        if let Some(ref bp) = self.backup_path {
            if bp.exists() {
                if let Err(e) = fs::remove_file(bp) {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "cleanup account migration backup",
                    );
                    // Justification: Benign temporary backup removal after verification already committed.
                }
            }
        }
    }
}

impl Drop for AccountGuard {
    fn drop(&mut self) {
        if !self.committed && self.backup_path.is_some() {
            crate::modules::logger::log_warn(
                "[AccountGuard] AccountGuard dropped uncommitted; executing automated safety rollback.",
            );
            let _ = self.trigger_rollback();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_accounts_equivalence_success() {
        let snap1 = AccountsVerification {
            account_count: 2,
            emails: vec!["a@example.com".to_string(), "b@example.com".to_string()],
            sha256: "hash123".to_string(),
        };
        let snap2 = snap1.clone();
        assert!(verify_accounts_equivalence(&snap1, &snap2).is_ok());
    }

    #[test]
    fn test_verify_accounts_equivalence_count_mismatch() {
        let snap1 = AccountsVerification {
            account_count: 2,
            emails: vec!["a@example.com".to_string(), "b@example.com".to_string()],
            sha256: "hash123".to_string(),
        };
        let mut snap2 = snap1.clone();
        snap2.account_count = 1;
        assert!(verify_accounts_equivalence(&snap1, &snap2).is_err());
    }

    #[test]
    fn test_verify_accounts_equivalence_hash_mismatch() {
        let snap1 = AccountsVerification {
            account_count: 1,
            emails: vec!["a@example.com".to_string()],
            sha256: "hash123".to_string(),
        };
        let mut snap2 = snap1.clone();
        snap2.sha256 = "different_hash".to_string();
        assert!(verify_accounts_equivalence(&snap1, &snap2).is_err());
    }
}
