//! Repo DB: tree

use super::models::AgmProjectTreeNode;
use super::schema::connect_db;
use chrono::Utc;
use rusqlite::params;
use std::sync::{Mutex, OnceLock};

static TREE_COMPUTE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn read_cached_tree(cache_key: &str, inst_key: &str, now: i64) -> Option<Vec<AgmProjectTreeNode>> {
    let conn = connect_db().ok()?;
    let (tree_json, updated_at, ttl_seconds): (String, i64, i64) = conn
        .query_row(
            "SELECT tree_json, updated_at, ttl_seconds FROM prompt_tree_cache WHERE cache_key = ?1 AND instance_id = ?2",
            params![cache_key, inst_key],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok()?;
    if now - updated_at < ttl_seconds {
        serde_json::from_str::<Vec<AgmProjectTreeNode>>(&tree_json).ok()
    } else {
        None
    }
}

/// Build the AGM Project -> Conversation -> 200-Word Prompt Tree across ALL registered instances
/// and persist Dual AGM (`P001`/`C001`) + GitMap (`GM:#1`/`GM:<short_id>`) Sequence IDs in `repo_prompts.db`.
/// Checks `prompt_tree_cache` first. If cache is valid (within TTL and not forced), returns cached JSON.
pub fn get_project_conversation_tree(
    max_words: usize,
    only_running: bool,
) -> Vec<AgmProjectTreeNode> {
    get_project_conversation_tree_cached(None, max_words, only_running, false)
}

/// Retrieve project conversation tree with cache control
pub fn get_project_conversation_tree_cached(
    instance_id: Option<&str>,
    max_words: usize,
    only_running: bool,
    force: bool,
) -> Vec<AgmProjectTreeNode> {
    let inst_key = instance_id.unwrap_or("all");
    let cache_key = format!(
        "tree:v{}:{}:{}:{}",
        env!("CARGO_PKG_VERSION"),
        inst_key,
        max_words,
        only_running
    );
    let now = Utc::now().timestamp();

    if !force {
        if let Some(nodes) = read_cached_tree(&cache_key, inst_key, now) {
            return nodes;
        }
    }

    // Serialize uncached scans so concurrent callers share one workspace pass instead of
    // fanning out across the blocking thread pool.
    let lock = TREE_COMPUTE_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().unwrap_or_else(|e| e.into_inner());

    if !force {
        let now_after_wait = Utc::now().timestamp();
        if let Some(nodes) = read_cached_tree(&cache_key, inst_key, now_after_wait) {
            return nodes;
        }
    }

    let tree_nodes = compute_project_conversation_tree(instance_id, max_words, only_running);
    let now = Utc::now().timestamp();

    let project_count = tree_nodes.len();
    let conversation_count: usize = tree_nodes.iter().map(|p| p.conversations.len()).sum();
    if let Ok(json_str) = serde_json::to_string(&tree_nodes) {
        if let Ok(conn) = connect_db() {
            // Justification: prompt-tree cache write has a 120-second TTL; the computed tree is returned regardless
            crate::error::record_ignored(
                conn.execute(
                "INSERT INTO prompt_tree_cache (cache_key, instance_id, tree_json, project_count, conversation_count, updated_at, ttl_seconds)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 120)
                 ON CONFLICT(cache_key) DO UPDATE SET
                    instance_id = excluded.instance_id,
                    tree_json = excluded.tree_json,
                    project_count = excluded.project_count,
                    conversation_count = excluded.conversation_count,
                    updated_at = excluded.updated_at,
                    ttl_seconds = excluded.ttl_seconds",
                params![
                    &cache_key,
                    inst_key,
                    &json_str,
                    project_count as i64,
                    conversation_count as i64,
                    now,
                ],
            ),
                "cache computed prompt tree",
            );
        }
    }

    tree_nodes
}

/// Invalidate cached project conversation trees in SQLite `prompt_tree_cache`
pub fn invalidate_prompt_tree_cache(instance_id: Option<&str>) {
    let Ok(conn) = connect_db() else { return };
    match instance_id {
        Some(id) if !id.trim().is_empty() && id != "all" => {
            let norm = crate::modules::instance::resolve_instance_id(id)
                .unwrap_or_else(|_| id.to_string());
            let pattern1 = format!("%:{}%", id);
            let pattern2 = format!("%:{}%", norm);
            // Justification: cache invalidation is best-effort; stale entries expire via TTL anyway
            crate::error::record_ignored(
                conn.execute(
                    "DELETE FROM prompt_tree_cache WHERE instance_id = ?1 OR instance_id = ?2 OR instance_id = 'all' OR cache_key LIKE ?3 OR cache_key LIKE ?4",
                    rusqlite::params![id, &norm, pattern1, pattern2],
                ),
                "invalidate prompt tree cache for instance",
            );
        }
        _ => {
            // Justification: cache invalidation is best-effort; stale entries expire via TTL anyway
            crate::error::record_ignored(
                conn.execute("DELETE FROM prompt_tree_cache", []),
                "invalidate all prompt tree cache",
            );
        }
    }
}
pub(crate) fn compute_project_conversation_tree(
    target_instance: Option<&str>,
    max_words: usize,
    only_running: bool,
) -> Vec<AgmProjectTreeNode> {
    let target = target_instance.filter(|t| !t.is_empty() && *t != "all");
    let registry = crate::modules::instance::load_registry().unwrap_or_default();
    let now = chrono::Utc::now().timestamp();

    super::tree_phases::transition_stale_inflight_prompts(now);
    let projects = super::tree_phases::query_tree_projects(target);
    let default_email = crate::modules::account::get_current_account()
        .ok()
        .flatten()
        .map(|a| a.email);
    let projects = super::tree_phases::discover_fallback_projects(target, projects);
    super::tree_phases::log_tree_telemetry_probe(target, &registry, &projects);

    let (conn_opt, convs_by_inst_and_path, active_prompts_by_inst_and_path, candidate_dirs) =
        super::tree_collect::collect_tree_conversation_items(target);

    let mut tree_nodes: Vec<AgmProjectTreeNode> = Vec::new();
    let word_cap = if max_words == 0 { 200 } else { max_words };
    let mut seen_project_keys: std::collections::HashSet<(String, String)> =
        std::collections::HashSet::new();

    let ctx = super::tree_nodes::TreeNodeCtx {
        registry: &registry,
        conn_opt: &conn_opt,
        default_email: &default_email,
        convs_by_inst_and_path: &convs_by_inst_and_path,
        active_prompts_by_inst_and_path: &active_prompts_by_inst_and_path,
        candidate_dirs: &candidate_dirs,
        only_running,
        word_cap,
        now,
    };
    for proj in &projects {
        let tree_node_count = tree_nodes.len();
        if let Some(node) = super::tree_nodes::build_project_tree_node(
            &ctx,
            proj,
            &mut seen_project_keys,
            tree_node_count,
        ) {
            tree_nodes.push(node);
        }
    }

    tree_nodes.sort_by_key(|n| n.seq_id);
    tree_nodes
}
