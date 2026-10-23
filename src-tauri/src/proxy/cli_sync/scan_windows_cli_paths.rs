use super::*;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Windows common CLI installation path scanning
#[cfg(target_os = "windows")]
pub(crate) fn scan_windows_cli_paths(cmd: &str) -> Option<PathBuf> {
    let mut common_paths: Vec<PathBuf> = Vec::new();

    // Common Windows installation paths, sorted by priority (only absolute paths that can be deduced are added to avoid misjudgment of empty/relative paths)
    if let Some(app_data) = std::env::var_os("APPDATA") {
        let npm_base = PathBuf::from(app_data).join("npm");
        common_paths.push(npm_base.join(format!("{}.cmd", cmd)));
        common_paths.push(npm_base.join(cmd));
    }

    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let pnpm_base = PathBuf::from(&local_app_data).join("pnpm");
        common_paths.push(pnpm_base.join(format!("{}.cmd", cmd)));
        common_paths.push(pnpm_base.join(cmd));

        let yarn_base = PathBuf::from(local_app_data).join("Yarn").join("bin");
        common_paths.push(yarn_base.join(format!("{}.cmd", cmd)));
        common_paths.push(yarn_base.join(cmd));
    }

    if let Some(home) = dirs::home_dir() {
        let bun_base = home.join(".bun").join("bin");
        common_paths.push(bun_base.join(format!("{}.exe", cmd)));
        common_paths.push(bun_base.join(cmd));

        let local_bin = home.join(".local").join("bin");
        common_paths.push(local_bin.join(format!("{}.exe", cmd)));
        common_paths.push(local_bin.join(format!("{}.cmd", cmd)));
        common_paths.push(local_bin.join(cmd));

        let cargo_bin = home.join(".cargo").join("bin");
        common_paths.push(cargo_bin.join(format!("{}.exe", cmd)));
        common_paths.push(cargo_bin.join(format!("{}.cmd", cmd)));
        common_paths.push(cargo_bin.join(cmd));

        let grok_bin = home.join(".grok").join("bin");
        common_paths.push(grok_bin.join(format!("{}.exe", cmd)));
        common_paths.push(grok_bin.join(format!("{}.cmd", cmd)));
        common_paths.push(grok_bin.join(cmd));

        let grokbuild_bin = home.join(".grokbuild").join("bin");
        common_paths.push(grokbuild_bin.join(format!("{}.exe", cmd)));
        common_paths.push(grokbuild_bin.join(format!("{}.cmd", cmd)));
        common_paths.push(grokbuild_bin.join(cmd));
    }

    for path in common_paths {
        if is_safe_path(&path) {
            tracing::debug!(
                "[CLI-Sync] Detected {} via Windows explicit path: {:?}",
                cmd,
                path
            );
            return Some(path);
        }
    }

    // Scan NVM Windows directory
    if let Ok(nvm_home) = std::env::var("NVM_HOME") {
        let nvm_path = PathBuf::from(nvm_home);
        if nvm_path.is_dir() {
            // NVM Windows structure: %NVM_HOME%\v {version} \ {cmd} .cmd
            if let Ok(entries) = fs::read_dir(&nvm_path) {
                for entry in entries.flatten() {
                    let cmd_path = entry.path().join(format!("{}.cmd", cmd));
                    if is_safe_path(&cmd_path) {
                        tracing::debug!("[CLI-Sync] Detected {} via NVM_HOME: {:?}", cmd, cmd_path);
                        return Some(cmd_path);
                    }
                    // Also check the .exe version
                    let exe_path = entry.path().join(format!("{}.exe", cmd));
                    if is_safe_path(&exe_path) {
                        tracing::debug!("[CLI-Sync] Detected {} via NVM_HOME: {:?}", cmd, exe_path);
                        return Some(exe_path);
                    }
                }
            }
        }
    }

    None
}

/// Parse the where command output to get the first valid path
#[cfg(target_os = "windows")]
pub(crate) fn parse_where_output(output: &[u8]) -> Option<PathBuf> {
    let stdout = String::from_utf8_lossy(output);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if is_safe_path(&path) {
                return Some(path);
            }
        }
    }
    None
}

/// Detects the existence of an alternate CLI alias command and returns its path
pub(crate) fn detect_fallback_binary(name: &str) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let mut c = Command::new("where");
        c.arg(name);
        c.creation_flags(CREATE_NO_WINDOW);
        if let Ok(out) = c.output() {
            if out.status.success() {
                return parse_where_output(&out.stdout);
            }
        }
        None
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(out) = Command::new("which").arg(name).output() {
            if out.status.success() {
                return Some(PathBuf::from(name));
            }
        }
        None
    }
}

/// Check if the path is a .cmd/.bat file
#[cfg(target_os = "windows")]
pub(crate) fn is_cmd_file(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"))
        .unwrap_or(false)
}

