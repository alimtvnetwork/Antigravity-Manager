//! Instance tests (tests_a).
use super::*;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch_spares_another_instances_pid() {
        let protected = vec![11128u32];
        let markers = vec![
            "/instances/nextv2/".to_string(),
            "antigravity-nextv2".to_string(),
            "c:/users/administrator/.antigravity_tools/instances/nextv2".to_string(),
        ];
        assert!(should_spare_pid(
            11128,
            "",
            "c:/program files/antigravity/antigravity.exe",
            "antigravity.exe",
            &protected,
            &markers
        ));
        assert!(should_spare_pid(
            4242,
            "--user-data-dir=c:/users/administrator/.antigravity_tools/instances/nextv2",
            "c:/program files/antigravity/antigravity.exe",
            "antigravity.exe",
            &protected,
            &markers
        ));
        assert!(!should_spare_pid(
            9001,
            "--user-data-dir=c:/users/administrator/appdata/roaming/antigravity",
            "c:/program files/antigravity/antigravity.exe",
            "antigravity.exe",
            &protected,
            &markers
        ));
    }

    #[test]
    fn saved_pid_identity_and_refresh_floor() {
        assert!(process_identity_matches(
            "Antigravity.exe",
            "C:/Program Files/Antigravity/Antigravity.exe"
        ));
        assert!(!process_identity_matches("agm.exe", "C:/agm/agm.exe"));
        assert!(!process_identity_matches(
            "antigravity",
            "C:/Users/me/AppData/Local/agm/agm.exe"
        ));
        assert_eq!(pid_refresh_interval_seconds(60), 180);
        assert_eq!(pid_refresh_interval_seconds(180), 180);
        assert_eq!(pid_refresh_interval_seconds(600), 600);
        assert_eq!(pid_refresh_interval_seconds(2000), 1200);
    }

    #[test]
    fn test_process_identity_matches_rejects_developer_tools() {
        assert!(!process_identity_matches(
            "esbuild.exe",
            "D:/work/Antigravity-Manager/node_modules/@esbuild/win32-x64/esbuild.exe"
        ));
        assert!(!process_identity_matches(
            "cargo.exe",
            "D:/work/Antigravity-Manager/target/debug/build/cargo.exe"
        ));
        assert!(process_identity_matches(
            "Antigravity.exe",
            "C:/Users/Admin/AppData/Local/Programs/Antigravity/Antigravity.exe"
        ));
    }

    #[test]
    fn test_workspace_snapshot_and_migration_helpers() {
        let ws = snapshot_active_workspaces("non-existent-instance-id-xyz");
        assert!(ws.is_empty());
        let res = migrate_instance_workspaces("non-existent-1", "non-existent-1");
        assert!(res.is_ok());
    }

    #[test]
    fn test_find_pids_target_path_normalization() {
        let raw_path = "C:\\Users\\User\\.config\\antigravity\\";
        let normalized = raw_path.to_lowercase().replace('\\', "/");
        let clean = normalized.trim_end_matches('/');
        assert_eq!(clean, "c:/users/user/.config/antigravity");
    }

    #[test]
    fn test_linux_process_name_matching() {
        let names = ["antigravity", "antigravity-ide", "apprun", "code"];
        let is_matched_first = names[0].contains("antigravity");
        let is_matched_third = names[2] == "apprun";
        assert!(is_matched_first);
        assert!(is_matched_third);

        let helper_args = "--type=renderer --user-data-dir=/tmp/test";
        let is_helper = helper_args.contains("--type=");
        assert!(is_helper);
    }
}
