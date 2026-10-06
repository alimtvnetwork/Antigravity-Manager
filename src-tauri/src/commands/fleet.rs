use crate::error::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetDeployResult {
    pub success: bool,
    pub message: String,
    pub raw_output: String,
}

#[tauri::command]
pub async fn check_gitmap_available() -> AppResult<bool> {
    let res = std::process::Command::new("gitmap").arg("version").output();
    Ok(res.map(|o| o.status.success()).unwrap_or(false))
}

#[tauri::command]
pub async fn check_gitmap_installed() -> AppResult<bool> {
    check_gitmap_available().await
}

#[tauri::command]
pub async fn deploy_accounts_to_fleet(
    include_main: Option<bool>,
    except_nodes: Option<Vec<String>>,
) -> AppResult<FleetDeployResult> {
    let mut cmd = std::process::Command::new("gitmap");
    cmd.args(["nodes", "deploy", "agm-accounts", "--json"]);

    if include_main.unwrap_or(false) {
        cmd.arg("--include-main");
    }

    if let Some(nodes) = except_nodes {
        let trimmed_nodes: Vec<String> = nodes
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !trimmed_nodes.is_empty() {
            cmd.arg("--except").arg(trimmed_nodes.join(","));
        }
    }

    let output = match cmd.output() {
        Ok(out) => out,
        Err(err) => {
            return Ok(FleetDeployResult {
                success: false,
                message: format!("Failed to execute gitmap: {}", err),
                raw_output: String::new(),
            });
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let raw_output = if !stderr.trim().is_empty() {
        if !stdout.trim().is_empty() {
            format!("{}\n{}", stdout.trim(), stderr.trim())
        } else {
            stderr.trim().to_string()
        }
    } else {
        stdout.trim().to_string()
    };

    let success = output.status.success();
    let message = if success {
        if raw_output.is_empty() || raw_output == "null" {
            "Fleet deployment completed successfully.".to_string()
        } else if let Ok(val) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(msg) = val.get("message").and_then(|m| m.as_str()) {
                msg.to_string()
            } else {
                "Fleet deployment completed successfully.".to_string()
            }
        } else {
            "Fleet deployment completed successfully.".to_string()
        }
    } else if raw_output.is_empty() {
        format!(
            "Fleet deployment exited with error code {:?}",
            output.status.code()
        )
    } else {
        format!("Fleet deployment failed: {}", raw_output)
    };

    Ok(FleetDeployResult {
        success,
        message,
        raw_output,
    })
}
