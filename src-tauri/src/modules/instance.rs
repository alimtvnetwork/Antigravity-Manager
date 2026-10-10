//! Instance management: lifecycle, switching, process cache, and settings.
//!
//! Thin facade — implementation lives in the submodules below. Every public
//! path (`crate::modules::instance::<item>`) keeps resolving via the
//! re-exports, so call sites are unaffected by the split.

#[path = "instance/active.rs"]
pub mod active;
#[path = "instance/clone.rs"]
pub mod clone;
#[path = "instance/clone_helpers.rs"]
pub mod clone_helpers;
#[path = "instance/close.rs"]
pub mod close;
#[path = "instance/create.rs"]
pub mod create;
#[path = "instance/credentials.rs"]
pub mod credentials;
#[path = "instance/db_clone.rs"]
pub mod db_clone;
#[path = "instance/executable.rs"]
pub mod executable;
#[path = "instance/fs_sync.rs"]
pub mod fs_sync;
#[path = "instance/instance_mgmt.rs"]
pub mod instance_mgmt;
#[path = "instance/launch.rs"]
pub mod launch;
pub(crate) use launch::launch_instance_inner_with_extra_workspaces;
#[path = "instance/lifecycle.rs"]
pub mod lifecycle;
#[path = "instance/merge.rs"]
pub mod merge;
#[path = "instance/misc.rs"]
pub mod misc;
#[path = "instance/observe.rs"]
pub mod observe;
#[path = "instance/paths.rs"]
pub mod paths;
#[path = "instance/pid_scan.rs"]
pub mod pid_scan;
#[path = "instance/pid_store.rs"]
pub mod pid_store;
#[path = "instance/pid_util.rs"]
pub mod pid_util;
#[path = "instance/process_cache.rs"]
pub mod process_cache;
#[path = "instance/process_record.rs"]
pub mod process_record;
#[path = "instance/projects.rs"]
pub mod projects;
#[path = "instance/projects_copy.rs"]
pub mod projects_copy;
#[path = "instance/protection.rs"]
pub mod protection;
#[path = "instance/query.rs"]
pub mod query;
#[path = "instance/registry.rs"]
pub mod registry;
#[path = "instance/settings.rs"]
pub mod settings;
#[path = "instance/settings_copy.rs"]
pub mod settings_copy;
#[path = "instance/smart_run.rs"]
pub mod smart_run;
#[path = "instance/spawn.rs"]
pub mod spawn;
#[path = "instance/storage.rs"]
pub mod storage;
#[path = "instance/switch.rs"]
pub mod switch;
#[path = "instance/switch_guards.rs"]
pub mod switch_guards;
#[path = "instance/sync.rs"]
pub mod sync;
#[cfg(test)]
#[path = "instance/tests_a.rs"]
pub mod tests_a;
#[cfg(test)]
#[path = "instance/tests_b.rs"]
pub mod tests_b;
#[cfg(test)]
#[path = "instance/tests_clone_a.rs"]
pub mod tests_clone_a;
#[cfg(test)]
#[path = "instance/tests_clone_b.rs"]
pub mod tests_clone_b;
#[cfg(test)]
#[path = "instance/tests_e2e.rs"]
pub mod tests_e2e;
#[path = "instance/tuning.rs"]
pub mod tuning;
#[path = "instance/workspace.rs"]
pub mod workspace;

