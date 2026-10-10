// Patch normalization (split from apply_patch_preflight.rs).
use super::cwd::recall_cwd_candidates;
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
use crate::proxy::adapters::apply_patch_preflight::Repair;

/// 一条 pre-flight 处理记录(给诊断页 / 日志)。

pub(crate) fn diagnose_absolute_paths(v4a: &str, primary: Option<&str>) -> Vec<Repair> {
    let mut known_cwds: Vec<String> = Vec::new();
    if let Some(cwd) = primary {
        if !cwd.is_empty() {
            known_cwds.push(cwd.to_string());
        }
    }
    known_cwds.extend(recall_cwd_candidates());
    known_cwds.sort();
    known_cwds.dedup();

    if known_cwds.is_empty() {
        return Vec::new();
    }

    let cwd_paths: Vec<PathBuf> = known_cwds.iter().map(PathBuf::from).collect();
    let mut repairs = Vec::new();
    for line in v4a.lines() {
        let Some(file) = line
            .strip_prefix("*** Update File: ")
            .or_else(|| line.strip_prefix("*** Add File: "))
            .or_else(|| line.strip_prefix("*** Delete File: "))
        else {
            continue;
        };
        let file = file.trim();
        let path = Path::new(file);
        if !path.is_absolute() {
            continue;
        }
        let inside_known_cwd = cwd_paths.iter().any(|cwd| path.starts_with(cwd));
        if !inside_known_cwd {
            repairs.push(Repair {
                file: file.to_string(),
                kind: "diagnostic:absolute_path_outside_known_cwd".to_string(),
                detail: format!(
                    "absolute patch target is outside known cwd candidates: {}",
                    known_cwds.join(" | ")
                ),
            });
        }
    }
    repairs
}

/// **规则:双边 `@@ … @@` → 单边 `@@ …`**(prompt gotcha #1 / chat-path #1)。V4A 的 `@@` 是
/// **单边** anchor(`@@ <header>`);模型常写成双边 `@@ <header> @@`,Codex 把尾部 `@@` 当字面文本
/// → `Failed to find context '... @@'`。仅处理**列 0 的 `@@` 头行**(正文行有 `+`/`-`/空格 前缀,不碰),
/// 去掉尾部 `@@` 及其前导空白;**裸 `@@`(section 分隔)不动**。
pub(crate) fn strip_trailing_at(v4a: &str) -> (String, Vec<Repair>) {
    let mut changed = 0usize;
    let out: Vec<String> = v4a
        .lines()
        .map(|l| {
            if l.starts_with("@@") {
                let t = l.trim_end();
                // 裸 `@@`(len==2)是合法 section 分隔,跳过;`@@ x @@` 才去尾。
                if t.len() > 2 && t.ends_with("@@") {
                    let body = t[..t.len() - 2].trim_end();
                    if !body.is_empty() && body != "@@" {
                        changed += 1;
                        return body.to_owned();
                    }
                }
            }
            l.to_owned()
        })
        .collect();
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    let repairs = if changed > 0 {
        vec![Repair {
            file: "(@@ header)".to_owned(),
            kind: "repaired".to_owned(),
            detail: format!("双边 @@ → 单边: {changed} 行(prompt gotcha #1)"),
        }]
    } else {
        Vec::new()
    };
    (joined, repairs)
}

fn normalized_diff_path(path: &str) -> Option<String> {
    let mut p = path.trim();
    if p == "/dev/null" || p.is_empty() {
        return None;
    }
    if (p.starts_with("a/") || p.starts_with("b/")) && p.len() > 2 {
        p = &p[2..];
    }
    Some(p.to_string())
}

/// Gemini 在非原生 OpenAI apply_patch tool 上常把补丁写成 unified diff:
/// `--- path` / `+++ path` / `@@ -a,+b`。Codex V4A 需要文件操作头
/// `*** Update File: path`,所以这里只做无语义损失的头部转换。
pub(crate) fn convert_unified_file_headers(v4a: &str) -> (String, Vec<Repair>) {
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut repairs = Vec::new();
    let mut i = 0usize;
    let mut converted = 0usize;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if let Some(old_path) = line.strip_prefix("--- ") {
            if let Some(next) = lines.get(i + 1) {
                if let Some(new_path) = next.strip_prefix("+++ ") {
                    if let Some(path) =
                        normalized_diff_path(new_path).or_else(|| normalized_diff_path(old_path))
                    {
                        out.push(format!("*** Update File: {path}"));
                        repairs.push(Repair {
                            file: path,
                            kind: "repaired".to_string(),
                            detail: "unified diff file headers -> V4A Update File".to_string(),
                        });
                        converted += 1;
                        i += 2;
                        continue;
                    }
                }
            }
        }

        if let Some(path) = trimmed
            .strip_prefix("file: ")
            .or_else(|| trimmed.strip_prefix("File: "))
            .and_then(normalized_diff_path)
        {
            out.push(format!("*** Update File: {path}"));
            repairs.push(Repair {
                file: path,
                kind: "repaired".to_string(),
                detail: "file: header -> V4A Update File".to_string(),
            });
            converted += 1;
            i += 1;
            continue;
        }

        out.push(line.to_string());
        i += 1;
    }

    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    if converted == 0 {
        (v4a.to_owned(), Vec::new())
    } else {
        (joined, repairs)
    }
}

fn is_unified_range_token(token: &str, sign: char) -> bool {
    let Some(rest) = token.strip_prefix(sign) else {
        return false;
    };
    let mut pieces = rest.split(',');
    let Some(start) = pieces.next() else {
        return false;
    };
    if start.is_empty() || !start.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if let Some(count) = pieces.next() {
        if count.is_empty() || !count.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }
    pieces.next().is_none()
}

pub(crate) fn is_unified_hunk_range_header(line: &str) -> bool {
    if !line.starts_with("@@") {
        return false;
    }
    let mut body = line[2..].trim();
    if let Some(stripped) = body.strip_suffix("@@") {
        body = stripped.trim_end();
    }
    let mut parts = body.split_whitespace();
    let Some(old_range) = parts.next() else {
        return false;
    };
    let Some(new_range) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && is_unified_range_token(old_range, '-')
        && is_unified_range_token(new_range, '+')
}

/// unified diff 的 `@@ -1,2 +1,3` 行号头不是 V4A anchor。Codex 会把 `@@ <text>`
/// 当成要查找的文本上下文,所以这里将纯行号范围规整为裸 `@@`。
pub(crate) fn strip_unified_hunk_ranges(v4a: &str) -> (String, Vec<Repair>) {
    let mut changed = 0usize;
    let out: Vec<String> = v4a
        .lines()
        .map(|line| {
            if is_unified_hunk_range_header(line) {
                changed += 1;
                "@@".to_string()
            } else {
                line.to_string()
            }
        })
        .collect();
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    let repairs = if changed > 0 {
        vec![Repair {
            file: "(@@ range header)".to_owned(),
            kind: "repaired".to_owned(),
            detail: format!("unified @@ 行号范围 -> V4A 裸 @@: {changed} 行"),
        }]
    } else {
        Vec::new()
    };
    (joined, repairs)
}
