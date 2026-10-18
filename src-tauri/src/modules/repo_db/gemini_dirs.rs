//! Repo DB: gemini dirs

use std::collections::HashSet;
use std::path::{Path, PathBuf};

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

/// Helper to verify whether a path belongs to an isolated secondary instance sandbox
fn is_secondary_sandbox_path(path: &Path) -> bool {
    if let Ok(instances_dir) = crate::modules::instance::get_instances_dir() {
        if path.starts_with(&instances_dir) {
            let default_sandbox = instances_dir.join("default");
            if !path.starts_with(&default_sandbox) {
                return true;
            }
        }
    }
    if let Ok(registry) = crate::modules::instance::load_registry() {
        for inst in &registry.instances {
            let is_default = inst.is_default || inst.id == "default" || inst.id == "__default__";
            if !is_default {
                let data_path = PathBuf::from(&inst.data_dir);
                if path.starts_with(&data_path) {
                    return true;
                }
                if let Ok(home) = crate::modules::instance::get_instance_home_dir(&inst.id) {
                    if path.starts_with(&home) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn gemini_dirs_for_instance(instance_id: &str) -> Vec<PathBuf> {
    let is_cli = instance_id == "__cli__"
        || instance_id == "cli"
        || instance_id == "antigravity-cli"
        || instance_id == "caller-cli";
    let is_named_instance = instance_id != "all"
        && instance_id != "default"
        && !instance_id.is_empty()
        && instance_id != "__default__";

    let mut dirs = Vec::new();

    if is_cli {
        let mut candidate_homes = Vec::new();
        if let Some(host_home) = get_canonical_host_home() {
            if !is_secondary_sandbox_path(&host_home) {
                candidate_homes.push(host_home);
            }
        }
        if let Some(raw_home) = dirs::home_dir() {
            if !is_secondary_sandbox_path(&raw_home) && !candidate_homes.contains(&raw_home) {
                candidate_homes.push(raw_home);
            }
        }
        for home in candidate_homes {
            let path = home.join(".gemini").join("antigravity-cli");
            if path.exists() && !is_secondary_sandbox_path(&path) && !dirs.contains(&path) {
                dirs.push(path);
            }
        }
        return dirs;
    }

    if is_named_instance {
        if let Ok(home) = crate::modules::instance::get_instance_home_dir(instance_id) {
            for sub in ["antigravity", "antigravity-ide"] {
                let path = home.join(".gemini").join(sub);
                if path.exists() && !dirs.contains(&path) {
                    dirs.push(path);
                }
            }
        }
    } else {
        // 1. Host canonical default home (including raw dirs::home_dir())
        let mut candidate_homes = Vec::new();
        if let Some(host_home) = get_canonical_host_home() {
            if !is_secondary_sandbox_path(&host_home) {
                candidate_homes.push(host_home);
            }
        }
        if let Some(raw_home) = dirs::home_dir() {
            if !is_secondary_sandbox_path(&raw_home) && !candidate_homes.contains(&raw_home) {
                candidate_homes.push(raw_home);
            }
        }
        for home in candidate_homes {
            let gemini_root = home.join(".gemini");
            if gemini_root.exists()
                && !is_secondary_sandbox_path(&gemini_root)
                && !dirs.contains(&gemini_root)
            {
                dirs.push(gemini_root.clone());
            }
            for sub in ["antigravity", "antigravity-ide"] {
                let path = gemini_root.join(sub);
                if path.exists() && !is_secondary_sandbox_path(&path) && !dirs.contains(&path) {
                    dirs.push(path);
                }
            }
        }

        // 2. Sandboxed default instance home (if present)
        if let Ok(instances_dir) = crate::modules::instance::get_instances_dir() {
            let default_sandbox = instances_dir.join("default").join("home");
            if default_sandbox.exists() {
                let sandbox_gemini = default_sandbox.join(".gemini");
                if sandbox_gemini.exists() && !dirs.contains(&sandbox_gemini) {
                    dirs.push(sandbox_gemini.clone());
                }
                for sub in ["antigravity", "antigravity-ide"] {
                    let path = sandbox_gemini.join(sub);
                    if path.exists() && !dirs.contains(&path) {
                        dirs.push(path);
                    }
                }
            }
        }

        // 3. APPDATA / WorkspaceStorage Data Dir: get_default_antigravity_data_dir()
        let def_data_dir = crate::modules::instance::get_default_antigravity_data_dir();
        if def_data_dir.exists() && !is_secondary_sandbox_path(&def_data_dir) {
            if !dirs.contains(&def_data_dir) {
                dirs.push(def_data_dir.clone());
            }
            let def_gemini = def_data_dir.join(".gemini");
            if def_gemini.exists() && !dirs.contains(&def_gemini) {
                dirs.push(def_gemini.clone());
            }
            for sub in ["antigravity", "antigravity-ide"] {
                let path = def_gemini.join(sub);
                if path.exists() && !dirs.contains(&path) {
                    dirs.push(path);
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
    let mut secondary_dirs = HashSet::new();
    if let Ok(reg) = crate::modules::instance::load_registry() {
        for inst in &reg.instances {
            let is_default = inst.is_default || inst.id == "default" || inst.id == "__default__";
            if !is_default {
                for dir in gemini_dirs_for_instance(&inst.id) {
                    secondary_dirs.insert(dir.clone());
                    tagged.push((inst.id.clone(), dir));
                }
            }
        }
    }

    // 2. Add default directories, strictly excluding any secondary sandbox paths
    for dir in gemini_dirs_for_instance("default") {
        let is_secondary = secondary_dirs.contains(&dir) || is_secondary_sandbox_path(&dir);
        if !is_secondary {
            tagged.push(("default".to_string(), dir));
        }
    }

    // 3. Add isolated CLI directories, tagged with "__cli__"
    for dir in gemini_dirs_for_instance("__cli__") {
        let is_secondary = secondary_dirs.contains(&dir) || is_secondary_sandbox_path(&dir);
        if !is_secondary && !tagged.iter().any(|(_, d)| d == &dir) {
            tagged.push(("__cli__".to_string(), dir));
        }
    }

    tagged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_dirs_default_excludes_cli() {
        let default_dirs = gemini_dirs_for_instance("default");
        for d in default_dirs {
            let s = d.to_string_lossy().to_ascii_lowercase();
            assert!(
                !s.ends_with("antigravity-cli"),
                "Default instance directory candidate list must not contain antigravity-cli: {:?}",
                d
            );
        }
    }

    #[test]
    fn test_gemini_dirs_tagged_tags_cli() {
        let tagged = gemini_dirs_tagged(Some("all"));
        for (tag, path) in tagged {
            let s = path.to_string_lossy().to_ascii_lowercase();
            if s.ends_with("antigravity-cli") {
                assert_eq!(
                    tag, "__cli__",
                    "antigravity-cli directory must be tagged with __cli__, found: {}",
                    tag
                );
            }
        }
    }
}
