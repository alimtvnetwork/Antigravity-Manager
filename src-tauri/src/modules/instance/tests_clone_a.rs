//! Instance clone-tree tests.
use super::*;
use std::fs;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(test)]
mod clone_tree_tests {
    use super::*;

    #[test]
    fn clone_copies_settings_workspace_db_and_gemini_repo() {
        let root = std::env::temp_dir().join(format!("agm-clone-trees-{}", std::process::id()));
        // Justification: test cleanup; absence is the normal case
        crate::error::record_ignored(fs::remove_dir_all(&root), "clean test dir");
        let src_data = root.join("src-data");
        let dst_data = root.join("dst-data");
        let src_home = root.join("src-home");
        let dst_home = root.join("dst-home");
        fs::create_dir_all(src_data.join("User").join("globalStorage")).unwrap();
        fs::create_dir_all(src_data.join("User").join("workspaceStorage").join("proj")).unwrap();
        fs::write(
            src_data.join("User").join("settings.json"),
            br#"{"workbench.colorTheme":"Tokyo Night"}"#,
        )
        .unwrap();
        fs::write(
            src_data.join("User").join("keybindings.json"),
            b"[{\"key\": \"ctrl+k\"}]",
        )
        .unwrap();
        fs::create_dir_all(src_data.join("User").join("snippets")).unwrap();
        fs::write(
            src_data.join("User").join("snippets").join("rust.json"),
            b"{\"snippet\": \"test\"}",
        )
        .unwrap();
        fs::write(
            src_data
                .join("User")
                .join("workspaceStorage")
                .join("proj")
                .join("state.vscdb"),
            b"workspace-repo-db",
        )
        .unwrap();
        fs::create_dir_all(src_home.join(".gemini").join("antigravity")).unwrap();
        fs::write(
            src_home.join(".gemini").join("antigravity").join("repo.db"),
            b"gemini-repo-db",
        )
        .unwrap();
        fs::create_dir_all(&dst_data).unwrap();

        copy_required_ide_files(&src_data, &dst_data).unwrap();
        copy_gemini_trees(&src_home, &dst_home).unwrap();

        assert_eq!(
            fs::read(dst_data.join("User").join("settings.json")).unwrap(),
            br#"{"workbench.colorTheme":"Tokyo Night"}"#
        );
        assert_eq!(
            fs::read(dst_data.join("User").join("keybindings.json")).unwrap(),
            b"[{\"key\": \"ctrl+k\"}]"
        );
        assert_eq!(
            fs::read(dst_data.join("User").join("snippets").join("rust.json")).unwrap(),
            b"{\"snippet\": \"test\"}"
        );
        assert_eq!(
            fs::read(
                dst_data
                    .join("User")
                    .join("workspaceStorage")
                    .join("proj")
                    .join("state.vscdb")
            )
            .unwrap(),
            b"workspace-repo-db"
        );
        assert_eq!(
            fs::read(dst_home.join(".gemini").join("antigravity").join("repo.db")).unwrap(),
            b"gemini-repo-db"
        );
        // Justification: test cleanup; absence is the normal case
        crate::error::record_ignored(fs::remove_dir_all(&root), "clean test dir");
    }

