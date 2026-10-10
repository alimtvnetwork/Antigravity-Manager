// Preflight tests (split from apply_patch_preflight.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;
use serde_json::json;
use std::io::Write;

fn tmp_file(name: &str, content: &str) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    (dir, name.to_owned())
}

#[test]
fn update_nonempty_file_not_converted() {
    let (dir, name) = tmp_file("nonempty.txt", "existing\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a =
        format!("*** Begin Patch\n*** Update File: {name}\n-existing\n+changed\n*** End Patch\n");
    let (out, reps) = recover_update_empty_file(&v4a, Some(cwd));
    assert_eq!(out, v4a, "非空文件 Update 不碰");
    assert!(reps.is_empty());
}

#[test]
fn add_file_missing_plus_prefix_is_filled() {
    // Add File 里有的行漏 `+`(模型常见)、有空行 → 全补 `+`,已有 `+` 的不动。
    let v4a = "*** Begin Patch\n*** Add File: new.md\n+# Title\nplain line no plus\n\n+already plus\n*** End Patch\n";
    let (out, reps) = ensure_add_file_plus(v4a);
    assert!(
        out.contains("\n+plain line no plus\n"),
        "漏 + 的行应补:\n{out}"
    );
    assert!(out.contains("\n+\n+already plus"), "空行 → 裸 +:\n{out}");
    assert!(!out.contains("++already plus"), "已是 + 的不重复");
    assert_eq!(reps[0].kind, "repaired");
    assert!(
        reps[0].detail.contains("2 行"),
        "漏 + 的 plain 行 + 空行 = 2: {:?}",
        reps
    );
}

#[test]
fn add_file_all_plus_untouched_and_update_not_affected() {
    // 全 + 的 Add File 不动;Update section 的非 + 行(context/-)绝不被 G 碰。
    let v4a = "*** Begin Patch\n*** Add File: a\n+x\n+y\n*** Update File: b\n cont\n-old\n+new\n*** End Patch\n";
    let (out, reps) = ensure_add_file_plus(v4a);
    assert_eq!(out, v4a, "Add 全 + + Update 不动:\n{out}");
    assert!(reps.is_empty());
}

#[test]
fn at_header_aligned_to_unique_file_line() {
    // 真机 seq181:@@ 头残缺(漏 `## 6. `),唯一包含于一个文件行 → 对齐。
    let (dir, name) = tmp_file("doc.md", "intro\n## 6. 系统架构建议\n建议分层\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n@@ 系统架构建议\n 建议分层\n+新增一行\n*** End Patch\n"
    );
    let (out, reps) = align_at_headers(&v4a, Some(cwd));
    assert!(
        out.contains("@@ ## 6. 系统架构建议"),
        "@@ 应对齐成文件真实整行:\n{out}"
    );
    assert_eq!(reps[0].kind, "repaired");
}

#[test]
fn at_header_exact_or_ambiguous_untouched() {
    // 已是文件真实整行 → 不动;多处包含(歧义)→ 不动。
    let (dir, name) = tmp_file("doc2.md", "## A\nx\n## A\n");
    let cwd = dir.path().to_str().unwrap();
    // 精确整行 `## A` 存在,但歧义(两行)→ 不动
    let v4a = format!("*** Update File: {name}\n@@ ## A\n x\n+y\n");
    let (out, reps) = align_at_headers(&v4a, Some(cwd));
    assert_eq!(out, v4a);
    assert!(reps.is_empty());
    // 子串 `A` 在 `## A` 两行里出现 → 歧义不动
    let v4a2 = format!("*** Update File: {name}\n@@ A\n x\n+y\n");
    let (out2, reps2) = align_at_headers(&v4a2, Some(cwd));
    assert_eq!(out2, v4a2);
    assert!(reps2.is_empty());
}

#[test]
fn unprefixed_dup_of_plus_line_dropped() {
    // 真机 seq235:无前缀行 + 紧跟 `+<同内容>` → 删废行(内容在 + 行,不丢)。
    let (dir, name) = tmp_file("u.md", "other\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Update File: {name}\n*data source*\n+*data source*\n+more\n");
    let (out, reps) = fix_unprefixed_lines(&v4a, Some(cwd));
    assert!(!out.contains("\n*data source*\n"), "无前缀废行应删:\n{out}");
    assert!(out.contains("+*data source*"), "+ 行保留(内容不丢)");
    assert_eq!(reps[0].kind, "repaired");
}

#[test]
fn unprefixed_existing_file_line_gets_context_space() {
    // 无前缀行在文件里有同行 → context 漏空格 → 补 ` `。
    let (dir, name) = tmp_file("u2.md", "alpha\nkeepme\nbeta\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Update File: {name}\nkeepme\n+added\n");
    let (out, reps) = fix_unprefixed_lines(&v4a, Some(cwd));
    assert!(out.contains("\n keepme\n"), "应补空格成 context:\n{out}");
    assert_eq!(reps[0].kind, "repaired");
}

#[test]
fn unprefixed_unknown_passes_through() {
    // 不在文件、非重复 → 透过(不猜)。
    let (dir, name) = tmp_file("u3.md", "real\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Update File: {name}\nhallucinated garbage line\n+x\n");
    let (out, reps) = fix_unprefixed_lines(&v4a, Some(cwd));
    assert_eq!(out, v4a, "未知无前缀行原样透过");
    assert!(reps.is_empty());
}

#[test]
fn cwd_candidates_remember_recall_and_resolve() {
    // MOC-263 P1:候选历史(deque)+ read_patch_file 按候选解析(全局态,只做非 flaky 断言)。
    let (dir, name) = tmp_file("cand_moc263.txt", "x\n");
    let real = dir.path().to_str().unwrap().to_owned();
    // 模拟并发污染:先记一个不含该文件的 stale cwd,再记真实 cwd(置顶)。
    remember_cwd("/tmp/stale_zzz_moc263_a");
    remember_cwd(&real);
    assert!(recall_cwd_candidates().iter().any(|c| c == &real));
    assert!(has_cwd_candidate(None), "有候选历史 → true");
    // primary=None(apply_patch 工具循环请求),真实 cwd 在候选里 → 读到文件
    // (stale cwd 无此文件,逐个试时自动跳过)。这是 P1 的核心:不再被 stale 单槽废掉。
    let got = read_patch_file(&name, None, &[(false, "x")]);
    assert!(got.is_some(), "应经候选 cwd 读到文件");
    assert_eq!(got.unwrap().1, "x\n");
    // turn-start 请求抽 cwd 入候选(供后续 apply_patch 回退)。
    let req = json!({"input":[{"type":"message","role":"user","content":"<environment_context>\n  <cwd>/tmp/ts_proj_b3f9</cwd>\n</environment_context>"}]});
    remember_cwd_from_request(Some(&req));
    assert!(recall_cwd_candidates()
        .iter()
        .any(|c| c == "/tmp/ts_proj_b3f9"));
}

#[test]
fn read_patch_file_picks_by_probe_not_first_readable() {
    // MOC-263 P2(chatgpt-codex-connector review):并发会话共享相对路径(README.md 等)、且 stale
    // 会话更新时,按 patch 锚点 probe 选**真正含锚点的候选**,而非取队首(most-recent=stale)可读的。
    let stale = tempfile::tempdir().unwrap();
    let real = tempfile::tempdir().unwrap();
    std::fs::write(stale.path().join("shared_moc263.txt"), "stale_only_line\n").unwrap();
    std::fs::write(
        real.path().join("shared_moc263.txt"),
        "real_anchor_line\nmore\n",
    )
    .unwrap();
    // 真实 cwd 先记、stale 后记 → stale 在候选队首(most-recent),模拟 review 担心的场景。
    remember_cwd(real.path().to_str().unwrap());
    remember_cwd(stale.path().to_str().unwrap());
    // probe 命中 real(含 real_anchor_line)、不命中 stale → 应选 real,不取队首 stale。
    let got = read_patch_file("shared_moc263.txt", None, &[(false, "real_anchor_line")]);
    assert!(got.is_some(), "应选到含锚点的候选");
    assert_eq!(
        got.unwrap().1,
        "real_anchor_line\nmore\n",
        "应选 real(含 probe 锚点)而非队首 stale"
    );
}

#[test]
fn read_patch_file_single_candidate_partial_header_not_skipped() {
    // MOC-263 P2 二轮(chatgpt-codex-connector review):probe 只含残缺 `@@` header(文件里无 exact
    // 行,真实是 `## 6. 系统架构建议`),**单一候选**不应因 probe 0 命中被判 unreadable —— 仍返回
    // 文件,让 align_at_headers 做子串修复。probe 是 tie-breaker 不是 gate。
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("doc_moc263.md"),
        "intro\n## 6. 系统架构建议\n建议分层\n",
    )
    .unwrap();
    remember_cwd(dir.path().to_str().unwrap());
    let got = read_patch_file("doc_moc263.md", None, &[(true, "系统架构建议")]);
    assert!(
        got.is_some(),
        "单候选 + 残缺 header probe 不应被判 unreadable"
    );
    assert!(got.unwrap().1.contains("## 6. 系统架构建议"));
}

