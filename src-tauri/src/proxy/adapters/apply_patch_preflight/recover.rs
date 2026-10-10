// Patch recovery (split from apply_patch_preflight.rs).
use super::repair::resolve_path;
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

use serde_json::Value;

/// 一条 pre-flight 处理记录(给诊断页 / 日志)。

/// **规则:`Update File` 目标是空文件 → `Delete File + Add File`**(prompt gotcha #3,无损)。
/// `*** Update File:` 无法作用于空文件(Codex 报 `cannot operate on a completely empty file`)。
/// 当目标文件存在且**为空**(真正 0 字节,非纯空白)、且 Update body 是**纯 `+` 行**(纯写内容,无 `-`/context 可
/// 匹配)时,转成 `*** Delete File: X` + `*** Add File: X` + 原 `+` body(空文件无内容可丢 → 无损)。
/// body 含 `-`/context(模型在空文件上写了匹配行,本就矛盾)/ 含 Move(交给 empty-move 规则)→ 不动。需 `cwd`。
pub(crate) fn recover_update_empty_file(v4a: &str, cwd: Option<&str>) -> (String, Vec<Repair>) {
    let Some(cwd) = cwd else {
        return (v4a.to_owned(), Vec::new());
    };
    if !v4a.contains("*** Update File:") {
        return (v4a.to_owned(), Vec::new());
    }
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut repairs = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if let Some(path) = lines[i].strip_prefix("*** Update File: ") {
            let p = path.trim();
            // 只认**真正 0 字节**(Codex 仅对 `completely empty file` 报错;纯空白文件仍是可读内容、
            // 能正常 Update)。用 `c.trim().is_empty()` 会把纯空白文件也转 Delete+Add → 丢掉那些
            // 空白字节(破坏性,codex-connector #435 P2)。
            let is_empty = std::fs::read_to_string(resolve_path(p, cwd))
                .map(|c| c.is_empty())
                .unwrap_or(false);
            if is_empty {
                let body_start = i + 1;
                let mut j = body_start;
                while j < lines.len() && !lines[j].starts_with("*** ") {
                    j += 1;
                }
                let body = &lines[body_start..j];
                let has_move = body
                    .first()
                    .map(|l| l.starts_with("*** Move to:"))
                    .unwrap_or(false);
                let content: Vec<&&str> = body
                    .iter()
                    .filter(|l| !l.trim().is_empty() && !l.starts_with("@@"))
                    .collect();
                let all_plus = !content.is_empty() && content.iter().all(|l| l.starts_with('+'));
                if !has_move && all_plus {
                    out.push(format!("*** Delete File: {p}"));
                    out.push(format!("*** Add File: {p}"));
                    for b in body {
                        if b.starts_with('+') {
                            out.push((*b).to_owned());
                        }
                    }
                    repairs.push(Repair {
                        file: p.to_owned(),
                        kind: "repaired".to_owned(),
                        detail: "Update 空文件 → Delete+Add 写入(prompt gotcha #3)".to_owned(),
                    });
                    i = j;
                    continue;
                }
            }
        }
        out.push(lines[i].to_owned());
        i += 1;
    }
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    (joined, repairs)
}

