use super::common::VERSION;
use antigravity_tools_lib::modules::repo_db::ActivePrompt;
use antigravity_tools_lib::modules::{account, db, instance, integration, oauth, repo_db};
use std::path::{Path, PathBuf};

/// Mutable state threaded through the sequential test-flow steps.
/// Each field is set by the step that computes it; steps run in order.
pub(crate) struct TestFlowState {
    pub is_json: bool,
    pub clone_source: Option<String>,
    pub default_data_dir_str: String,
    pub protected_pids: std::collections::HashSet<u32>,
    pub acc1: Option<account::Account>,
    pub acc2: Option<account::Account>,
    pub new_inst: Option<instance::InstanceConfig>,
    pub rt: Option<tokio::runtime::Runtime>,
    pub gitmap_dir: String,
    pub heartbeat_log_str: String,
    pub py_heartbeat_runner: String,
    pub running_prompt: Option<ActivePrompt>,
    pub queued_prompt_1: Option<ActivePrompt>,
    pub queued_prompt_2: Option<ActivePrompt>,
    pub initial_heartbeat_status: String,
    pub spawned_pid: u32,
    pub root_dir: PathBuf,
    pub py_script: PathBuf,
    pub shot1_path: PathBuf,
    pub post_switch_pid: u32,
    pub shot2_path: PathBuf,
    pub final_pid: u32,
    pub shot3_path: PathBuf,
}

impl TestFlowState {
    pub(crate) fn new(is_json: bool, clone_source: Option<String>) -> Self {
        Self {
            is_json,
            clone_source,
            default_data_dir_str: String::new(),
            protected_pids: std::collections::HashSet::new(),
            acc1: None,
            acc2: None,
            new_inst: None,
            rt: None,
            gitmap_dir: String::new(),
            heartbeat_log_str: String::new(),
            py_heartbeat_runner: String::new(),
            running_prompt: None,
            queued_prompt_1: None,
            queued_prompt_2: None,
            initial_heartbeat_status: String::new(),
            spawned_pid: 0,
            root_dir: PathBuf::new(),
            py_script: PathBuf::new(),
            shot1_path: PathBuf::new(),
            post_switch_pid: 0,
            shot2_path: PathBuf::new(),
            final_pid: 0,
            shot3_path: PathBuf::new(),
        }
    }
}
impl TestFlowState {
    pub(crate) fn assert_not_protected(&self, pid: u32, context: &str) {
        if pid > 0 && self.protected_pids.contains(&pid) {
            eprintln!(
                "[FATAL ERROR] Refusing to touch PID {} during {}: Belongs to protected main IDE!",
                pid, context
            );
            std::process::exit(1);
        }
    }
    pub(crate) fn step0(&mut self) {
        // 0. Safety Invariant Check: Identify the Running Main Antigravity IDE (PID & Data Dir)
        let default_data_dir = instance::get_default_antigravity_data_dir();
        self.default_data_dir_str = default_data_dir.to_string_lossy().to_string();
        self.protected_pids = instance::find_pids_for_data_dir(&self.default_data_dir_str, true)
            .into_iter()
            .collect();

        println!("[SAFETY] Inspecting active host processes to protect current working IDE...");
        println!(
            "         ● Protected Default IDE Data Dir: {}",
            self.default_data_dir_str
        );
        println!(
            "         ● Protected Main IDE Process IDs: {:?}",
            self.protected_pids
        );
        println!(
            "         ● SAFETY INVARIANT: None of these PIDs will ever be closed or terminated!"
        );
        println!(
            "--------------------------------------------------------------------------------"
        );
    }

