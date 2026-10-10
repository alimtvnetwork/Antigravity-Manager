//! Split Repo DB Module
//! Dedicated SQLite State Database for tracking running projects,
//! backing up active prompts, and directly dispatching prompts upon profile switch.
//!
//! This is a thin facade: domain logic lives in submodules below.
//! All public paths (`crate::modules::repo_db::<item>`) are re-exported here.

#![allow(dead_code)]

pub mod agy;
pub mod auto_resume;
pub mod backup;
pub mod clone_rows;
pub mod detection;
pub mod discovery;
pub mod dispatch;
pub mod failed_commands;
pub mod gemini_dirs;
pub mod goal_heartbeat;
pub mod liveness;
pub mod models;
pub mod project_queries;
pub mod prompts_crud;
pub mod queue;
pub mod resend;
pub mod restore;
pub mod schema;
pub mod sequences;
pub mod state;
pub mod text_utils;
pub mod transcript;
pub mod tree;
pub mod tree_collect;
pub mod tree_format;
pub mod tree_node_conv;
pub mod tree_node_liveness;
pub mod tree_nodes;
pub mod tree_phases;
pub mod tree_text;

// ---- models ----
pub use models::{
    ActivePrompt, AgmConversationNode, AgmProjectTreeNode, AgmSequenceResolution,
    AutoResumePromptInfo, AutoResumeResult, FailedCommandRecord, ProjectExecutionInfo,
    ProjectLivenessEvaluation, PromptGoalHeartbeatConfig, PromptLivenessProbe, RunningProject,
    SwitchPromptSnap, TranscriptInspection,
};
// ---- state ----
pub use state::{get_dispatched_prompts_cache, get_memory_prompt, reset_dispatched_prompts_cache};
// ---- schema ----
pub use schema::{connect_db, get_repo_db_path, purge_corrupted_running_projects};
// ---- failed_commands ----
pub use failed_commands::{
    clear_failed_commands, count_failed_commands, decode_uri_to_path_pub, list_failed_commands,
    log_failed_command,
};
// ---- detection ----
pub use detection::{
    detect_running_projects, extract_image_payload_or_path, live_prompt_text, resume_task_document,
};
// ---- gemini_dirs ----
pub use gemini_dirs::{gemini_dirs_for_instance, gemini_dirs_tagged, get_canonical_host_home};
// ---- discovery ----
pub use discovery::discover_running_prompts_from_antigravity;
// ---- backup ----
pub use backup::{
    backup_running_prompts, count_backed_up_prompts, needs_prompt_channel_wait,
    verify_prompts_running,
};
// ---- dispatch ----
pub use dispatch::{dispatch_running_prompts, parse_flexible_timestamp};
// ---- liveness ----
pub use liveness::is_prompt_running_for_project;
// ---- queue ----
pub use queue::{
    check_and_dispatch_enqueued_prompts, count_enqueued_prompts, enqueue_prompt_for_instance_full,
    requeue_running_conversations_for_instance,
};
// ---- project_queries ----
pub use project_queries::{
    get_live_project_execution_info, get_project_execution_status, is_any_prompt_actively_running,
    list_all_prompts, list_running_projects,
};
// ---- restore ----
pub use restore::{
    list_backed_up_prompts, restore_prompts_from_backup_json, switch_prompt_snapshot,
};
// ---- prompts_crud ----
pub use prompts_crud::{insert_active_prompt, list_active_prompts, save_or_requeue_prompt};
// ---- text_utils ----
pub use text_utils::{
    extract_clean_user_prompt, extract_smart_prompt_summary, format_friendly_workspace_label,
    get_git_branch_for_path, lookup_conversation_title,
};
// ---- agy ----
pub use agy::{
    copy_to_system_clipboard, enqueue_prompt_for_instance, send_prompt_now_for_instance,
    spawn_prompt_via_agy,
};
// ---- resend ----
pub use resend::{resend_all_running_commands, resend_running_commands_for_instance};
// ---- auto_resume ----
pub use auto_resume::auto_resume_recent_prompts;
// ---- sequences ----
pub use sequences::{extract_prompt_tail_snippet, extract_prompt_words_preview};
// ---- transcript ----
pub use transcript::resolve_transcript_path;
// ---- tree ----
pub use tree::{
    get_project_conversation_tree, get_project_conversation_tree_cached,
    invalidate_prompt_tree_cache,
};
// ---- tree_format ----
pub use tree_format::{
    get_project_conversation_tree_for_instance, group_identical_conversation_runs,
    list_running_prompts_for_instance, prompt_target_by_sequence_scoped,
    resolve_agm_sequence_target,
};
// ---- tree_text ----
pub use tree_text::{format_tree_view_cli, format_tree_view_telegram_html};
// ---- goal_heartbeat ----
pub use goal_heartbeat::{
    ensure_prompt_goals_running_for_instance, inspect_prompt_goal_status,
    run_prompt_goal_worker_loop, start_prompt_goal_heartbeat,
    stop_prompt_goal_workers_for_instance,
};
// ---- clone_rows ----
pub use clone_rows::clone_instance_repo_rows;

// ---- pub(crate) items (internal cross-module + test access) ----
pub(crate) use clone_rows::clone_repo_rows_on;
pub(crate) use discovery::read_conversation_summary_rows;
pub(crate) use failed_commands::decode_uri_to_path;
pub(crate) use project_queries::normalize_path_for_compare;
pub(crate) use schema::init_tables;
pub(crate) use sequences::{ensure_conversation_sequence_in_conn, ensure_project_sequence_in_conn};
pub(crate) use state::{get_active_agy_workers, get_memory_prompts_map};
pub(crate) use transcript::inspect_conversation_transcript;
pub(crate) use tree::compute_project_conversation_tree;
pub(crate) use tree_collect::{collect_tree_conversation_items, RawConvItem};
pub(crate) use tree_node_conv::{build_active_prompt_conv_nodes, build_transcript_conv_nodes};
pub(crate) use tree_node_liveness::evaluate_project_liveness;
pub(crate) use tree_nodes::{build_project_tree_node, TreeNodeCtx};
pub(crate) use tree_phases::{
    discover_fallback_projects, log_tree_telemetry_probe, query_tree_projects,
    transition_stale_inflight_prompts,
};
