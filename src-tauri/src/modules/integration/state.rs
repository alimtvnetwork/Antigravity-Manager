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

/// 根据目标参数、进程运行态及可执行文件存在性决策最终切换环境
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

    // target_ide 为 None 或未指定时进行智能环境探查（经典版桌面端优先，严禁仅凭静态 IDE 数据库文件劫持经典版目标）
    let mut is_ide = false;
    if classic_running {
        // 原生经典版正在运行，确定目标为经典版
        is_ide = false;
    } else if ide_running {
        // 经典版未运行，但 IDE 正在运行，推导为 IDE
        is_ide = true;
    } else if has_classic_exe {
        // 原生经典版可执行文件存在，优先保持经典版
        is_ide = false;
    } else if let Some(exe_str) = ide_exe_path {
        // 原生经典版不存在，检查是否存在 IDE 可执行文件
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
