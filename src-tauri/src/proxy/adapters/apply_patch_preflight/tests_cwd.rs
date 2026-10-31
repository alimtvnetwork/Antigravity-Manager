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

    // codex-connector #435 P2: Windows path backslashes cannot be doubled (traverse Value to get the unescaped original text,
    // The entire request cannot be serialized first). json! in "C:\\Users\\me\\repo" = actual single backslash path.
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
    // File context lines have no trailing spaces; patch context lines with trailing spaces → should be aligned to actual file bytes.
    let (dir, name) = tmp_file("a.txt", "fn main() {\n    let x = 1;\n    let y = 2;\n}\n");
    let cwd = dir.path().to_str().unwrap();
    // patch: add a line after `let x = 1;`; context with trailing spaces (common model error).
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n    let x = 1;   \n+    let z = 9;\n    let y = 2;\n*** End Patch\n"
    );
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        out.contains("    let x = 1;\n"),
        "Trailing spaces should be aligned:\n {out}"
    );
    assert!(out.contains("+    let z = 9;"), "Keep new rows");
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
    assert_eq!(out, v4a, "Exact match without changing one byte");
}

#[test]
fn ambiguous_match_is_skipped_not_guessed() {
    // The anchor point `x` appears in many places in the file → ambiguity → let it go without guessing.
    let (dir, name) = tmp_file("c.txt", "x\nx\nx\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Begin Patch\n*** Update File: {name}\n x   \n+added\n*** End Patch\n");
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert!(reps[0].kind.starts_with("skipped"), "{:?}", reps);
    assert_eq!(out, v4a, "Ambiguity remains unchanged");
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
    // Real machine seq230 form: only Add File + content, no Begin/End.
    let body = "*** Add File: outputs/x.md\n+# Title\n+body\n";
    let (out, rep) = ensure_v4a_envelope(body);
    assert!(out.starts_with("*** Begin Patch\n"), "{out}");
    assert!(out.trim_end().ends_with("*** End Patch"), "{out}");
    assert!(
        out.contains("+# Title") && out.contains("+body"),
        "No content is lost"
    );
    assert!(rep.is_some());
}

#[test]
fn envelope_only_end_added() {
    let body = "*** Begin Patch\n*** Add File: x\n+a\n";
    let (out, rep) = ensure_v4a_envelope(body);
    assert_eq!(
        out.matches("*** Begin Patch").count(),
        1,
        "Do not add Begin repeatedly"
    );
    assert!(out.trim_end().ends_with("*** End Patch"));
    assert!(rep.unwrap().detail.contains("End Patch"));
}

#[test]
fn envelope_prefixed_end_stripped_for_code_file() {
    // MOC-268 (User Agreed "Strip by File Type"): **Only `+*** End Patch`** (Add line error prefix) in code/structured configuration files
    // Here (the bare `*** End Patch` cannot be the last line of the legal source code), the prefix is stripped and transformed into a bare terminator, with no appending, zero residue, and zero content loss.
    for path in ["x.rs", "c.json", "s.toml", "w.vue"] {
        let body = format!("*** Begin Patch\n*** Add File: {path}\n+a\n+b\n+*** End Patch");
        let (out, rep) = ensure_v4a_envelope(&body);
        assert!(
            out.trim_end().ends_with("\n*** End Patch"),
            "Code files should be stripped to bare terminators ( {path} ):\n {out}"
        );
        assert!(
            !out.contains("+*** End Patch"),
            "Prefixed terminators should not remain ( {path} ):\n {out}"
        );
        assert_eq!(
            out.matches("*** End Patch").count(),
            1,
            "Only one terminator ( {path} ):\n {out}"
        );
        assert!(
            rep.unwrap()
                .detail
                .contains("Stripping and adding prefix by mistake"),
            "{path}"
        );
    }
}

#[test]
fn envelope_deletion_or_context_end_line_not_stripped() {
    // MOC-268(chatgpt-codex-connector review):` *** End Patch`(context)/ `-*** End Patch`(deletion)
    // It is a ** legal Update hunk line ** (for example, the model uses Update to delete the *** End Patch remaining in the file), and ** must not be used as a terminator strip**
    // (stripping `-` = silently discard removal). The last two lines → the normal completion terminator, the hunk line are **left as is**. Even if the target is a code file.
    for last in ["-*** End Patch", " *** End Patch"] {
        let body = format!("*** Begin Patch\n*** Update File: src/foo.rs\n keep\n{last}");
        let (out, rep) = ensure_v4a_envelope(&body);
        assert!(
            out.contains(last),
            "The legal hunk line {last:?} must be retained as is (no stripping = no deletion without loss):\n {out}"
        );
        assert!(
            out.trim_end().ends_with("\n*** End Patch"),
            "The terminator ( {last:?} ) should be complemented normally:\n {out}"
        );
        let r = rep.unwrap();
        assert!(
            r.detail.contains("End Patch")
                && !r.detail.contains("Stripping and adding prefix by mistake"),
            "Normal append, non-stripping ( {last:?} ): {}",
            r.detail
        );
    }
}