#[test]
fn read_patch_file_partial_header_substring_picks_real_over_stale() {
    // MOC-263 P2 三轮(chatgpt-codex-connector review):多候选 + 纯残缺 `@@` 头,exact 全 0 → 退
    // 子串评分,选**子串含该头的 real**,而非盲取队首 stale(否则 align 会对 stale 子串修复)。
    let stale = tempfile::tempdir().unwrap();
    let real = tempfile::tempdir().unwrap();
    std::fs::write(
        stale.path().join("doc2_moc263.md"),
        "stale intro\nunrelated heading\n",
    )
    .unwrap();
    std::fs::write(
        real.path().join("doc2_moc263.md"),
        "intro\n## 6. 系统架构建议\n建议分层\n",
    )
    .unwrap();
    remember_cwd(real.path().to_str().unwrap());
    remember_cwd(stale.path().to_str().unwrap()); // stale 在队首(most-recent)
    let got = read_patch_file("doc2_moc263.md", None, &[(true, "系统架构建议")]);
    assert!(got.is_some());
    assert!(
        got.unwrap().1.contains("## 6. 系统架构建议"),
        "子串评分应选含该头的 real,而非队首 stale"
    );
}

#[test]
fn read_patch_file_tied_score_is_ambiguous_none() {
    // MOC-263 P2 四轮(chatgpt-codex-connector review):两候选 probe 分数并列(共享相同锚点行)→
    // 歧义 → None(不猜),而非取队首 stale。下游 skip,patch 透过交 Codex / 模型自纠。
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    std::fs::write(a.path().join("tie_moc263.txt"), "SHARED_ANCHOR\naaa\n").unwrap();
    std::fs::write(b.path().join("tie_moc263.txt"), "SHARED_ANCHOR\nbbb\n").unwrap();
    remember_cwd(a.path().to_str().unwrap());
    remember_cwd(b.path().to_str().unwrap());
    let got = read_patch_file("tie_moc263.txt", None, &[(false, "SHARED_ANCHOR")]);
    assert!(got.is_none(), "并列分数(歧义)应返回 None 不猜");
}

