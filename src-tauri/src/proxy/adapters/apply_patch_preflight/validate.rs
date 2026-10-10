// v4a validation (split from apply_patch_preflight.rs).
use super::cwd::{anchor_probe, has_cwd_candidate};
use super::normalize::is_unified_hunk_range_header;
use super::read::read_patch_file;
// apply_patch **pre-flight 自动修复**:在把 V4A patch 发给 Codex apply 之前,读目标文件比对,
// 自动对齐**安全**的上下文失配(尾随空格 / 首尾空白差异),消灭 V4A 头号失败
// `apply_patch verification failed: Failed to find expected lines`。
//
// ## 为什么需要
// 弱一点的 chat 模型(非 OpenAI)在大文件上常无法逐字节复刻 `Update File` 的 context/删除行
// (尾随空格、缩进、记忆偏差)→ Codex 找不到锚点 → apply 失败 → 模型整文件重写,浪费时间和 token。
// 实测真机报错(rollout 地面真相)正是这类。
//
// ## 安全边界(绝不损坏文件 —— 对齐用户「不做破坏性降级」硬规则)
// - **只动锚点**:`Update File` 里的 context(空格前缀)/ 删除(`-`)行。`+新增` 行**绝不改动**。
// - **只在唯一匹配时修**:锚点块在文件里按「忽略尾随空格 / 首尾空白」找候选,**恰好一个**位置才对齐;
//   0 个(模型真改错内容)或 ≥2 个(歧义)一律**原样放行**,交给 Codex parse_patch 暴露真坏,绝不靠猜。
// - **Add File / Delete File 不碰**(无锚点,不涉及匹配)。读不到文件 / 无 cwd → 原样放行。
// - 每条修复 / 放行都记进 apply-patch 诊断页,可审计。

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::proxy::adapters::apply_patch_preflight::Repair;
use serde_json::Value;

/// 一条 pre-flight 处理记录(给诊断页 / 日志)。

/// 发送给 Codex 自定义 apply_patch 前的后验校验。发现明确非法的 V4A 时,调用方应把该工具项
/// 标成 incomplete,避免 Codex 执行后再把失败历史喂回模型形成循环。
pub fn validate_v4a_for_codex(v4a: &str) -> Option<(usize, String)> {
    let meaningful: Vec<(usize, &str)> = v4a
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .collect();
    let Some((first_line_no, first)) = meaningful.first() else {
        return Some((1, "empty apply_patch input".to_string()));
    };
    if first.trim() != "*** Begin Patch" {
        return Some((
            first_line_no + 1,
            "apply_patch input must start with *** Begin Patch".to_string(),
        ));
    }
    let Some((last_line_no, last)) = meaningful.last() else {
        return Some((1, "empty apply_patch input".to_string()));
    };
    if last.trim() != "*** End Patch" {
        return Some((
            last_line_no + 1,
            "apply_patch input must end with *** End Patch".to_string(),
        ));
    }

    let mut has_hunk = false;
    for (idx, line) in v4a.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "*** Begin Patch" && idx != *first_line_no {
            return Some((
                idx + 1,
                "apply_patch input contains a nested or repeated *** Begin Patch".to_string(),
            ));
        }
        if trimmed == "*** End Patch" && idx != *last_line_no {
            return Some((
                idx + 1,
                "apply_patch input contains an early or repeated *** End Patch".to_string(),
            ));
        }
        if trimmed.starts_with("*** Add File:")
            || trimmed.starts_with("*** Update File:")
            || trimmed.starts_with("*** Delete File:")
        {
            has_hunk = true;
        }
        if line.starts_with("--- ") || line.starts_with("+++ ") {
            return Some((
                idx + 1,
                "V4A apply_patch does not accept unified diff file header lines (---/+++)"
                    .to_string(),
            ));
        }
        if is_unified_hunk_range_header(line) {
            return Some((
                idx + 1,
                "V4A apply_patch uses bare @@ or @@ text anchors, not unified @@ -a,+b ranges"
                    .to_string(),
            ));
        }
        if idx != *first_line_no && idx != *last_line_no && !trimmed.is_empty() {
            match line.chars().next() {
                Some('+') | Some('-') | Some(' ') => {}
                _ if trimmed.starts_with("@@") => {}
                _ if trimmed.starts_with("*** Add File:")
                    || trimmed.starts_with("*** Update File:")
                    || trimmed.starts_with("*** Delete File:")
                    || trimmed.starts_with("*** Move to:")
                    || trimmed == "*** End of File" => {}
                _ if trimmed.starts_with("***") => {
                    return Some((
                        idx + 1,
                        format!(
                            "unrecognized V4A operation: {}",
                            trimmed.chars().take(80).collect::<String>()
                        ),
                    ));
                }
                _ => {
                    return Some((
                        idx + 1,
                        "line missing V4A prefix (expected '+', '-', ' ', '@@', or '*** ' marker)"
                            .to_string(),
                    ));
                }
            }
        }
    }
    if !has_hunk {
        return Some((
            1,
            "apply_patch input has no file operation hunk".to_string(),
        ));
    }
    None
}

