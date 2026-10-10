//! Repo DB: tree project node builder

use super::models::{ActivePrompt, AgmConversationNode, AgmProjectTreeNode, RunningProject};
use super::project_queries::normalize_path_for_compare;
use super::sequences::ensure_project_sequence_in_conn;
use super::tree_format::group_identical_conversation_runs;
use std::collections::HashSet;

pub(crate) struct TreeNodeCtx<'a> {
    pub registry: &'a crate::models::instance::InstanceRegistry,
    pub conn_opt: &'a Option<rusqlite::Connection>,
    pub default_email: &'a Option<String>,
    pub convs_by_inst_and_path:
        &'a std::collections::HashMap<(String, String), Vec<super::tree_collect::RawConvItem>>,
    pub active_prompts_by_inst_and_path:
        &'a std::collections::HashMap<(String, String), Vec<ActivePrompt>>,
    pub candidate_dirs: &'a [(String, std::path::PathBuf)],
    pub only_running: bool,
    pub word_cap: usize,
    pub now: i64,
}

pub(crate) fn build_project_tree_node(
    ctx: &TreeNodeCtx,
    proj: &RunningProject,
    seen_project_keys: &mut std::collections::HashSet<(String, String)>,
    tree_node_count: usize,
) -> Option<AgmProjectTreeNode> {
    let registry = ctx.registry;
    let conn_opt = ctx.conn_opt;
    let default_email = ctx.default_email;
    let only_running = ctx.only_running;
    let norm_path = normalize_path_for_compare(&proj.repo_path);
    let project_key = if !norm_path.is_empty() {
        norm_path.clone()
    } else {
        proj.id.clone()
    };

    if !seen_project_keys.insert((project_key.clone(), proj.instance_id.clone())) {
        return None;
    }

    let is_inst_alive = if proj.instance_id == "default" || proj.instance_id == "__default__" {
        crate::modules::process::is_antigravity_running(None)
    } else if let Some(inst) = registry
        .instances
        .iter()
        .find(|i| i.id == proj.instance_id || i.name == proj.instance_id)
    {
        crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid)
    } else {
        false
    };

    let p_seq = if let Some(ref conn) = conn_opt {
        ensure_project_sequence_in_conn(
            conn,
            &project_key,
            &proj.id,
            &proj.repo_name,
            &proj.repo_path,
            &proj.instance_id,
        )
    } else {
        (tree_node_count as i64) + 1
    };

    let norm_proj_inst = if proj.instance_id == "default" || proj.instance_id == "__default__" {
        "default".to_string()
    } else {
        crate::modules::instance::resolve_instance_id(&proj.instance_id)
            .unwrap_or_else(|_| proj.instance_id.clone())
    };

    let (instance_seq_num, instance_name, bound_email, inst_pid, instance_exe_name) =
        if proj.instance_id == "default" || proj.instance_id == "__default__" {
            let exe_name = crate::modules::instance::resolve_instance_exe_name("default", None);
            (
                Some(1),
                "default".to_string(),
                default_email.clone(),
                crate::modules::process::get_antigravity_pids(None)
                    .first()
                    .copied(),
                exe_name,
            )
        } else if let Some(inst) = registry
            .instances
            .iter()
            .find(|i| i.id == proj.instance_id || i.name == proj.instance_id)
        {
            let exe_name = crate::modules::instance::resolve_instance_exe_name(
                &inst.id,
                inst.executable_path.as_deref(),
            );
            (
                inst.seq_num,
                inst.name.clone(),
                inst.bound_email.clone(),
                inst.pid,
                exe_name,
            )
        } else {
            let exe_name =
                crate::modules::instance::resolve_instance_exe_name(&proj.instance_id, None);
            (None, proj.instance_id.clone(), None, None, exe_name)
        };

    let mut conv_nodes: Vec<AgmConversationNode> = Vec::new();
    super::tree_node_conv::build_transcript_conv_nodes(
        ctx,
        proj,
        &norm_path,
        &norm_proj_inst,
        &project_key,
        is_inst_alive,
        &mut conv_nodes,
    );
    super::tree_node_conv::build_active_prompt_conv_nodes(
        ctx,
        proj,
        &norm_path,
        &norm_proj_inst,
        &project_key,
        is_inst_alive,
        &mut conv_nodes,
    );
    let (proj_is_running, rationale) = super::tree_node_liveness::evaluate_project_liveness(
        ctx,
        proj,
        &project_key,
        is_inst_alive,
        &conv_nodes,
    );
    crate::modules::logger::log_info(&format!(
        "[PROMPT_LIVENESS_PROBE][PROJECT] instance_id=\"{}\" project=\"{}\" is_running={} rationale=\"{}\"",
        proj.instance_id, proj.repo_name, proj_is_running, rationale
    ));

    crate::modules::logger::log_instance_prompt_audit(
        &proj.instance_id,
        &instance_name,
        &proj.repo_name,
        &proj.repo_path,
        proj.workspace_storage_path.as_deref().unwrap_or("none"),
        inst_pid,
        "ProjectConversationTreeLiveness",
        proj_is_running,
        &rationale,
    );

    if only_running {
        let has_any_running_conv = conv_nodes.iter().any(|c| c.is_running);
        if !proj_is_running && !has_any_running_conv {
            return None;
        }
    }

    let grouped_convs = group_identical_conversation_runs(conv_nodes);
    let proj_byte_size: usize = grouped_convs.iter().map(|c| c.byte_size.unwrap_or(0)).sum();
    let running_count = grouped_convs.iter().filter(|c| c.is_running).count();
    let queued_count = grouped_convs.iter().filter(|c| c.is_queued).count();

    return Some(AgmProjectTreeNode {
        seq_id: p_seq,
        seq_code: format!("P{:03}", p_seq),
        gitmap_seq_code: format!("GM:#{}", p_seq),
        project_id: proj.id.clone(),
        repo_name: proj.repo_name.clone(),
        repo_path: proj.repo_path.clone(),
        instance_id: proj.instance_id.clone(),
        instance_seq_num,
        instance_name,
        instance_exe_name,
        bound_email,
        is_running: proj_is_running,
        running_count,
        queued_count,
        conversations: grouped_convs,
        byte_size: Some(proj_byte_size),
        repeat_count: None,
        repeat_badge: None,
    });
}