pub use crate::models::instance::{InstanceConfig, InstanceRegistry, InstanceStatus};
pub use active::{
    bind_account_to_instance, get_active_instance_id, set_active_instance_id, set_default_instance,
};
pub use clone::copy_instance_with_options;
pub(crate) use clone_helpers::{
    copy_gemini_trees, copy_required_ide_files, pick_best_settings_path, source_profile_home,
};
pub use clone_helpers::{
    copy_source_ide_trees, copy_source_user_settings, rename_instance,
    sanitize_cloned_instance_summaries, GEMINI_CLONE_DIRS, REQUIRED_IDE_REL_PATHS,
};
pub(crate) use close::clear_stale_instance_lockfiles;
pub use close::{close_instance, close_instance_verified};
pub use create::{create_instance, create_instance_with_account, stop_instance};
pub(crate) use credentials::{inject_account_credentials, CredentialInjectOptions};
pub use db_clone::{copy_dir_recursive, safe_clone_sqlite_db};
pub(crate) use executable::resolve_launch_executable;
pub use executable::{
    clone_instance_executable, export_instances_json, import_instances_json,
    set_instance_executable,
};
pub use fs_sync::{
    ensure_default_instance_exists, sync_directory_recursive, sync_instance_ide_parity,
};
pub use instance_mgmt::{delete_instance, wipe_instance_session};
pub use lifecycle::{clone_instance, copy_instance, restart_instance};
pub use merge::{
    deep_merge_json, merge_state_vscdb_recent_paths, merge_storage_json_recent_paths,
    purge_recent_project_paths,
};
pub use misc::{count_instances, resolve_instance_exe_name};
pub use observe::{observe_instance, resolve_instance_id, ObservedInstanceState};
pub use paths::{
    get_instance_db_path, get_instance_home_dir, get_instances_dir, get_registry_path,
    open_instance_db,
};
pub use pid_scan::find_pids_for_data_dir;
pub(crate) use pid_scan::{
    get_cached_antigravity_processes, CachedProcessInfo, PROCESS_SCAN_CACHE,
};
pub use pid_store::{get_instance_saved_pid, mark_instance_stopped, record_instance_pid};
pub(crate) use pid_util::force_refresh_process_cache;
pub use pid_util::{
    is_instance_running, pid_refresh_interval_seconds, process_identity_matches,
    refresh_pid_cache_if_due, refresh_saved_instance_pids, resolve_instance_pid_for_switch,
    saved_pid_matches,
};
pub use process_cache::is_instance_process_running_smart;
pub(crate) use process_cache::{check_cached_pid_alive, focus_running_instance};
#[cfg(test)]
pub use process_record::MOCK_PID_ALIVE;
pub use process_record::{
    ensure_instance_running_for_dispatch, get_cached_instance_process,
    get_or_detect_instance_process, invalidate_instance_process_cache, is_pid_alive_os,
    is_pid_alive_targeted, update_cached_instance_process, InstanceProcessCacheItem,
    InstanceProcessRecord, INSTANCE_PROCESS_CACHE, SMART_PROCESS_CACHE,
};
pub use projects::{
    assign_project_to_instance, get_instance_running_process_count,
    scan_and_cache_all_running_instances, wait_for_instance_prompt_channel,
    warm_up_smart_process_cache,
};
pub use projects_copy::{copy_instance_projects, find_instance_by_executable};
pub use protection::{other_instance_protection, should_spare_pid};
pub use query::list_instances;
pub use registry::{export_instances_envelope, load_registry, save_registry};
pub use settings::{
    compute_ide_ending_sequence, compute_instance_window_title, find_instance_settings_path,
    get_instance_settings_targets, inject_instance_settings,
};
pub use settings_copy::{copy_instance_settings, enforce_default_settings};
pub(crate) use smart_run::get_macos_candidate_paths;
pub use smart_run::{
    ensure_instance_running_smart, focus_or_launch_instance,
    focus_or_launch_instance_with_workspace, focus_or_launch_workspace, launch_instance,
    launch_instance_with_workspaces, launch_instance_without_prompt_reinject,
};
pub(crate) use spawn::record_spawned_instance_process;
pub use storage::{
    get_default_antigravity_data_dir, purge_volatile_instance_sessions,
    update_instance_app_storage, write_keyring_bypass_markers,
};
pub use switch::switch_account_to_instance;
pub(crate) use switch_guards::check_switch_lease_guard;
pub use sync::{sync_all_instances_and_quotas_logic, sync_instance_pid_and_quota_logic};
pub use tuning::{
    export_instance_settings, get_instance_theme, import_instance_settings,
    set_instance_plan_review, set_instance_theme, set_instance_turbo_mode,
};
pub use workspace::{
    get_instance_workspace_folders, get_instance_workspace_paths, migrate_instance_workspaces,
    restore_and_inject_prompts_for_instance, snapshot_active_workspaces,
};
