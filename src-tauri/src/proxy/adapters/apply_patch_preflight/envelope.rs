// v4a envelope (split from apply_patch_preflight.rs).
use super::cwd::{anchor_probe, has_cwd_candidate};
use super::read::read_patch_file;
use super::recover::last_op_is_add_file_code;
use super::repair::repair_update_section;
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

/// **缺信封自动补全**:模型常只写 `*** Add/Update File:` + 内容,漏掉 `*** Begin Patch` /
/// `*** End Patch` 头尾 → Codex(及本 adapter 的 V4A 校验)判 incomplete → 模型被迫重试。
/// 当 patch 含至少一个 `*** Add/Update/Delete File:` 操作、JSON 已完整(调用方先 gate
/// `detect_json_truncation` 为 None 才调本函数,确保不是流式截断)、但缺 Begin/End 信封时,
/// **纯补标记**(不改一字节内容、不猜),返回 `(补全后, Some(Repair))`;本就完整 / 非 patch 体
/// 返回 `(原样, None)`。
///
/// 安全:缺 Begin 时**仅当首个非空行就是操作行**才在最前补 `*** Begin Patch`(有前导散文则不动,
/// 交给 `repair_v4a_envelope` / Codex);缺 End 时去尾随空白后补 `*** End Patch`。
pub fn ensure_v4a_envelope(input: &str) -> (String, Option<Repair>) {
    let is_op = |l: &str| {
        let t = l.trim_end();
        t.starts_with("*** Add File:")
            || t.starts_with("*** Update File:")
            || t.starts_with("*** Delete File:")
    };
    if !input.lines().any(is_op) {
        return (input.to_owned(), None); // 不是可识别的 patch 体,不碰
    }
    let has_begin = input.lines().any(|l| l.trim_end() == "*** Begin Patch");
    let has_end = input.lines().any(|l| l.trim_end() == "*** End Patch");
    if has_begin && has_end {
        return (input.to_owned(), None);
    }
    let mut body = input.to_owned();
    let mut added: Vec<&str> = Vec::new();
    if !has_begin {
        // 仅当首个非空行就是操作行才安全(无前导散文混入信封内)。
        let first_nonempty = input.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        if !is_op(first_nonempty) {
            return (input.to_owned(), None);
        }
        body = format!("*** Begin Patch\n{body}");
        added.push("Begin Patch");
    }
    if !has_end {
        let trimmed = body.trim_end();
        let last = trimmed.lines().last().unwrap_or("");
        // [MOC-268] **只有 `+*** End Patch`(Add 行前缀)** 才是「模型给终止符误加前缀」的形态。
        // ` *** End Patch`(context)/ `-*** End Patch`(deletion)是**合法 Update hunk 行**——例如模型
        // 用 Update **删除**文件里之前残留的 `*** End Patch`(`-*** End Patch`),或用它当 context 锚点;
        // 把它们当终止符剥会**静默丢弃删除 / 破坏锚点**(chatgpt-codex-connector review)→ 故 ` `/`-` 一律
        // 走下面正常 append(补真终止符,hunk 行原样保留)。
        // 对 `+*** End Patch` 再**按文件类型消歧**(用户拍板):
        //   · 代码 / 结构化配置文件(裸 `*** End Patch` 不可能是合法源码末行)→ 必是误前缀终止符 → **剥前缀**
        //     (`head` 切到末行起点、保留其前换行;末行 ASCII,边界安全)。
        //   · 文档 / 文本 / 未知(可能是正文末行)→ **不猜**:不剥(免删正文)、不追加(免残留),留 incomplete
        //     交下游判截断、模型按 guidance 规则2 重发。prompt 才是根治,中间层只在确定安全时介入。
        if last == "+*** End Patch" {
            if last_op_is_add_file_code(&body) {
                let head = &trimmed[..trimmed.len() - last.len()];
                body = format!("{head}*** End Patch");
                added.push("End Patch(代码文件·剥误加前缀终止符)");
            } else {
                return (
                    body,
                    Some(Repair {
                        file: "(envelope)".to_owned(),
                        kind: "skipped:ambiguous_prefixed_end".to_owned(),
                        detail:
                            "末行 +*** End Patch 且目标非代码文件(可能是正文)→ 不猜不补全,留 incomplete"
                                .to_owned(),
                    }),
                );
            }
        } else {
            // 含 ` *** End Patch` / `-*** End Patch`(合法 hunk 行)及普通内容末行 → 正常补真终止符。
            body = format!("{trimmed}\n*** End Patch");
            added.push("End Patch");
        }
    }
    (
        body,
        Some(Repair {
            file: "(envelope)".to_owned(),
            kind: "repaired".to_owned(),
            detail: format!("模型漏写信封,自动补全: {}", added.join(" + ")),
        }),
    )
}

