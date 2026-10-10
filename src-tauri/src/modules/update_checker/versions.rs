use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

/// Compare two semantic versions (supports pre-release tags like "4.8.1-beta.2" vs "4.8.1-beta.1")
pub fn compare_versions(latest: &str, current: &str) -> bool {
    let parse_semver = |v: &str| -> (Vec<u32>, Option<(String, u32)>) {
        let clean = v.trim().trim_start_matches('v');
        if let Some((main_part, pre_part)) = clean.split_once('-') {
            let nums: Vec<u32> = main_part
                .split('.')
                .filter_map(|s| s.parse::<u32>().ok())
                .collect();
            // 解析预发布段，如 beta.2 -> ("beta", 2)
            let pre_info = if let Some((tag, num_str)) = pre_part.split_once('.') {
                Some((tag.to_lowercase(), num_str.parse::<u32>().unwrap_or(0)))
            } else {
                Some((pre_part.to_lowercase(), 0))
            };
            (nums, pre_info)
        } else {
            let nums: Vec<u32> = clean
                .split('.')
                .filter_map(|s| s.parse::<u32>().ok())
                .collect();
            (nums, None)
        }
    };

    let (latest_nums, latest_pre) = parse_semver(latest);
    let (current_nums, current_pre) = parse_semver(current);

    // 1. 先比较主版本号 [major, minor, patch]
    for i in 0..latest_nums.len().max(current_nums.len()) {
        let l = latest_nums.get(i).copied().unwrap_or(0);
        let c = current_nums.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    // 2. 主版本号完全相同时，检查 pre-release (标准 SemVer 规则：无 pre-release > 有 pre-release)
    match (latest_pre, current_pre) {
        (None, Some(_)) => true,  // e.g. latest 4.8.1 正式版 > current 4.8.1-beta.2
        (Some(_), None) => false, // e.g. latest 4.8.1-beta.2 < current 4.8.1 正式版
        (Some((l_tag, l_num)), Some((c_tag, c_num))) => {
            if l_tag != c_tag {
                l_tag > c_tag
            } else {
                l_num > c_num // e.g. beta.2 > beta.1
            }
        }
        (None, None) => false, // 完全相同版本
    }
}

/// Check if enough time has passed since last check
pub fn should_check_for_updates(settings: &UpdateSettings) -> bool {
    if !settings.auto_check {
        return false;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let elapsed_hours = (now - settings.last_check_time) / 3600;
    let interval = if settings.check_interval_hours > 0 {
        settings.check_interval_hours
    } else {
        DEFAULT_CHECK_INTERVAL_HOURS
    };
    elapsed_hours >= interval
}

/// Load update settings from config file
pub fn load_update_settings() -> Result<UpdateSettings, String> {
    let data_dir = crate::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get data dir: {}", e))?;
    let settings_path = data_dir.join("update_settings.json");

    if !settings_path.exists() {
        return Ok(UpdateSettings::default());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))
}

/// Save update settings to config file
pub fn save_update_settings(settings: &UpdateSettings) -> Result<(), String> {
    let data_dir = crate::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get data dir: {}", e))?;
    let settings_path = data_dir.join("update_settings.json");

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))
}

/// Update last check time
pub fn update_last_check_time() -> Result<(), String> {
    let mut settings = load_update_settings()?;
    settings.last_check_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    save_update_settings(&settings)
}
