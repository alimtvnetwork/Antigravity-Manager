// Hunk repair (split from apply_patch_preflight.rs).
use super::cwd::anchor_probe;
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

use serde_json::Value;

/// 一条 pre-flight 处理记录(给诊断页 / 日志)。
#[derive(Debug, Clone, PartialEq)]

/// [MOC-263 P0] 在 `file[floor..]` 里找从 `anchors[0]` 起、能**唯一**匹配的最长连续块。
/// 锚点用「忽略尾随空格」比较(段内字节漂移留给后续 repair_hunk 对齐)。返回 `(块长 = 匹配的锚点数,
/// 文件起点)`。最长且唯一 → Some;最长的非空匹配若 >1 处(歧义)→ None(更短只会更歧义);全 0 → None。
fn longest_unique_block(anchors: &[&str], file: &[&str], floor: usize) -> Option<(usize, usize)> {
    if anchors.is_empty() || floor >= file.len() {
        return None;
    }
    // [MOC-263 P1] 段首锚点必须在 `file[floor..]` **全局唯一**,否则该段起点歧义 —— 同一行在别处也出现时,
    // 贪心最长块会靠更长块的"唯一性"选中**无关的更早区域**(file 有旧 `A/B/C` 块 + 真实 `A/B…gap…C/D`
    // 区,body `A/-B/ C/-D` 被切成旧块的 hunk → 从错块删 B),而不切分时本会安全失败。起点不唯一 = 切分
    // 非唯一确定 → bail,原样透过交模型自纠(chatgpt-codex-connector review;不猜不丢)。
    let first = anchors[0].trim_end();
    if file[floor..]
        .iter()
        .filter(|l| l.trim_end() == first)
        .count()
        != 1
    {
        return None;
    }
    let max_len = anchors.len().min(file.len() - floor);
    for len in (1..=max_len).rev() {
        let block = &anchors[..len];
        let mut hits: Vec<usize> = Vec::new();
        let mut start = floor;
        while start + len <= file.len() {
            if (0..len).all(|t| file[start + t].trim_end() == block[t].trim_end()) {
                hits.push(start);
                if hits.len() > 1 {
                    break;
                }
            }
            start += 1;
        }
        match hits.len() {
            1 => return Some((len, hits[0])),
            0 => continue,    // 太长(跨越文件里的跳变)→ 缩短再试
            _ => return None, // 最长非空匹配即歧义 → 放弃(不猜不丢)
        }
    }
    None
}