/// Verify the path is safe (prevent command injection)
#[cfg(target_os = "windows")]
pub(crate) fn is_safe_path(path: &PathBuf) -> bool {
    // Check if the path exists and is a file
    if !path.exists() || !path.is_file() {
        return false;
    }

    // Must be an absolute path, avoid executing relative path files
    if !path.is_absolute() {
        return false;
    }

    // Check if path contains dangerous characters
    let path_str = path.to_string_lossy();
    let dangerous_chars = ['&', '|', ';', '<', '>', '(', ')', '`', '$', '^', '%', '!'];
    if path_str.chars().any(|c| dangerous_chars.contains(&c)) {
        tracing::warn!(
            "[CLI-Sync] Path contains dangerous characters: {}",
            path_str
        );
        return false;
    }

    true
}

/// Execute version command (Windows special processing .cmd/.bat)
#[cfg(target_os = "windows")]
pub(crate) fn run_version_command(executable_path: &PathBuf) -> Option<String> {
    // Security verification: Verify that the path does not contain dangerous characters
    if !is_safe_path(executable_path) {
        return None;
    }

    let output = if is_cmd_file(executable_path) {
        // Use quotes to prevent injection and use the /S switch to ensure safe parsing.
        let quoted_path = format!("\"{}\"", executable_path.to_string_lossy());
        Command::new("cmd.exe")
            .arg("/C")
            .arg(&quoted_path)
            .arg("--version")
            .creation_flags(CREATE_NO_WINDOW)
            .output()
    } else {
        let mut cmd = Command::new(executable_path);
        cmd.arg("--version");
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output()
    };

    match output {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            // Use regular expressions to extract version numbers (more precise)
            extract_version(&s)
        }
        _ => None,
    }
}

