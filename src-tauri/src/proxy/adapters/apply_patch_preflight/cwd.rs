// CWD tracking (split from apply_patch_preflight.rs).
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
const CWD_CANDIDATES_CAP: usize = 12;
static CWD_HISTORY: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

fn cwd_history() -> &'static Mutex<VecDeque<String>> {
    CWD_HISTORY.get_or_init(|| Mutex::new(VecDeque::new()))
}

/// 记一个最近见到的 cwd(去重置顶,超 [`CWD_CANDIDATES_CAP`] 淘汰最旧)。空串忽略。
pub(crate) fn remember_cwd(cwd: &str) {
    if cwd.is_empty() {
        return;
    }
    if let Ok(mut q) = cwd_history().lock() {
        if let Some(pos) = q.iter().position(|c| c == cwd) {
            q.remove(pos);
        }
        q.push_front(cwd.to_owned());
        while q.len() > CWD_CANDIDATES_CAP {
            q.pop_back();
        }
    }
}

/// 最近见过的 cwd 候选(most-recent-first)。
pub(crate) fn recall_cwd_candidates() -> Vec<String> {
    cwd_history()
        .lock()
        .map(|q| q.iter().cloned().collect())
        .unwrap_or_default()
}

/// 是否有任何可用 cwd(当前请求的 `primary` 或历史候选)。byte-exact 规则据此短路。
pub(crate) fn has_cwd_candidate(primary: Option<&str>) -> bool {
    primary.map(|c| !c.is_empty()).unwrap_or(false) || !recall_cwd_candidates().is_empty()
}

/// patch section 的「锚点 probe」,每项 `(is_header, 文本)`:
/// - context(` `)/ 删除(`-`)行去前缀 → `(false, 行内容)`,在候选文件里按**整行 exact**(trim)比对;
/// - `@@ <header>` 的 header 文本 → `(true, header)`,按**子串**比对(残缺头是真实整行的子串,如
///   `系统架构建议` ⊂ `## 6. 系统架构建议`)。两类分开评分:exact 头若被 stale 的同名整行命中会误选,
///   故 header 不进 exact(chatgpt-codex-connector review)。供 [`read_patch_file`] 在同名候选间挑目标文件。
pub(crate) fn anchor_probe<'a>(body: &[&'a str]) -> Vec<(bool, &'a str)> {
    let mut probe = Vec::new();
    for l in body {
        match l.chars().next() {
            Some(' ') | Some('-') => probe.push((false, &l[1..])),
            Some('+') => {} // 新增行 —— 不在目标文件,不作 probe
            _ => {
                if let Some(h) = l.strip_prefix("@@ ") {
                    let h = h.trim();
                    if !h.is_empty() {
                        probe.push((true, h));
                    }
                } else if !l.is_empty() && !l.starts_with("@@") && !l.starts_with("*** ") {
                    // 无前缀行(模型漏写前缀,fix_unprefixed_lines 要按文件整行 exact 匹配来修)→ 整行作
                    // exact probe,使空-probe 路径(漏前缀是唯一锚点时)也能在同名候选间挑对文件
                    // (chatgpt-codex-connector review)。
                    probe.push((false, l));
                }
            }
        }
    }
    probe
}

/// 按候选 cwd 解析并读取 patch 目标文件(MOC-263 P1 + P2)。`primary`(当前请求 cwd,apply_patch
/// 请求通常 None)优先,再按最近 cwd 历史逐个试。`probe` = patch 的 context/删除锚点行内容
/// ([`anchor_probe`]):多个候选 cwd 都存在同名相对文件时(并发会话共享 `README.md`/`package.json`
/// 等),**选内容里命中最多 probe 锚点行的候选**(= patch 真正针对的文件),而非取第一个可读的
/// (chatgpt-codex-connector review P2:取第一个会对错文件对齐)。所有候选 probe 命中均为 0 → 没有
/// 候选是目标 → 返回 None(skip,安全)。`probe` 为空(纯新增 patch 无锚点)→ 退回第一个可读
/// (无从判别、最好努力)。绝对路径直接读。

/// 从 Codex Responses 请求里抽 `<cwd>...</cwd>`(Codex 注入的 environment_context 块,
/// 形如 `<environment_context>\n  <cwd>/abs/path</cwd>\n  <shell>zsh</shell>...`)。
///
/// **遍历 Value 树**找含 `<cwd>` 的字符串节点(其值已是 serde 反转义后的原文)再抽取 —— **不能**
/// 先 `serde_json::to_string(整个请求)` 再搜:那会把字符串值**重新 JSON 转义**,Windows 路径
/// `C:\Users\...` 的反斜杠被翻倍成 `C:\\Users\\...`,resolve_path 拿到错路径(codex-connector #435 P2)。
/// 不依赖 `<cwd>` 落在 instructions 还是某条 input message(任意层级的 string 节点都扫)。
pub fn extract_cwd(request: Option<&Value>) -> Option<String> {
    fn find_in_value(v: &Value) -> Option<String> {
        match v {
            Value::String(s) => extract_cwd_from_str(s),
            Value::Array(a) => a.iter().find_map(find_in_value),
            Value::Object(o) => o.values().find_map(find_in_value),
            _ => None,
        }
    }
    find_in_value(request?)
}

/// 从单个(已反转义的)字符串里抽 `<cwd>...</cwd>`。
fn extract_cwd_from_str(s: &str) -> Option<String> {
    let start = s.find("<cwd>")? + "<cwd>".len();
    let rest = &s[start..];
    let end = rest.find("</cwd>")?;
    let cwd = rest[..end].trim();
    if cwd.is_empty() {
        None
    } else {
        Some(cwd.to_owned())
    }
}

/// [MOC-194 关键] 把请求里的 `<cwd>` 记入进程级缓存。**必须对每个请求调用**(不止 apply_patch):
/// 带 `<cwd>` 的是 **turn-start 请求**(不产生 apply_patch、不调 [`optimize_patch`]),而 apply_patch
/// 出现在**不带 cwd 的工具循环后续请求**里。只在 `optimize_patch` 里记忆 → 永远学不到 cwd(实测:
/// `LAST_CWD` 一直 None、所有 Tier B 读盘规则全程 no-op)。故记忆点必须在每请求都经过的地方
/// (转换器 `with_original_request`),turn-start 的 cwd 才能被后续 apply_patch 请求回退到。
pub fn remember_cwd_from_request(request: Option<&Value>) {
    if let Some(cwd) = extract_cwd(request) {
        remember_cwd(&cwd);
    }
}

pub fn remember_cwd_from_text(text: &str) -> bool {
    let Some(cwd) = extract_cwd_from_str(text) else {
        return false;
    };
    remember_cwd(&cwd);
    true
}
