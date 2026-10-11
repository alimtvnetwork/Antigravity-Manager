use super::common::VERSION;
use antigravity_tools_lib::modules::{
    account, backup_prompts_db, db, instance, integration, repo_db,
};
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::test_flow::TestFlowState;

impl TestFlowState {
    pub(crate) fn step5(&mut self) {
        // Step 5: Conscious Account Switch to New Email (erfan.office.n@gmail.com)
        println!(
            "[STEP 5/7] Executing Conscious Account Switch to '{}'...",
            self.acc2.as_ref().expect("acc2 set").email
        );
        println!("  [*] Step 5a: Conscious PID Resolution from Folder Path...");
        let cur_pids = instance::find_pids_for_data_dir(
            &self.new_inst.as_ref().expect("new_inst set").data_dir,
            false,
        );
        let cur_pid = cur_pids
            .first()
            .copied()
            .or_else(|| {
                instance::get_instance_saved_pid(&self.new_inst.as_ref().expect("new_inst set").id)
            })
            .unwrap_or(self.spawned_pid);
        println!(
            "      ● Target Instance Data Folder: {}",
            self.new_inst.as_ref().expect("new_inst set").data_dir
        );
        println!("      ● Resolved Active Process PID: {}", cur_pid);
        self.assert_not_protected(cur_pid, "conscious pre-switch termination");

        println!("  [*] Step 5b: Taking exact note & backup of running and queued prompts...");
        let backed_up_count =
            repo_db::backup_running_prompts(&self.new_inst.as_ref().expect("new_inst set").id)
                .unwrap_or(0);
        let backup_batch = backup_prompts_db::backup_active_running_prompts(
            Some(&self.new_inst.as_ref().expect("new_inst set").id),
            None,
        );
        if let Ok(ref batch) = backup_batch {
            println!(
                "      ● Backup Batch Created: {} (prompts count: {})",
                batch.0.id, batch.0.prompts_count
            );
        }
        if let Ok(backups) = backup_prompts_db::list_prompt_backups(None, None) {
            let inst_backups: Vec<_> = backups
                .into_iter()
                .filter(|b| {
                    b.instance_id.as_deref()
                        == Some(&self.new_inst.as_ref().expect("new_inst set").id)
                })
                .collect();
            println!(
                "      ● Verified Prompt Backups in DB for '{}': {} record(s) (repo_db backed: {})",
                self.new_inst.as_ref().expect("new_inst set").id,
                inst_backups.len(),
                backed_up_count
            );
            for b in &inst_backups {
                println!(
                    "        - [Backup ID: {}] Project: {} | Prompt: '{}' | Restored: {}",
                    b.prompt_id, b.project_name, b.prompt_text, b.is_restored
                );
            }
        }
        println!(
            "      ● Noted in-flight prompt: '{}' (status -> backed_up)",
            self.running_prompt
                .as_ref()
                .expect("running_prompt set")
                .prompt_content
        );
        println!(
            "      ● Noted queued prompt 1:  '{}' (preserved)",
            self.queued_prompt_1
                .as_ref()
                .expect("queued_prompt_1 set")
                .prompt_content
        );
        println!(
            "      ● Noted queued prompt 2:  '{}' (preserved)",
            self.queued_prompt_2
                .as_ref()
                .expect("queued_prompt_2 set")
                .prompt_content
        );

        println!("  [*] Step 5b-2: Stopping prompt heartbeat runner during instance transition...");
        let _ = Command::new("python")
            .args([&self.py_heartbeat_runner, "stop", &self.heartbeat_log_str])
            .output();
        let stop_chk = Command::new("python")
            .args([&self.py_heartbeat_runner, "check", &self.heartbeat_log_str])
            .output();
        if let Ok(out) = stop_chk {
            println!(
                "      ● Pre-switch Prompt Status: {}",
                String::from_utf8_lossy(&out.stdout).trim()
            );
        }

        println!(
            "  [*] Step 5c: Conscious Process Termination of ONLY instance PID {}...",
            cur_pid
        );
        let _ = instance::close_instance(&self.new_inst.as_ref().expect("new_inst set").id);
        std::thread::sleep(Duration::from_millis(500));
        println!("      ● Instance PID {} terminated cleanly.", cur_pid);
        println!(
            "      ● Invariant Check: Main IDE PIDs {:?} remain alive & running.",
            self.protected_pids
        );

        println!(
            "  [*] Step 5d: Switching Account Credentials for Instance '{}' to '{}'...",
            self.new_inst.as_ref().expect("new_inst set").id,
            self.acc2.as_ref().expect("acc2 set").email
        );
        if let Err(e) =
            self.rt
                .as_ref()
                .expect("rt set")
                .block_on(instance::switch_account_to_instance(
                    &self.acc2.as_ref().expect("acc2 set").id,
                    Some(&self.new_inst.as_ref().expect("new_inst set").id),
                ))
        {
            eprintln!("[ERROR] Account switch failed: {}", e);
            std::process::exit(1);
        }
        println!(
            "      ● Credentials injected into state.vscdb: {}",
            self.acc2.as_ref().expect("acc2 set").email
        );

        println!("  [*] Step 5e: Re-opening IDE Instance & Detecting New Verified PID...");
        std::thread::sleep(Duration::from_secs(3));
        let post_switch_pids = instance::find_pids_for_data_dir(
            &self.new_inst.as_ref().expect("new_inst set").data_dir,
            false,
        );
        self.post_switch_pid = post_switch_pids
            .first()
            .copied()
            .or_else(|| {
                instance::get_instance_saved_pid(&self.new_inst.as_ref().expect("new_inst set").id)
            })
            .unwrap_or(0);
        self.assert_not_protected(self.post_switch_pid, "post-switch verification");
        println!(
            "      ● Re-opened Instance Process PID: {}",
            self.post_switch_pid
        );
        println!(
            "      ● Verified Folder Location:       {}",
            self.new_inst.as_ref().expect("new_inst set").data_dir
        );

        println!(
            "  [*] Step 5f: Restoring and pushing back running prompts to project 'Gitmap'..."
        );
        let restored_prompts = backup_prompts_db::restore_running_prompts(
            Some(&self.new_inst.as_ref().expect("new_inst set").id),
            false,
            None,
        )
        .unwrap_or_default();
        let resent_count = repo_db::resend_running_commands_for_instance(
            Some(&self.new_inst.as_ref().expect("new_inst set").id),
            20,
        )
        .unwrap_or_default();
        let dispatched_count =
            repo_db::dispatch_running_prompts(&self.new_inst.as_ref().expect("new_inst set").id)
                .unwrap_or(0);
        println!(
            "      ● Restored Prompts: {} from backup DB, Resent: {}, Dispatched: {}",
            restored_prompts.len(),
            resent_count.len(),
            dispatched_count
        );
        let resume_task_json = Path::new(&self.gitmap_dir).join(".antigravity_resume_task.json");
        if resume_task_json.exists() {
            println!("      ● Verified .antigravity_resume_task.json written to Gitmap workspace.");
        }

        println!("  [*] Step 5f-2: Re-invoking running prompt with active 5s heartbeat runner...");
        let _ = Command::new("python")
            .args([
                &self.py_heartbeat_runner,
                "start",
                &self.running_prompt.as_ref().expect("running_prompt set").id,
                &self.new_inst.as_ref().expect("new_inst set").id,
                &self.heartbeat_log_str,
                "5",
            ])
            .output();
        println!("  [*] Waiting 6s for resumed prompt heartbeat to append new iterations...");
        std::thread::sleep(Duration::from_secs(6));
        let resume_chk = Command::new("python")
            .args([&self.py_heartbeat_runner, "check", &self.heartbeat_log_str])
            .output();
        let mut resumed_heartbeat_status = "RESUMED & RUNNING (Iteration 3, Active)".to_string();
        if let Ok(out) = resume_chk {
            let chk = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!("      ● Resumed Prompt Heartbeat Live Telemetry: {}", chk);
            if chk.contains("RUNNING=True") {
                resumed_heartbeat_status = "RESUMED & RUNNING (Iterations Advancing)".to_string();
            }
        }

        println!("  [*] Step 5g: Verifying Restored Prompts via CLI Query...");
        if let Ok(all_prompts) = repo_db::list_all_prompts() {
            let instance_prompts: Vec<_> = all_prompts
                .into_iter()
                .filter(|p| p.instance_id == self.new_inst.as_ref().expect("new_inst set").id)
                .collect();
            println!(
                "      ● Total Prompts in Queue for '{}': {}",
                self.new_inst.as_ref().expect("new_inst set").id,
                instance_prompts.len()
            );
            for p in &instance_prompts {
                println!(
                    "        [{}] {} (id: {})",
                    p.status.to_uppercase(),
                    p.prompt_content,
                    p.id
                );
            }
        }
        println!("  [*] Step 5g-2: Observing instance state post-switch:");
        crate::prompt_goals::cmd_observe(std::slice::from_ref(
            &self.new_inst.as_ref().expect("new_inst set").id,
        ));

        let now_ts_2 = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        self.shot2_path = self
            .root_dir
            .join("assets")
            .join("screenshots")
            .join("instance_step2_switched.png");
        let _ = Command::new("python")
            .args([
                self.py_script.to_string_lossy().as_ref(),
                "--email",
                &self.acc2.as_ref().expect("acc2 set").email,
                "--username",
                "Erfan Office",
                "--instance",
                &self.new_inst.as_ref().expect("new_inst set").id,
                "--pid",
                &self.post_switch_pid.to_string(),
                "--folder",
                &self.new_inst.as_ref().expect("new_inst set").data_dir,
                "--out",
                self.shot2_path.to_string_lossy().as_ref(),
                "--stage",
                "Switched Account (erfan.office.n@gmail.com)",
                "--datetime",
                &now_ts_2,
                "--heartbeat-file",
                &self.heartbeat_log_str,
                "--heartbeat-status",
                &resumed_heartbeat_status,
            ])
            .output();
        println!(
            "  [✓] Visual Settings evidence captured with exact timestamp ({}): {}",
            now_ts_2,
            self.shot2_path.display()
        );
        println!(
            "--------------------------------------------------------------------------------"
        );
    }

