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
fn extract_cwd_from_env_block() {
    let req = json!({
        "input": [{"type":"message","role":"user","content":"<environment_context>\n  <cwd>/Users/x/proj</cwd>\n  <shell>zsh</shell>\n</environment_context>"}]
    });
    assert_eq!(extract_cwd(Some(&req)).as_deref(), Some("/Users/x/proj"));
    assert_eq!(extract_cwd(None), None);
    assert_eq!(extract_cwd(Some(&json!({"input":[]}))), None);

    // codex-connector #435 P2:Windows 路径反斜杠不能被翻倍(遍历 Value 取反转义原文,
    // 不能先序列化整个请求)。json! 里 "C:\\Users\\me\\repo" = 实际单反斜杠路径。
    let win = json!({
        "input": [{"type":"message","role":"user","content":"<environment_context>\n  <cwd>C:\\Users\\me\\repo</cwd>\n</environment_context>"}]
    });
    assert_eq!(
        extract_cwd(Some(&win)).as_deref(),
        Some(r"C:\Users\me\repo")
    );
}

#[test]
fn trailing_whitespace_anchor_is_repaired_to_file_bytes() {
    // 文件 context 行无尾随空格;patch 的 context 行带尾随空格 → 应被对齐成文件真实字节。
    let (dir, name) = tmp_file("a.txt", "fn main() {\n    let x = 1;\n    let y = 2;\n}\n");
    let cwd = dir.path().to_str().unwrap();
    // patch: 在 `let x = 1;` 后加一行;context 带尾随空格(模型常见错)。
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n    let x = 1;   \n+    let z = 9;\n    let y = 2;\n*** End Patch\n"
    );
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        out.contains("    let x = 1;\n"),
        "尾随空格应被对齐掉:\n{out}"
    );
    assert!(out.contains("+    let z = 9;"), "新增行保留");
    assert_eq!(reps[0].kind, "repaired", "{:?}", reps);
}

#[test]
fn exact_match_left_clean() {
    let (dir, name) = tmp_file("b.txt", "alpha\nbeta\ngamma\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n alpha\n-beta\n+BETA\n gamma\n*** End Patch\n"
    );
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert_eq!(reps[0].kind, "clean");
    assert_eq!(out, v4a, "精确匹配不改一字节");
}

#[test]
fn ambiguous_match_is_skipped_not_guessed() {
    // 锚点 ` x` 在文件里多处出现 → 歧义 → 放行不猜。
    let (dir, name) = tmp_file("c.txt", "x\nx\nx\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Begin Patch\n*** Update File: {name}\n x   \n+added\n*** End Patch\n");
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert!(reps[0].kind.starts_with("skipped"), "{:?}", reps);
    assert_eq!(out, v4a, "歧义不改");
}

#[test]
fn no_match_skipped() {
    let (dir, name) = tmp_file("d.txt", "real content\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n model hallucinated line\n+x\n*** End Patch\n"
    );
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert!(reps[0].kind.starts_with("skipped"));
    assert_eq!(out, v4a);
}

#[test]
fn unreadable_file_passes_through() {
    let v4a = "*** Begin Patch\n*** Update File: nonexistent_zzz.txt\n a\n+b\n*** End Patch\n";
    let (out, reps) = preflight_repair(v4a, Some("/tmp/no_such_dir_xyz"));
    assert_eq!(out, v4a);
    assert_eq!(reps[0].kind, "skipped:unreadable");
}

#[test]
fn envelope_added_when_model_omits_begin_end() {
    // 真机 seq230 形态:只有 Add File + 内容,无 Begin/End。
    let body = "*** Add File: outputs/x.md\n+# Title\n+body\n";
    let (out, rep) = ensure_v4a_envelope(body);
    assert!(out.starts_with("*** Begin Patch\n"), "{out}");
    assert!(out.trim_end().ends_with("*** End Patch"), "{out}");
    assert!(
        out.contains("+# Title") && out.contains("+body"),
        "内容不丢"
    );
    assert!(rep.is_some());
}

#[test]
fn envelope_only_end_added() {
    let body = "*** Begin Patch\n*** Add File: x\n+a\n";
    let (out, rep) = ensure_v4a_envelope(body);
    assert_eq!(out.matches("*** Begin Patch").count(), 1, "不重复加 Begin");
    assert!(out.trim_end().ends_with("*** End Patch"));
    assert!(rep.unwrap().detail.contains("End Patch"));
}