#[test]
fn read_patch_file_header_probe_not_beaten_by_stale_exact_fragment() {
    // MOC-263 P2 五轮(chatgpt-codex-connector review):probe 只有残缺 `@@` 头;stale 恰有一整行 =
    // 该 fragment,real 是其子串(`## 6. X`)。header 按**子串**评分(不进 exact)→ 两候选都子串命中
    // → 并列 → None,**不会**因 stale 的 exact 整行被误选。
    let stale = tempfile::tempdir().unwrap();
    let real = tempfile::tempdir().unwrap();
    std::fs::write(stale.path().join("h_moc263.md"), "系统架构建议\nx\n").unwrap();
    std::fs::write(real.path().join("h_moc263.md"), "## 6. 系统架构建议\ny\n").unwrap();
    remember_cwd(real.path().to_str().unwrap());
    remember_cwd(stale.path().to_str().unwrap()); // stale 在队首
    let got = read_patch_file("h_moc263.md", None, &[(true, "系统架构建议")]);
    assert!(
        got.is_none(),
        "header 子串并列应 None,不被 stale 的 exact 整行误选"
    );
}

#[test]
fn context_line_starting_with_at_at_does_not_block_split() {
    // MOC-263 P2 五轮(chatgpt-codex-connector review):context 行内容以 @@ 开头(` @@ ...`,列 0 是
    // 空格)不应被当 hunk 分隔符而禁用自动切分(分割器只认列 0 @@)。含此类 context 行的多区纯删除仍应切。
    let content = "@@ banner\nkeep_a\nREMOVE_1\nmid1\nmid2\nmid3\nREMOVE_2\nkeep_b\n";
    let (dir, name) = tmp_file("atat_moc263.txt", content);
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n @@ banner\n keep_a\n-REMOVE_1\n mid1\n-REMOVE_2\n keep_b\n*** End Patch\n"
    );
    let (out, _reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        out.contains("\n@@\n"),
        "含 ` @@` context 行的多区删除仍应自动切段(列 0 判定):\n{out}"
    );
}