/// [MOC-263 P0] 把**无 `@@`** 的 Update body 按文件真实位置切成多个 hunk。
///
/// 真机最主要 apply 失败因(phase-1 seg1/seg3):模型把多个**不连续**编辑组拼进一个 Update File 块、
/// 漏写 `@@` 分隔 → applier 把整块当一段连续上下文匹配 → `Failed to find expected lines`。
/// 这里按锚点(context/delete)序列贪心切成**有序、不重叠、各自唯一匹配**的 N 段(每段 = 从上一段
/// 之后起、唯一匹配的最长连续锚点块),`+` 新增行随相邻段保留。仅 **N≥2 且每段都能唯一定位**时返回
/// `Some`;单段 / 任一段歧义或无法定位 → `None`(调用方原样透过,不猜不丢)。调用方用裸 `@@` 串接各段。
fn segment_no_at_body<'a>(body: &[&'a str], file: &[&str]) -> Option<Vec<Vec<&'a str>>> {
    let anchors: Vec<(usize, &str)> = body
        .iter()
        .enumerate()
        .filter_map(|(idx, l)| match l.chars().next() {
            Some(' ') | Some('-') => Some((idx, &l[1..])),
            _ => None,
        })
        .collect();
    if anchors.len() < 2 {
        return None;
    }
    let anchor_contents: Vec<&str> = anchors.iter().map(|(_, c)| *c).collect();

    // 贪心分段:每段 = 从 floor 起唯一匹配的最长连续锚点块。
    // 记 (anchor_start, anchor_end_excl, file_start, file_end_excl);floor 单调推进保证有序不重叠。
    let mut raw: Vec<(usize, usize, usize, usize)> = Vec::new();
    let mut ai = 0usize;
    let mut floor = 0usize;
    while ai < anchors.len() {
        let (len, pos) = longest_unique_block(&anchor_contents[ai..], file, floor)?;
        raw.push((ai, ai + len, pos, pos + len));
        ai += len;
        floor = pos + len;
    }

    // 合并相邻段:段间 file 间隙若**全空行**(模型漏写文件里的空行)→ 同一 hunk,不在此切,
    // 交 repair_hunk 的 EP-1 blank-tolerant 处理(否则会把空行漂移误切成两段,破坏既有行为)。
    // 只在间隙含**非空行**(真·不连续编辑区域)时才保留为独立段。
    let mut groups: Vec<(usize, usize, usize, usize)> = Vec::new();
    for g in raw {
        if let Some(last) = groups.last_mut() {
            let gap = &file[last.3..g.2];
            if gap.iter().all(|l| l.trim().is_empty()) {
                last.1 = g.1;
                last.3 = g.3;
                continue;
            }
        }
        groups.push(g);
    }
    if groups.len() < 2 {
        return None; // 单段(或全因空行间隙合并成单段)→ 没必要切,交回常规路径
    }

    // [MOC-263 P0 安全防护] 段间「浮动 `+` 插入行」落点歧义 → 一律 bail。
    // 若前段末锚点与后段首锚点之间存在**任何** `+` 新增行,该 `+` 的落点都无法从 V4A 唯一确定 ——
    // 它可能是前段末尾的插入,也可能是模型写给后段的「引入行」;段间隔着非空内容时两种落点是文件里
    // 不同位置,猜错就是**静默错误 apply**(违反不猜不丢)。**关键**:即便前段末锚点是 `-` 删除也不安全
    // (chatgpt-codex-connector review 指出的混合 replace+insert:`-return 1`/`+return 42`/`+@memoize`/
    // ` def beta():` —— `+return 42` 是替换、`+@memoize` 却是给后段的引入行,二者无法区分)→ 不做
    // "前段有删除就放行"的豁免,只要 gap 里有 `+` 就放弃整次分段、原样透过(交模型自纠)。
    // 纯删除 / 纯上下文的多区(gap 无 `+`)仍安全切。
    for gi in 0..groups.len() - 1 {
        let last_anchor_line = anchors[groups[gi].1 - 1].0;
        let next_anchor_line = anchors[groups[gi + 1].0].0;
        let gap_has_add = body[last_anchor_line + 1..next_anchor_line]
            .iter()
            .any(|l| l.starts_with('+'));
        if gap_has_add {
            return None;
        }
    }

    // 段 g 的 body 行区间:首段含开头前导行(body[0..首锚点]);其余段从其首锚点起,
    // 到下一段首锚点止 → 段内 / 段后的 `+` 行随**前**段保留。
    let mut subhunks: Vec<Vec<&'a str>> = Vec::new();
    for gi in 0..groups.len() {
        let line_start = if gi == 0 { 0 } else { anchors[groups[gi].0].0 };
        let line_end = if gi + 1 < groups.len() {
            anchors[groups[gi + 1].0].0
        } else {
            body.len()
        };
        subhunks.push(body[line_start..line_end].to_vec());
    }
    Some(subhunks)
}

