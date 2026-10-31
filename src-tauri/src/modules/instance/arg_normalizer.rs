//! Instance Launch Arg Normalizer.
//! Standardizes required Electron / Chromium launch flags for sandboxed execution.

use std::path::Path;

/// Standard keyring bypass arguments across Linux and Chromium.
pub const KEYRING_BYPASS_ARGS: &[&str] = &["--password-store=basic", "--use-mock-keychain"];
/// Dynamic OS-allocated ephemeral port flag.
pub const DYNAMIC_CDP_PORT_ARG: &str = "--remote-debugging-port=0";

/// Normalizes CLI launch arguments for an isolated instance profile, ensuring all required flags exist.
pub fn normalize_instance_launch_args(data_dir: &Path, existing_args: &[String]) -> Vec<String> {
    let mut args = existing_args.to_vec();
    let data_dir_str = data_dir.to_string_lossy().to_string();
    let user_data_prefix = "--user-data-dir=";

    // 1. Ensure --user-data-dir is present and points to data_dir
    let has_user_data = args.iter().any(|a| a.starts_with(user_data_prefix));
    if !has_user_data {
        args.push(format!("{}{}", user_data_prefix, data_dir_str));
    }

    // 2. Ensure dynamic CDP port argument is present
    let has_cdp = args
        .iter()
        .any(|a| a.starts_with("--remote-debugging-port="));
    if !has_cdp {
        args.push(DYNAMIC_CDP_PORT_ARG.to_string());
    }

    // 3. Ensure keyring bypass arguments are present
    for bypass_arg in KEYRING_BYPASS_ARGS {
        if !args.iter().any(|a| a == *bypass_arg) {
            args.push(bypass_arg.to_string());
        }
    }

    args
}
