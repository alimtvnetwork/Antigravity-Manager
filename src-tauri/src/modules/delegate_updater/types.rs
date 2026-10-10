use std::path::{Path, PathBuf};

use super::*;

#[derive(Debug, Clone, Default)]
pub struct DelegateUpdateOptions {
    pub wait_pid: Option<u32>,
    pub is_relaunch: bool,
    pub target_exe: Option<PathBuf>,
    pub install_dir: Option<PathBuf>,
    pub target_version: Option<String>,
    pub is_delegated_worker: bool,
}