/// Extract version number (using more precise semver matching)
pub(crate) fn extract_version(s: &str) -> Option<String> {
    // Matches semver format: x.y.z or x.y
    let re = regex::Regex::new(r"(\d+\.\d+(?:\.\d+)?)").ok()?;
    re.captures(s)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum CliApp {
    Claude,
    Codex,
    Gemini,
    OpenCode,
    JeikCode,
    GrokBuild,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CliConfigFile {
    pub name: String,
    pub path: PathBuf,
}

impl CliApp {
    pub fn as_str(&self) -> &'static str {
        match self {
            CliApp::Claude => "claude",
            CliApp::Codex => "codex",
            CliApp::Gemini => "gemini",
            CliApp::OpenCode => "opencode",
            CliApp::JeikCode => "jeikcode",
            CliApp::GrokBuild => "grok",
        }
    }

    pub fn config_files(&self) -> Vec<CliConfigFile> {
        let home = match dirs::home_dir() {
            Some(p) => p,
            None => return vec![],
        };
        match self {
            CliApp::Claude => vec![
                CliConfigFile {
                    name: ".claude.json".to_string(),
                    path: home.join(".claude.json"),
                },
                CliConfigFile {
                    name: "settings.json".to_string(),
                    path: home.join(".claude").join("settings.json"),
                },
            ],
            CliApp::Codex => {
                let codex_dir = if home.join(".chatgpt").exists() || !home.join(".codex").exists() {
                    home.join(".chatgpt")
                } else {
                    home.join(".codex")
                };
                vec![
                    CliConfigFile {
                        name: "auth.json".to_string(),
                        path: codex_dir.join("auth.json"),
                    },
                    CliConfigFile {
                        name: "config.toml".to_string(),
                        path: codex_dir.join("config.toml"),
                    },
                ]
            }
            CliApp::Gemini => vec![
                CliConfigFile {
                    name: ".env".to_string(),
                    path: home.join(".gemini").join(".env"),
                },
                CliConfigFile {
                    name: "settings.json".to_string(),
                    path: home.join(".gemini").join("settings.json"),
                },
                CliConfigFile {
                    name: "config.json".to_string(),
                    path: home.join(".gemini").join("config.json"),
                },
            ],
            CliApp::OpenCode => vec![CliConfigFile {
                name: "config.json".to_string(),
                path: home.join(".opencode").join("config.json"),
            }],
            CliApp::JeikCode => {
                let jeikcode_dir = if let Ok(custom_home) = std::env::var("JEIKCODE_HOME") {
                    PathBuf::from(custom_home)
                } else {
                    home.join(".jeikcode")
                };
                vec![CliConfigFile {
                    name: "config.toml".to_string(),
                    path: jeikcode_dir.join("config.toml"),
                }]
            }
            CliApp::GrokBuild => {
                let grok_dir = if let Ok(custom) =
                    std::env::var("GROK_HOME").or_else(|_| std::env::var("GROKBUILD_HOME"))
                {
                    PathBuf::from(custom)
                } else if home.join(".grok").exists() || !home.join(".grokbuild").exists() {
                    home.join(".grok")
                } else {
                    home.join(".grokbuild")
                };
                vec![CliConfigFile {
                    name: "config.toml".to_string(),
                    path: grok_dir.join("config.toml"),
                }]
            }
        }
    }

    pub fn default_url(&self) -> &'static str {
        match self {
            CliApp::Claude => "https://api.anthropic.com",
            CliApp::Codex => "https://api.openai.com/v1",
            CliApp::Gemini => "https://generativelanguage.googleapis.com",
            CliApp::OpenCode => "https://api.openai.com/v1",
            CliApp::JeikCode => "http://127.0.0.1:8046/v1",
            CliApp::GrokBuild => "https://api.x.ai/v1",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub is_synced: bool,
    pub has_backup: bool,
    pub current_base_url: Option<String>,
    pub files: Vec<String>, // Returns a list of associated file names for front-end display
}

/// Detect whether the CLI is installed and get the version
pub fn check_cli_installed(app: &CliApp) -> (bool, Option<String>) {
    let cmd = app.as_str();
    // The command name is used by default. If fallback finds the path, it will be updated to the absolute path.
    let mut executable_path = PathBuf::from(cmd);

    // 1. Prioritize using which/where detection (follow PATH)
    let which_output = if cfg!(target_os = "windows") {
        let mut c = Command::new("where");
        c.arg(cmd);
        #[cfg(target_os = "windows")]
        c.creation_flags(CREATE_NO_WINDOW);
        c.output()
    } else {
        Command::new("which").arg(cmd).output()
    };

    let mut installed = match &which_output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    };

    #[cfg(target_os = "windows")]
    if installed {
        if let Ok(out) = &which_output {
            if let Some(found_path) = parse_where_output(&out.stdout) {
                executable_path = found_path;
            }
        }
    }

    #[cfg(target_os = "windows")]
    if !installed {
        if let Some(found_path) = scan_windows_cli_paths(cmd) {
            installed = true;
            executable_path = found_path;
        }
    }

    // [FIX #765] macOS enhanced detection: explicit search for common binary paths if which fails
    // Solve the problem that the Tauri process PATH may be incomplete, resulting in the installed CLI not being detected
    if !installed && !cfg!(target_os = "windows") {
        let home = dirs::home_dir().unwrap_or_default();
        let mut common_paths = vec![
            home.join(".local/bin"),
            home.join(".bun/bin"),
            home.join(".bun/install/global/node_modules/.bin"),
            home.join(".npm-global/bin"),
            home.join(".volta/bin"),
            home.join("bin"),
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/bin"),
        ];

        // Enhancement: Scan all node versions in the nvm directory
        let nvm_base = home.join(".nvm/versions/node");
        if nvm_base.exists() {
            if let Ok(entries) = std::fs::read_dir(&nvm_base) {
                for entry in entries.flatten() {
                    let bin_path = entry.path().join("bin");
                    if bin_path.exists() {
                        common_paths.push(bin_path);
                    }
                }
            }
        }

        for path in common_paths {
            let full_path = path.join(cmd);
            if full_path.exists() {
                tracing::debug!(
                    "[CLI-Sync] Detected {} via explicit path: {:?}",
                    cmd,
                    full_path
                );
                installed = true;
                executable_path = full_path;
                break;
            }
        }
    }

    // If it's JeikCode and regular detection misses, try to detect if the atomcode alias or configuration file exists
    if !installed && app == &CliApp::JeikCode {
        if let Some(p) = detect_fallback_binary("atomcode") {
            executable_path = p;
            installed = true;
        } else if let Some(home) = dirs::home_dir() {
            let jeikcode_dir = if let Ok(custom_home) = std::env::var("JEIKCODE_HOME") {
                PathBuf::from(custom_home)
            } else {
                home.join(".jeikcode")
            };
            if jeikcode_dir.join("config.toml").exists() || jeikcode_dir.exists() {
                installed = true;
            }
        }
    }

    // If it's GrokBuild and regular detection misses, try to detect whether the grokbuild alias or configuration file exists
    if !installed && app == &CliApp::GrokBuild {
        if let Some(p) = detect_fallback_binary("grokbuild") {
            executable_path = p;
            installed = true;
        } else if let Some(home) = dirs::home_dir() {
            let grok_dir = home.join(".grok");
            let grokbuild_dir = home.join(".grokbuild");
            if grok_dir.join("config.toml").exists()
                || grokbuild_dir.join("config.toml").exists()
                || grok_dir.exists()
                || grokbuild_dir.exists()
            {
                installed = true;
            }
        }
    }

    if !installed {
        return (false, None);
    }

    // 2. Get the version (Windows uses special processing .cmd/.bat)
    #[cfg(target_os = "windows")]
    let version = run_version_command(&executable_path);

    #[cfg(not(target_os = "windows"))]
    let version = {
        let mut ver_cmd = Command::new(&executable_path);
        ver_cmd.arg("--version");
        let version_output = ver_cmd.output();
        match version_output {
            Ok(out) if out.status.success() => {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let cleaned = s
                    .split(|c: char| !c.is_numeric() && c != '.')
                    .filter(|part| !part.is_empty())
                    .last()
                    .map(|p| p.trim())
                    .unwrap_or(&s)
                    .to_string();
                Some(cleaned)
            }
            _ => None,
        }
    };

    (true, version)
}
