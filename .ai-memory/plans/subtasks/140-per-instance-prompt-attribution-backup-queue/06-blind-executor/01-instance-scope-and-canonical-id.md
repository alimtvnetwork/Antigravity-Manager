# Step 01: InstanceScope, canonical_instance_id, resolve_instance_input

You add the storage-layer instance key and the input-layer resolver. Nothing calls them yet except the tests; later steps switch callers over. `resolve_instance_id` stays exactly as it is.

## 1. Depends on

None. This is the first step.

## 2. Files you may edit

- `src-tauri/src/modules/instance.rs`

## 3. Find it

| Change | Anchor (`fn` signature) | Unique search literal | Line hint |
|---|---|---|---|
| A. Insert new code above `resolve_instance_id` | `pub fn resolve_instance_id(specifier: &str) -> Result<String, String> {` | `/// Resolve an instance query string (seq_num like` | 4370 (doc comment), 4372 (fn) |
| B. Add tests | `mod tests {` | `fn switch_spares_another_instances_pid()` | 6613 (`mod tests`), 6617 (first test) |

Search commands:

```text
gitmap aum search "Resolve an instance query string" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "fn switch_spares_another_instances_pid" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "InstanceScope" src-tauri/src --ext .rs
gitmap aum search "canonical_instance_id" src-tauri/src --ext .rs
gitmap aum search "InstanceScope|canonical_instance_id|resolve_instance_input|default_instance_id_in" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "InstanceScope|canonical_instance_id|resolve_instance_input|default_instance_id_in" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "InstanceScope|canonical_instance_id|resolve_instance_input|default_instance_id_in" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "InstanceScope|canonical_instance_id|resolve_instance_input|default_instance_id_in" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
```

The last six searches must return 0 hits before you start. If they return hits, someone already did this step (or a name collides): STOP.

This step only adds names; no existing signature changes, so no caller in `src-tauri/src`, `src-tauri/src/bin/agm.rs` or `src-tauri/tests/*.rs` needs an edit (checked 2026-10-06).

## 4. Current code

### Change A: the lines directly above and at the start of `resolve_instance_id` (around line 4366 to 4377)

You do not replace this code. You insert the new code between the closing `}` of `observe_instance` and the `///` doc comment of `resolve_instance_id`.

```rust
        last_heartbeat_line: last_line,
    })
}

/// Resolve an instance query string (seq_num like "1", ID like "inst-xyz", name like "Instance 1", or "default"/"active")
/// to a valid concrete instance ID.
pub fn resolve_instance_id(specifier: &str) -> Result<String, String> {
    let registry = load_registry()?;
    let clean = specifier.trim();
    if clean.is_empty() || clean.eq_ignore_ascii_case("active") {
        return get_active_instance_id();
    }
```

### Change B: the start of the test module (around line 6612 to 6617)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch_spares_another_instances_pid() {
```

Facts you can rely on (already verified):

- `InstanceConfig` and `InstanceRegistry` are in scope in `instance.rs` through `pub use crate::models::instance::{InstanceConfig, InstanceRegistry, InstanceStatus};` (line 2).
- `InstanceRegistry` has exactly two fields: `active_instance_id: String` and `instances: Vec<InstanceConfig>`.
- `InstanceConfig` has exactly these fields: `id: String`, `name: String`, `data_dir: String`, `executable_path: Option<String>`, `extensions_dir: Option<String>`, `bound_account_id: Option<String>`, `bound_email: Option<String>`, `created_at: i64`, `last_used: i64`, `is_default: bool`, `pid: Option<u32>`, `seq_num: Option<u32>`.
- `load_registry()` is at line 411 and `get_active_instance_id()` at line 4196, both return `Result<_, String>`.

## 5. New code

### Change A: insert this block immediately above the line `/// Resolve an instance query string (seq_num like "1", ...`

Leave one empty line between the closing `}` of `observe_instance` and this block, and one empty line between this block and the `///` line of `resolve_instance_id`.

