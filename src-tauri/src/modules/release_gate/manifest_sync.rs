//! Manifest Version Synchronization Validator.
//! Enforces atomic version synchronization across package.json, Cargo.toml, and tauri.conf.json.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestSyncReport {
    pub is_synced: bool,
    pub package_version: Option<String>,
    pub cargo_version: Option<String>,
    pub tauri_version: Option<String>,
}

/// Evaluates whether all provided manifest versions match identically.
pub fn verify_manifest_versions(
    package_ver: Option<&str>,
    cargo_ver: Option<&str>,
    tauri_ver: Option<&str>,
) -> ManifestSyncReport {
    let p = package_ver.map(|s| s.trim().to_string());
    let c = cargo_ver.map(|s| s.trim().to_string());
    let t = tauri_ver.map(|s| s.trim().to_string());

    let is_synced = match (&p, &c, &t) {
        (Some(p_val), Some(c_val), Some(t_val)) => p_val == c_val && c_val == t_val,
        _ => false,
    };

    ManifestSyncReport {
        is_synced,
        package_version: p,
        cargo_version: c,
        tauri_version: t,
    }
}