/// 修复一个 `Update File` section 的 body。`path` 是 patch 里的(相对)路径。
/// `cwd` 是当前请求 cwd(primary hint),读盘经 [`read_patch_file`] 再按候选历史解析(MOC-263)。
pub(crate) fn repair_update_section(
    path: &str,
    body: &[&str],
    cwd: Option<&str>,
) -> (Vec<String>, Repair) {
    let probe = anchor_probe(body);
    let Some((_abs, content)) = read_patch_file(path, cwd, &probe) else {
        return (
            body.iter().map(|l| (*l).to_owned()).collect(),
            Repair {
                file: path.to_owned(),
                kind: "skipped:unreadable".to_owned(),
                detail: format!("读不到文件 {path}(候选 cwd 均无)→ 原样放行"),
            },
        );
    };
    let file_lines: Vec<&str> = content.lines().collect();

    // [MOC-263 P0] body 无 `@@` 但含多个不连续编辑组(模型漏写 `@@` 分隔)→ 自动按文件位置切段、
    // 用裸 `@@` 串接,使 applier 把各段当独立 hunk 定位(否则整块当一段连续上下文必失配)。
    // 仅唯一可分段时才动;单段 / 歧义 → 保持原 body 交常规路径。
    let mut split_owned: Vec<&str> = Vec::new();
    // 用**列 0** `@@`(不 trim_start)判断是否已有 hunk 分隔,与下方实际分割器(`l.starts_with("@@")`)
    // 一致 —— 否则 context 行 ` @@ ...`(前导空格、内容以 @@ 开头,如 markdown/diff 文本)会被误当分隔符、
    // 错误禁用自动切分,而分割器又不切它 → 仍失败(chatgpt-codex-connector review)。
    let did_split = if !body.iter().any(|l| l.starts_with("@@")) {
        match segment_no_at_body(body, &file_lines) {
            Some(subhunks) => {
                for (k, sub) in subhunks.iter().enumerate() {
                    if k > 0 {
                        split_owned.push("@@");
                    }
                    split_owned.extend_from_slice(sub);
                }
                true
            }
            None => false,
        }
    } else {
        false
    };
    let effective_body: &[&str] = if did_split { &split_owned } else { body };

    // 把 body 切成 hunk(按 `@@` 行分段;`@@` 行本身保留、不参与锚点匹配)。
    let mut new_body: Vec<String> = Vec::with_capacity(effective_body.len());
    let mut repaired_hunks = 0;
    let mut clean_hunks = 0;
    let mut skipped: Vec<String> = Vec::new();
    let mut hunk: Vec<&str> = Vec::new();
    let flush = |hunk: &mut Vec<&str>,
                 new_body: &mut Vec<String>,
                 repaired_hunks: &mut usize,
                 clean_hunks: &mut usize,
                 skipped: &mut Vec<String>| {
        if hunk.is_empty() {
            return;
        }
        match repair_hunk(hunk, &file_lines) {
            HunkOutcome::Clean => {
                *clean_hunks += 1;
                new_body.extend(hunk.iter().map(|l| (*l).to_owned()));
            }
            HunkOutcome::Repaired(fixed) => {
                *repaired_hunks += 1;
                new_body.extend(fixed);
            }
            HunkOutcome::Skipped(reason) => {
                skipped.push(reason);
                new_body.extend(hunk.iter().map(|l| (*l).to_owned()));
            }
        }
        hunk.clear();
    };

    for &l in effective_body {
        if l.starts_with("@@") {
            flush(
                &mut hunk,
                &mut new_body,
                &mut repaired_hunks,
                &mut clean_hunks,
                &mut skipped,
            );
            new_body.push(l.to_owned());
        } else {
            hunk.push(l);
        }
    }
    flush(
        &mut hunk,
        &mut new_body,
        &mut repaired_hunks,
        &mut clean_hunks,
        &mut skipped,
    );

    let kind = if repaired_hunks > 0 || did_split {
        "repaired"
    } else if skipped.is_empty() {
        "clean"
    } else {
        "skipped:no_unique_match"
    };
    let detail = format!(
        "{}hunk: 修复 {repaired_hunks} / 本就匹配 {clean_hunks} / 放行 {}{}",
        if did_split {
            "多 hunk 无 @@ 分隔 → 自动按文件位置切段插裸 @@; "
        } else {
            ""
        },
        skipped.len(),
        if skipped.is_empty() {
            String::new()
        } else {
            format!(" ({})", skipped.join("; "))
        }
    );
    (
        new_body,
        Repair {
            file: path.to_owned(),
            kind: kind.to_owned(),
            detail,
        },
    )
}

