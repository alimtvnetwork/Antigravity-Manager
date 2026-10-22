//! Instance management: lifecycle, switching, process cache, and settings.
//!
//! Thin facade — implementation lives in the submodules below. Every public
//! path (`crate::modules::instance::<item>`) keeps resolving via the
//! re-exports, so call sites are unaffected by the split.

#[path = "instance/active.rs"]
pub mod active;
#[path = "instance/arg_normalizer.rs"]
pub mod arg_normalizer;
#[path = "instance/cdp_onboarding.rs"]
pub mod cdp_onboarding;
#[path = "instance/clone.rs"]
pub mod clone;
#[path = "instance/child_health_monitor.rs"]
pub mod child_health_monitor;
#[path = "instance/child_process_tree_enumerator.rs"]
pub mod child_process_tree_enumerator;
#[path = "instance/child_termination_guard.rs"]
pub mod child_termination_guard;
#[path = "instance/clone_checksum.rs"]
pub mod clone_checksum;
#[path = "instance/clone_helpers.rs"]
pub mod clone_helpers;
#[path = "instance/close.rs"]
pub mod close;
#[path = "instance/cloned_guid_reseeder.rs"]
pub mod cloned_guid_reseeder;
#[path = "instance/cloned_telemetry_salt.rs"]
pub mod cloned_telemetry_salt;
#[path = "instance/crash_loop_backoff.rs"]
pub mod crash_loop_backoff;
#[path = "instance/crash_loop_guard.rs"]
pub mod crash_loop_guard;
#[path = "instance/create.rs"]
pub mod create;
#[path = "instance/credentials.rs"]
pub mod credentials;
#[path = "instance/db_clone.rs"]
pub mod db_clone;
#[path = "instance/env_allowlist_filter.rs"]
pub mod env_allowlist_filter;
#[path = "instance/env_whitelist_filter.rs"]
pub mod env_whitelist_filter;
#[path = "instance/executable.rs"]
pub mod executable;
#[path = "instance/extension_sanitizer.rs"]
pub mod extension_sanitizer;
#[path = "instance/fs_sync.rs"]
pub mod fs_sync;
#[path = "instance/health_probe.rs"]
pub mod health_probe;
#[path = "instance/identity_reseeder.rs"]
pub mod identity_reseeder;
#[path = "instance/instance_mgmt.rs"]
pub mod instance_mgmt;
#[path = "instance/ipc_socket_cleaner.rs"]
pub mod ipc_socket_cleaner;
#[path = "instance/keychain_file_guard.rs"]
pub mod keychain_file_guard;
#[path = "instance/keychain_path_resolver.rs"]
pub mod keychain_path_resolver;
#[path = "instance/keyring_flags.rs"]
pub mod keyring_flags;
#[path = "instance/launch.rs"]
pub mod launch;
pub(crate) use launch::launch_instance_inner_with_extra_workspaces;
#[path = "instance/launch_argv_builder.rs"]
pub mod launch_argv_builder;
#[path = "instance/launch_argv_sanitizer.rs"]
pub mod launch_argv_sanitizer;
#[path = "instance/launch_env_sanitizer.rs"]
pub mod launch_env_sanitizer;
#[path = "instance/lifecycle.rs"]
pub mod lifecycle;
#[path = "instance/lock_reclaimer.rs"]
pub mod lock_reclaimer;
#[path = "instance/merge.rs"]
pub mod merge;
#[path = "instance/misc.rs"]
pub mod misc;
#[path = "instance/keyring_mock_writer.rs"]
pub mod keyring_mock_writer;
#[path = "instance/mock_keyring_writer.rs"]
pub mod mock_keyring_writer;
#[path = "instance/observe.rs"]
pub mod observe;
#[path = "instance/orphan_socket_sweeper.rs"]
pub mod orphan_socket_sweeper;
#[path = "instance/path_sanitizer.rs"]
pub mod path_sanitizer;
#[path = "instance/project_path_canonical_normalizer.rs"]
pub mod project_path_canonical_normalizer;
#[path = "instance/paths.rs"]
pub mod paths;
#[path = "instance/pid_scan.rs"]
pub mod pid_scan;
#[path = "instance/pid_store.rs"]
pub mod pid_store;
#[path = "instance/pid_util.rs"]
pub mod pid_util;
#[path = "instance/pid_reclaim_barrier.rs"]
pub mod pid_reclaim_barrier;
#[path = "instance/pipe_socket_cleaner.rs"]
pub mod pipe_socket_cleaner;
#[path = "instance/port_collision_evictor.rs"]
pub mod port_collision_evictor;
#[path = "instance/port_detector.rs"]
pub mod port_detector;
#[path = "instance/port_matrix.rs"]
pub mod port_matrix;
#[path = "instance/port_reclaim_sentinel.rs"]
pub mod port_reclaim_sentinel;
#[path = "instance/port_reclaim_watchdog.rs"]
pub mod port_reclaim_watchdog;
#[path = "instance/port_lease_tracker.rs"]
pub mod port_lease_tracker;
#[path = "instance/remote_debugging_port_allocator.rs"]
pub mod remote_debugging_port_allocator;
#[path = "instance/post_switch_resume.rs"]
pub mod post_switch_resume;
#[path = "instance/process_cache.rs"]
pub mod process_cache;
#[path = "instance/process_record.rs"]
pub mod process_record;
#[path = "instance/process_vitality_probe.rs"]
pub mod process_vitality_probe;
#[path = "instance/profile_clone_manifest.rs"]
pub mod profile_clone_manifest;
#[path = "instance/profile_clone_verifier.rs"]
pub mod profile_clone_verifier;
#[path = "instance/profile_fingerprint_hasher.rs"]
pub mod profile_fingerprint_hasher;
#[path = "instance/profile_dir_validator.rs"]
pub mod profile_dir_validator;
#[path = "instance/project_uri.rs"]
pub mod project_uri;
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
#[path = "instance/runtime_crash_counter.rs"]
pub mod runtime_crash_counter;
#[path = "instance/sandbox_purger.rs"]
pub mod sandbox_purger;
#[path = "instance/sandboxed_storage_cleaner.rs"]
pub mod sandboxed_storage_cleaner;
#[path = "instance/session_file_reclaimer.rs"]
pub mod session_file_reclaimer;
#[path = "instance/session_cleanup_worker.rs"]
pub mod session_cleanup_worker;
#[path = "instance/session_uuid.rs"]
pub mod session_uuid;
#[path = "instance/session_trace_uuid.rs"]
pub mod session_trace_uuid;
#[path = "instance/settings.rs"]
pub mod settings;
#[path = "instance/settings_copy.rs"]
pub mod settings_copy;
#[path = "instance/smart_run.rs"]
pub mod smart_run;
#[path = "instance/spawn.rs"]
pub mod spawn;
#[path = "instance/state_snapshot.rs"]
pub mod state_snapshot;
#[path = "instance/state_snapshot_exporter.rs"]
pub mod state_snapshot_exporter;
#[path = "instance/state_snapshot_rollback.rs"]
pub mod state_snapshot_rollback;
#[path = "instance/storage.rs"]
pub mod storage;
#[path = "instance/storage_checksum_guard.rs"]
pub mod storage_checksum_guard;
#[path = "instance/storage_migration_guard.rs"]
pub mod storage_migration_guard;
#[path = "instance/storage_scrubber.rs"]
pub mod storage_scrubber;
#[path = "instance/telemetry_storage_cleaner.rs"]
pub mod telemetry_storage_cleaner;
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
#[cfg(test)]
#[path = "instance/tests_sandboxed_environment.rs"]
pub mod tests_sandboxed_environment;
#[cfg(test)]
#[path = "instance/tests_stale_pid_reaper.rs"]
pub mod tests_stale_pid_reaper;
#[cfg(test)]
#[path = "instance/tests_profile_hasher.rs"]
pub mod tests_profile_hasher;
#[cfg(test)]
#[path = "instance/tests_binary_execution_probe.rs"]
pub mod tests_binary_execution_probe;
#[cfg(test)]
#[path = "instance/tests_keyring_mock_writer.rs"]
pub mod tests_keyring_mock_writer;
#[cfg(test)]
#[path = "instance/tests_session_file_reclaimer.rs"]
pub mod tests_session_file_reclaimer;
#[cfg(test)]
#[path = "instance/tests_port_collision_evictor.rs"]
pub mod tests_port_collision_evictor;
#[cfg(test)]
#[path = "instance/tests_cloned_guid_reseeder.rs"]
pub mod tests_cloned_guid_reseeder;
#[cfg(test)]
#[path = "instance/tests_cloned_telemetry_salt.rs"]
pub mod tests_cloned_telemetry_salt;
#[cfg(test)]
#[path = "instance/tests_env_token_scrubber.rs"]
pub mod tests_env_token_scrubber;
#[cfg(test)]
#[path = "instance/tests_crash_loop_backoff.rs"]
pub mod tests_crash_loop_backoff;
#[cfg(test)]
#[path = "instance/tests_crash_loop_guard.rs"]
pub mod tests_crash_loop_guard;
#[cfg(test)]
#[path = "instance/tests_child_health_monitor.rs"]
pub mod tests_child_health_monitor;
#[cfg(test)]
#[path = "instance/tests_window_state_restorer.rs"]
pub mod tests_window_state_restorer;
#[cfg(test)]
#[path = "instance/tests_window_layout_anchor.rs"]
pub mod tests_window_layout_anchor;
#[cfg(test)]
#[path = "instance/tests_vitality_scanner.rs"]
pub mod tests_vitality_scanner;
#[cfg(test)]
#[path = "instance/tests_profile_fingerprint_hasher.rs"]
pub mod tests_profile_fingerprint_hasher;
#[cfg(test)]
#[path = "instance/tests_remote_debugging_port_allocator.rs"]
pub mod tests_remote_debugging_port_allocator;
#[cfg(test)]
#[path = "instance/tests_session_trace_uuid.rs"]
pub mod tests_session_trace_uuid;
#[cfg(test)]
#[path = "instance/tests_child_termination_guard.rs"]
pub mod tests_child_termination_guard;
#[cfg(test)]
#[path = "instance/tests_process_vitality_probe.rs"]
pub mod tests_process_vitality_probe;
#[cfg(test)]
#[path = "instance/tests_ipc_socket_cleaner.rs"]
pub mod tests_ipc_socket_cleaner;
#[cfg(test)]
#[path = "instance/tests_profile_dir_validator.rs"]
pub mod tests_profile_dir_validator;
#[cfg(test)]
#[path = "instance/tests_workspace_dir_validator.rs"]
pub mod tests_workspace_dir_validator;
#[cfg(test)]
#[path = "instance/tests_sandboxed_storage_cleaner.rs"]
pub mod tests_sandboxed_storage_cleaner;
#[cfg(test)]
#[path = "instance/tests_sandbox_purger.rs"]
pub mod tests_sandbox_purger;
#[path = "instance/terminate_fallback_ladder.rs"]
pub mod terminate_fallback_ladder;
#[path = "instance/tuning.rs"]
pub mod tuning;
#[path = "instance/vitality_scanner.rs"]
pub mod vitality_scanner;
#[path = "instance/window_layout_anchor.rs"]
pub mod window_layout_anchor;
#[path = "instance/window_state_restorer.rs"]
pub mod window_state_restorer;
#[path = "instance/workspace.rs"]
pub mod workspace;
#[path = "instance/workspace_continuity.rs"]
pub mod workspace_continuity;
#[path = "instance/workspace_dir_validator.rs"]
pub mod workspace_dir_validator;
#[path = "instance/workspace_hash.rs"]
pub mod workspace_hash;
#[path = "instance/workspace_hash_sentinel.rs"]
pub mod workspace_hash_sentinel;
#[path = "instance/workspace_storage_migrator.rs"]
pub mod workspace_storage_migrator;
#[path = "instance/workspace_storage_sanitizer.rs"]
pub mod workspace_storage_sanitizer;
#[path = "instance/env_token_scrubber.rs"]
pub mod env_token_scrubber;
#[path = "instance/socket_lock_recovery.rs"]
pub mod socket_lock_recovery;
#[path = "instance/profile_telemetry_isolation.rs"]
pub mod profile_telemetry_isolation;
#[path = "instance/process_spawn_ladder.rs"]
pub mod process_spawn_ladder;
#[path = "instance/port_allocation_guard.rs"]
pub mod port_allocation_guard;
#[path = "instance/workspace_directory_guard.rs"]
pub mod workspace_directory_guard;
#[path = "instance/env_path_normalizer.rs"]
pub mod env_path_normalizer;
#[path = "instance/stale_pid_reaper.rs"]
pub mod stale_pid_reaper;
#[path = "instance/profile_state_hasher.rs"]
pub mod profile_state_hasher;
#[path = "instance/binary_execution_probe.rs"]
pub mod binary_execution_probe;
#[path = "instance/remote_port_guard.rs"]
pub mod remote_port_guard;
#[path = "instance/directory_permission_guard.rs"]
pub mod directory_permission_guard;

