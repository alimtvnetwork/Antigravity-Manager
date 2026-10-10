use crate::error::AppError;
use crate::modules::repo_db;
use crate::modules::*;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

use super::*;

/// Format available prompt templates with slug, title, and preview snippet
pub fn format_prompts_templates_report() -> String {
    let templates = [
        (
            "read-all",
            "Enhanced Read Memory & Project Ingestion",
            "Executes mandatory pre-flight protocol: defensively reads project identity, recent git commits, CODE RED rules, specs in 02-spec/, pending plans in .ai-memory/, and ambiguities before touching codebase. Prevents hallucinations and guarantees full architectural context.",
        ),
        (
            "execute-pending-tasks",
            "Autonomous Queued Tasks Execution Loop",
            "Autonomous orchestration loop discovering, prioritizing, and executing all pending plans and subtasks in .ai-memory/plans/pending/. Enforces quality gates, isolated branch hygiene, pre-flight checks, and clean final-state commits with strict author attribution.",
        ),
        (
            "execute-parent-task",
            "Parent Task Decomposition & N-Step Loop",
            "Decomposes complex, multi-layered architectural initiatives into structured, isolated subtasks. Runs an autonomous continuous N-step self-loop until completion, validating each milestone against specifications, running unit tests, and preserving git hygiene across polyglot stacks.",
        ),
        (
            "ci-cd-fix",
            "Grounded 4-Part RCA & CI/CD Self-Healing",
            "Diagnose and repair CI/CD pipeline failures using grounded 4-part Root Cause Analysis without guessing. Inspects workflow logs, reproduces failures locally with test runner scripts, patches the root cause, and verifies passing GitHub Actions pipelines.",
        ),
        (
            "coding-guidelines",
            "Grounded Coding Guidelines Audit & Enforcement",
            "Repository-wide audit and enforcement of grounded coding standards: PascalCase database tables, positive boolean prefixes, AppError wrappers, universal response envelopes, zero-allocation string folding, and small function boundaries across Rust, Go, TypeScript, and Python.",
        ),
        (
            "smart-test-runner",
            "Smart Incremental Test Runner & Inventory",
            "Orchestrates incremental test execution across the repository using centralized manifests, dual-queue worker pools, heavy test isolation, and dynamic ETA sleep protocols. Tracks test durations to prioritize fast feedback and prevent CI timeouts.",
        ),
        (
            "minor-bump",
            "Automated Minor Release Ceremony",
            "Executes minor version bump, synchronizes package.json, Cargo.toml, tauri.conf.json, updates CHANGELOG.md with strict author attribution, tags git commit, and triggers release pipeline with zero untracked file drift.",
        ),
        (
            "is-done",
            "Task Completion & Quality Verification Gate",
            "Rigorous quality gate confirming all user requirements, acceptance criteria, unit tests, and linters pass cleanly. Verifies cargo fmt, cargo clippy, and frontend build before marking task completed in .ai-memory/.",
        ),
    ];

    let mut rows = String::new();
    for (i, (slug, title, preview)) in templates.iter().enumerate() {
        rows.push_str(&format!(
            "{}. 📌 <b>{}</b> (<code>{}</code>)\n   <i>\"{}\"</i>\n   • <b>Run:</b> <code>/prompt default {}</code>\n\n",
            i + 1,
            title,
            slug,
            preview,
            slug
        ));
    }

    format!(
        "📋 <b>Available Reusable Prompt Templates ({} Templates)</b>\n\n\
        {}\
        💡 <b>How to Run Templates:</b>\n\
        • <b>Local workspace:</b> <code>/prompt &lt;project-id&gt; &lt;template-slug&gt;</code>\n\
        • <b>Target specific node:</b> <code>&lt;node-alias&gt;:/prompt &lt;project-id&gt; &lt;template-slug&gt;</code>\n\
        • <b>With custom instruction:</b> <code>/prompt &lt;project-id&gt; &lt;template-slug&gt; with custom notes...</code>",
        templates.len(),
        rows
    )
}

/// Format active and queued prompts from state database
pub fn format_prompts_list() -> String {
    match repo_db::list_all_prompts() {
        Ok(prompts) if !prompts.is_empty() => {
            let total = prompts.len();
            let mut rows = String::new();
            for (i, p) in prompts.iter().take(10).enumerate() {
                let badge = match p.status.as_str() {
                    "running" => "🟢",
                    "dispatched" => "📤",
                    "backed_up" => "💾",
                    "completed" => "✅",
                    _ => "⚪",
                };
                let clean = repo_db::extract_clean_user_prompt(&p.prompt_content);
                rows.push_str(&format!(
                    "{}. [{}] <code>{}</code>\n   • <b>Project:</b> <code>{}</code>\n   • <i>\"{}\"</i>\n\n",
                    i + 1,
                    badge,
                    clean_for_telegram_html(&p.id, 24),
                    clean_for_telegram_html(&p.project_id, 32),
                    clean_for_telegram_html(&clean, 100)
                ));
            }
            format!(
                "📝 <b>State Database Prompt Queue ({} Prompts, Showing Top {})</b>\n\n\
                {}\
                💡 Send <code>/restore</code> to re-queue backed-up prompts.",
                total,
                prompts.len().min(10),
                rows
            )
        }
        _ => "📝 <b>Prompt Queue:</b> No prompts currently registered in split SQLite database."
            .to_string(),
    }
}

