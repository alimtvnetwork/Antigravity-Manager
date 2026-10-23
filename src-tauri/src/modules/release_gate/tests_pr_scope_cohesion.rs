//! Unit tests for Single Problem Scope & PR Cohesion Verifier.

use crate::modules::release_gate::pr_scope_cohesion_verifier::{
    audit_pr_scope, calculate_cohesion_score, check_mixed_concerns, classify_changeset,
    classify_file_path, validate_commit_revertability, ChangesetCategory, CommitMetadata,
    FileChangeset,
};

#[test]
fn test_classify_file_path() {
    // Governance
    assert_eq!(classify_file_path("AGENTS.md"), ChangesetCategory::Governance);
    assert_eq!(classify_file_path("./agents.md"), ChangesetCategory::Governance);
    assert_eq!(classify_file_path(".agents/skills/my-skill/skill.md"), ChangesetCategory::Governance);
    assert_eq!(classify_file_path("02-spec/21-app/spec.md"), ChangesetCategory::Governance);
    assert_eq!(classify_file_path(".ai-memory/plans/subtask.md"), ChangesetCategory::Governance);
    assert_eq!(classify_file_path("LICENSE"), ChangesetCategory::Governance);
    assert_eq!(classify_file_path(".github/pull_request_template.md"), ChangesetCategory::Governance);

    // ReleaseTooling
    assert_eq!(classify_file_path("scripts/release.py"), ChangesetCategory::ReleaseTooling);
    assert_eq!(classify_file_path(".github/workflows/ci.yml"), ChangesetCategory::ReleaseTooling);
    assert_eq!(classify_file_path("src-tauri/src/modules/release_gate/channel_gate.rs"), ChangesetCategory::ReleaseTooling);
    assert_eq!(classify_file_path("package.json"), ChangesetCategory::ReleaseTooling);
    assert_eq!(classify_file_path("Cargo.toml"), ChangesetCategory::ReleaseTooling);
    assert_eq!(classify_file_path("Dockerfile"), ChangesetCategory::ReleaseTooling);

    // Documentation
    assert_eq!(classify_file_path("README.md"), ChangesetCategory::Documentation);
    assert_eq!(classify_file_path("README_EN.md"), ChangesetCategory::Documentation);
    assert_eq!(classify_file_path("docs/architecture_guide.md"), ChangesetCategory::Documentation);
    assert_eq!(classify_file_path("changelog.md"), ChangesetCategory::Documentation);

    // UIComponents
    assert_eq!(classify_file_path("src/components/common/PillButton.tsx"), ChangesetCategory::UIComponents);
    assert_eq!(classify_file_path("src/stores/error-store.ts"), ChangesetCategory::UIComponents);
    assert_eq!(classify_file_path("src/views/SettingsModal.tsx"), ChangesetCategory::UIComponents);
    assert_eq!(classify_file_path("src/styles/tailwind.css"), ChangesetCategory::UIComponents);

    // CorePipeline
    assert_eq!(classify_file_path("src-tauri/src/proxy/server/router_proxy.rs"), ChangesetCategory::CorePipeline);
    assert_eq!(classify_file_path("src-tauri/src/modules/pipeline/thinking_budget.rs"), ChangesetCategory::CorePipeline);
    assert_eq!(classify_file_path("src-tauri/src/engine/model_routing.rs"), ChangesetCategory::CorePipeline);
}

#[test]
fn test_category_isolation_properties() {
    assert!(ChangesetCategory::Governance.is_isolated_concern());
    assert!(ChangesetCategory::ReleaseTooling.is_isolated_concern());
    assert!(ChangesetCategory::Documentation.is_isolated_concern());
    assert!(!ChangesetCategory::CorePipeline.is_isolated_concern());
    assert!(!ChangesetCategory::UIComponents.is_isolated_concern());

    // Same category can always mix
    assert!(ChangesetCategory::CorePipeline.can_mix_with(ChangesetCategory::CorePipeline));
    assert!(ChangesetCategory::Governance.can_mix_with(ChangesetCategory::Governance));

    // CorePipeline and UIComponents can mix in full-stack features
    assert!(ChangesetCategory::CorePipeline.can_mix_with(ChangesetCategory::UIComponents));
    assert!(ChangesetCategory::UIComponents.can_mix_with(ChangesetCategory::CorePipeline));

    // Governance and ReleaseTooling cannot mix with functional code
    assert!(!ChangesetCategory::Governance.can_mix_with(ChangesetCategory::CorePipeline));
    assert!(!ChangesetCategory::Governance.can_mix_with(ChangesetCategory::UIComponents));
    assert!(!ChangesetCategory::ReleaseTooling.can_mix_with(ChangesetCategory::CorePipeline));
    assert!(!ChangesetCategory::ReleaseTooling.can_mix_with(ChangesetCategory::UIComponents));
    assert!(!ChangesetCategory::Governance.can_mix_with(ChangesetCategory::ReleaseTooling));
}

