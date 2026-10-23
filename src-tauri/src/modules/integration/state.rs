use super::*;

pub(crate) static PROMPT_REINJECTED: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

pub fn note_prompt_reinjected(value: bool) {
    if let Ok(mut slot) = PROMPT_REINJECTED.lock() {
        *slot = value;
    }
}

pub fn take_prompt_reinjected() -> bool {
    PROMPT_REINJECTED
        .lock()
        .map(|mut slot| {
            let value = *slot;
            *slot = false;
            value
        })
        .unwrap_or(false)
}

/// The final decision is made to switch the environment based on the target parameters, process running status and executable file existence.
pub fn resolve_effective_target(
    target_ide: Option<&str>,
    classic_running: bool,
    ide_running: bool,
    has_classic_exe: bool,
    ide_exe_path: Option<&str>,
) -> (bool, Option<&'static str>) {
    let is_explicit_ide = target_ide == Some("ide");
    let is_explicit_classic = target_ide == Some("classic");

    if is_explicit_ide {
        return (true, Some("ide"));
    }
    if is_explicit_classic {
        return (false, Some("classic"));
    }

    // Intelligent environment detection is performed when target_ide is None or not specified (the classic version desktop version is preferred, and it is strictly prohibited to hijack the classic version target with only static IDE database files)
    let mut is_ide = false;
    if classic_running {
        // The native classic version is running, and the target is determined to be the classic version.
        is_ide = false;
    } else if ide_running {
        // Classic version is not running, but the IDE is running, inferred as IDE
        is_ide = true;
    } else if has_classic_exe {
        // The native classic version executable file exists, priority is given to keeping the classic version
        is_ide = false;
    } else if let Some(exe_str) = ide_exe_path {
        // The native classic version does not exist, check whether the IDE executable file exists
        let path_lower = exe_str.to_lowercase();
        if path_lower.contains("antigravity ide") || path_lower.contains("antigravity-ide") {
            is_ide = true;
        }
    }

    let effective = if is_ide {
        Some("ide")
    } else if is_explicit_classic {
        Some("classic")
    } else {
        None
    };

    (is_ide, effective)
}
