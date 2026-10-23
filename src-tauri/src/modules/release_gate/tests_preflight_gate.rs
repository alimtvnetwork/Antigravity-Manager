#[cfg(test)]
mod tests {
    use super::super::preflight_gate_runner::{classify_preflight_requirements, PreflightRequirements};

    #[test]
    fn test_rust_only_changes() {
        let paths = &["src-tauri/src/main.rs", "src-tauri/src/modules/proxy.rs"];
        let reqs = classify_preflight_requirements(paths);

        assert!(reqs.requires_rust_fmt);
        assert!(reqs.requires_cargo_clippy);
        assert!(!reqs.requires_frontend_build);
        assert!(!reqs.is_trivial_only);
        assert_eq!(reqs.planned_commands.len(), 2);
    }

    #[test]
    fn test_frontend_only_changes() {
        let paths = &["src/components/common/Toast.tsx", "package.json"];
        let reqs = classify_preflight_requirements(paths);

        assert!(!reqs.requires_rust_fmt);
        assert!(!reqs.requires_cargo_clippy);
        assert!(reqs.requires_frontend_build);
        assert!(!reqs.is_trivial_only);
        assert_eq!(reqs.planned_commands, vec!["npm run build"]);
    }

    #[test]
    fn test_both_rust_and_frontend_changes() {
        let paths = &["src-tauri/Cargo.toml", "src/App.tsx"];
        let reqs = classify_preflight_requirements(paths);

        assert!(reqs.requires_rust_fmt);
        assert!(reqs.requires_cargo_clippy);
        assert!(reqs.requires_frontend_build);
        assert!(!reqs.is_trivial_only);
        assert_eq!(reqs.planned_commands.len(), 3);
    }

    #[test]
    fn test_docs_and_specs_only_trivial() {
        let paths = &[
            "02-spec/21-app/spec.md",
            ".ai-memory/plans/readme.md",
            "README.md",
            "changelog.md",
        ];
        let reqs = classify_preflight_requirements(paths);

        assert!(!reqs.requires_rust_fmt);
        assert!(!reqs.requires_cargo_clippy);
        assert!(!reqs.requires_frontend_build);
        assert!(reqs.is_trivial_only);
        assert!(reqs.planned_commands.is_empty());
    }

    #[test]
    fn test_windows_backslashes_normalized() {
        let paths = &["src-tauri\\src\\lib.rs"];
        let reqs = classify_preflight_requirements(paths);

        assert!(reqs.requires_rust_fmt);
        assert!(reqs.requires_cargo_clippy);
    }
}
