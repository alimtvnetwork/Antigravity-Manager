use super::*;

pub(crate) fn print_help() {
    println!("AGM Delegated Out-of-Process Update CLI:");
    println!("  agm delegate-update [--wait-pid <PID>] [--install-dir <PATH>] [--target-exe <PATH>] [--relaunch]");
    println!("  agm open-ui [--target-exe <PATH>] [--install-dir <PATH>]");
    println!();
    println!("Description:");
    println!("  3-Stage Self-Delegating CLI Update Pipeline:");
    println!("  1. Copies CLI to isolated %TEMP%\\agm-updater\\agm-update-cli-<pid>.exe and waits for UI exit.");
    println!("  2. Executes `agm-update-cli update --force --no-launch --install-dir <PATH>`.");
    println!("  3. Executes `agm-update-cli open-ui --target-exe <PATH>` to automatically reopen the updated UI.");
    println!();
    println!("Options:");
    println!("  --wait-pid <PID>     Wait for specified GUI process ID to exit before updating");
    println!("  --install-dir <PATH> Exact application installation directory to update");
    println!("  --target-exe <PATH>  Specific UI executable path to launch after update");
    println!("  --relaunch           Relaunch UI upon update completion (default: true)");
    println!("  --no-relaunch        Do not relaunch UI upon update completion");
    println!("  --version <VER>      Target release version to install");
}