#[test]
fn envelope_prefixed_end_stripped_for_code_file() {
    // MOC-268(用户拍板「按文件类型剥」):**仅 `+*** End Patch`**(Add 行误前缀)在代码/结构化配置文件
    // 里(裸 `*** End Patch` 不可能是合法源码末行)剥前缀规整成裸终止符,不追加、零残留、零内容丢失。
    for path in ["x.rs", "c.json", "s.toml", "w.vue"] {
        let body = format!("*** Begin Patch\n*** Add File: {path}\n+a\n+b\n+*** End Patch");
        let (out, rep) = ensure_v4a_envelope(&body);
        assert!(
            out.trim_end().ends_with("\n*** End Patch"),
            "代码文件应剥成裸终止符 ({path}):\n{out}"
        );
        assert!(
            !out.contains("+*** End Patch"),
            "不应残留带前缀终止符 ({path}):\n{out}"
        );
        assert_eq!(
            out.matches("*** End Patch").count(),
            1,
            "只一个终止符 ({path}):\n{out}"
        );
        assert!(rep.unwrap().detail.contains("剥误加前缀"), "{path}");
    }
}

#[test]
fn envelope_deletion_or_context_end_line_not_stripped() {
    // MOC-268(chatgpt-codex-connector review):` *** End Patch`(context)/ `-*** End Patch`(deletion)
    // 是**合法 Update hunk 行**(如模型用 Update 删文件里残留的 *** End Patch),**绝不能当终止符剥**
    // (剥 `-` = 静默丢弃删除)。这俩末行 → 正常补真终止符、hunk 行**原样保留**。即便目标是代码文件。
    for last in ["-*** End Patch", " *** End Patch"] {
        let body = format!("*** Begin Patch\n*** Update File: src/foo.rs\n keep\n{last}");
        let (out, rep) = ensure_v4a_envelope(&body);
        assert!(
            out.contains(last),
            "合法 hunk 行 {last:?} 必须原样保留(不剥=不丢删除):\n{out}"
        );
        assert!(
            out.trim_end().ends_with("\n*** End Patch"),
            "应正常补真终止符 ({last:?}):\n{out}"
        );
        let r = rep.unwrap();
        assert!(
            r.detail.contains("End Patch") && !r.detail.contains("剥误加前缀"),
            "走正常 append、非剥 ({last:?}):{}",
            r.detail
        );
    }
}

#[test]
fn envelope_prefixed_end_left_incomplete_for_doc_file() {
    // MOC-268(silent-failure review):文档/文本/未知类型里裸 `*** End Patch` **可能是正文末行**(本仓
    // V4A 文档就有这串字)→ 歧义不猜:既不剥(免删正文)也不追加(免残留)→ 不补全留 incomplete。
    for path in ["notes.md", "readme.txt", "data"] {
        let body =
            format!("*** Begin Patch\n*** Add File: {path}\n+How to end a patch:\n+*** End Patch");
        let (out, rep) = ensure_v4a_envelope(&body);
        assert_eq!(out, body, "文档文件歧义末行应原样保留 ({path}):\n{out}");
        assert!(
            out.trim_end().ends_with("+*** End Patch"),
            "正文行应保留不删 ({path}):\n{out}"
        );
        assert!(
            !out.trim_end().ends_with("\n*** End Patch"),
            "不应追加裸终止符 ({path}):\n{out}"
        );
        assert_eq!(
            rep.unwrap().kind,
            "skipped:ambiguous_prefixed_end",
            "{path}"
        );
    }
}

#[test]
fn envelope_prefixed_end_not_stripped_for_update_even_code() {
    // MOC-268(chatgpt-codex-connector review):`*** Update File:` 的 `+*** End Patch` 是**新增行**
    // (可能往代码文件的字符串/fixture 里加这串字),不是 Add File 的误前缀终止符 → **即便目标是代码
    // 文件也不剥**(剥了=丢新增),走歧义 → 留 incomplete。剥仅限末操作是 Add File。
    let body = "*** Begin Patch\n*** Update File: src/foo.rs\n keep\n+*** End Patch".to_string();
    let (out, rep) = ensure_v4a_envelope(&body);
    assert_eq!(out, body, "Update 的 +*** End Patch 不应被剥/动:\n{out}");
    assert!(
        out.trim_end().ends_with("+*** End Patch"),
        "新增行应保留:\n{out}"
    );
    assert!(
        !out.trim_end().ends_with("\n*** End Patch"),
        "不应追加裸终止符:\n{out}"
    );
    assert_eq!(rep.unwrap().kind, "skipped:ambiguous_prefixed_end");
}

#[test]
fn envelope_complete_untouched() {
    let body = "*** Begin Patch\n*** Add File: x\n+a\n*** End Patch\n";
    let (out, rep) = ensure_v4a_envelope(body);
    assert_eq!(out, body);
    assert!(rep.is_none());
}

