# Step 04: gemini_dirs_for_instance and gemini_dirs_tagged(&InstanceScope)

Today `gemini_dirs_tagged(None)` means "all", `gemini_dirs_for_instance("all")` silently returns the default dirs, and a named instance's dirs are found through `get_instance_home_dir` (which used to create folders). After this step:

- `gemini_dirs_for_instance_in(registry, id, host_home, instances_root)` takes a canonical id and returns `Err` for `"all"` or any unknown id.
- `gemini_dirs_tagged_in(registry, &InstanceScope, host_home, instances_root)` returns `(instance_id, dir)` pairs. `All` lists the default instance first, then every other registry id. Each dir appears once; if two owners would share a dir, the first owner (default) keeps it and the skip is logged. Nothing is ever tagged `"all"`.
- The thin wrappers `gemini_dirs_for_instance(&str)` and `gemini_dirs_tagged(&InstanceScope)` load the registry and call the cores.

## 1. Depends on

Step 01 (`InstanceScope`, `canonical_instance_id`, `canonical_instance_id_in`, `default_instance_id_in`).

## 2. Files you may edit

- `src-tauri/src/modules/repo_db.rs`

## 3. Find it

| Change | Anchor | Unique search literal | Line hint |
|---|---|---|---|
| A. Import | top of file | `use chrono::Utc;` | 9 |
| B. Replace both functions | `pub fn gemini_dirs_for_instance(instance_id: &str) -> Vec<PathBuf> {` and `pub fn gemini_dirs_tagged(instance_id: Option<&str>) -> Vec<(String, PathBuf)> {` | `pub fn gemini_dirs_tagged(instance_id: Option<&str>)` | 824 to 905 |
| C1. Caller | `pub fn get_live_project_execution_info() -> Vec<ProjectExecutionInfo> {` (line 2480) | `let candidate_dirs = gemini_dirs_tagged(None);` | 2490 |
| C2. Caller | `fn compute_project_conversation_tree(` (line 4042) | `let candidate_dirs_fb = gemini_dirs_tagged(target);` | 4107 |
| C3. Caller | `fn compute_project_conversation_tree(` (line 4042) | `let candidate_dirs = gemini_dirs_tagged(target);` | 4245 |
| D. Tests | `mod tests {` | `fn test_uri_decoding()` | about 5880 after step 03 |

```text
gitmap aum search "gemini_dirs_tagged(" src-tauri/src --ext .rs
gitmap aum search "gemini_dirs_for_instance(" src-tauri/src --ext .rs
gitmap aum search "gemini_dirs_tagged|gemini_dirs_for_instance" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "gemini_dirs_tagged|gemini_dirs_for_instance" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "gemini_dirs_tagged|gemini_dirs_for_instance" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "gemini_dirs_tagged|gemini_dirs_for_instance" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
```

The last four searches must return 0 hits (checked 2026-10-06). `gemini_dirs_tagged` changes its parameter from `Option<&str>` to `&InstanceScope`, so any hit there is a caller this step must update; if one appears, STOP and report it.

Expected before you start: `gemini_dirs_tagged(` has 4 hits (definition 867, callers 2490, 4107, 4245). `gemini_dirs_for_instance(` has 6 hits (definition 824, internal calls 877, 889, 898 inside the old `gemini_dirs_tagged`, callers 959 and 1911). Line numbers shift by about 20 after step 03; anchor on the literals.

Callers of `gemini_dirs_for_instance` that need NO change (the wrapper keeps the signature `(&str) -> Vec<PathBuf>`):

- `repo_db.rs:959` in `discover_running_prompts_from_antigravity` (rewritten in step 05).
- `repo_db.rs:1911` in `is_prompt_running_for_project`: `let candidate_dirs = gemini_dirs_for_instance(norm_inst);`. `norm_inst` is already a resolved id.

## 4. Current code

### A. Imports (lines 7 to 19)

```rust
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, Once, OnceLock};
use std::time::Duration;
use uuid::Uuid;
```

### B. Both functions (lines 824 to 905)

```rust
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
```

### C1 (line 2490)

```rust
    // 1. Inspect Antigravity conversation_summaries.db
    let candidate_dirs = gemini_dirs_tagged(None);
```

### C2 (lines 4105 to 4107)

```rust
    if projects.is_empty() {
        let now_fb = Utc::now().timestamp();
        let candidate_dirs_fb = gemini_dirs_tagged(target);
```

