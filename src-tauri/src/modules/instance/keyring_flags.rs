//! Keyring Mock Mode Arguments Injector.
//! Injects OS-specific credentials isolation flags into spawned IDE instances.

/// Injects OS keyring bypass arguments to prevent sandbox profiles from modifying the host user's keychain.
pub fn inject_keyring_isolation_flags(args: &mut Vec<String>, is_isolated_sandbox: bool) {
    if !is_isolated_sandbox {
        return;
    }

    let os = std::env::consts::OS;

    match os {
        "linux" => {
            if !args.iter().any(|a| a == "--password-store=basic") {
                args.push("--password-store=basic".to_string());
            }
        }
        "macos" => {
            if !args.iter().any(|a| a == "--use-mock-keychain") {
                args.push("--use-mock-keychain".to_string());
            }
        }
        "windows" => {
            if !args.iter().any(|a| a == "--password-store=basic") {
                args.push("--password-store=basic".to_string());
            }
        }
        _ => {}
    }
}
