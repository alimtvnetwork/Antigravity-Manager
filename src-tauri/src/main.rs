// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let sub = args[1]
            .trim_start_matches('/')
            .trim_start_matches('-')
            .to_lowercase();
        if sub == "delegate-update" || sub == "update-ui" || sub == "ui-update-runner" {
            antigravity_tools_lib::modules::delegate_updater::run(&args[2..]);
            return;
        }
        if sub == "update" || sub == "update-all" || sub == "ua" {
            let ok = antigravity_tools_lib::modules::delegate_updater::run_cli_update(&args[2..]);
            std::process::exit(if ok { 0 } else { 1 });
        }
        if sub == "open-ui" || sub == "ui" || sub == "launch-ui" || sub == "start-ui" {
            antigravity_tools_lib::modules::delegate_updater::open_ui(&args[2..]);
            return;
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Fix for transparent window on some Linux systems
        // See: https://github.com/spacedriveapp/spacedrive/issues/1512#issuecomment-1758550164
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    antigravity_tools_lib::run()
}
