use std::fs;
use std::path::{Path, PathBuf};

use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_parse_delegate_args() {
        let args = vec![
            "--wait-pid".to_string(),
            "4321".to_string(),
            "--install-dir".to_string(),
            "C:\\Users\\Test\\AppData\\Local\\Programs\\agm-alim".to_string(),
            "--target-exe".to_string(),
            "C:\\Users\\Test\\AppData\\Local\\Programs\\agm-alim\\agm-alim.exe".to_string(),
            "--relaunch".to_string(),
            "--delegated-worker".to_string(),
        ];
        let opts = parse_delegate_args(&args);
        assert_eq!(opts.wait_pid, Some(4321));
        assert!(opts.is_relaunch);
        assert!(opts.is_delegated_worker);
        assert_eq!(
            opts.install_dir.unwrap().to_string_lossy(),
            "C:\\Users\\Test\\AppData\\Local\\Programs\\agm-alim"
        );
        assert_eq!(
            opts.target_exe.unwrap().to_string_lossy(),
            "C:\\Users\\Test\\AppData\\Local\\Programs\\agm-alim\\agm-alim.exe"
        );
    }

    #[test]
    pub(crate) fn test_resolve_default_target_exe_filters_temp_updater() {
        let install_dir = PathBuf::from("C:\\Users\\Test\\AppData\\Local\\Programs\\agm-alim");
        let temp_exe = Some(PathBuf::from(
            "C:\\Temp\\agm-updater\\agm-update-cli-999.exe",
        ));
        let resolved = resolve_default_target_exe(temp_exe, &install_dir);
        #[cfg(target_os = "macos")]
        assert!(
            resolved
                .to_string_lossy()
                .contains("Antigravity Manager Tools.app")
                || resolved.to_string_lossy().contains("agm-alim")
        );
        #[cfg(not(target_os = "macos"))]
        assert!(resolved.to_string_lossy().contains("agm-alim"));
        assert!(!resolved.to_string_lossy().contains("agm-update-cli"));
    }

    #[test]
    pub(crate) fn test_resolve_default_target_exe_filters_trash_path() {
        let install_dir = PathBuf::from("C:\\Users\\Test\\AppData\\Local\\Programs\\agm-alim");
        let trashed_exe = Some(PathBuf::from(
            "/Users/test/.Trash/Antigravity Manager Tools 13-20-55-339.app",
        ));
        let resolved = resolve_default_target_exe(trashed_exe, &install_dir);
        assert!(!resolved.to_string_lossy().to_lowercase().contains(".trash"));
    }

    #[test]
    pub(crate) fn test_prepare_isolated_update_cli_creates_temp_binary() {
        let res = prepare_isolated_update_cli(987654);
        assert!(res.is_ok(), "prepare_isolated_update_cli failed: {:?}", res);
        let path = res.unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("agm-update-cli-987654"));
        // Justification: best-effort cleanup; a leftover file is harmless
        crate::error::record_ignored(fs::remove_file(&path), "remove_file");
    }
}
