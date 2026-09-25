fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    if target_os == "windows" && target_env == "msvc" {
        let windows = tauri_build::WindowsAttributes::new_without_app_manifest();
        let attrs = tauri_build::Attributes::new().windows_attributes(windows);
        tauri_build::try_build(attrs).expect("failed to run tauri-build");

        // Embed Common-Controls 6.0 and Windows 10/11 compatibility manifest into all executables
        // (both agm-alim.exe binary and antigravity_tools_lib test runner executables)
        // using new_without_app_manifest to prevent duplicate resource (CVT1100) collisions.
        let manifest_path = std::path::Path::new("windows-test.manifest");
        if manifest_path.exists() {
            if let Ok(abs_path) = manifest_path.canonicalize() {
                println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
                println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", abs_path.display());
            }
        }

        let expected_arch_dir = match target_arch.as_str() {
            "x86_64" => "x64",
            "x86" => "x86",
            "aarch64" => "arm64",
            _ => "x64",
        };

        if let Ok(out_dir) = std::env::var("OUT_DIR") {
            let out_path = std::path::PathBuf::from(out_dir);
            if let Some(debug_dir) = out_path.ancestors().nth(3) {
                let deps_dir = debug_dir.join("deps");
                let _ = std::fs::create_dir_all(&deps_dir);

                let build_dir = debug_dir.join("build");
                if build_dir.exists() {
                    let _ = copy_dll_recursive(&build_dir, &deps_dir, expected_arch_dir);
                    let _ = copy_dll_recursive(&build_dir, debug_dir, expected_arch_dir);
                }
            }
        }
    } else {
        tauri_build::build();
    }
}

fn copy_dll_recursive(
    src: &std::path::Path,
    dst_deps: &std::path::Path,
    expected_arch: &str,
) -> std::io::Result<()> {
    if src.is_dir() {
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let _ = copy_dll_recursive(&path, dst_deps, expected_arch);
            } else if path.is_file() {
                let is_webview2 = path
                    .file_name()
                    .map(|n| {
                        n.to_string_lossy()
                            .eq_ignore_ascii_case("WebView2Loader.dll")
                    })
                    .unwrap_or(false);
                if is_webview2 {
                    let is_matching_arch = path
                        .parent()
                        .and_then(|p| p.file_name())
                        .map(|n| n.to_string_lossy().eq_ignore_ascii_case(expected_arch))
                        .unwrap_or(false);
                    if is_matching_arch {
                        let _ = std::fs::copy(&path, dst_deps.join("WebView2Loader.dll"));
                    }
                }
            }
        }
    }
    Ok(())
}
