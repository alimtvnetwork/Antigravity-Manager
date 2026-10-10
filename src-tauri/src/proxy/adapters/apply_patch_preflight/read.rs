// Patch file reading (split from apply_patch_preflight.rs).
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

/// 一条 pre-flight 处理记录(给诊断页 / 日志)。

pub(crate) fn read_patch_file(
    relpath: &str,
    primary: Option<&str>,
    probe: &[(bool, &str)],
) -> Option<(PathBuf, String)> {
    let p = Path::new(relpath);
    if p.is_absolute() {
        return std::fs::read_to_string(p)
            .ok()
            .map(|c| (p.to_path_buf(), c));
    }
    // ① fresh primary 权威:当前请求自带 cwd 且文件可读 → 直接用,交下游决定匹配(含 align_at_headers
    //    的 partial `@@` 子串修复)。**probe 只在多个同名候选间做 tie-breaker,绝不当 gate** —— 否则
    //    残缺 `@@` 头 / 单一候选会因 probe 0 命中被误判 unreadable(chatgpt-codex-connector review P2 二轮)。
    if let Some(c) = primary {
        if !c.is_empty() {
            let abs = Path::new(c).join(p);
            if let Ok(content) = std::fs::read_to_string(&abs) {
                return Some((abs, content));
            }
        }
    }
    // ② 否则用最近 cwd 候选历史(most-recent-first),读出所有存在的同名文件。
    let mut readable: Vec<(PathBuf, String)> = Vec::new();
    for c in recall_cwd_candidates() {
        let abs = Path::new(&c).join(p);
        if let Ok(content) = std::fs::read_to_string(&abs) {
            readable.push((abs, content));
        }
    }
    match readable.len() {
        0 => return None,
        // 单候选 → 直接用(下游决定匹配,partial header 子串修复才有机会);不因 probe 0 命中而 skip。
        1 => return readable.into_iter().next(),
        _ => {}
    }
    // ③ 多个同名候选(并发会话共享 README.md/package.json 等)→ 按锚点 probe 挑 patch 真正针对的文件。
    //    评分:context/删除行(非 header)按**整行 exact**(trim)命中;`@@` 头(header)按**子串**命中
    //    真实整行(残缺头是整行子串,如 `系统架构建议` ⊂ `## 6. 系统架构建议`)。两类合并计分,**唯一
    //    最高分**才选;并列 / 全 0 → None(歧义不猜,违反"不猜不丢"则会对 stale 文件对齐)。
    //    header 不进 exact:否则 stale 的同名整行(恰=残缺头)会以 exact 胜过 real 的子串(review)。
    let probe: Vec<(bool, &str)> = probe
        .iter()
        .map(|&(h, t)| (h, t.trim()))
        .filter(|(_, t)| !t.is_empty())
        .collect();
    if probe.is_empty() {
        return readable.into_iter().next(); // 无锚点(纯新增 patch)→ 最 recent(无需对齐,下游 no-op)
    }
    let scores: Vec<usize> = readable
        .iter()
        .map(|(_, c)| {
            let fl: Vec<&str> = c.lines().map(str::trim).collect();
            probe
                .iter()
                .filter(|&&(is_header, t)| {
                    if is_header {
                        fl.iter().any(|line| !line.is_empty() && line.contains(t))
                    } else {
                        fl.iter().any(|line| *line == t)
                    }
                })
                .count()
        })
        .collect();
    let max = scores.iter().copied().max().unwrap_or(0);
    if max == 0 {
        return None; // 没有候选含任何锚点 → 都不是目标 → skip(安全)
    }
    if scores.iter().filter(|s| **s == max).count() != 1 {
        return None; // 并列最高 = 歧义 → 不猜
    }
    let best_idx = scores.iter().position(|s| *s == max).unwrap();
    Some(readable.swap_remove(best_idx))
}