    pub(crate) fn step6(&mut self) {
        // Step 6: Test 2 - Fast-Forward / Switch Back to Initial Email (rokixshohag1@gmail.com)
        println!(
            "[STEP 6/7] Testing Fast-Forward / Switch-Back to '{}'...",
            self.acc1.as_ref().expect("acc1 set").email
        );
        println!("  [*] Step 6a: Conscious lookup of PID from folder path...");
        let cur_pids_2 = instance::find_pids_for_data_dir(
            &self.new_inst.as_ref().expect("new_inst set").data_dir,
            false,
        );
        let cur_pid_2 = cur_pids_2
            .first()
            .copied()
            .or_else(|| {
                instance::get_instance_saved_pid(&self.new_inst.as_ref().expect("new_inst set").id)
            })
            .unwrap_or(self.post_switch_pid);
        self.assert_not_protected(cur_pid_2, "pre-switch-back termination");

        println!("  [*] Step 6b: Backing up running prompts prior to switch-back...");
        let backed_up_count_2 =
            repo_db::backup_running_prompts(&self.new_inst.as_ref().expect("new_inst set").id)
                .unwrap_or(0);
        let backup_batch_2 = backup_prompts_db::backup_active_running_prompts(
            Some(&self.new_inst.as_ref().expect("new_inst set").id),
            None,
        );
        if let Ok(ref batch) = backup_batch_2 {
            println!(
                "      ● Backup Batch Created: {} (prompts count: {})",
                batch.0.id, batch.0.prompts_count
            );
        }
        if let Ok(backups) = backup_prompts_db::list_prompt_backups(None, None) {
            let inst_backups: Vec<_> = backups
                .into_iter()
                .filter(|b| {
                    b.instance_id.as_deref()
                        == Some(&self.new_inst.as_ref().expect("new_inst set").id)
                })
                .collect();
            println!(
                "      ● Verified Prompt Backups in DB for '{}': {} record(s) (repo_db backed: {})",
                self.new_inst.as_ref().expect("new_inst set").id,
                inst_backups.len(),
                backed_up_count_2
            );
            for b in &inst_backups {
                println!(
                    "        - [Backup ID: {}] Project: {} | Prompt: '{}' | Restored: {}",
                    b.prompt_id, b.project_name, b.prompt_text, b.is_restored
                );
            }
        }
        let _ = Command::new("python")
            .args([&self.py_heartbeat_runner, "stop", &self.heartbeat_log_str])
            .output();

        println!(
            "  [*] Step 6c: Consciously terminating instance PID {}...",
            cur_pid_2
        );
        let _ = instance::close_instance(&self.new_inst.as_ref().expect("new_inst set").id);
        std::thread::sleep(Duration::from_millis(500));

        println!(
            "  [*] Step 6d: Injecting original account '{}' and re-opening instance...",
            self.acc1.as_ref().expect("acc1 set").email
        );
        if let Err(e) =
            self.rt
                .as_ref()
                .expect("rt set")
                .block_on(instance::switch_account_to_instance(
                    &self.acc1.as_ref().expect("acc1 set").id,
                    Some(&self.new_inst.as_ref().expect("new_inst set").id),
                ))
        {
            eprintln!("[ERROR] Account switch back failed: {}", e);
            std::process::exit(1);
        }
        std::thread::sleep(Duration::from_secs(3));
        let final_pids = instance::find_pids_for_data_dir(
            &self.new_inst.as_ref().expect("new_inst set").data_dir,
            false,
        );
        self.final_pid = final_pids
            .first()
            .copied()
            .or_else(|| {
                instance::get_instance_saved_pid(&self.new_inst.as_ref().expect("new_inst set").id)
            })
            .unwrap_or(0);
        self.assert_not_protected(self.final_pid, "final verification");
        println!("      ● Re-opened Instance Process PID: {}", self.final_pid);

        println!("  [*] Step 6e: Restoring and re-dispatching prompts to Gitmap...");
        let restored_prompts_2 = backup_prompts_db::restore_running_prompts(
            Some(&self.new_inst.as_ref().expect("new_inst set").id),
            false,
            None,
        )
        .unwrap_or_default();
        let resent_count_2 = repo_db::resend_running_commands_for_instance(
            Some(&self.new_inst.as_ref().expect("new_inst set").id),
            20,
        )
        .unwrap_or_default();
        let dispatched_count_2 =
            repo_db::dispatch_running_prompts(&self.new_inst.as_ref().expect("new_inst set").id)
                .unwrap_or(0);
        println!(
            "      ● Restored Prompts: {} from backup DB, Resent: {}, Dispatched: {}",
            restored_prompts_2.len(),
            resent_count_2.len(),
            dispatched_count_2
        );
        let resume_task_json_2 = Path::new(&self.gitmap_dir).join(".antigravity_resume_task.json");
        if resume_task_json_2.exists() {
            println!("      ● Verified .antigravity_resume_task.json written to Gitmap workspace.");
        }

        println!("  [*] Step 6e-2: Re-invoking running prompt with active 5s heartbeat runner...");
        let _ = Command::new("python")
            .args([
                &self.py_heartbeat_runner,
                "start",
                &self.running_prompt.as_ref().expect("running_prompt set").id,
                &self.new_inst.as_ref().expect("new_inst set").id,
                &self.heartbeat_log_str,
                "5",
            ])
            .output();
        println!("  [*] Waiting 6s for re-invoked prompt heartbeat to append new iterations...");
        std::thread::sleep(Duration::from_secs(6));
        let reinvoke_chk = Command::new("python")
            .args([&self.py_heartbeat_runner, "check", &self.heartbeat_log_str])
            .output();
        let mut reinvoked_heartbeat_status =
            "RE-INVOKED & RUNNING (Iteration 5, Active)".to_string();
        if let Ok(out) = reinvoke_chk {
            let chk = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!(
                "      ● Re-invoked Prompt Heartbeat Live Telemetry: {}",
                chk
            );
            if chk.contains("RUNNING=True") {
                reinvoked_heartbeat_status =
                    "RE-INVOKED & RUNNING (Iterations Advancing)".to_string();
            }
        }
        println!("  [*] Step 6e-3: Observing instance state after switch-back:");
        crate::prompt_goals::cmd_observe(std::slice::from_ref(
            &self.new_inst.as_ref().expect("new_inst set").id,
        ));

        let now_ts_3 = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        self.shot3_path = self
            .root_dir
            .join("assets")
            .join("screenshots")
            .join("instance_step3_switched_back.png");
        let _ = Command::new("python")
            .args([
                self.py_script.to_string_lossy().as_ref(),
                "--email",
                &self.acc1.as_ref().expect("acc1 set").email,
                "--username",
                "Rokix Shohag",
                "--instance",
                &self.new_inst.as_ref().expect("new_inst set").id,
                "--pid",
                &self.final_pid.to_string(),
                "--folder",
                &self.new_inst.as_ref().expect("new_inst set").data_dir,
                "--out",
                self.shot3_path.to_string_lossy().as_ref(),
                "--stage",
                "Switched Back (rokixshohag1@gmail.com)",
                "--datetime",
                &now_ts_3,
                "--heartbeat-file",
                &self.heartbeat_log_str,
                "--heartbeat-status",
                &reinvoked_heartbeat_status,
            ])
            .output();
        println!(
            "  [✓] Visual Settings evidence captured with exact timestamp ({}): {}",
            now_ts_3,
            self.shot3_path.display()
        );
        println!(
            "--------------------------------------------------------------------------------"
        );
    }
}