/// Resolve canonical prompt template from 01-prompts directory or fallback table
pub fn resolve_prompt_template_content(query: &str) -> Option<String> {
    let q = query.trim().to_lowercase();
    let q_clean = q.strip_prefix('/').unwrap_or(&q);

    // 1. Check known canonical templates
    let fallback = match q_clean {
        "read-all" | "read-memory-enhanced" | "read" => Some(
            "Execute enhanced read memory protocol: inspect project identity, recent git commits, CODE RED rules, specs in 02-spec/, pending plans in .ai-memory/, and ambiguities before touching codebase. Defensively ingest full architecture to guarantee grounded execution."
        ),
        "execute-pending-tasks" | "execute-pending" | "pending" => Some(
            "Execute pending tasks loop: systematically discover, catalog, and execute all pending plans and subtasks in .ai-memory/plans/pending/ with full QA gates, focused unit tests, and clean final-state commits."
        ),
        "execute-parent-task" | "parent-task" => Some(
            "Decompose parent task into discrete subtasks and run an autonomous N-step continuous loop until completion with strict specification adherence and coding guidelines."
        ),
        "ci-cd-fix" | "ci-fix" => Some(
            "Diagnose and repair CI/CD pipeline failures using grounded 4-part Root Cause Analysis without guessing. Inspect workflow failure logs, reproduce locally with test runner, patch defect, and verify passing pipeline."
        ),
        "coding-guidelines" | "cg" => Some(
            "Audit and enforce repository-wide grounded coding guidelines across all touched modules: PascalCase SQLite tables, positive boolean prefixes, AppError wrappers, zero-allocation strings, and small function boundaries."
        ),
        "smart-test-runner" | "test-runner" | "test" => Some(
            "Run smart incremental test runner across repository test inventory. Enforce heavy test isolation, worker pools, and ETA sleep protocols."
        ),
        "minor-bump" | "bump" => Some(
            "Execute minor version bump, synchronize manifests (package.json, Cargo.toml, tauri.conf.json), update CHANGELOG.md with strict author attribution, and trigger release pipeline."
        ),
        "is-done" | "done" => Some(
            "Verify all requirements and quality gates: ensure all touched modules pass cargo fmt, cargo clippy, npm run build, and unit tests with zero regressions before completing task."
        ),
        _ => None,
    };

    if let Some(fb) = fallback {
        return Some(fb.to_string());
    }

    // 2. Search on disk in 01-prompts/
    let mut search_roots = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        search_roots.push(cwd.join("01-prompts"));
        if let Some(parent) = cwd.parent() {
            search_roots.push(parent.join("coding-guidelines").join("01-prompts"));
            search_roots.push(parent.join("Antigravity-Manager").join("01-prompts"));
        }
    }

    for root in search_roots {
        if !root.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                let fname = entry.file_name().to_string_lossy().to_lowercase();
                if fname.contains(q_clean) {
                    if path.is_file() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            return Some(content.trim().to_string());
                        }
                    } else if path.is_dir() {
                        if let Ok(sub_entries) = std::fs::read_dir(&path) {
                            let mut md_files: Vec<std::path::PathBuf> = sub_entries
                                .flatten()
                                .map(|e| e.path())
                                .filter(|p| p.is_file())
                                .collect();
                            md_files.sort();
                            if let Some(first_file) = md_files.first() {
                                if let Ok(content) = std::fs::read_to_string(first_file) {
                                    return Some(content.trim().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Helper to wrap telegram prompt text with template prefixes, suffixes, or voice instruction notes
pub fn wrap_telegram_prompt(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let lower_slug = trimmed.to_lowercase();
    if !lower_slug.contains(' ') {
        if let Some(tpl) = resolve_prompt_template_content(&lower_slug) {
            return tpl;
        }
    }

    let mut prefix_slug: Option<String> = None;
    let mut suffix_slug: Option<String> = None;
    let mut clean_words: Vec<String> = Vec::new();

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let t = tokens[i];
        if t == "--prefix" || t == "-prefix" {
            if i + 1 < tokens.len() {
                prefix_slug = Some(tokens[i + 1].to_string());
                i += 2;
                continue;
            }
        } else if t == "--suffix" || t == "-suffix" {
            if i + 1 < tokens.len() {
                suffix_slug = Some(tokens[i + 1].to_string());
                i += 2;
                continue;
            }
        } else {
            clean_words.push(t.to_string());
        }
        i += 1;
    }

    let core_body = clean_words.join(" ");
    let mut parts = Vec::new();
    if let Some(ref pref) = prefix_slug {
        if let Some(tpl) = resolve_prompt_template_content(pref) {
            parts.push(tpl);
        } else {
            parts.push(format!("[Template Prefix: {}]", pref));
        }
    }
    if !core_body.is_empty() {
        parts.push(core_body);
    }
    if let Some(ref suff) = suffix_slug {
        if let Some(tpl) = resolve_prompt_template_content(suff) {
            parts.push(tpl);
        } else {
            parts.push(format!("[Template Suffix: {}]", suff));
        }
    }

    if parts.is_empty() {
        trimmed.to_string()
    } else {
        parts.join("\n\n")
    }
}