#[test]
fn anchor_probe_includes_unprefixed_lines() {
    // MOC-263 P2 六轮(chatgpt-codex-connector review):无前缀行(fix_unprefixed_lines 要按文件整行
    // 匹配修)也要进 probe,否则它是唯一锚点时空-probe 路径会在同名候选间选错文件。`+` / `***` 不进。
    let body = vec![
        " ctx",
        "-del",
        "+add",
        "@@ hdr",
        "unprefixed line",
        "*** End Patch",
    ];
    let p = anchor_probe(&body);
    assert!(p.contains(&(false, "ctx")), "context 行进 probe");
    assert!(p.contains(&(false, "del")), "删除行进 probe");
    assert!(p.contains(&(true, "hdr")), "@@ 头进 probe(header)");
    assert!(
        p.contains(&(false, "unprefixed line")),
        "无前缀行应作 exact probe"
    );
    assert!(!p.iter().any(|(_, t)| *t == "add"), "+ 新增行不进 probe");
    assert!(
        !p.iter().any(|(_, t)| t.starts_with("***")),
        "*** 控制行不进 probe"
    );
}

#[test]
fn preflight_aligns_via_candidate_cwd_with_none_primary() {
    // MOC-263 P1 端到端:apply_patch 请求 cwd=None,真实 cwd 仅在候选历史里 → 仍能读盘对齐。
    let (dir, name) = tmp_file("p1_e2e_moc263.txt", "alpha\nbeta\ngamma\n");
    let real = dir.path().to_str().unwrap().to_owned();
    remember_cwd(&real);
    // context 带尾随空格(需读盘对齐);primary=None,靠候选历史找到真实文件。
    let v4a =
        format!("*** Begin Patch\n*** Update File: {name}\n beta   \n+inserted\n*** End Patch\n");
    let (out, reps) = preflight_repair(&v4a, None);
    assert!(
        out.contains("\n beta\n"),
        "应经候选 cwd 读盘对齐尾随空格:\n{out}"
    );
    assert!(out.contains("+inserted"), "新增行保留");
    assert_eq!(reps[0].kind, "repaired", "{:?}", reps);
}

#[test]
fn multi_hunk_pure_delete_no_at_auto_split() {
    // MOC-263 P0:多个不连续**纯删除/上下文**区拼一个 Update File 块、无 @@ → 安全自动切段插裸 @@。
    // (带插入 `+` 的多区因落点歧义不在此切,见 mixed_replace_insert_gap_passthrough)
    let content = "keep_top\nREMOVE_1\nmiddle\nmiddle2\nmiddle3\nREMOVE_2\nkeep_bottom\n";
    let (dir, name) = tmp_file("multi_del.txt", content);
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n keep_top\n-REMOVE_1\n middle\n-REMOVE_2\n keep_bottom\n*** End Patch\n"
    );
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert!(out.contains("\n@@\n"), "两个不连续删除区应插裸 @@:\n{out}");
    assert_eq!(reps[0].kind, "repaired", "{:?}", reps);
    assert!(reps[0].detail.contains("自动按文件位置切段"), "{:?}", reps);
    assert!(
        out.contains("-REMOVE_1") && out.contains("-REMOVE_2"),
        "删除行保留:\n{out}"
    );
}

