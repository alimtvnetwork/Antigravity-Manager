//! Repo DB: gemini dirs

use std::collections::HashSet;
use std::path::PathBuf;

pub fn get_canonical_host_home() -> Option<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if home_str.contains(".antigravity_tools") {
            let mut curr = home.as_path();
            while let Some(parent) = curr.parent() {
                if let Some(name) = curr.file_name().and_then(|n| n.to_str()) {
                    if name.eq_ignore_ascii_case(".antigravity_tools") {
                        return Some(parent.to_path_buf());
                    }
                }
                curr = parent;
            }
        }
        return Some(home);
    }
    None
}

pub fn gemini_dirs_for_instance(instance_id: &str) -> Vec<PathBuf> {
    let named = instance_id != "all"
        && instance_id != "default"
        && !instance_id.is_empty()
        && instance_id != "__default__";

    let mut dirs = Vec::new();

    if named {
        if let Ok(home) = crate::modules::instance::get_instance_home_dir(instance_id) {
            for sub in ["antigravity", "antigravity-ide", "antigravity-cli"] {
                let path = home.join(".gemini").join(sub);
                if path.exists() {
                    dirs.push(path);
                }
            }
        }
    } else {
        // 1. Host canonical default home
        if let Some(host_home) = get_canonical_host_home() {
            for sub in ["antigravity", "antigravity-ide", "antigravity-cli"] {
                let path = host_home.join(".gemini").join(sub);
                if path.exists() && !dirs.contains(&path) {
                    dirs.push(path);
                }
            }
        }
        // 2. Sandboxed default instance home (if present)
        if let Ok(instances_dir) = crate::modules::instance::get_instances_dir() {
            let default_sandbox = instances_dir.join("default").join("home");
            if default_sandbox.exists() {
                for sub in ["antigravity", "antigravity-ide", "antigravity-cli"] {
                    let path = default_sandbox.join(".gemini").join(sub);
                    if path.exists() && !dirs.contains(&path) {
                        dirs.push(path);
                    }
                }
            }
        }
    }
    dirs
}

pub fn gemini_dirs_tagged(instance_id: Option<&str>) -> Vec<(String, PathBuf)> {
    let mut tagged = Vec::new();
    let target = instance_id.unwrap_or("all");

    if target != "all" {
        let norm_id = if target == "__default__" || target.is_empty() {
            "default"
        } else {
            target
        };
        for dir in gemini_dirs_for_instance(norm_id) {
            tagged.push((norm_id.to_string(), dir));
        }
        return tagged;
    }

    // When target is "all":
    // 1. Collect all secondary instance directories first
    let mut secondary_dirs = std::collections::HashSet::new();
    if let Ok(reg) = crate::modules::instance::load_registry() {
        for inst in &reg.instances {
            if !inst.is_default && inst.id != "default" {
                for dir in gemini_dirs_for_instance(&inst.id) {
                    secondary_dirs.insert(dir.clone());
                    tagged.push((inst.id.clone(), dir));
                }
            }
        }
    }

    // 2. Add default directories, strictly excluding any secondary sandbox paths
    for dir in gemini_dirs_for_instance("default") {
        if !secondary_dirs.contains(&dir) {
            tagged.push(("default".to_string(), dir));
        }
    }

    tagged
}