```rust
/// Selects one canonical instance or every registered instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceScope {
    One(String),
    All,
}

/// Registry entry that owns the global store: `is_default` or id `"default"`; else `"default"`.
pub fn default_instance_id_in(registry: &InstanceRegistry) -> String {
    registry
        .instances
        .iter()
        .find(|i| i.is_default || i.id == "default")
        .map(|i| i.id.clone())
        .unwrap_or_else(|| "default".to_string())
}

fn known_instance_ids(registry: &InstanceRegistry) -> String {
    registry
        .instances
        .iter()
        .map(|i| i.id.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Storage-layer key. Exact registry match only. Never returns the active instance.
pub fn canonical_instance_id_in(registry: &InstanceRegistry, raw: &str) -> Result<String, String> {
    let clean = raw.trim();
    if clean.is_empty()
        || clean.eq_ignore_ascii_case("default")
        || clean.eq_ignore_ascii_case("__default__")
    {
        return Ok(default_instance_id_in(registry));
    }
    registry
        .instances
        .iter()
        .find(|i| i.id.eq_ignore_ascii_case(clean))
        .map(|i| i.id.clone())
        .ok_or_else(|| {
            format!(
                "unknown instance id '{}'; known: {}",
                clean,
                known_instance_ids(registry)
            )
        })
}

/// Storage-layer key loaded from the registry file. See `canonical_instance_id_in`.
pub fn canonical_instance_id(raw: &str) -> Result<String, String> {
    let registry = load_registry()?;
    canonical_instance_id_in(&registry, raw)
}

/// Input-layer resolver for CLI and UI text (`#2`, `2`, `8159`, display name, `active`).
/// `active_id` is used only when the input is the literal word `active`.
pub fn resolve_instance_input_in(
    registry: &InstanceRegistry,
    active_id: &str,
    input: &str,
) -> Result<String, String> {
    let clean = input.trim();
    if clean.is_empty() {
        return Err("instance required".to_string());
    }
    if clean.eq_ignore_ascii_case("active") {
        return canonical_instance_id_in(registry, active_id);
    }
    if let Ok(id) = canonical_instance_id_in(registry, clean) {
        return Ok(id);
    }
    if let Some(inst) = registry
        .instances
        .iter()
        .find(|i| i.name.eq_ignore_ascii_case(clean))
    {
        return canonical_instance_id_in(registry, &inst.id);
    }

    let lower = clean.to_ascii_lowercase();
    let seq_text = lower
        .strip_prefix('#')
        .or_else(|| lower.strip_prefix("instance-"))
        .or_else(|| lower.strip_prefix("ins-"))
        .unwrap_or(lower.as_str());
    if let Ok(num) = seq_text.parse::<u32>() {
        if let Some(inst) = registry.instances.iter().find(|i| i.seq_num == Some(num)) {
            return canonical_instance_id_in(registry, &inst.id);
        }
    }

    let suffix = lower.trim_start_matches('-');
    if suffix.is_empty() {
        return Err(format!(
            "unknown instance '{}'; known: {}",
            clean,
            known_instance_ids(registry)
        ));
    }
    let hyphen_suffix = format!("-{}", suffix);
    let candidates: Vec<&InstanceConfig> = registry
        .instances
        .iter()
        .filter(|i| {
            i.id.to_ascii_lowercase().ends_with(&hyphen_suffix)
                || i.name.to_ascii_lowercase().ends_with(&hyphen_suffix)
        })
        .collect();
    match candidates.as_slice() {
        [] => Err(format!(
            "unknown instance '{}'; known: {}",
            clean,
            known_instance_ids(registry)
        )),
        [only] => canonical_instance_id_in(registry, &only.id),
        many => Err(format!(
            "ambiguous instance '{}': candidates {}",
            clean,
            many.iter()
                .map(|i| i.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Input-layer resolver loaded from the registry file. See `resolve_instance_input_in`.
pub fn resolve_instance_input(input: &str) -> Result<String, String> {
    let clean = input.trim();
    if clean.is_empty() {
        return Err("instance required".to_string());
    }
    let registry = load_registry()?;
    let active_id = if clean.eq_ignore_ascii_case("active") {
        get_active_instance_id()?
    } else {
        String::new()
    };
    resolve_instance_input_in(&registry, &active_id, clean)
}
```

Do not change `resolve_instance_id`, `get_active_instance_id`, or any caller in this step.

## 6. Tests

Paste the two helpers and the seven tests directly after the line `    use super::*;` inside `mod tests` (before `#[test] fn switch_spares_another_instances_pid`). Step 02 reuses `scope_cfg` and `scope_registry`; do not rename them.

```rust
    fn scope_cfg(id: &str, name: &str, seq: u32, is_default: bool) -> InstanceConfig {
        InstanceConfig {
            id: id.to_string(),
            name: name.to_string(),
            data_dir: format!("/agm-test/{}/data", id),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default,
            pid: None,
            seq_num: Some(seq),
        }
    }

    fn scope_registry(active_id: &str) -> InstanceRegistry {
        InstanceRegistry {
            active_instance_id: active_id.to_string(),
            instances: vec![
                scope_cfg("default", "Default", 1, true),
                scope_cfg("work-1234", "Work", 2, false),
                scope_cfg("a-8159", "Alpha", 3, false),
                scope_cfg("b-8159", "Beta", 4, false),
            ],
        }
    }

    #[test]
    fn canonical_instance_id_maps_default_aliases() {
        let registry = scope_registry("default");
        for raw in ["", "  ", "default", "__default__", "DEFAULT", " Default "] {
            assert_eq!(
                canonical_instance_id_in(&registry, raw).unwrap(),
                "default",
                "alias {:?}",
                raw
            );
        }
        assert_eq!(
            canonical_instance_id_in(&registry, "WORK-1234").unwrap(),
            "work-1234"
        );

        let renamed_default = InstanceRegistry {
            active_instance_id: String::new(),
            instances: vec![scope_cfg("main-1", "Main", 1, true)],
        };
        assert_eq!(
            canonical_instance_id_in(&renamed_default, "__default__").unwrap(),
            "main-1"
        );

        let empty = InstanceRegistry::default();
        assert_eq!(canonical_instance_id_in(&empty, "").unwrap(), "default");
    }

    #[test]
    fn canonical_instance_id_rejects_unknown_all_and_active() {
        let registry = scope_registry("default");
        for raw in ["nope", "all", "active", "#2", "2", "Work"] {
            let err = canonical_instance_id_in(&registry, raw).unwrap_err();
            assert!(err.contains("unknown instance id"), "{}", err);
            assert!(err.contains("known: default, work-1234, a-8159, b-8159"), "{}", err);
        }
    }

    #[test]
    fn canonical_instance_id_never_returns_active_instance() {
        let registry = scope_registry("work-1234");
        assert_eq!(canonical_instance_id_in(&registry, "").unwrap(), "default");
        assert_eq!(
            canonical_instance_id_in(&registry, "default").unwrap(),
            "default"
        );
    }

    #[test]
    fn instance_id_and_ide_flavor_are_not_confused() {
        let registry = scope_registry("default");
        assert!(canonical_instance_id_in(&registry, "ide").is_err());
        assert!(canonical_instance_id_in(&registry, "agy").is_err());
        assert_eq!(
            canonical_instance_id_in(&registry, "default").unwrap(),
            "default"
        );
    }

    #[test]
    fn resolve_instance_input_rejects_ambiguous_suffix() {
        let registry = scope_registry("default");
        for input in ["8159", "-8159"] {
            let err = resolve_instance_input_in(&registry, "default", input).unwrap_err();
            assert!(err.starts_with("ambiguous instance"), "{}", err);
            assert!(err.contains("a-8159"), "{}", err);
            assert!(err.contains("b-8159"), "{}", err);
        }
    }

    #[test]
    fn resolve_instance_input_accepts_seq_and_name() {
        let registry = scope_registry("default");
        for input in ["#2", "2", "ins-2", "instance-2", "Work", "work", "work-1234", "1234"] {
            assert_eq!(
                resolve_instance_input_in(&registry, "default", input).unwrap(),
                "work-1234",
                "input {:?}",
                input
            );
        }
        assert_eq!(
            resolve_instance_input_in(&registry, "default", "a-8159").unwrap(),
            "a-8159"
        );
        assert_eq!(
            resolve_instance_input_in(&registry, "work-1234", "__default__").unwrap(),
            "default"
        );
    }

    #[test]
    fn resolve_instance_input_requires_input_and_uses_active_only_when_typed() {
        let registry = scope_registry("work-1234");
        assert_eq!(
            resolve_instance_input_in(&registry, "work-1234", "").unwrap_err(),
            "instance required"
        );
        assert_eq!(
            resolve_instance_input_in(&registry, "work-1234", "active").unwrap(),
            "work-1234"
        );
        assert!(resolve_instance_input_in(&registry, "ghost", "active").is_err());
        assert!(resolve_instance_input_in(&registry, "work-1234", "nope").is_err());
    }
```

Notes for this step:

- The original subtask table listed `instance_id_and_ide_flavor_are_not_confused` under `repo_db.rs`. It is placed here because the rule it checks (an IDE flavor is never accepted as an instance id) lives in `canonical_instance_id_in`.
- No test calls `canonical_instance_id`, `resolve_instance_input`, or `load_registry`. They only call the `_in` cores.

## 7. Gate

Run from the repo root. All commands must exit 0.

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 canonical_instance_id; cargo test --lib -- --test-threads=1 resolve_instance_input; cargo test --lib -- --test-threads=1 instance_id_and_ide_flavor_are_not_confused; cd ..
```

Each `cargo test` line must report at least one test run and 0 failed. `canonical_instance_id` must run 3 tests, `resolve_instance_input` must run 3 tests.

## 8. Commit

```text
Fix: prompts - add InstanceScope and canonical instance id resolvers
```

Stage only `src-tauri/src/modules/instance.rs`.

## 9. Done when

- [ ] `gitmap aum search "pub enum InstanceScope" src-tauri/src --ext .rs` returns exactly 1 hit in `instance.rs`.
- [ ] `canonical_instance_id_in`, `canonical_instance_id`, `resolve_instance_input_in`, `resolve_instance_input`, `default_instance_id_in` exist and are `pub`.
- [ ] `resolve_instance_id` is byte-for-byte unchanged.
- [ ] No new code calls `get_active_instance_id` except `resolve_instance_input` (only for the literal input `active`).
- [ ] The 7 new tests pass; no test calls `load_registry`.
- [ ] The collision searches in section 3 were run on `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file, and returned 0 hits.
- [ ] Gate exited 0; commit pushed with the exact message.