#[test]
fn envelope_prefixed_end_left_incomplete_for_doc_file() {
    // MOC-268 (silent-failure review): Document/Text/Unknown Type `*** End Patch` **may be the last line of the text** (this warehouse
    // The V4A document has this string of words) → Don’t guess the ambiguity: neither stripping (avoiding deletion of the text) nor appending (avoiding residue) → leaving incomplete without completion.
    for path in ["notes.md", "readme.txt", "data"] {
        let body =
            format!("*** Begin Patch\n*** Add File: {path}\n+How to end a patch:\n+*** End Patch");
        let (out, rep) = ensure_v4a_envelope(&body);
        assert_eq!(
            out, body,
            "Document file ambiguity last line should be left intact ( {path} ):\n {out}"
        );
        assert!(
            out.trim_end().ends_with("+*** End Patch"),
            "The text line should be left undeleted ( {path} ):\n {out}"
        );
        assert!(
            !out.trim_end().ends_with("\n*** End Patch"),
            "The naked terminator ( {path} ) should not be appended:\n {out}"
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
    // MOC-268 (chatgpt-codex-connector review): `*** Update File:` of `+*** End Patch` is **new line**
    // (Maybe add this string of words to the string/fixture of the code file), it is not the wrong prefix terminator of Add File → **Even if the target is code
    // The file is not stripped** (stripped = lost and added), leaving ambiguity → leaving incomplete. The last operation to strip is Add File.
    let body = "*** Begin Patch\n*** Update File: src/foo.rs\n keep\n+*** End Patch".to_string();
    let (out, rep) = ensure_v4a_envelope(&body);
    assert_eq!(
        out, body,
        "Update's +*** End Patch should not be stripped/touched:\n {out}"
    );
    assert!(
        out.trim_end().ends_with("+*** End Patch"),
        "The new row should be retained:\n {out}"
    );
    assert!(
        !out.trim_end().ends_with("\n*** End Patch"),
        "No naked terminator should be appended:\n {out}"
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
    // Non-patch bodies are not touched
    let (o1, r1) = ensure_v4a_envelope("just some text\nno ops here\n");
    assert_eq!(o1, "just some text\nno ops here\n");
    assert!(r1.is_none());
    // Begin is missing and the first non-blank line is not an operation line (with leading prose) → Unsafe, Begin is not added
    let prose = "here is my patch:\n*** Add File: x\n+a\n*** End Patch\n";
    let (o2, _r2) = ensure_v4a_envelope(prose);
    assert!(
        !o2.starts_with("*** Begin Patch"),
        "Prose that has a lead should not be filled in hastily. Begin"
    );
}

#[test]
fn strip_trailing_at_double_sided_to_single() {
    let v4a = "*** Begin Patch\n*** Update File: x\n@@ def f(): @@\n-a\n+b\n*** End Patch\n";
    let (out, reps) = strip_trailing_at(v4a);
    assert!(
        out.contains("@@ def f():\n"),
        "The tail should be removed @@:\n {out}"
    );
    assert!(!out.contains("@@ def f(): @@"));
    assert_eq!(reps.len(), 1);
}

#[test]
fn strip_trailing_at_keeps_bare_and_single() {
    // Naked @@ (section separation) + single side @@ does not move
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
        "Copy original content:\n {out}"
    );
    assert!(!out.contains("*** Move to:"), "Move has been replaced");
    assert_eq!(reps[0].kind, "repaired");
}

#[test]
fn recover_empty_move_with_hunk_untouched() {
    // Update+Move but **has** hunk (rename + content change) → no touch (prompt allows).
    let (dir, name) = tmp_file("old2.md", "a\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n*** Move to: new2.md\n-a\n+b\n*** End Patch\n"
    );
    let (out, reps) = recover_empty_move(&v4a, Some(cwd));
    assert_eq!(out, v4a, "Move with hunk does not move");
    assert!(reps.is_empty());
}

#[test]
fn rename_with_eof_marker_hunk_not_treated_as_empty() {
    // codex-connector #435 P1:rename + `*** End of File` append hunk cannot be empty rename (otherwise converted to
    // Delete+Add) of lost content → Identified as a hunk → Not transferred through.
    let (dir, name) = tmp_file("eof_old.md", "a\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n*** Move to: eof_new.md\n*** End of File\n+tail\n*** End Patch\n"
    );
    let (out, reps) = recover_empty_move(&v4a, Some(cwd));
    assert_eq!(
        out, v4a,
        "Rename containing EOF hunk should be passed through:\n {out}"
    );
    assert!(reps.is_empty(), "{:?}", reps);
}

#[test]
fn add_on_existing_passes_through_unchanged() {
    // Rule #2 has been withdrawn: Add existing files are **no longer** converted to Delete+Add (to avoid overwriting and losing data), and use Codex as is
    // Report already exists, and the model self-corrects to a targeted Update.
    let (dir, name) = tmp_file("exists.md", "important old content\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Begin Patch\n*** Add File: {name}\n+new content\n*** End Patch\n");
    let (out, reps) = optimize_patch(&v4a, Some(cwd), true);
    assert!(
        !out.contains("*** Delete File:"),
        "Delete should no longer be inserted (removed rule #2):\n {out}"
    );
    assert!(
        out.contains(&format!("*** Add File: {name}")),
        "Add Leave as is"
    );
    assert!(
        !reps
            .iter()
            .any(|r| r.detail.contains("Delete File overwrite")),
        "There should be no coverage fixes: {:?}",
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
    assert!(!out.contains("*** Update File:"), "Update converted");
    assert_eq!(reps[0].kind, "repaired");
}

#[test]
fn update_whitespace_only_file_not_converted() {
    // codex-connector #435 P2: Purely blank files (non-0 bytes) are not considered empty → do not convert Delete+Add (otherwise blank bytes will be lost).
    let (dir, name) = tmp_file("ws.txt", "  \n\t\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Begin Patch\n*** Update File: {name}\n+line1\n*** End Patch\n");
    let (out, reps) = recover_update_empty_file(&v4a, Some(cwd));
    assert_eq!(
        out, v4a,
        "Purely blank files Update should not be converted to Delete+Add:\n {out}"
    );
    assert!(reps.is_empty());
}