#[test]
fn envelope_not_added_to_nonpatch_or_leading_prose() {
    // 非 patch 体不碰
    let (o1, r1) = ensure_v4a_envelope("just some text\nno ops here\n");
    assert_eq!(o1, "just some text\nno ops here\n");
    assert!(r1.is_none());
    // 缺 Begin 且首个非空行不是操作行(有前导散文)→ 不安全,不补 Begin
    let prose = "here is my patch:\n*** Add File: x\n+a\n*** End Patch\n";
    let (o2, _r2) = ensure_v4a_envelope(prose);
    assert!(
        !o2.starts_with("*** Begin Patch"),
        "有前导散文不应贸然补 Begin"
    );
}

#[test]
fn strip_trailing_at_double_sided_to_single() {
    let v4a = "*** Begin Patch\n*** Update File: x\n@@ def f(): @@\n-a\n+b\n*** End Patch\n";
    let (out, reps) = strip_trailing_at(v4a);
    assert!(out.contains("@@ def f():\n"), "应去尾部 @@:\n{out}");
    assert!(!out.contains("@@ def f(): @@"));
    assert_eq!(reps.len(), 1);
}

#[test]
fn strip_trailing_at_keeps_bare_and_single() {
    // 裸 @@(section 分隔)+ 单边 @@ 都不动
    let v4a = "*** Update File: x\n@@\n@@ class Foo\n-a\n+b\n";
    let (out, reps) = strip_trailing_at(v4a);
    assert_eq!(out, v4a);
    assert!(reps.is_empty());
}

#[test]
fn strip_unified_hunk_ranges_to_bare_at() {
    let v4a = "*** Begin Patch\n*** Update File: x\n@@ -6,2 +6,4 @@\n-a\n+b\n*** End Patch\n";
    let (out, reps) = strip_unified_hunk_ranges(v4a);
    assert!(out.contains("*** Update File: x\n@@\n-a\n+b"));
    assert!(!out.contains("-6,2 +6,4"));
    assert_eq!(reps.len(), 1);
}

#[test]
fn strip_unified_hunk_ranges_keeps_text_anchor() {
    let v4a = "*** Update File: x\n@@ class Foo\n-a\n+b\n";
    let (out, reps) = strip_unified_hunk_ranges(v4a);
    assert_eq!(out, v4a);
    assert!(reps.is_empty());
}

#[test]
fn optimize_patch_converts_unified_diff_headers_to_v4a_update() {
    let v4a = "*** Begin Patch\n--- C:/Users/32057/Documents/Codex/2026-07-05/zai/data_summary.md\n+++ C:/Users/32057/Documents/Codex/2026-07-05/zai/data_summary.md\n@@ -7,4 +7,5 @@\n old\n+new\n*** End Patch\n";
    let (out, reps) = optimize_patch(v4a, None, true);
    assert!(
        out.contains(
            "*** Update File: C:/Users/32057/Documents/Codex/2026-07-05/zai/data_summary.md"
        ),
        "{out}"
    );
    assert!(out.contains("@@\n old\n+new"), "{out}");
    assert!(validate_v4a_for_codex(&out).is_none(), "{out}");
    assert!(
        reps.iter()
            .any(|r| r.detail.contains("unified diff file headers")),
        "{reps:?}"
    );
}

#[test]
fn optimize_patch_converts_file_header_to_v4a_update() {
    let v4a = "*** Begin Patch\nfile: C:\\Users\\32057\\Documents\\Codex\\2026-07-05\\zai\\data_summary.md\n@@\n-old\n+new\n*** End Patch\n";
    let (out, reps) = optimize_patch(v4a, None, true);
    assert!(
        out.contains(
            "*** Update File: C:\\Users\\32057\\Documents\\Codex\\2026-07-05\\zai\\data_summary.md"
        ),
        "{out}"
    );
    assert!(validate_v4a_for_codex(&out).is_none(), "{out}");
    assert!(reps.iter().any(|r| r.detail.contains("file: header")));
}

#[test]
fn validate_v4a_rejects_unified_headers() {
    let v4a =
        "*** Begin Patch\n*** Update File: x\n--- a/x\n+++ b/x\n@@ -1 +1\n-a\n+b\n*** End Patch\n";
    let err = validate_v4a_for_codex(v4a).expect("unified diff should be rejected");
    assert!(err.1.contains("unified diff file header"));
}

#[test]
fn validate_v4a_rejects_double_envelope() {
    let v4a = "*** Begin Patch\n*** Update File: x\n@@\n-a\n+b\n*** End Patch\n*** Begin Patch\n*** Update File: y\n@@\n-c\n+d\n*** End Patch\n";
    let err = validate_v4a_for_codex(v4a).expect("double envelope should be rejected");
    assert!(
        err.1.contains("repeated *** Begin Patch") || err.1.contains("repeated *** End Patch"),
        "{err:?}"
    );
}