/// **规则 G:Add File 内容行漏 `+` 前缀 → 补全**(grammar `add_hunk: … add_line+`、
/// `add_line: "+" /(.*)/`)。Add File 语义 = 后续每行都是新文件的**字面内容**、必须 `+` 前缀;
/// 模型偶尔漏写 `+` → Codex 不认作内容。Add File section 内**无歧义**(全是新增),给非 `+` 行
/// 统一补 `+`(空行 → 裸 `+`);已是 `+` 的不动(不重复成 `++`)。纯字符串、不读盘。
pub(crate) fn ensure_add_file_plus(v4a: &str) -> (String, Vec<Repair>) {
    if !v4a.contains("*** Add File:") {
        return (v4a.to_owned(), Vec::new());
    }
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut repairs = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if let Some(path) = lines[i].strip_prefix("*** Add File: ") {
            out.push(lines[i].to_owned()); // header
            i += 1;
            let mut fixed = 0usize;
            // body 到下一个 `*** ` 控制行 / EOF;Add File body 全是 `+` 内容行。
            while i < lines.len() && !lines[i].starts_with("*** ") {
                if lines[i].starts_with('+') {
                    out.push(lines[i].to_owned());
                } else {
                    out.push(format!("+{}", lines[i]));
                    fixed += 1;
                }
                i += 1;
            }
            if fixed > 0 {
                repairs.push(Repair {
                    file: path.trim().to_owned(),
                    kind: "repaired".to_owned(),
                    detail: format!("Add File {fixed} 行漏 `+` 前缀 → 补全(lark add_line)"),
                });
            }
        } else {
            out.push(lines[i].to_owned());
            i += 1;
        }
    }
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    (joined, repairs)
}

/// **规则:`@@ <header>` 锚点对齐文件真实行**(真机 seq181:`Failed to find context 'X'`)。
/// V4A 的 `@@ <header>` 是单边锚点,Codex 按**精确整行**匹配文件里的 section 行;模型常写**残缺**
/// 头(如 `@@ 系统架构建议`,而文件真实行是 `## 6. 系统架构建议`)→ 找不到锚点。当 `<header>` 不是
/// 文件里任何**整行**、但**恰好唯一包含于**某一文件行时,把 `@@ <header>` 对齐成 `@@ <该文件整行>`;
/// 0 个 / 多个包含 → 歧义,原样放行(不猜)。裸 `@@`(无 header)不动。需 `cwd`。
pub(crate) fn align_at_headers(v4a: &str, cwd: Option<&str>) -> (String, Vec<Repair>) {
    if !has_cwd_candidate(cwd) {
        return (v4a.to_owned(), Vec::new());
    }
    if !v4a.contains("*** Update File:") {
        return (v4a.to_owned(), Vec::new());
    }
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut repairs = Vec::new();
    let mut file_lines: Vec<String> = Vec::new();
    let mut have_file = false;
    let mut fixed = 0usize;
    let mut i = 0;
    while i < lines.len() {
        if let Some(path) = lines[i].strip_prefix("*** Update File: ") {
            // 切到新 Update File section → 按候选 cwd + 锚点 probe 解析目标文件(MOC-263 P1/P2)
            let mut se = i + 1;
            while se < lines.len() && !lines[se].starts_with("*** ") {
                se += 1;
            }
            let probe = anchor_probe(&lines[i + 1..se]);
            file_lines = read_patch_file(path.trim(), cwd, &probe)
                .map(|(_, c)| c.lines().map(str::to_owned).collect())
                .unwrap_or_default();
            have_file = !file_lines.is_empty();
            out.push(lines[i].to_owned());
            i += 1;
            continue;
        }
        // `@@ <header>` 锚点(非裸 `@@`),且文件已载入
        if have_file {
            if let Some(header) = lines[i].strip_prefix("@@ ") {
                let h = header.trim();
                if !h.is_empty() && !file_lines.iter().any(|fl| fl == h) {
                    let hits: Vec<&String> =
                        file_lines.iter().filter(|fl| fl.contains(h)).collect();
                    if hits.len() == 1 {
                        out.push(format!("@@ {}", hits[0]));
                        fixed += 1;
                        i += 1;
                        continue;
                    }
                }
            }
        }
        out.push(lines[i].to_owned());
        i += 1;
    }
    if fixed > 0 {
        repairs.push(Repair {
            file: "(@@ anchor)".to_owned(),
            kind: "repaired".to_owned(),
            detail: format!("@@ 锚点残缺 → 对齐文件真实整行: {fixed} 处(Failed to find context)"),
        });
    }
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    (joined, repairs)
}