/// **规则:Update body 内**无前缀行**按文件判定补全**(真机 seq235:单行漏前缀 → validate 拒 →
/// 整份 Update 重写浪费)。grammar `change_line: ("+"|"-"|" ") /(.*)/` 要求每行带前缀;模型偶尔
/// 漏写一行的前缀。**非破坏性**修(只补前缀 / 删可证重复的废行,绝不丢内容):
/// - 无前缀行**与相邻 `+<同内容>` 行重复**(模型写了两遍)→ 删该废行(内容在 `+` 行里,不丢);
/// - 否则无前缀**非空**行**在目标文件里有完全相同的整行** → 它是 context 行漏了空格 → 补 ` `
///   (合法 context 且 byte-exact;最不破坏的解释:行保留。模型若本意是删,顶多没删成、无数据损失);
/// - 其余(不在文件、非重复、空行)→ 原样透过,交 validate 报错让模型自纠(不猜)。
///
/// 仅作用于 `*** Update File:` section(Add File 的漏 `+` 由 [`ensure_add_file_plus`] 管)。需 `cwd`。
pub(crate) fn fix_unprefixed_lines(v4a: &str, cwd: Option<&str>) -> (String, Vec<Repair>) {
    if !has_cwd_candidate(cwd) {
        return (v4a.to_owned(), Vec::new());
    }
    if !v4a.contains("*** Update File:") {
        return (v4a.to_owned(), Vec::new());
    }
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut repairs = Vec::new();
    let mut in_update = false;
    let mut file_lines: Vec<String> = Vec::new();
    let mut drop_dups = 0usize;
    let mut add_ctx = 0usize;
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        if let Some(path) = l.strip_prefix("*** Update File: ") {
            in_update = true;
            // 按候选 cwd + 锚点 probe 解析目标文件(MOC-263 P1/P2)。
            let mut se = i + 1;
            while se < lines.len() && !lines[se].starts_with("*** ") {
                se += 1;
            }
            let probe = anchor_probe(&lines[i + 1..se]);
            file_lines = read_patch_file(path.trim(), cwd, &probe)
                .map(|(_, c)| c.lines().map(str::to_owned).collect())
                .unwrap_or_default();
            out.push(l.to_owned());
            i += 1;
            continue;
        }
        if l.starts_with("*** ") {
            in_update = false; // 任何其它控制行结束 Update body
            out.push(l.to_owned());
            i += 1;
            continue;
        }
        let first = l.chars().next();
        let valid = matches!(first, Some('+') | Some('-') | Some(' '))
            || l.starts_with("@@")
            || l.is_empty();
        if in_update && !valid {
            // case1:与相邻 `+<同内容>` 重复的废行 → 删(内容在 + 行里,不丢)
            let plus_dup = format!("+{l}");
            let next_dup = lines.get(i + 1).map(|n| *n == plus_dup).unwrap_or(false);
            let prev_dup = out.last().map(|o| o == &plus_dup).unwrap_or(false);
            if next_dup || prev_dup {
                drop_dups += 1;
                i += 1;
                continue;
            }
            // case2:文件里有完全相同整行 → context 漏空格 → 补 ` `
            if file_lines.iter().any(|fl| fl == l) {
                out.push(format!(" {l}"));
                add_ctx += 1;
                i += 1;
                continue;
            }
            // else:透过(不猜)
        }
        out.push(l.to_owned());
        i += 1;
    }
    if drop_dups + add_ctx > 0 {
        repairs.push(Repair {
            file: "(unprefixed)".to_owned(),
            kind: "repaired".to_owned(),
            detail: format!(
                "Update 无前缀行修复: 补 context 空格 {add_ctx} / 删重复废行 {drop_dups}(lark change_line)"
            ),
        });
    }
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    (joined, repairs)
}

/// 对 V4A patch 做 pre-flight 修复。`cwd` 用于把 patch 的相对路径解析到真实文件。
/// 返回 `(修复后 V4A, 处理记录)`。无 cwd / 无 `Update File` / 读不到文件时 V4A 原样返回。
pub fn preflight_repair(v4a: &str, cwd: Option<&str>) -> (String, Vec<Repair>) {
    if !has_cwd_candidate(cwd) {
        return (v4a.to_owned(), Vec::new());
    }
    // 没有任何 Update File 直接短路(Add/Delete File 不涉及锚点匹配)。
    if !v4a.contains("*** Update File:") {
        return (v4a.to_owned(), Vec::new());
    }
    let mut repairs = Vec::new();
    let lines: Vec<&str> = v4a.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(path) = line.strip_prefix("*** Update File: ") {
            out.push(line.to_owned());
            i += 1;
            // 收集本 Update File section 的 body(到下一个 `*** ` 控制行为止)。
            let body_start = i;
            while i < lines.len() && !lines[i].starts_with("*** ") {
                i += 1;
            }
            let body = &lines[body_start..i];
            let (repaired_body, rep) = repair_update_section(path.trim(), body, cwd);
            out.extend(repaired_body);
            repairs.push(rep);
        } else {
            out.push(line.to_owned());
            i += 1;
        }
    }
    // 保留尾随换行语义:lines() 丢掉末尾换行,join 后若原文以 \n 结尾则补上。
    let mut joined = out.join("\n");
    if v4a.ends_with('\n') {
        joined.push('\n');
    }
    (joined, repairs)
}