#[test]
fn mixed_replace_insert_gap_passthrough() {
    // MOC-263 P0 安全(chatgpt-codex-connector review 指出):段间「替换 + 额外插入」混合 →
    // `+` 落点歧义(`+return 42` 是替换、`+@memoize` 是给后段 `def beta` 的引入行,无法区分)→
    // 不切、原样透过,避免把 @memoize 静默插到 return 后(错位)。即便前段末锚点是 `-` 删除也不豁免。
    let content = "alpha\nreturn 1\n# gap\ndef beta():\n";
    let (dir, name) = tmp_file("mixed.py", content);
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n-return 1\n+return 42\n+@memoize\n def beta():\n*** End Patch\n"
    );
    let (out, _reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        !out.contains("\n@@\n"),
        "混合 replace+insert 落点歧义不应切:\n{out}"
    );
    assert!(
        out.contains("+@memoize") && out.contains("+return 42"),
        "内容不丢"
    );
}

#[test]
fn single_contiguous_hunk_not_split() {
    // 单段连续 hunk → 不切(group<2 → None),走常规对齐。
    let (dir, name) = tmp_file("single.txt", "a\nb\nc\nd\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Begin Patch\n*** Update File: {name}\n a\n b\n+x\n c\n*** End Patch\n");
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert!(!out.contains("\n@@\n"), "单段连续 hunk 不应插 @@:\n{out}");
    assert!(!reps[0].detail.contains("切段"), "{:?}", reps);
}

#[test]
fn ambiguous_multi_region_passthrough() {
    // 锚点内容在文件里重复(歧义)→ longest_unique_block 返回 None → 不切,透过(不猜不丢)。
    let (dir, name) = tmp_file("amb.txt", "x\ny\nx\ny\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Begin Patch\n*** Update File: {name}\n-x\n+X\n-y\n+Y\n*** End Patch\n");
    let (out, _reps) = preflight_repair(&v4a, Some(cwd));
    assert!(!out.contains("\n@@\n"), "歧义不应切:\n{out}");
}

#[test]
fn greedy_split_bails_when_first_anchor_not_globally_unique() {
    // MOC-263 P1(chatgpt-codex-connector review):**块唯一但段首非唯一**的隐蔽歧义 —— file 有旧
    // ALPHA/BETA/GAMMA 块 + 真实 ALPHA/BETA…gap…GAMMA/DELTA 区。body ` ALPHA/-BETA/ GAMMA/-DELTA`
    // 的 [ALPHA,BETA,GAMMA] 作为连续块只在旧块唯一出现,贪心会选中旧块、从**错块**删 BETA;而段首
    // ALPHA 在文件里出现 2 次 = 起点歧义。修复后段首非全局唯一即 bail,不切分、原样透过(不猜不丢)。
    let content = "ALPHA\nBETA\nGAMMA\nmid_x\nmid_y\nALPHA\nBETA\nsep_gap\nGAMMA\nDELTA\n";
    let (dir, name) = tmp_file("greedy_moc263.txt", content);
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n ALPHA\n-BETA\n GAMMA\n-DELTA\n*** End Patch\n"
    );
    let (out, _reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        !out.contains("\n@@\n"),
        "段首锚点非全局唯一(起点歧义)时应 bail 不切分,避免从错块删行:\n{out}"
    );
}

#[test]
fn floating_add_after_context_passthrough_not_misplaced() {
    // MOC-263 P0 安全防护:`+` 浮动在两不连续区域之间、前段末锚点是 context(非 `-` 删除)→ 落点
    // 歧义(可能属前段尾插、也可能是后段引入行)→ 不切(否则会把 +@memoize 插到错位置、静默错误
    // apply)。这是 pre-push review 抓到的 BLOCKER 回归点。
    let content =
        "def alpha():\n    return 1\n# --- section break ---\ndef beta():\n    return 2\n";
    let (dir, name) = tmp_file("deco.py", content);
    let cwd = dir.path().to_str().unwrap();
    // 模型以为 `return 1` 与 `def beta():` 相邻、在中间加 +@memoize;实际隔着 section break。
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n     return 1\n+@memoize\n def beta():\n*** End Patch\n"
    );
    let (out, _reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        !out.contains("\n@@\n"),
        "浮动 + 落点歧义不应切段(防静默错误 apply):\n{out}"
    );
    assert!(out.contains("+@memoize"), "内容不丢");
}