    #[test]
    fn test_copy_source_user_settings_copies_to_data_and_appdata() {
        let root = std::env::temp_dir().join(format!("agm-user-settings-{}", std::process::id()));
        // Justification: test cleanup; absence is the normal case
        crate::error::record_ignored(fs::remove_dir_all(&root), "clean test dir");

        let src_data = root.join("src-inst").join("data");
        let dst_data = root.join("dst-inst").join("data");

        let src_user = src_data.join("User");
        fs::create_dir_all(&src_user).unwrap();
        fs::write(
            src_user.join("settings.json"),
            br#"{"workbench.colorTheme":"Catppuccin Mocha"}"#,
        )
        .unwrap();
        fs::write(
            src_user.join("keybindings.json"),
            br#"[{"key":"ctrl+shift+p"}]"#,
        )
        .unwrap();
        let src_snippets = src_user.join("snippets");
        fs::create_dir_all(&src_snippets).unwrap();
        fs::write(
            src_snippets.join("custom.json"),
            b"{\"prefix\": \"custom\"}",
        )
        .unwrap();

        let source = InstanceConfig {
            id: "src-inst".to_string(),
            name: "Source".to_string(),
            data_dir: src_data.to_string_lossy().to_string(),
            executable_path: None,
            extensions_dir: Some("D:/extensions".to_string()),
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(1),
        };

        let dest = InstanceConfig {
            id: "dst-inst".to_string(),
            name: "Dest".to_string(),
            data_dir: dst_data.to_string_lossy().to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(2),
        };

        // Justification: the unit under test; assertions below verify the outcome
        crate::error::record_ignored(
            copy_source_user_settings(&source, &dest),
            "run copy under test",
        );

        assert_eq!(
            fs::read(dst_data.join("User").join("settings.json")).unwrap(),
            br#"{"workbench.colorTheme":"Catppuccin Mocha"}"#
        );
        assert_eq!(
            fs::read(dst_data.join("User").join("keybindings.json")).unwrap(),
            br#"[{"key":"ctrl+shift+p"}]"#
        );
        assert_eq!(
            fs::read(dst_data.join("User").join("snippets").join("custom.json")).unwrap(),
            b"{\"prefix\": \"custom\"}"
        );

        // Justification: test cleanup; absence is the normal case
        crate::error::record_ignored(fs::remove_dir_all(&root), "clean test dir");
    }

    #[test]
    fn test_compute_ide_ending_sequence() {
        let default_inst = InstanceConfig {
            id: "default".to_string(),
            name: "Default".to_string(),
            data_dir: "/tmp/data".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: true,
            pid: None,
            seq_num: Some(1),
        };
        assert_eq!(compute_ide_ending_sequence(&default_inst), "Antigravity");

        let default_by_id = InstanceConfig {
            id: "default".to_string(),
            name: "Main Workspace".to_string(),
            data_dir: "/tmp/data".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(1),
        };
        assert_eq!(compute_ide_ending_sequence(&default_by_id), "Antigravity");

        let custom_inst = InstanceConfig {
            id: "inst-8136".to_string(),
            name: "8136".to_string(),
            data_dir: "/tmp/data".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(2),
        };
        assert_eq!(
            compute_ide_ending_sequence(&custom_inst),
            "antigravity-8136"
        );

        let fallback_inst = InstanceConfig {
            id: "inst-xyz".to_string(),
            name: "---".to_string(),
            data_dir: "/tmp/data".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(3),
        };
        assert_eq!(
            compute_ide_ending_sequence(&fallback_inst),
            "antigravity-inst-xyz"
        );
    }

    #[test]
    fn test_compute_instance_window_title() {
        let default_inst = InstanceConfig {
            id: "default".to_string(),
            name: "Default".to_string(),
            data_dir: "/tmp/data".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: true,
            pid: None,
            seq_num: Some(1),
        };
        assert_eq!(
            compute_instance_window_title(&default_inst),
            "#1 Default - Antigravity${separator}${dirty}${activeEditorShort}${separator}${rootName}"
        );

        let custom_inst = InstanceConfig {
            id: "inst-8136".to_string(),
            name: "8136".to_string(),
            data_dir: "/tmp/data".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(2),
        };
        assert_eq!(
            compute_instance_window_title(&custom_inst),
            "#2 8136 - antigravity-8136${separator}${dirty}${activeEditorShort}${separator}${rootName}"
        );
    }

    #[test]
    fn test_inject_instance_settings() {
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_settings_test_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let user_dir = temp_dir.join("User");
        // Justification: test fixture setup; a real failure fails the test at the next step
        crate::error::record_ignored(fs::create_dir_all(&user_dir), "create test dir");

        let inst = InstanceConfig {
            id: "inst-settings-test".to_string(),
            name: "Settings Test".to_string(),
            data_dir: temp_dir.to_string_lossy().to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(5),
        };

        assert!(inject_instance_settings(&inst).is_ok());

        let settings_path = user_dir.join("settings.json");
        assert!(settings_path.exists());
        let content = fs::read_to_string(&settings_path).unwrap();
        let val: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(
            val.get("window.title").and_then(|v| v.as_str()),
            Some("#5 Settings Test - antigravity-settings-test${separator}${dirty}${activeEditorShort}${separator}${rootName}")
        );

        // Justification: test cleanup; absence is the normal case
        crate::error::record_ignored(fs::remove_dir_all(&temp_dir), "clean test dir");
    }
}