enum HunkOutcome {
    /// 锚点精确匹配文件,无需改。
    Clean,
    /// 锚点对齐成文件真实字节后的整个 hunk(含原样的 `+` 行)。
    Repaired(Vec<String>),
    /// 未修(0 或多个匹配),附原因。
    Skipped(String),
}

/// 修一个 hunk:锚点 = context(空格前缀)+ 删除(`-`)行的**内容**(去前缀),按序应是文件里的
/// 连续块。精确匹配→Clean;否则按「忽略尾随空格 / 首尾空白」找候选,唯一→对齐,否则放行。
fn repair_hunk(hunk: &[&str], file_lines: &[&str]) -> HunkOutcome {
    // 锚点行在 hunk 里的下标 + 内容(去单字符前缀)。
    let anchors: Vec<(usize, &str)> = hunk
        .iter()
        .enumerate()
        .filter_map(|(idx, l)| match l.chars().next() {
            Some(' ') => Some((idx, &l[1..])),
            Some('-') => Some((idx, &l[1..])),
            _ => None, // '+' 新增行 / 空行 / 其它不作锚点
        })
        .collect();
    if anchors.is_empty() {
        return HunkOutcome::Clean; // 纯新增,无锚点
    }
    let anchor_contents: Vec<&str> = anchors.iter().map(|(_, c)| *c).collect();

    // 精确匹配:文件里存在连续块完全等于锚点内容 → 无需修(Codex 自己能找到)。
    if !find_block(file_lines, &anchor_contents, |a, b| a == b).is_empty() {
        return HunkOutcome::Clean;
    }

    // 模糊匹配:逐行「忽略尾随空格」相等;仍 0 个再退「首尾空白都忽略」。
    let mut matches = find_block(file_lines, &anchor_contents, |a, b| {
        a.trim_end() == b.trim_end()
    });
    let mut mode = "尾随空格";
    if matches.is_empty() {
        matches = find_block(file_lines, &anchor_contents, |a, b| a.trim() == b.trim());
        mode = "首尾空白";
    }
    match matches.len() {
        1 => {
            let pos = matches[0];
            // 把锚点行对齐成文件真实字节(保留 hunk 里 +/- /空格 的交错与 `+` 行)。
            let mut fixed: Vec<String> = hunk.iter().map(|l| (*l).to_owned()).collect();
            for (k, (idx, _)) in anchors.iter().enumerate() {
                let prefix = hunk[*idx].chars().next().unwrap(); // ' ' 或 '-'
                let file_line = file_lines[pos + k];
                fixed[*idx] = format!("{prefix}{file_line}");
            }
            HunkOutcome::Repaired(fixed)
        }
        n if n > 1 => HunkOutcome::Skipped(format!("{mode}下 {n} 处匹配(歧义)")),
        // 0 连续匹配 → 试「忽略空行差异」(EP-1:模型漏/多写空行致整块失配)。锚点**非空行**序列
        // 在文件里唯一定位(允许文件该区间含模型漏写的空行),命中则用文件真实区间(含空行 + 字节)
        // 重建锚点,`+` 插入行保持原位。0/多处仍放行(不猜)。
        _ => {
            // blank-tolerant 重建会丢弃空白锚点行、改用文件空行 → 无法忠实表达「删除一个空行」的 `-`
            // (会被静默转成 context = 该删没删)。若 hunk 含空白行删除,放弃 blank-tolerant、透过(不猜)。
            let has_blank_deletion = hunk
                .iter()
                .any(|l| l.starts_with('-') && l[1..].trim().is_empty());
            if has_blank_deletion {
                return HunkOutcome::Skipped(
                    "含空白行删除,blank-tolerant 不安全 → 放行".to_owned(),
                );
            }
            let regions = find_regions_blank_tolerant(file_lines, &anchor_contents);
            match regions.len() {
                1 => {
                    let (s, e) = regions[0];
                    HunkOutcome::Repaired(rebuild_hunk_with_region(hunk, &file_lines[s..e]))
                }
                0 => HunkOutcome::Skipped("锚点在文件中 0 匹配(疑模型改错内容)".to_owned()),
                n => HunkOutcome::Skipped(format!("忽略空行下 {n} 处匹配(歧义)")),
            }
        }
    }
}

