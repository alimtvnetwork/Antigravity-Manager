// Patch optimization pipeline (split from apply_patch_preflight.rs).
use super::cwd::remember_cwd;
use super::envelope::{ensure_v4a_envelope, fix_unprefixed_lines, preflight_repair};
use super::normalize::{
    convert_unified_file_headers, diagnose_absolute_paths, strip_trailing_at,
    strip_unified_hunk_ranges,
};
use super::recover::{recover_empty_move, recover_update_empty_file};
use super::repair::repair_update_section;
use super::validate::{align_at_headers, ensure_add_file_plus};
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

/// apply_patch **中间层总入口**:按白名单规则**逐条恢复已知格式错误**,使模型不遵循 prompt 时
/// 产出的畸形 patch 仍能被 Codex 正确 apply。**只动确定的已知坑;未知一律原样放行(不猜不丢)。**
///
/// 两层结构(对齐 [[MOC-194]] 方案):
///
/// **Tier A 语法规整**(镜像 Codex 给 GPT 的 lark 语法,纯字符串、不读盘 —— 把 GPT 靠语法约束生成
/// 保证的合法性,在第三方 chat 路径事后保证):
/// - [`strip_trailing_at`] — 双边 `@@ … @@` → 单边(grammar `change_context: "@@" | "@@ " /(.+)/`;实测 18×)。
/// - [`convert_unified_file_headers`] — `--- old` / `+++ new` 或 `file: path` → `*** Update File: path`。
/// - [`ensure_add_file_plus`] — Add File 内容行漏 `+` → 补全(grammar `add_line: "+" /(.*)/`,Add File 无歧义)。
/// - [`ensure_v4a_envelope`] — 缺 `*** Begin/End Patch` → 补全(grammar `start: begin_patch hunk+ end_patch`;
///   gotcha #6 + 真机 seq230)。**仅 `json_complete`(非流式截断)时做**,且**放最后**以包裹 Tier B 产物。
///
/// **Tier B 语义恢复**(grammar 管不到的文件状态/内容层,需 `cwd` 读盘):
/// - [`recover_update_empty_file`] — Update 空文件 → Delete+Add(实测 50×,无损)。
/// - [`align_at_headers`] — `@@ <header>` 残缺锚点 → 对齐文件真实整行(`Failed to find context`)。
/// - [`fix_unprefixed_lines`] — Update 内无前缀行 → 按文件判定补 context 空格 / 删重复废行(seq235)。
/// - [`recover_empty_move`] — 空 Update+Move(rename-only)→ Delete+Add 复制原内容(实测 76×)。
/// - [`preflight_repair`] — Update 上下文 byte-exact 失配 → 读盘对齐(实测 134×)。
///
/// 未覆盖的错点:**原样透过**,交 Codex applier 报错(不猜不丢)。
/// `json_complete`:调用方传 `detect_json_truncation(args).is_none()`(chat);gemini args 一次性完整传 `true`。
pub fn optimize_patch(v4a: &str, cwd: Option<&str>, json_complete: bool) -> (String, Vec<Repair>) {
    // [MOC-194/MOC-263] **两类 cwd,分流使用**:
    // - `fresh_cwd` = 当前请求自带的 `<cwd>`(apply_patch 请求通常 None)。**判定文件 == Codex 应用
    //   文件**,可信。
    // - 候选历史 = 跨请求记忆的最近 N 个不同 cwd([`recall_cwd_candidates`])。Codex 只在 turn-start
    //   请求发 `<cwd>`,apply_patch 工具循环后续请求不带 → 靠它回退。MOC-263:从单槽改候选列表,
    //   并发多会话不再被别项目 stale cwd 覆盖(读盘按候选逐个试、选第一个存在的)。
    //
    // **状态改写规则**(`recover_update_empty_file` / `recover_empty_move`:把 Update 转成 Delete+Add)
    // 的判定文件与应用文件(Codex 用 patch 相对路径在真实 cwd 应用)**可能不是同一个** → 错 cwd 下会
    // 删错项目的同名文件(破坏性)。故这两条**只用 fresh_cwd**(判定==应用才安全),**不查候选历史**;
    // apply_patch 请求无 fresh cwd → 自动跳过透过(安全)。
    // **byte-exact 对齐规则**(align/preflight/fix_unprefixed)传 `fresh_cwd` 作 primary,内部经
    // [`read_patch_file`] 再查候选历史:最坏命中错文件也只是「不唯一匹配 / byte 不符」→ 安全 no-op。
    let fresh_cwd = cwd;
    // 当前请求若带 cwd,记入候选历史(turn-start 的 cwd 主要由转换器 `remember_cwd_from_request`
    // 在每请求记入;这里兜底:万一 apply_patch 请求自带 cwd 也纳入)。
    if let Some(c) = cwd {
        remember_cwd(c);
    }
    let mut repairs = Vec::new();
    let mut s = v4a.to_owned();
    repairs.extend(diagnose_absolute_paths(&s, fresh_cwd));

    // ── Tier A 语法规整(纯字符串)──
    let (s1, r1) = strip_trailing_at(&s);
    s = s1;
    repairs.extend(r1);

    let (s_d, r_d) = convert_unified_file_headers(&s);
    s = s_d;
    repairs.extend(r_d);

    let (s_r, r_r) = strip_unified_hunk_ranges(&s);
    s = s_r;
    repairs.extend(r_r);

    let (s_g, r_g) = ensure_add_file_plus(&s);
    s = s_g;
    repairs.extend(r_g);

    // ── Tier B 语义恢复 ──
    // 注:`Add File 已存在 → Delete+Add 覆盖` 规则**已撤销**(2026-06-09)。它会覆盖已有文件、
    // 可能丢失 Add 内容里没有的现存内容(破坏性降级);且会抢走模型收到 `already exists` 后
    // 自纠为**针对性 Update**(无损)的机会。改为原样透过、交 Codex 报 `already exists` 让模型自纠。
    //
    // 状态改写规则 → **fresh_cwd**(防 stale 删错文件,见上)。
    let (s_f, r_f) = recover_update_empty_file(&s, fresh_cwd);
    s = s_f;
    repairs.extend(r_f);

    let (s3, r3) = recover_empty_move(&s, fresh_cwd);
    s = s3;
    repairs.extend(r3);

    // byte-exact 对齐规则 → 传 fresh_cwd 作 primary,内部 read_patch_file 再查候选历史(最坏安全 no-op)。
    let (s_h, r_h) = align_at_headers(&s, fresh_cwd);
    s = s_h;
    repairs.extend(r_h);

    let (s_u, r_u) = fix_unprefixed_lines(&s, fresh_cwd);
    s = s_u;
    repairs.extend(r_u);

    let (s2, r2) = preflight_repair(&s, fresh_cwd);
    s = s2;
    repairs.extend(r2);

    // ── 信封补全放最后:包裹 Tier B 可能新增的 Delete+Add 等结构 ──
    if json_complete {
        let (s4, r4) = ensure_v4a_envelope(&s);
        s = s4;
        if let Some(r) = r4 {
            repairs.push(r);
        }
    }
    (s, repairs)
}