    pub(crate) fn step1(&mut self) {
        // Step 1: Clean up any existing test instances & stale test prompts
        println!(
            "[STEP 1/7] Cleaning up existing non-default sandbox instances and stale prompts..."
        );
        let mut removed_ids: Vec<String> = Vec::new();
        if let Ok(instances) = instance::list_instances() {
            for inst in instances {
                if inst.config.is_default || inst.config.id == "default" {
                    continue;
                }
                if !inst.config.id.starts_with("test-cli-flow")
                    && !inst.config.id.starts_with("test-diag")
                {
                    continue;
                }
                if let Some(pid) = inst.pid {
                    self.assert_not_protected(pid, "stale instance cleanup");
                }
                let _ = instance::close_instance(&inst.config.id);
                if let Err(e) = instance::delete_instance(&inst.config.id) {
                    eprintln!(
                        "  [WARN] Failed to delete instance '{}': {}",
                        inst.config.id, e
                    );
                } else {
                    removed_ids.push(inst.config.id.clone());
                    println!(
                        "  [✓] Removed stale instance '{}' ({})",
                        inst.config.name, inst.config.id
                    );
                }
            }
        }
        if let Ok(conn) = repo_db::connect_db() {
            for id in &removed_ids {
                let _ = conn.execute("DELETE FROM active_prompts WHERE instance_id = ?1", [id]);
                let _ = conn.execute("DELETE FROM running_projects WHERE instance_id = ?1", [id]);
            }
        }
        println!("  [SUCCESS] All stale sandbox instances and test prompts purged.");
        println!(
            "--------------------------------------------------------------------------------"
        );
    }

    pub(crate) fn step2(&mut self) {
        // Step 2: Clone the default IDE, or create an empty instance when --new is set
        let test_inst_id = "test-cli-flow";
        let test_inst_name = "Test-CLI-Flow".to_string();
        match &self.clone_source {
            Some(source) => println!(
                "[STEP 2/7] Cloning instance from '{}' into '{}'...",
                source, test_inst_id
            ),
            None => println!(
                "[STEP 2/7] Creating a new empty instance '{}' (no IDE data copied)...",
                test_inst_id
            ),
        }

        let accounts = account::list_accounts().unwrap_or_default();
        self.acc1 = Some(
            accounts
                .iter()
                .find(|a| a.email.starts_with("rokixshohag1"))
                .or_else(|| accounts.first())
                .expect("No accounts found in vault")
                .clone(),
        );
        self.acc2 = Some(
            accounts
                .iter()
                .find(|a| a.email.starts_with("erfan.office.n"))
                .or_else(|| accounts.get(1))
                .expect("No alternative account found in vault")
                .clone(),
        );

        self.new_inst = Some(match &self.clone_source {
            Some(source) => {
                let resolved =
                    instance::resolve_instance_id(source).unwrap_or_else(|_| source.to_string());
                match instance::copy_instance(&resolved, test_inst_name.clone(), Some("full")) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        eprintln!(
                            "[ERROR] Failed to clone instance from '{}': {}",
                            resolved, e
                        );
                        std::process::exit(1);
                    }
                }
            }
            None => match instance::create_instance(test_inst_name.clone()) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("[ERROR] Failed to create a new empty instance: {}", e);
                    std::process::exit(1);
                }
            },
        });
        let _ = instance::bind_account_to_instance(
            &self.new_inst.as_ref().expect("new_inst set").id,
            &self.acc1.as_ref().expect("acc1 set").id,
            &self.acc1.as_ref().expect("acc1 set").email,
        );
        self.rt = Some(tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"));
        let mut acc1_loaded = match account::load_account(&self.acc1.as_ref().expect("acc1 set").id)
        {
            Ok(a) => a,
            Err(_) => self.acc1.as_ref().expect("acc1 set").clone(),
        };
        if let Ok(fresh) = self
            .rt
            .as_ref()
            .expect("rt set")
            .block_on(oauth::ensure_fresh_token(
                &acc1_loaded.token,
                Some(&acc1_loaded.id),
            ))
        {
            acc1_loaded.token = fresh;
            let _ = account::save_account(&acc1_loaded);
        }

        let target_data_path =
            PathBuf::from(&self.new_inst.as_ref().expect("new_inst set").data_dir);
        let _ = instance::update_instance_app_storage(
            &target_data_path,
            Some(&acc1_loaded.email),
            acc1_loaded.token.is_gcp_tos,
        );
        instance::purge_volatile_instance_sessions(&target_data_path);
        let db_path = target_data_path
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        let _ = db::inject_token(
            &db_path,
            &acc1_loaded.token.access_token,
            &acc1_loaded.token.refresh_token,
            acc1_loaded.token.expiry_timestamp,
            &acc1_loaded.email,
            acc1_loaded.token.is_gcp_tos,
            acc1_loaded.token.project_id.as_deref(),
            acc1_loaded.token.id_token.as_deref(),
            acc1_loaded.token.oauth_client_key.as_deref(),
            None,
        );
        #[cfg(target_os = "windows")]
        {
            let appdata_db_dir = target_data_path
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage");
            let _ = std::fs::create_dir_all(&appdata_db_dir);
            let appdata_db_path = appdata_db_dir.join("state.vscdb");
            let _ = db::inject_token(
                &appdata_db_path,
                &acc1_loaded.token.access_token,
                &acc1_loaded.token.refresh_token,
                acc1_loaded.token.expiry_timestamp,
                &acc1_loaded.email,
                acc1_loaded.token.is_gcp_tos,
                acc1_loaded.token.project_id.as_deref(),
                acc1_loaded.token.id_token.as_deref(),
                acc1_loaded.token.oauth_client_key.as_deref(),
                None,
            );
            if let Ok(inst_home) =
                instance::get_instance_home_dir(&self.new_inst.as_ref().expect("new_inst set").id)
            {
                let home_appdata_db_dir = inst_home
                    .join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage");
                let _ = std::fs::create_dir_all(&home_appdata_db_dir);
                let home_appdata_db_path = home_appdata_db_dir.join("state.vscdb");
                let _ = db::inject_token(
                    &home_appdata_db_path,
                    &acc1_loaded.token.access_token,
                    &acc1_loaded.token.refresh_token,
                    acc1_loaded.token.expiry_timestamp,
                    &acc1_loaded.email,
                    acc1_loaded.token.is_gcp_tos,
                    acc1_loaded.token.project_id.as_deref(),
                    acc1_loaded.token.id_token.as_deref(),
                    acc1_loaded.token.oauth_client_key.as_deref(),
                    None,
                );
                let _ = integration::write_to_file_credentials_at(&inst_home, &acc1_loaded);
                instance::write_keyring_bypass_markers(&target_data_path, Some(&inst_home));
            } else {
                instance::write_keyring_bypass_markers(&target_data_path, None);
            }
        }
        let _ = integration::write_to_file_credentials_at(&target_data_path, &acc1_loaded);

        println!(
            "  [SUCCESS] Cloned IDE profile into instance '{}':",
            self.new_inst.as_ref().expect("new_inst set").id
        );
        println!(
            "            ● Name:            {}",
            self.new_inst.as_ref().expect("new_inst set").name
        );
        println!(
            "            ● Folder Location: {}",
            self.new_inst.as_ref().expect("new_inst set").data_dir
        );
        println!(
            "            ● Bound Account:   {} (ID: {})",
            self.acc1.as_ref().expect("acc1 set").email,
            self.acc1.as_ref().expect("acc1 set").id
        );
        println!("  [*] Initial Live Observation of newly cloned instance:");
        crate::prompt_goals::cmd_observe(std::slice::from_ref(
            &self.new_inst.as_ref().expect("new_inst set").id,
        ));
        println!(
            "--------------------------------------------------------------------------------"
        );
    }
}

