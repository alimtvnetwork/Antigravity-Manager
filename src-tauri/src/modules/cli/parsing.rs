use crate::modules::{account, auto_switcher, config, instance, repo_db};

use super::*;

pub(crate) fn extract_flag_value(args: &[String], flags: &[&str]) -> Option<String> {
    for (idx, arg) in args.iter().enumerate() {
        if flags.iter().any(|f| arg == f) {
            if let Some(val) = args.get(idx + 1) {
                if !val.starts_with('-') {
                    return Some(val.clone());
                }
            }
        }
    }
    None
}

pub(crate) fn extract_positional_args(args: &[String]) -> Vec<String> {
    let mut positional = Vec::new();
    let mut is_skipping_next = false;
    for arg in args {
        if is_skipping_next {
            is_skipping_next = false;
            continue;
        }
        if arg == "-i"
            || arg == "--instance"
            || arg == "--profile"
            || arg == "-r"
            || arg == "--repo"
            || arg == "--workspace"
            || arg == "-l"
            || arg == "--limit"
            || arg == "-k"
            || arg == "--keep"
        {
            is_skipping_next = true;
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        positional.push(arg.clone());
    }
    positional
}