pub use crate::models::instance::{InstanceConfig, InstanceRegistry, InstanceStatus};
pub use arg_normalizer::*;
pub use clone_checksum::*;
pub use extension_sanitizer::*;
pub use health_probe::*;
pub use identity_reseeder::*;
pub use keyring_flags::*;
pub use lock_reclaimer::*;
pub use path_sanitizer::*;
pub use port_detector::*;
pub use port_matrix::*;
pub use project_uri::*;
pub use sandbox_purger::*;
pub use session_uuid::*;
pub use state_snapshot::*;
pub use storage_scrubber::*;
pub use vitality_scanner::*;
pub use workspace_continuity::*;
pub use workspace_hash::*;

pub use active::{
    bind_account_to_instance, get_active_instance_id, set_active_instance_id, set_default_instance,
};
pub use cdp_onboarding::{
    dismiss_security_notice_dialog_for_data_dir, spawn_security_notice_dismisser,
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
#[cfg(target_os = "windows")]
pub use close::force_terminate_pid_win32;
pub use close::{
    close_instance, close_instance_verified, detect_caller_instance_id, refuse_if_caller_instance,
};
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
pub(crate) use observe::verify_instance_relaunched;
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
pub(crate) use pid_util::{first_live_window_pid, force_refresh_process_cache};
pub use pid_util::{
    is_instance_running, pid_refresh_interval_seconds, process_identity_matches,
    refresh_pid_cache_if_due, refresh_saved_instance_pids, resolve_instance_pid_for_switch,
    saved_pid_matches,
};
pub use post_switch_resume::{cancel_post_switch_resume, schedule_post_switch_resume};
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
    clear_instance_onboarding_acceptance, get_default_antigravity_data_dir,
    purge_volatile_instance_sessions, record_onboarding_acceptance_fallback,
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
    restore_and_inject_prompts_for_instance, schedule_prompt_restore_after_launch,
    snapshot_active_workspaces,
};
pub use child_health_monitor::*;
pub use child_process_tree_enumerator::*;
pub use child_termination_guard::*;
pub use cloned_guid_reseeder::*;
pub use cloned_telemetry_salt::*;
pub use crash_loop_backoff::*;
pub use crash_loop_guard::*;
pub use env_allowlist_filter::*;
pub use env_whitelist_filter::*;
pub use ipc_socket_cleaner::*;
pub use keychain_file_guard::*;
pub use keychain_path_resolver::*;
pub use keyring_mock_writer::*;
pub use launch_argv_builder::*;
pub use launch_argv_sanitizer::*;
pub use launch_env_sanitizer::*;
pub use mock_keyring_writer::*;
pub use orphan_socket_sweeper::*;
pub use pid_reclaim_barrier::*;
pub use pipe_socket_cleaner::*;
pub use port_collision_evictor::*;
pub use port_lease_tracker::*;
pub use port_reclaim_sentinel::*;
pub use port_reclaim_watchdog::*;
pub use process_vitality_probe::*;
pub use profile_clone_manifest::*;
pub use profile_clone_verifier::*;
pub use profile_dir_validator::*;
pub use profile_fingerprint_hasher::*;
pub use project_path_canonical_normalizer::*;
pub use remote_debugging_port_allocator::*;
pub use runtime_crash_counter::*;
pub use sandboxed_storage_cleaner::*;
pub use session_cleanup_worker::*;
pub use session_file_reclaimer::*;
pub use session_trace_uuid::*;
pub use state_snapshot_exporter::*;
pub use state_snapshot_rollback::*;
pub use storage_checksum_guard::*;
pub use storage_migration_guard::*;
pub use telemetry_storage_cleaner::*;
pub use terminate_fallback_ladder::*;
pub use window_layout_anchor::*;
pub use window_state_restorer::*;
pub use vitality_scanner::*;
pub use workspace_dir_validator::*;
pub use workspace_hash_sentinel::*;
pub use workspace_storage_migrator::*;
pub use workspace_storage_sanitizer::*;
pub use env_token_scrubber::*;
pub use socket_lock_recovery::*;
pub use profile_telemetry_isolation::*;
pub use process_spawn_ladder::*;
pub use port_allocation_guard::*;
pub use workspace_directory_guard::*;
pub use env_path_normalizer::*;
pub use stale_pid_reaper::*;
pub use profile_state_hasher::*;
pub use binary_execution_probe::*;
pub use remote_port_guard::*;
pub use directory_permission_guard::*;
