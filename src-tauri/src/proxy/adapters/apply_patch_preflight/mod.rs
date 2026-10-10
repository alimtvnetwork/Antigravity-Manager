//! Apply-patch preflight repair (split from apply_patch_preflight.rs).
//! Facade: implementation lives in submodules, each <= 500 lines.

//! apply_patch **pre-flight 自动修复**:在把 V4A patch 发给 Codex apply 之前,读目标文件比对,
//! 自动对齐**安全**的上下文失配(尾随空格 / 首尾空白差异),消灭 V4A 头号失败
//! `apply_patch verification failed: Failed to find expected lines`。
//!
//! ## 为什么需要
//! 弱一点的 chat 模型(非 OpenAI)在大文件上常无法逐字节复刻 `Update File` 的 context/删除行
//! (尾随空格、缩进、记忆偏差)→ Codex 找不到锚点 → apply 失败 → 模型整文件重写,浪费时间和 token。
//! 实测真机报错(rollout 地面真相)正是这类。
//!
//! ## 安全边界(绝不损坏文件 —— 对齐用户「不做破坏性降级」硬规则)
//! - **只动锚点**:`Update File` 里的 context(空格前缀)/ 删除(`-`)行。`+新增` 行**绝不改动**。
//! - **只在唯一匹配时修**:锚点块在文件里按「忽略尾随空格 / 首尾空白」找候选,**恰好一个**位置才对齐;
//!   0 个(模型真改错内容)或 ≥2 个(歧义)一律**原样放行**,交给 Codex parse_patch 暴露真坏,绝不靠猜。
//! - **Add File / Delete File 不碰**(无锚点,不涉及匹配)。读不到文件 / 无 cwd → 原样放行。
//! - 每条修复 / 放行都记进 apply-patch 诊断页,可审计。

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde_json::Value;

/// 一条 pre-flight 处理记录(给诊断页 / 日志)。
#[derive(Debug, Clone, PartialEq)]

pub struct Repair {
    /// patch 里的文件路径(相对,原样)。
    pub file: String,
    /// `repaired`(对齐了锚点)/ `clean`(本就精确匹配,未改)/ `skipped:<原因>`(放行未修)。
    pub kind: String,
    /// 人类可读详情(改了几行 / 为何放行)。
    pub detail: String,
}

/// [MOC-194/MOC-263] 进程级「最近见过的 cwd」候选历史(most-recent-first,去重,容量上限)。
///
/// **为什么从单槽改成候选列表(MOC-263 P1)**:Codex 只在 turn-start 请求发 `<cwd>`,apply_patch
/// 工具循环后续请求不带 cwd → 靠跨请求记忆。旧实现是**进程级单槽**,多个 Codex 会话并发时(真机
/// 常态:同时开 N 个对话改不同项目)单槽被**别的会话**的 turn-start cwd 持续覆盖 → apply_patch
/// 请求回退到的是**别项目的 stale cwd** → Tier B 读盘规则解析到错目录 → 全程 `skipped:unreadable`
/// (实测 phase-1:5/5 段兜底全废)。改成**最近 N 个不同 cwd 的候选列表**:读盘时对每个候选试
/// `cwd/相对路径` 是否存在,选**第一个存在**的(真项目 cwd 才有该文件,stale cwd 没有 → 自动选对)。
/// 命中错 cwd 的同名文件最坏让后续锚点匹配失败 → 安全 skip,绝不误改(保持「不猜不丢」)。
pub mod cwd;
pub mod envelope;
pub mod normalize;
pub mod pipeline;
pub mod read;
pub mod recover;
pub mod repair;
pub mod validate;

#[cfg(test)]
mod tests_cwd;
#[cfg(test)]
mod tests_repair;

pub use cwd::{extract_cwd, remember_cwd_from_request, remember_cwd_from_text};
pub use envelope::{ensure_v4a_envelope, preflight_repair};
pub use pipeline::optimize_patch;
pub use validate::validate_v4a_for_codex;