### C3 (line 4245)

```rust
    let candidate_dirs = gemini_dirs_tagged(target);
```

In C2 and C3, `target` is `Option<&str>`, defined at line 4047 as `let target = target_instance.filter(|t| !t.is_empty() && *t != "all");`. The only external caller (`src-tauri/src/commands/instance.rs:312`, `get_project_conversation_tree`) passes a registry id from the UI.

Helpers you can rely on (already verified): `get_canonical_host_home() -> Option<PathBuf>` (line 805), `normalize_path_for_compare(p: &str) -> String` (private, line 2473, usable inside this module), `crate::modules::instance::get_instances_dir() -> Result<PathBuf, String>`, `crate::modules::instance::load_registry() -> Result<InstanceRegistry, String>`, `crate::modules::logger::log_warn(&str)`.

## 5. New code

### A. Add this import line directly after `use chrono::Utc;`

```rust
use crate::modules::instance::{InstanceRegistry, InstanceScope};
```

`cargo fmt` may move it; that is fine.

### B. Replace the whole block from 4 B with

```rust
const GEMINI_FLAVOR_DIRS: [&str; 3] = ["antigravity", "antigravity-ide", "antigravity-cli"];

fn push_existing_gemini_dirs(home: &Path, dirs: &mut Vec<PathBuf>) {
    for sub in GEMINI_FLAVOR_DIRS {
        let path = home.join(".gemini").join(sub);
        if path.exists() && !dirs.contains(&path) {
            dirs.push(path);
        }
    }
}

/// Gemini store dirs of one registered instance. Never creates folders.
/// The default instance owns the host home and `<instances>/default/home`.
pub fn gemini_dirs_for_instance_in(
    registry: &InstanceRegistry,
    instance_id: &str,
    host_home: Option<&Path>,
    instances_root: &Path,
) -> Result<Vec<PathBuf>, String> {
    let canonical_id = crate::modules::instance::canonical_instance_id_in(registry, instance_id)?;
    let default_id = crate::modules::instance::default_instance_id_in(registry);
    let mut dirs = Vec::new();
    if canonical_id == default_id {
        if let Some(home) = host_home {
            push_existing_gemini_dirs(home, &mut dirs);
        }
        push_existing_gemini_dirs(&instances_root.join("default").join("home"), &mut dirs);
        if default_id != "default" {
            push_existing_gemini_dirs(&instances_root.join(&default_id).join("home"), &mut dirs);
        }
    } else {
        push_existing_gemini_dirs(&instances_root.join(&canonical_id).join("home"), &mut dirs);
    }
    Ok(dirs)
}

/// `(instance_id, dir)` pairs for a scope. Each dir appears once; `All` lists default first.
pub fn gemini_dirs_tagged_in(
    registry: &InstanceRegistry,
    scope: &InstanceScope,
    host_home: Option<&Path>,
    instances_root: &Path,
) -> Vec<(String, PathBuf)> {
    let requested_ids: Vec<String> = match scope {
        InstanceScope::One(id) => vec![id.clone()],
        InstanceScope::All => {
            let default_id = crate::modules::instance::default_instance_id_in(registry);
            let mut ids = vec![default_id.clone()];
            for inst in &registry.instances {
                if inst.id != default_id {
                    ids.push(inst.id.clone());
                }
            }
            ids
        }
    };

    let mut tagged: Vec<(String, PathBuf)> = Vec::new();
    let mut seen_dirs: HashSet<String> = HashSet::new();
    for requested_id in requested_ids {
        let owner_id =
            match crate::modules::instance::canonical_instance_id_in(registry, &requested_id) {
                Ok(id) => id,
                Err(e) => {
                    crate::modules::logger::log_warn(&format!(
                        "[RepoDB] Gemini dirs skipped: {}",
                        e
                    ));
                    continue;
                }
            };
        let dirs = gemini_dirs_for_instance_in(registry, &owner_id, host_home, instances_root)
            .unwrap_or_default();
        for dir in dirs {
            if seen_dirs.insert(normalize_path_for_compare(&dir.to_string_lossy())) {
                tagged.push((owner_id.clone(), dir));
            } else {
                crate::modules::logger::log_warn(&format!(
                    "[RepoDB] Gemini dir {} already has an owner; not tagged as '{}'",
                    dir.display(),
                    owner_id
                ));
            }
        }
    }
    tagged
}

pub fn gemini_dirs_for_instance(instance_id: &str) -> Vec<PathBuf> {
    let registry = match crate::modules::instance::load_registry() {
        Ok(registry) => registry,
        Err(e) => {
            crate::modules::logger::log_warn(&format!("[RepoDB] Gemini dirs skipped: {}", e));
            return Vec::new();
        }
    };
    let instances_root = match crate::modules::instance::get_instances_dir() {
        Ok(root) => root,
        Err(e) => {
            crate::modules::logger::log_warn(&format!("[RepoDB] Gemini dirs skipped: {}", e));
            return Vec::new();
        }
    };
    let host_home = get_canonical_host_home();
    match gemini_dirs_for_instance_in(
        &registry,
        instance_id,
        host_home.as_deref(),
        &instances_root,
    ) {
        Ok(dirs) => dirs,
        Err(e) => {
            crate::modules::logger::log_warn(&format!(
                "[RepoDB] Gemini dirs for '{}' skipped: {}",
                instance_id, e
            ));
            Vec::new()
        }
    }
}

pub fn gemini_dirs_tagged(scope: &InstanceScope) -> Vec<(String, PathBuf)> {
    let registry = match crate::modules::instance::load_registry() {
        Ok(registry) => registry,
        Err(e) => {
            crate::modules::logger::log_warn(&format!("[RepoDB] Gemini dirs skipped: {}", e));
            return Vec::new();
        }
    };
    let instances_root = match crate::modules::instance::get_instances_dir() {
        Ok(root) => root,
        Err(e) => {
            crate::modules::logger::log_warn(&format!("[RepoDB] Gemini dirs skipped: {}", e));
            return Vec::new();
        }
    };
    let host_home = get_canonical_host_home();
    gemini_dirs_tagged_in(&registry, scope, host_home.as_deref(), &instances_root)
}

/// Scope for an optional tree target. `None` means every instance; an unknown id gives `None`.
fn instance_scope_for_target(target: Option<&str>) -> Option<InstanceScope> {
    match target {
        None => Some(InstanceScope::All),
        Some(raw) => match crate::modules::instance::canonical_instance_id(raw) {
            Ok(id) => Some(InstanceScope::One(id)),
            Err(e) => {
                crate::modules::logger::log_warn(&format!(
                    "[RepoDB] Prompt tree target ignored: {}",
                    e
                ));
                None
            }
        },
    }
}
```

