//! remediate_cmds — CLI command handlers, split from agm.rs.

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) fn remediate_repo_gitignore_native(args: &[String]) {
    let target_dir = args
        .iter()
        .find(|a| !a.starts_with('-') && *a != "agm" && *a != "agy")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let target_files = [
        "antigravity-resume_task.json",
        ".antigravity_resume_task.json",
        "antigravity_resume_task.json",
        ".antigravity-resume_task.json",
    ];

    println!(
        "[*] Remediating resume task files and .gitignore in {:?}...",
        target_dir
    );

    let mut tracked_files: Vec<String> = Vec::new();
    for tf in &target_files {
        let check = Command::new("git")
            .args(["-C", &target_dir.to_string_lossy(), "ls-files", "--", tf])
            .output();
        if let Ok(out) = check {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                tracked_files.push(tf.to_string());
            }
        }
    }

    let mut was_delete_committed = false;
    if !tracked_files.is_empty() {
        let mut rm_cmd = vec![
            "-C".to_string(),
            target_dir.to_string_lossy().to_string(),
            "rm".to_string(),
            "--cached".to_string(),
            "-f".to_string(),
            "--ignore-unmatch".to_string(),
            "--".to_string(),
        ];
        rm_cmd.extend(tracked_files.clone());
        let _ = Command::new("git").args(&rm_cmd).status();

        let commit_res = Command::new("git")
            .args([
                "-C",
                &target_dir.to_string_lossy(),
                "commit",
                "-m",
                "chore(git): remove antigravity-resume_task.json from repository",
            ])
            .status();
        if let Ok(st) = commit_res {
            was_delete_committed = st.success();
        }
    }

    for tf in &target_files {
        let p = target_dir.join(tf);
        if p.is_file() {
            let _ = fs::remove_file(p);
        }
    }

    let gitignore_path = target_dir.join(".gitignore");
    let mut content = fs::read_to_string(&gitignore_path).unwrap_or_default();
    let mut missing_entries = Vec::new();
    for tf in &target_files {
        let exists = content
            .lines()
            .any(|line| line.trim() == *tf || line.trim() == format!("/{}", tf));
        if !exists {
            missing_entries.push(*tf);
        }
    }

    let mut was_ignore_committed = false;
    if !missing_entries.is_empty() {
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        for me in &missing_entries {
            content.push_str(me);
            content.push('\n');
        }
        if fs::write(&gitignore_path, content).is_ok() {
            let _ = Command::new("git")
                .args(["-C", &target_dir.to_string_lossy(), "add", ".gitignore"])
                .status();
            let commit_res = Command::new("git")
                .args([
                    "-C",
                    &target_dir.to_string_lossy(),
                    "commit",
                    "-m",
                    "chore(git): ignore antigravity-resume_task.json in .gitignore",
                ])
                .status();
            if let Ok(st) = commit_res {
                was_ignore_committed = st.success();
            }
        }
    }

    let repo_name = target_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("repo");
    if was_delete_committed && was_ignore_committed {
        println!(
            "  ✓ [{}] Deleted antigravity-resume_task.json from Git & committed, then added to .gitignore & committed",
            repo_name
        );
    } else if was_delete_committed {
        println!(
            "  ✓ [{}] Deleted antigravity-resume_task.json from Git and committed",
            repo_name
        );
    } else if was_ignore_committed {
        println!(
            "  ✓ [{}] Added antigravity-resume_task.json to .gitignore and committed",
            repo_name
        );
    } else {
        println!(
            "  ✓ [{}] antigravity-resume_task.json is already in .gitignore and clean.",
            repo_name
        );
    }
}
