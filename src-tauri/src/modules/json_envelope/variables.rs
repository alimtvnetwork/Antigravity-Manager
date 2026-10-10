use serde_json::Value;
use std::collections::HashMap;

use super::*;

pub(crate) fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

/// Merges top-level and workDirectory variable dictionaries into a single lookup table.
pub fn merge_variables(
    top_vars: Option<&HashMap<String, Value>>,
    work_dir_vars: Option<&HashMap<String, Value>>,
) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if let Some(w_vars) = work_dir_vars {
        for (k, v) in w_vars {
            out.insert(k.clone(), value_to_string(v));
        }
    }
    if let Some(t_vars) = top_vars {
        for (k, v) in t_vars {
            out.insert(k.clone(), value_to_string(v));
        }
    }
    resolve_chained_variables(&mut out);
    out
}

pub(crate) fn resolve_chained_variables(vars: &mut HashMap<String, String>) {
    for _ in 0..2 {
        let snapshot = vars.clone();
        for val in vars.values_mut() {
            if val.contains('$') {
                *val = resolve_string_variable(val, &snapshot);
            }
        }
    }
}

/// Replaces `${varName}`, `${variables.varName}`, and `$variables.varName` in input string.
pub fn resolve_string_variable(input: &str, vars: &HashMap<String, String>) -> String {
    if !input.contains('$') || vars.is_empty() {
        return input.to_string();
    }
    let mut res = input.to_string();
    for _ in 0..2 {
        let before = res.clone();
        for (k, v) in vars {
            let pat1 = format!("${{{}}}", k);
            let pat2 = format!("${{variables.{}}}", k);
            let pat3 = format!("$variables.{}", k);
            if res.contains(&pat1) {
                res = res.replace(&pat1, v);
            }
            if res.contains(&pat2) {
                res = res.replace(&pat2, v);
            }
            if res.contains(&pat3) {
                res = res.replace(&pat3, v);
            }
        }
        if res == before {
            break;
        }
    }
    res
}

/// Recursively expands variables across all string values in a Serde JSON Value.
pub fn expand_variables_in_value(val: &mut Value, vars: &HashMap<String, String>) {
    if vars.is_empty() {
        return;
    }
    match val {
        Value::String(s) => {
            if s.contains('$') {
                *s = resolve_string_variable(s, vars);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                expand_variables_in_value(item, vars);
            }
        }
        Value::Object(map) => {
            for (_k, v) in map.iter_mut() {
                expand_variables_in_value(v, vars);
            }
        }
        _ => {}
    }
}