### C1: replace the line `    let candidate_dirs = gemini_dirs_tagged(None);` with

```rust
    let candidate_dirs = gemini_dirs_tagged(&InstanceScope::All);
```

### C2: replace the line `        let candidate_dirs_fb = gemini_dirs_tagged(target);` with

```rust
        let candidate_dirs_fb = instance_scope_for_target(target)
            .map(|scope| gemini_dirs_tagged(&scope))
            .unwrap_or_default();
```

### C3: replace the line `    let candidate_dirs = gemini_dirs_tagged(target);` with

```rust
    let candidate_dirs = instance_scope_for_target(target)
        .map(|scope| gemini_dirs_tagged(&scope))
        .unwrap_or_default();
```

Do not change anything else in `get_live_project_execution_info` or `compute_project_conversation_tree` (their `resolve_instance_id` calls are replaced in step 08).

## 6. Tests

Paste after `    use super::*;` in `mod tests` of `repo_db.rs`. They build an in-memory registry and temp folders; no environment variable is touched and `load_registry` is never called.

```rust
    fn dirs_cfg(id: &str, is_default: bool) -> crate::modules::instance::InstanceConfig {
        crate::modules::instance::InstanceConfig {
            id: id.to_string(),
            name: id.to_string(),
            data_dir: String::new(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default,
            pid: None,
            seq_num: None,
        }
    }

    fn dirs_registry() -> InstanceRegistry {
        InstanceRegistry {
            active_instance_id: "inst-a".to_string(),
            instances: vec![
                dirs_cfg("default", true),
                dirs_cfg("inst-a", false),
                dirs_cfg("inst-b", false),
            ],
        }
    }

    fn make_gemini_dir(home: &Path, flavor: &str) -> PathBuf {
        let dir = home.join(".gemini").join(flavor);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn gemini_dirs_tagged_all_never_stamps_all() {
        let tmp = tempfile::tempdir().unwrap();
        let host_home = tmp.path().join("host");
        let root = tmp.path().join("instances");
        let host_dir = make_gemini_dir(&host_home, "antigravity");
        let sandbox_dir = make_gemini_dir(&root.join("default").join("home"), "antigravity-ide");
        let a_dir = make_gemini_dir(&root.join("inst-a").join("home"), "antigravity");
        let registry = dirs_registry();

        let tagged = gemini_dirs_tagged_in(
            &registry,
            &InstanceScope::All,
            Some(host_home.as_path()),
            &root,
        );

        assert!(tagged.iter().all(|(id, _)| id != "all" && !id.is_empty()));
        let mut seen = HashSet::new();
        for (_, dir) in &tagged {
            assert!(seen.insert(dir.clone()), "dir listed twice: {}", dir.display());
        }
        assert_eq!(tagged[0].0, "default");
        assert!(tagged.contains(&("default".to_string(), host_dir)));
        assert!(tagged.contains(&("default".to_string(), sandbox_dir)));
        assert!(tagged.contains(&("inst-a".to_string(), a_dir)));
        assert!(tagged.iter().all(|(id, _)| id != "inst-b"));
    }

    #[test]
    fn gemini_dirs_tagged_all_keeps_default_owner_for_shared_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("instances");
        let a_home = root.join("inst-a").join("home");
        let shared = make_gemini_dir(&a_home, "antigravity");
        let registry = dirs_registry();

        let tagged =
            gemini_dirs_tagged_in(&registry, &InstanceScope::All, Some(a_home.as_path()), &root);

        let owners: Vec<&str> = tagged
            .iter()
            .filter(|(_, dir)| *dir == shared)
            .map(|(id, _)| id.as_str())
            .collect();
        assert_eq!(owners, vec!["default"]);
    }

    #[test]
    fn gemini_dirs_tagged_one_named_scope_reads_only_its_home() {
        let tmp = tempfile::tempdir().unwrap();
        let host_home = tmp.path().join("host");
        let root = tmp.path().join("instances");
        make_gemini_dir(&host_home, "antigravity");
        let a_dir = make_gemini_dir(&root.join("inst-a").join("home"), "antigravity-cli");
        let registry = dirs_registry();

        let tagged = gemini_dirs_tagged_in(
            &registry,
            &InstanceScope::One("inst-a".to_string()),
            Some(host_home.as_path()),
            &root,
        );

        assert_eq!(tagged, vec![("inst-a".to_string(), a_dir)]);
    }

    #[test]
    fn gemini_dirs_for_instance_rejects_all_and_unknown() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("instances");
        let registry = dirs_registry();

        assert!(gemini_dirs_for_instance_in(&registry, "all", None, &root).is_err());
        assert!(gemini_dirs_for_instance_in(&registry, "ghost", None, &root).is_err());
        assert!(gemini_dirs_tagged_in(
            &registry,
            &InstanceScope::One("ghost".to_string()),
            None,
            &root
        )
        .is_empty());
        assert!(!root.join("ghost").exists());
        assert!(gemini_dirs_for_instance_in(&registry, "__default__", None, &root)
            .unwrap()
            .is_empty());
    }
```

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 gemini_dirs; cd ..
```

`gemini_dirs` must run 4 tests, 0 failed.

## 8. Commit

```text
Fix: prompts - tag gemini dirs by instance scope, never as all
```

Stage only `src-tauri/src/modules/repo_db.rs`.

## 9. Done when

- [ ] `gitmap aum search "gemini_dirs_tagged(None)" src-tauri/src --ext .rs` returns 0 hits.
- [ ] `gitmap aum search "gemini_dirs_tagged(target)" src-tauri/src --ext .rs` returns 0 hits.
- [ ] `gemini_dirs_for_instance_in` and `gemini_dirs_tagged_in` take `&InstanceRegistry`; neither calls `load_registry` or `get_instance_home_dir`.
- [ ] The string `"all"` is not compared anywhere in the new code.
- [ ] Callers of `gemini_dirs_tagged` and `gemini_dirs_for_instance` were searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file; the only callers are the three in `repo_db.rs` (C1 to C3) plus the two unchanged `gemini_dirs_for_instance` calls.
- [ ] The 4 tests pass.
- [ ] Gate exited 0; commit pushed with the exact message.

Known temporary gap: until step 05 is committed, `discover_running_prompts_from_antigravity("all")` (called by `backup_running_prompts("all")`) gets no dirs, because `"all"` is no longer silently treated as default. Before this step it stamped those rows `"all"` (bug B04). Step 05 closes the gap by scanning `InstanceScope::All`.