/// EP-1 辅助:在 `file_lines` 里找锚点**非空行**序列能唯一定位的区间(允许文件区间内含模型漏写的
/// 空行,但不允许有额外的非空行)。返回所有匹配区间 `[start, end)`(end 为最后一个匹配非空行的下一位)。
fn find_regions_blank_tolerant(
    file_lines: &[&str],
    anchor_contents: &[&str],
) -> Vec<(usize, usize)> {
    let nb: Vec<&str> = anchor_contents
        .iter()
        .map(|c| c.trim_end())
        .filter(|c| !c.trim().is_empty())
        .collect();
    if nb.is_empty() {
        return Vec::new();
    }
    let mut regions = Vec::new();
    for start in 0..file_lines.len() {
        if file_lines[start].trim().is_empty() || file_lines[start].trim_end() != nb[0] {
            continue;
        }
        let mut fi = start;
        let mut ai = 0;
        let mut ok = true;
        while ai < nb.len() {
            if fi >= file_lines.len() {
                ok = false;
                break;
            }
            let fl = file_lines[fi];
            if fl.trim().is_empty() {
                fi += 1; // 跳过文件空行(模型可能漏写)
                continue;
            }
            if fl.trim_end() == nb[ai] {
                ai += 1;
                fi += 1;
            } else {
                ok = false; // 出现额外非空行 → 此 start 不匹配
                break;
            }
        }
        if ok && ai == nb.len() {
            regions.push((start, fi));
        }
    }
    regions
}

/// EP-1 辅助:用文件真实区间 `region`(含空行)重建 hunk —— 锚点(context/`-`)对齐成文件字节、
/// 补回模型漏写的文件空行(作 context),`+` 插入行按 hunk 原序保持。模型自带的空白锚点行丢弃
/// (改用文件的空行,避免重复)。
fn rebuild_hunk_with_region(hunk: &[&str], region: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut fi = 0usize; // region 游标
    for &hl in hunk {
        match hl.chars().next() {
            Some('+') => out.push(hl.to_owned()), // 插入行原样保位
            Some(' ') | Some('-') => {
                let prefix = hl.chars().next().unwrap();
                let content = &hl[1..];
                if content.trim().is_empty() {
                    continue; // 模型的空锚点行丢弃,用文件空行
                }
                // 先补回文件里模型漏写的空行(作 context)
                while fi < region.len() && region[fi].trim().is_empty() {
                    out.push(format!(" {}", region[fi]));
                    fi += 1;
                }
                if fi < region.len() {
                    out.push(format!("{prefix}{}", region[fi]));
                    fi += 1;
                } else {
                    out.push(hl.to_owned());
                }
            }
            _ => {} // 无前缀空行等丢弃,用文件空行
        }
    }
    out
}

/// 在 `file_lines` 里找所有起点 `i`,使 `file_lines[i..i+anchor.len()]` 与 `anchor` 逐行 `eq` 为真。
/// 返回所有匹配起点。
fn find_block<F: Fn(&str, &str) -> bool>(
    file_lines: &[&str],
    anchor: &[&str],
    eq: F,
) -> Vec<usize> {
    if anchor.is_empty() || anchor.len() > file_lines.len() {
        return Vec::new();
    }
    let mut hits = Vec::new();
    for i in 0..=(file_lines.len() - anchor.len()) {
        if (0..anchor.len()).all(|k| eq(file_lines[i + k], anchor[k])) {
            hits.push(i);
        }
    }
    hits
}

/// 把 patch 路径解析到绝对路径。绝对路径原样;相对路径对 `cwd` 拼接。
pub(crate) fn resolve_path(path: &str, cwd: &str) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        Path::new(cwd).join(p)
    }
}