/// **规则:空 `Update File + Move to`(rename-only)→ `Delete File + Add File`**(prompt gotcha #7)。
/// 模型想纯重命名却写 `*** Update File: X` + `*** Move to: Y` 且**无 hunk** → Codex 报
/// `Update file hunk for path 'X' is empty`。按 prompt **自身建议**恢复:读 X 原内容,转成
/// `*** Delete File: X` + `*** Add File: Y` + 逐行 `+` 复制(空行为裸 `+`)。读不到 X → 原样放行。
pub(crate) fn recover_empty_move(v4a: &str, cwd: Option<&str>) -> (String, Vec<Repair>) {
    let Some(cwd) = cwd else {
        return (v4a.to_owned(), Vec::new());
    };
    if !v4a.contains("*** Move to:") {
        return (v4a.to_owned(), Vec::new());
    }
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut repairs = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        // 匹配 `*** Update File: X` 紧跟 `*** Move to: Y`,且 Move 后到下一个 `*** ` 控制行之间无 hunk 行。
        if let Some(old) = lines[i].strip_prefix("*** Update File: ") {
            if i + 1 < lines.len() {
                if let Some(new) = lines[i + 1].strip_prefix("*** Move to: ") {
                    // 看 Move 之后、下一个**文件操作**控制行之前有没有 hunk 内容行。
                    // 注:`*** End of File` 是文档化的 **hunk 内标记**(prompt RENAME/MOVE 段),不是
                    // section 边界 —— 不能停在它(否则 rename+EOF 追加会被误判成空 rename、转成丢内容的
                    // Delete+Add,codex-connector #435 P1)。它本身即表示「有 hunk」,继续往后扫。
                    let mut j = i + 2;
                    let mut has_hunk = false;
                    while j < lines.len() {
                        let t = lines[j];
                        if t.trim_end() == "*** End of File" {
                            has_hunk = true;
                            j += 1;
                            continue;
                        }
                        if t.starts_with("*** ") {
                            break; // 真正的下一个文件操作 / End Patch 边界
                        }
                        if t.starts_with('+')
                            || t.starts_with('-')
                            || t.starts_with(' ')
                            || t.starts_with("@@")
                        {
                            has_hunk = true;
                        }
                        j += 1;
                    }
                    if !has_hunk {
                        // 空 rename-only → 读原文件转 Delete+Add。读不到 / 内容为空 → 不转(空 Add File
                        // 体可能被 Codex 拒)→ 原样放行交 Codex 处理。
                        let abs = resolve_path(old.trim(), cwd);
                        match std::fs::read_to_string(&abs) {
                            Ok(content) if !content.is_empty() => {
                                out.push(format!("*** Delete File: {}", old.trim()));
                                out.push(format!("*** Add File: {}", new.trim()));
                                for cl in content.lines() {
                                    out.push(format!("+{cl}"));
                                }
                                repairs.push(Repair {
                                    file: old.trim().to_owned(),
                                    kind: "repaired".to_owned(),
                                    detail: format!(
                                        "空 Update+Move(rename-only)→ Delete+Add 复制原内容 → {}(prompt gotcha #7)",
                                        new.trim()
                                    ),
                                });
                                i = j; // 跳过原 Update/Move(+空体)
                                continue;
                            }
                            _ => {
                                repairs.push(Repair {
                                    file: old.trim().to_owned(),
                                    kind: "skipped:unreadable_or_empty".to_owned(),
                                    detail: "空 Update+Move 但原文件读不到 / 为空 → 原样放行"
                                        .to_owned(),
                                });
                            }
                        }
                    }
                }
            }
        }
        out.push(lines[i].to_owned());
        i += 1;
    }
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    (joined, repairs)
}

/// 末个 patch 操作是否为「`*** Add File:` + 代码/结构化配置文件目标」。用于 [`ensure_v4a_envelope`] 判定
/// 末行 `+*** End Patch` 可否安全剥成终止符:**仅 Add File**(新建文件,裸 `*** End Patch` 不可能是合法
/// 源码的**末行** → 必是误前缀终止符)才剥;`*** Update File:` 的 `+*** End Patch` 是**新增行**(可能往
/// 字符串 / fixture 里加这串字),剥了=丢新增 → 不剥(chatgpt-codex-connector review:限定 Add File)。
/// 文档 / 文本 / 未知扩展也不剥(可能是正文,留 incomplete 不猜)。allowlist 保守。MOC-268。
pub(crate) fn last_op_is_add_file_code(body: &str) -> bool {
    let last_op = body.lines().rev().find(|l| {
        let t = l.trim_end();
        t.starts_with("*** Add File: ")
            || t.starts_with("*** Update File: ")
            || t.starts_with("*** Delete File: ")
    });
    let Some(path) = last_op.and_then(|l| l.trim_end().strip_prefix("*** Add File: ")) else {
        return false; // 无操作,或末操作是 Update/Delete(非 Add File)→ 不剥
    };
    let ext = std::path::Path::new(path.trim())
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase());
    matches!(
        ext.as_deref(),
        Some(
            "rs" | "ts"
                | "tsx"
                | "js"
                | "jsx"
                | "mjs"
                | "cjs"
                | "py"
                | "go"
                | "java"
                | "kt"
                | "kts"
                | "c"
                | "h"
                | "cc"
                | "cpp"
                | "cxx"
                | "hpp"
                | "hh"
                | "cs"
                | "rb"
                | "php"
                | "swift"
                | "scala"
                | "lua"
                | "sql"
                | "sh"
                | "bash"
                | "zsh"
                | "css"
                | "scss"
                | "sass"
                | "less"
                | "html"
                | "htm"
                | "xml"
                | "vue"
                | "svelte"
                | "json"
                | "toml"
                | "yaml"
                | "yml"
                | "gradle"
                | "cmake"
                | "proto"
                | "graphql"
                | "dart"
                | "r"
        )
    )
}