#[test]
fn validate_v4a_rejects_missing_line_prefix() {
    let v4a =
        "*** Begin Patch\n*** Update File: x\n@@\nold line without prefix\n+new\n*** End Patch\n";
    let err = validate_v4a_for_codex(v4a).expect("missing prefix should be rejected");
    assert!(err.1.contains("line missing V4A prefix"), "{err:?}");
}

#[test]
fn optimize_patch_strips_unified_range_header() {
    let v4a = "*** Begin Patch\n*** Update File: x\n@@ -6 +6\n-a\n+b\n*** End Patch\n";
    let (out, reps) = optimize_patch(v4a, None, true);
    assert!(out.contains("*** Update File: x\n@@\n-a\n+b"));
    assert!(validate_v4a_for_codex(&out).is_none(), "{out}");
    assert!(reps.iter().any(|r| r.file == "(@@ range header)"));
}

#[test]
fn recover_empty_move_to_delete_add() {
    let (dir, name) = tmp_file("old.md", "line1\nline2\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a =
        format!("*** Begin Patch\n*** Update File: {name}\n*** Move to: new.md\n*** End Patch\n");
    let (out, reps) = recover_empty_move(&v4a, Some(cwd));
    assert!(out.contains(&format!("*** Delete File: {name}")), "{out}");
    assert!(out.contains("*** Add File: new.md"), "{out}");
    assert!(
        out.contains("+line1") && out.contains("+line2"),
        "复制原内容:\n{out}"
    );
    assert!(!out.contains("*** Move to:"), "Move 已被替换");
    assert_eq!(reps[0].kind, "repaired");
}

#[test]
fn recover_empty_move_with_hunk_untouched() {
    // Update+Move 但**有** hunk(rename + 内容改)→ 不碰(prompt 允许)。
    let (dir, name) = tmp_file("old2.md", "a\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n*** Move to: new2.md\n-a\n+b\n*** End Patch\n"
    );
    let (out, reps) = recover_empty_move(&v4a, Some(cwd));
    assert_eq!(out, v4a, "有 hunk 的 Move 不动");
    assert!(reps.is_empty());
}

#[test]
fn rename_with_eof_marker_hunk_not_treated_as_empty() {
    // codex-connector #435 P1:rename + `*** End of File` 追加 hunk 不能被当空 rename(否则转成
    // 丢内容的 Delete+Add)→ 识别为有 hunk → 透过不转。
    let (dir, name) = tmp_file("eof_old.md", "a\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n*** Move to: eof_new.md\n*** End of File\n+tail\n*** End Patch\n"
    );
    let (out, reps) = recover_empty_move(&v4a, Some(cwd));
    assert_eq!(out, v4a, "含 EOF hunk 的 rename 应透过不转:\n{out}");
    assert!(reps.is_empty(), "{:?}", reps);
}

#[test]
fn add_on_existing_passes_through_unchanged() {
    // 规则 #2 已撤:Add 已存在文件**不再**转 Delete+Add(避免覆盖丢数据),原样透过让 Codex
    // 报 already exists、模型自纠为针对性 Update。
    let (dir, name) = tmp_file("exists.md", "important old content\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Begin Patch\n*** Add File: {name}\n+new content\n*** End Patch\n");
    let (out, reps) = optimize_patch(&v4a, Some(cwd), true);
    assert!(
        !out.contains("*** Delete File:"),
        "不应再插 Delete(已撤规则#2):\n{out}"
    );
    assert!(
        out.contains(&format!("*** Add File: {name}")),
        "Add 原样保留"
    );
    assert!(
        !reps.iter().any(|r| r.detail.contains("Delete File 覆盖")),
        "不应有覆盖类修复: {:?}",
        reps
    );
}

#[test]
fn update_empty_file_to_delete_add() {
    let (dir, name) = tmp_file("empty.txt", "");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Begin Patch\n*** Update File: {name}\n+line1\n+line2\n*** End Patch\n");
    let (out, reps) = recover_update_empty_file(&v4a, Some(cwd));
    assert!(out.contains(&format!("*** Delete File: {name}")), "{out}");
    assert!(out.contains(&format!("*** Add File: {name}")), "{out}");
    assert!(out.contains("+line1") && out.contains("+line2"));
    assert!(!out.contains("*** Update File:"), "Update 已转换");
    assert_eq!(reps[0].kind, "repaired");
}

#[test]
fn update_whitespace_only_file_not_converted() {
    // codex-connector #435 P2:纯空白文件(非 0 字节)不算空 → 不转 Delete+Add(否则丢空白字节)。
    let (dir, name) = tmp_file("ws.txt", "  \n\t\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Begin Patch\n*** Update File: {name}\n+line1\n*** End Patch\n");
    let (out, reps) = recover_update_empty_file(&v4a, Some(cwd));
    assert_eq!(out, v4a, "纯空白文件 Update 不应转 Delete+Add:\n{out}");
    assert!(reps.is_empty());
}