#[test]
fn blank_line_drift_block_realigned() {
    // EP-1 真机 seq111:模型 context 块漏了文件里的空行 → 整块失配。忽略空行唯一定位 → 重建
    // (补回文件空行 + 对齐字节),`+` 插入保位。
    let (dir, name) = tmp_file(
        "main.py",
        "from a import x\nfrom b import y\n\nfrom c import z\nfrom d import w\n",
    );
    let cwd = dir.path().to_str().unwrap();
    // patch 的 context 漏了 `from b` 与 `from c` 之间的空行,想在 `from d` 后插一行。
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n from a import x\n from b import y\n from c import z\n from d import w\n+from e import v\n*** End Patch\n"
    );
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert!(out.contains("+from e import v"), "插入行保留:\n{out}");
    // 重建后 context 块应含被补回的空行(裸 ' ')。
    assert!(out.contains("\n \n"), "应补回文件空行作 context:\n{out}");
    assert_eq!(reps[0].kind, "repaired", "{:?}", reps);
}

#[test]
fn blank_tolerant_skips_blank_line_deletion() {
    // 含「删除一个空行」的 `-` → blank-tolerant 重建无法忠实表达 → 放行不改(不静默转 context)。
    let (dir, name) = tmp_file("bd.txt", "x\ny\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Update File: {name}\n x\n-\n y\n+z\n");
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert_eq!(out, v4a, "含空白行删除应放行不改:\n{out}");
    assert!(reps[0].kind.starts_with("skipped"), "{:?}", reps);
}

#[test]
fn blank_tolerant_ambiguous_passthrough() {
    // 精确失配(文件 p/q 间有空行,patch 没写)但忽略空行后**多处**匹配 → 歧义放行不猜。
    let (dir, name) = tmp_file("dup.txt", "p\n\nq\nX\np\n\nq\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Update File: {name}\n p\n q\n+r\n");
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert_eq!(out, v4a, "歧义(忽略空行后多处)不改:\n{out}");
    assert!(reps[0].kind.starts_with("skipped"), "{:?}", reps);
}

#[test]
fn optimize_pipeline_fixes_multiple_issues() {
    // 一个 patch 同时:漏信封 + 双边 @@ + 尾随空格上下文 → 全恢复。
    let (dir, name) = tmp_file("multi.txt", "fn main() {\n    let x = 1;\n}\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Update File: {name}\n@@ fn main() {{ @@\n    let x = 1;   \n+    let y = 2;\n"
    );
    let (out, reps) = optimize_patch(&v4a, Some(cwd), true);
    assert!(out.starts_with("*** Begin Patch\n"), "补信封:\n{out}");
    assert!(out.trim_end().ends_with("*** End Patch"), "补 End:\n{out}");
    assert!(out.contains("@@ fn main() {\n"), "双边 @@ 转单边:\n{out}");
    assert!(out.contains("    let x = 1;\n"), "尾随空格对齐:\n{out}");
    assert!(out.contains("+    let y = 2;"), "新增行保留");
    // 至少 3 类修复都记录
    let kinds: Vec<&str> = reps.iter().map(|r| r.kind.as_str()).collect();
    assert!(
        kinds.iter().filter(|k| **k == "repaired").count() >= 2,
        "{:?}",
        reps
    );
}

#[test]
fn add_file_untouched_no_cwd_noop() {
    let v4a = "*** Begin Patch\n*** Add File: new.txt\n+hello\n*** End Patch\n";
    // 无 Update File → 短路原样返回(即便给 cwd)。
    let (out, reps) = preflight_repair(v4a, Some("/tmp"));
    assert_eq!(out, v4a);
    assert!(reps.is_empty());
    // 无 cwd → 原样
    let (out2, reps2) = preflight_repair(v4a, None);
    assert_eq!(out2, v4a);
    assert!(reps2.is_empty());
}
