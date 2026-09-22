fn main() {
    tauri_build::build();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_os == "windows" && target_env == "msvc" {
        let manifest_path = std::path::Path::new("windows-test.manifest");
        if manifest_path.exists() {
            println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
            if let Ok(abs_path) = manifest_path.canonicalize() {
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

                if let Some(target_dir) = debug_dir.parent() {
                    let _ = copy_dll_recursive(target_dir, &deps_dir, expected_arch_dir);
                }
            }
        }
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