pub(crate) fn cmd_test_instance_flow(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Instance Switch Test:");
        println!("  agm test-instance-flow [--from default|<inst>] [--new] [--json]");
        println!("  agm tif --from default");
        println!("  agm tif --new");
        println!("\nModes:");
        println!("  (default)              Clone the whole default IDE data directory");
        println!("  --from, -f <inst>      Clone from default, #N, an id, or a name");
        println!("  --new                  Create an empty instance. Do not copy IDE data");
        println!("\nExamples:");
        println!("  agm test-instance-flow --from default");
        println!("  agm instances create \"Clone From Default\" --from default");
        println!("  agm instances create \"New Empty Instance\"");
        return;
    }

    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    let is_new = args.iter().any(|a| a == "--new");
    let mut from_source: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "--from" || args[i] == "-f") && i + 1 < args.len() {
            from_source = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        i += 1;
    }
    if is_new && from_source.is_some() {
        eprintln!("[ERROR] Use either --new or --from, not both.");
        std::process::exit(1);
    }
    let clone_source = if is_new {
        None
    } else {
        Some(from_source.unwrap_or_else(|| "default".to_string()))
    };

    println!("================================================================================");
    println!("  AGM Autonomous CLI Instance Switching & Prompt Preservation Workflow Engine");
    println!("================================================================================");

    let mut state = TestFlowState::new(is_json, clone_source);
    state.step0();
    state.step1();
    state.step2();
    state.step3();
    state.step4();
    state.step5();
    state.step6();
    state.step7();
}