#[test]
fn test_mixed_concern_detection() {
    // Pure CorePipeline changeset
    let clean_files = classify_changeset(&[
        "src-tauri/src/proxy/server/router_proxy.rs",
        "src-tauri/src/modules/pipeline/thinking_filter.rs",
    ]);
    let violations = check_mixed_concerns(&clean_files);
    assert!(violations.is_empty());

    // Mixed changeset: CorePipeline + Governance (AGENTS.md)
    let mixed_files = classify_changeset(&[
        "src-tauri/src/proxy/server/router_proxy.rs",
        "AGENTS.md",
    ]);
    let mixed_violations = check_mixed_concerns(&mixed_files);
    assert_eq!(mixed_violations.len(), 1);
    assert_eq!(mixed_violations[0].isolated_category, ChangesetCategory::Governance);
    assert_eq!(mixed_violations[0].conflicting_category, ChangesetCategory::CorePipeline);
}

#[test]
fn test_cohesion_scoring() {
    // 100% cohesive PR
    let files = classify_changeset(&[
        "src-tauri/src/proxy/server/router_proxy.rs",
        "src-tauri/src/proxy/handlers/gemini.rs",
        "src-tauri/src/engine/route_balancer.rs",
    ]);
    let (primary, score) = calculate_cohesion_score(&files);
    assert_eq!(primary, Some(ChangesetCategory::CorePipeline));
    assert!((score - 1.0).abs() < f64::EPSILON);

    // Mixed concern PR gets penalized score
    let mixed = classify_changeset(&[
        "src-tauri/src/proxy/server/router_proxy.rs",
        "AGENTS.md",
    ]);
    let (_, mixed_score) = calculate_cohesion_score(&mixed);
    // Base 0.5 * 0.5 (penalty) = 0.25
    assert!(mixed_score <= 0.25);
}

#[test]
fn test_commit_revertability_clean() {
    let commit = CommitMetadata {
        hash: "a1b2c3d".to_string(),
        message: "feat(proxy): implement wildcard model routing for Gemini 2.0".to_string(),
        files_changed: vec![
            "src-tauri/src/proxy/server/router_proxy.rs".to_string(),
            "src-tauri/src/engine/route_balancer.rs".to_string(),
        ],
        author: Some("Alim".to_string()),
    };

    let report = validate_commit_revertability(&commit);
    assert!(report.is_revertable);
    assert!(report.violations.is_empty());
}

#[test]
fn test_commit_revertability_rejects_noisy_message() {
    let commit = CommitMetadata {
        hash: "e5f6g7h".to_string(),
        message: "wip".to_string(),
        files_changed: vec!["src-tauri/src/proxy/server/router_proxy.rs".to_string()],
        author: Some("Alim".to_string()),
    };

    let report = validate_commit_revertability(&commit);
    assert!(!report.is_revertable);
    assert!(report.violations.iter().any(|v| v.contains("lacks self-contained rationale")));
}

#[test]
fn test_commit_revertability_rejects_tangled_concerns() {
    let commit = CommitMetadata {
        hash: "1122334".to_string(),
        message: "feat: update proxy router and modify governance agents doc".to_string(),
        files_changed: vec![
            "src-tauri/src/proxy/server/router_proxy.rs".to_string(),
            "AGENTS.md".to_string(),
        ],
        author: Some("Alim".to_string()),
    };

    let report = validate_commit_revertability(&commit);
    assert!(!report.is_revertable);
    assert!(report.violations.iter().any(|v| v.contains("Tangled changeset")));
}

#[test]
fn test_audit_pr_scope_success() {
    let commits = vec![
        CommitMetadata {
            hash: "1111111".to_string(),
            message: "feat(pipeline): implement thinking budget parser".to_string(),
            files_changed: vec!["src-tauri/src/modules/pipeline/thinking_budget.rs".to_string()],
            author: Some("Alim".to_string()),
        },
        CommitMetadata {
            hash: "2222222".to_string(),
            message: "feat(proxy): route thinking blocks through adapter".to_string(),
            files_changed: vec!["src-tauri/src/proxy/server/router_proxy.rs".to_string()],
            author: Some("Alim".to_string()),
        },
    ];

    let files = &[
        "src-tauri/src/modules/pipeline/thinking_budget.rs",
        "src-tauri/src/proxy/server/router_proxy.rs",
    ];

    let audit = audit_pr_scope(&commits, files);
    assert!(audit.is_single_problem_scope);
    assert_eq!(audit.primary_category, Some(ChangesetCategory::CorePipeline));
    assert!(audit.mixed_concerns.is_empty());
    assert!(audit.summary.contains("PR adheres to Single Problem Scope"));
}

#[test]
fn test_audit_pr_scope_failure_mixed_governance() {
    let commits = vec![
        CommitMetadata {
            hash: "3333333".to_string(),
            message: "fix(proxy): update model endpoints and update AGENTS.md rules".to_string(),
            files_changed: vec![
                "src-tauri/src/proxy/server/router_proxy.rs".to_string(),
                "AGENTS.md".to_string(),
            ],
            author: Some("Alim".to_string()),
        },
    ];

    let files = &[
        "src-tauri/src/proxy/server/router_proxy.rs",
        "AGENTS.md",
    ];

    let audit = audit_pr_scope(&commits, files);
    assert!(!audit.is_single_problem_scope);
    assert!(!audit.mixed_concerns.is_empty());
    assert!(audit.summary.contains("violates AGENTS.md Single Problem Scope"));
}
