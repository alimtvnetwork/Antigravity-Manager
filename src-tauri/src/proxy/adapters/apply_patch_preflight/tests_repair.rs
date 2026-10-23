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
    assert_eq!(out, v4a, "Non-empty files are not touched by Update");
    assert!(reps.is_empty());
}

#[test]
fn add_file_missing_plus_prefix_is_filled() {
    // In Add File, there are missing lines like `+` (common in models), empty lines → full complement of `+`, and those that already have `+` are not affected.
    let v4a = "*** Begin Patch\n*** Add File: new.md\n+# Title\nplain line no plus\n\n+already plus\n*** End Patch\n";
    let (out, reps) = ensure_add_file_plus(v4a);
    assert!(
        out.contains("\n+plain line no plus\n"),
        "Lines with missing + should be filled in:\n {out}"
    );
    assert!(
        out.contains("\n+\n+already plus"),
        "Empty line → bare +:\n {out}"
    );
    assert!(
        !out.contains("++already plus"),
        "It is already + and does not repeat"
    );
    assert_eq!(reps[0].kind, "repaired");
    assert!(
        reps[0].detail.contains("2 lines"),
        "Leaking + plain rows + empty rows = 2: {:?}",
        reps
    );
}

#[test]
fn add_file_all_plus_untouched_and_update_not_affected() {
    // The Add File with full + will not move; the non-+ lines (context/-) of the Update section will never be touched by G.
    let v4a = "*** Begin Patch\n*** Add File: a\n+x\n+y\n*** Update File: b\n cont\n-old\n+new\n*** End Patch\n";
    let (out, reps) = ensure_add_file_plus(v4a);
    assert_eq!(out, v4a, "Add Full + + Update Not moving:\n {out}");
    assert!(reps.is_empty());
}

#[test]
fn at_header_aligned_to_unique_file_line() {
    // Real machine seq181:@@ The header is incomplete (missing `## 6. `), the only one contained in one file line → alignment.
    let (dir, name) = tmp_file(
        "doc.md",
        "intro\n## 6. System architecture recommendations\nLayering is recommended\n",
    );
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Begin Patch\n*** Update File:");
    let (out, reps) = align_at_headers(&v4a, Some(cwd));
    assert!(
        out.contains("@@ ## 6. System architecture suggestions"),
        "@@ Should be the entire line of the complete file:\n {out}"
    );
    assert_eq!(reps[0].kind, "repaired");
}

#[test]
fn at_header_exact_or_ambiguous_untouched() {
    // Already the entire line of the file → does not move; multiple inclusions (ambiguity) → does not move.
    let (dir, name) = tmp_file("doc2.md", "## A\nx\n## A\n");
    let cwd = dir.path().to_str().unwrap();
    // Exact whole line `## A` exists, but ambiguity (two lines) → does not move
    let v4a = format!("*** Update File: {name}\n@@ ## A\n x\n+y\n");
    let (out, reps) = align_at_headers(&v4a, Some(cwd));
    assert_eq!(out, v4a);
    assert!(reps.is_empty());
    // Substring `A` appears in two lines `## A` → no ambiguity
    let v4a2 = format!("*** Update File: {name}\n@@ A\n x\n+y\n");
    let (out2, reps2) = align_at_headers(&v4a2, Some(cwd));
    assert_eq!(out2, v4a2);
    assert!(reps2.is_empty());
}

#[test]
fn unprefixed_dup_of_plus_line_dropped() {
    // Real machine seq235: No prefix line + followed by `+<same content>` → delete the discarded line (the content is in the + line and will not be lost).
    let (dir, name) = tmp_file("u.md", "other\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Update File: {name}\n*data source*\n+*data source*\n+more\n");
    let (out, reps) = fix_unprefixed_lines(&v4a, Some(cwd));
    assert!(
        !out.contains("\n*data source*\n"),
        "Unused lines without prefix should be deleted:\n {out}"
    );
    assert!(
        out.contains("+*data source*"),
        "+ Row preservation (content is not lost)"
    );
    assert_eq!(reps[0].kind, "repaired");
}

#[test]
fn unprefixed_existing_file_line_gets_context_space() {
    // Lines without prefixes have the same counterpart in the file → context leaks spaces → adds ` `.
    let (dir, name) = tmp_file("u2.md", "alpha\nkeepme\nbeta\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Update File: {name}\nkeepme\n+added\n");
    let (out, reps) = fix_unprefixed_lines(&v4a, Some(cwd));
    assert!(
        out.contains("\n keepme\n"),
        "The spaces should be filled in as context:\n {out}"
    );
    assert_eq!(reps[0].kind, "repaired");
}

#[test]
fn unprefixed_unknown_passes_through() {
    // Not in file, non-duplicate → through (no guessing).
    let (dir, name) = tmp_file("u3.md", "real\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Update File: {name}\nhallucinated garbage line\n+x\n");
    let (out, reps) = fix_unprefixed_lines(&v4a, Some(cwd));
    assert_eq!(
        out, v4a,
        "Unknown unprefixed lines are passed through as-is"
    );
    assert!(reps.is_empty());
}

#[test]
fn cwd_candidates_remember_recall_and_resolve() {
    // MOC-263 P1: Candidate history (deque) + read_patch_file parsed by candidate (global state, only non-flaky assertions are made).
    let (dir, name) = tmp_file("cand_moc263.txt", "x\n");
    let real = dir.path().to_str().unwrap().to_owned();
    // Simulate concurrency pollution: first record a stale cwd that does not contain the file, and then record the real cwd (top).
    remember_cwd("/tmp/stale_zzz_moc263_a");
    remember_cwd(&real);
    assert!(recall_cwd_candidates().iter().any(|c| c == &real));
    assert!(has_cwd_candidate(None), "has candidate history → true");
    // primary=None (apply_patch tool loop request), the real cwd is in the candidate → read the file
    // (stale cwd does not have this file and will be automatically skipped when trying one by one). This is the core of P1: no longer ruined by stale single slots.
    let got = read_patch_file(&name, None, &[(false, "x")]);
    assert!(got.is_some(), "The file should be read via candidate cwd");
    assert_eq!(got.unwrap().1, "x\n");
    // turn-start requests to extract cwd into candidates (for subsequent apply_patch rollback).
    let req = json!({"input":[{"type":"message","role":"user","content":"<environment_context>\n  <cwd>/tmp/ts_proj_b3f9</cwd>\n</environment_context>"}]});
    remember_cwd_from_request(Some(&req));
    assert!(recall_cwd_candidates()
        .iter()
        .any(|c| c == "/tmp/ts_proj_b3f9"));
}

#[test]
fn read_patch_file_picks_by_probe_not_first_readable() {
    // MOC-263 P2 (chatgpt-codex-connector review): Concurrent sessions share relative paths (README.md, etc.), and stale
    // When the session is updated, press the patch anchor point probe to select the candidate that actually contains the anchor point, rather than the one readable by the first (most-recent=stale).
    let stale = tempfile::tempdir().unwrap();
    let real = tempfile::tempdir().unwrap();
    std::fs::write(stale.path().join("shared_moc263.txt"), "stale_only_line\n").unwrap();
    std::fs::write(
        real.path().join("shared_moc263.txt"),
        "real_anchor_line\nmore\n",
    )
    .unwrap();
    // Real cwd is written first, stale is written later → stale is at the head of the candidate team (most-recent), simulating the scenario that the review is worried about.
    remember_cwd(real.path().to_str().unwrap());
    remember_cwd(stale.path().to_str().unwrap());
    // The probe hits real (including real_anchor_line), but fails to hit stale → real should be selected, and the leader stale of the team should not be taken.
    let got = read_patch_file("shared_moc263.txt", None, &[(false, "real_anchor_line")]);
    assert!(
        got.is_some(),
        "The candidate containing the anchor point should be selected"
    );
    assert_eq!(
        got.unwrap().1,
        "real_anchor_line\nmore\n",
        "Real (including probe anchor point) should be selected instead of team leader stale"
    );
}

#[test]
fn read_patch_file_single_candidate_partial_header_not_skipped() {
    // MOC-263 P2 second round (chatgpt-codex-connector review): probe only contains incomplete `@@` header (there is no exact in the file
    // OK, the truth is `## 6. System architecture suggestion`), **Single candidate** should not be judged as unreadable due to probe 0 hit - still returns
    // File, let align_at_headers do substring repair. A probe is a tie-breaker, not a gate.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("doc_moc263.md"),
        "intro\n## 6. System architecture recommendations\nLayering is recommended\n",
    )
    .unwrap();
    remember_cwd(dir.path().to_str().unwrap());
    let got = read_patch_file(
        "doc_moc263.md",
        None,
        &[(true, "System architecture suggestions")],
    );
    assert!(
        got.is_some(),
        "Single candidate + incomplete header probe should not be judged as unreadable"
    );
    assert!(got
        .unwrap()
        .1
        .contains("## 6. System architecture suggestions"));
}

#[test]
fn read_patch_file_partial_header_substring_picks_real_over_stale() {
    // MOC-263 P2 three rounds (chatgpt-codex-connector review): multiple candidates + pure incomplete `@@` header, exact all 0 → Retire
    // For substring scoring, select the substring containing the real substring of the header instead of blindly taking the first stale (otherwise align will repair the stale substring).
    let stale = tempfile::tempdir().unwrap();
    let real = tempfile::tempdir().unwrap();
    std::fs::write(
        stale.path().join("doc2_moc263.md"),
        "stale intro\nunrelated heading\n",
    )
    .unwrap();
    std::fs::write(
        real.path().join("doc2_moc263.md"),
        "intro\n## 6. System architecture recommendations\nLayering is recommended\n",
    )
    .unwrap();
    remember_cwd(real.path().to_str().unwrap());
    remember_cwd(stale.path().to_str().unwrap()); // stale is at the head of the team (most-recent)
    let got = read_patch_file(
        "doc2_moc263.md",
        None,
        &[(true, "System architecture suggestions")],
    );
    assert!(got.is_some());
    assert!(
        got.unwrap().1.contains("## 6. System architecture suggestions"),
        "The substring score should be selected as real containing the header, rather than stale as the header."
    );
}

#[test]
fn read_patch_file_tied_score_is_ambiguous_none() {
    // MOC-263 P2 four rounds (chatgpt-codex-connector review): Two candidate probe scores are tied (share the same anchor row)→
    // Ambiguity → None (no guess), instead of taking the first stale. Downstream skip and patch are self-corrected by passing Codex/model.
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    std::fs::write(a.path().join("tie_moc263.txt"), "SHARED_ANCHOR\naaa\n").unwrap();
    std::fs::write(b.path().join("tie_moc263.txt"), "SHARED_ANCHOR\nbbb\n").unwrap();
    remember_cwd(a.path().to_str().unwrap());
    remember_cwd(b.path().to_str().unwrap());
    let got = read_patch_file("tie_moc263.txt", None, &[(false, "SHARED_ANCHOR")]);
    assert!(
        got.is_none(),
        "Tie scores (ambiguity) should return None without guessing"
    );
}

#[test]
fn read_patch_file_header_probe_not_beaten_by_stale_exact_fragment() {
    // MOC-263 P2 fifth round (chatgpt-codex-connector review): probe only has a missing `@@` header; stale has a whole line =
    // The fragment,real is its substring (`## 6. X`). The header is scored according to **substring** (exact is not included) → both candidates have substring hits.
    // → Tie → None, **will not** be mistakenly selected due to stale's exact entire row.
    let stale = tempfile::tempdir().unwrap();
    let real = tempfile::tempdir().unwrap();
    std::fs::write(
        stale.path().join("h_moc263.md"),
        "System architecture recommendations\nx\n",
    )
    .unwrap();
    std::fs::write(
        real.path().join("h_moc263.md"),
        "## 6. System architecture suggestions\ny\n",
    )
    .unwrap();
    remember_cwd(real.path().to_str().unwrap());
    remember_cwd(stale.path().to_str().unwrap()); // stale is at the head of the team
    let got = read_patch_file(
        "h_moc263.md",
        None,
        &[(true, "System architecture suggestions")],
    );
    assert!(
        got.is_none(),
        "Header substring parallelization should be None, not mistakenly selected by stale's exact entire line"
    );
}

#[test]
fn context_line_starting_with_at_at_does_not_block_split() {
    // MOC-263 P2 fifth round (chatgpt-codex-connector review): context row content starts with @@ (` @@ ...`, column 0 is
    // spaces) should not be used as hunk delimiters to disable automatic splitting (the delimiter only recognizes columns 0 @@). Multi-region pure deletes containing such context lines should still work.
    let content = "@@ banner\nkeep_a\nREMOVE_1\nmid1\nmid2\nmid3\nREMOVE_2\nkeep_b\n";
    let (dir, name) = tmp_file("atat_moc263.txt", content);
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n @@ banner\n keep_a\n-REMOVE_1\n mid1\n-REMOVE_2\n keep_b\n*** End Patch\n"
    );
    let (out, _reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        out.contains("\n@@\n"),
        "Multi-region deletion of lines containing ` @@` context should still be automatically segmented (column 0 is judged):\n {out}"
    );
}

#[test]
fn anchor_probe_includes_unprefixed_lines() {
    // MOC-263 P2 six rounds (chatgpt-codex-connector review): unprefixed lines (fix_unprefixed_lines should be based on the entire line of the file
    // Matching repair) must also enter the probe, otherwise it is the only anchor point space-time-probe path and the wrong file will be selected among the candidates with the same name. `+` / `***` does not advance.
    let body = vec![
        " ctx",
        "-del",
        "+add",
        "@@ hdr",
        "unprefixed line",
        "*** End Patch",
    ];
    let p = anchor_probe(&body);
    assert!(p.contains(&(false, "ctx")), "context traveling probe");
    assert!(p.contains(&(false, "del")), "Delete traveling probe");
    assert!(p.contains(&(true, "hdr")), "@@ Header probe(header)");
    assert!(
        p.contains(&(false, "unprefixed line")),
        "Unprefixed lines should be used as exact probes"
    );
    assert!(
        !p.iter().any(|(_, t)| *t == "add"),
        "+ Added new failure probe"
    );
    assert!(
        !p.iter().any(|(_, t)| t.starts_with("***")),
        "*** Control probe"
    );
}

#[test]
fn preflight_aligns_via_candidate_cwd_with_none_primary() {
    // MOC-263 P1 end-to-end: apply_patch request cwd=None, the real cwd is only in the candidate history → disk alignment can still be read.
    let (dir, name) = tmp_file("p1_e2e_moc263.txt", "alpha\nbeta\ngamma\n");
    let real = dir.path().to_str().unwrap().to_owned();
    remember_cwd(&real);
    // context with trailing spaces (requires disk alignment); primary=None, rely on candidate history to find the real file.
    let v4a =
        format!("*** Begin Patch\n*** Update File: {name}\n beta   \n+inserted\n*** End Patch\n");
    let (out, reps) = preflight_repair(&v4a, None);
    assert!(
        out.contains("\n beta\n"),
        "Trailing spaces should be aligned by candidate cwd reads:\n {out}"
    );
    assert!(out.contains("+inserted"), "Keep new rows");
    assert_eq!(reps[0].kind, "repaired", "{:?}", reps);
}

#[test]
fn multi_hunk_pure_delete_no_at_auto_split() {
    // MOC-263 P0: Multiple discontinuous **pure delete/context** areas are combined into one Update File block, without @@ → Safe automatic segmentation and insertion @@.
    // (Multiple regions with insertion `+` are not covered here due to the ambiguity of placement points, see mixed_replace_insert_gap_passthrough)
    let content = "keep_top\nREMOVE_1\nmiddle\nmiddle2\nmiddle3\nREMOVE_2\nkeep_bottom\n";
    let (dir, name) = tmp_file("multi_del.txt", content);
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n keep_top\n-REMOVE_1\n middle\n-REMOVE_2\n keep_bottom\n*** End Patch\n"
    );
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        out.contains("\n@@\n"),
        "Two non-consecutive deleted areas should be inserted naked @@:\n {out}"
    );
    assert_eq!(reps[0].kind, "repaired", "{:?}", reps);
    assert!(
        reps[0]
            .detail
            .contains("Automatically cut into segments based on file location"),
        "{:?}",
        reps
    );
    assert!(
        out.contains("-REMOVE_1") && out.contains("-REMOVE_2"),
        "Deleted rows retained:\n {out}"
    );
}

#[test]
fn mixed_replace_insert_gap_passthrough() {
    // MOC-263 P0 security (chatgpt-codex-connector review pointed out): inter-segment "replacement + extra insertion" mixing →
    // `+` placement ambiguity (`+return 42` is a replacement, `+@memoize` is an introduction line to the later `def beta`, and cannot be distinguished)→
    // Do not cut, pass through as it is, avoid inserting @memoize silently after return (misplaced). Even if the anchor point at the end of the previous paragraph is `-`, deletion will not be exempted.
    let content = "alpha\nreturn 1\n# gap\ndef beta():\n";
    let (dir, name) = tmp_file("mixed.py", content);
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n-return 1\n+return 42\n+@memoize\n def beta():\n*** End Patch\n"
    );
    let (out, _reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        !out.contains("\n@@\n"),
        "Mixed replace+insert placement ambiguity should not be cut:\n {out}"
    );
    assert!(
        out.contains("+@memoize") && out.contains("+return 42"),
        "No content is lost"
    );
}

#[test]
fn single_contiguous_hunk_not_split() {
    // Single segment continuous hunk → no cutting (group<2 → None), normal alignment.
    let (dir, name) = tmp_file("single.txt", "a\nb\nc\nd\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Begin Patch\n*** Update File: {name}\n a\n b\n+x\n c\n*** End Patch\n");
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        !out.contains("\n@@\n"),
        "Single segment continuous hunk should not be inserted @@:\n {out}"
    );
    assert!(!reps[0].detail.contains("Cut into sections"), "{:?}", reps);
}

#[test]
fn ambiguous_multi_region_passthrough() {
    // The anchor content is repeated in the file (ambiguity) → longest_unique_block returns None → do not cut, pass through (no guessing, no loss).
    let (dir, name) = tmp_file("amb.txt", "x\ny\nx\ny\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Begin Patch\n*** Update File: {name}\n-x\n+X\n-y\n+Y\n*** End Patch\n");
    let (out, _reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        !out.contains("\n@@\n"),
        "Ambiguity should not be resolved:\n {out}"
    );
}

#[test]
fn greedy_split_bails_when_first_anchor_not_globally_unique() {
    // MOC-263 P1 (chatgpt-codex-connector review): Hidden ambiguity in **block is unique but segment header is not unique** - file has old
    // ALPHA/BETA/GAMMA block + real ALPHA/BETA…gap…GAMMA/DELTA zone. body `ALPHA/-BETA/ GAMMA/-DELTA`
    // [ALPHA, BETA, GAMMA] as a continuous block only appears in the old block. Greedy will select the old block and delete BETA from the **wrong block**; and the beginning of the paragraph
    // ALPHA appears 2 times in file = starting point ambiguity. After the repair, the segment header is not globally unique, that is, bail, which is not divided and passed through as it is (no guessing, no loss).
    let content = "ALPHA\nBETA\nGAMMA\nmid_x\nmid_y\nALPHA\nBETA\nsep_gap\nGAMMA\nDELTA\n";
    let (dir, name) = tmp_file("greedy_moc263.txt", content);
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n ALPHA\n-BETA\n GAMMA\n-DELTA\n*** End Patch\n"
    );
    let (out, _reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        !out.contains("\n@@\n"),
        "When the segment head anchor point is not globally unique (starting point ambiguity), bail should not be split to avoid deleting lines from the wrong block:\n {out}"
    );
}

#[test]
fn floating_add_after_context_passthrough_not_misplaced() {
    // MOC-263 P0 safety protection: `+` floats between two discontinuous areas, and the last anchor point of the previous segment is context (not `-` deleted)→ drop point
    // Ambiguity (may be end insertion in the previous paragraph, or introduction line in the later paragraph) → Do not cut (otherwise +@memoize will be inserted into the wrong position, silent error
    // apply). This is the BLOCKER regression point caught in the pre-push review.
    let content =
        "def alpha():\n    return 1\n# --- section break ---\ndef beta():\n    return 2\n";
    let (dir, name) = tmp_file("deco.py", content);
    let cwd = dir.path().to_str().unwrap();
    // The model assumes that `return 1` and `def beta():` are adjacent and add +@memoize; in the middle, which is actually separated by a section break.
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n     return 1\n+@memoize\n def beta():\n*** End Patch\n"
    );
    let (out, _reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        !out.contains("\n@@\n"),
        "Float + drop point ambiguity should not be segmented (to prevent silent errors apply):\n {out}"
    );
    assert!(out.contains("+@memoize"), "No content is lost");
}

#[test]
fn blank_line_drift_block_realigned() {
    // EP-1 real machine seq111: The model context block misses an empty line in the file → the entire block is mismatched. Ignore empty lines unique positioning → Rebuild
    // (Complement empty lines in the file + alignment bytes), `+` inserts a guaranteed bit.
    let (dir, name) = tmp_file(
        "main.py",
        "from a import x\nfrom b import y\n\nfrom c import z\nfrom d import w\n",
    );
    let cwd = dir.path().to_str().unwrap();
    // The context of patch misses the blank line between `from b` and `from c`, and I want to insert a line after `from d`.
    let v4a = format!(
        "*** Begin Patch\n*** Update File: {name}\n from a import x\n from b import y\n from c import z\n from d import w\n+from e import v\n*** End Patch\n"
    );
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert!(
        out.contains("+from e import v"),
        "Inserted rows retained:\n {out}"
    );
    // After rebuilding the context block should contain padded empty lines (bare ' ').
    assert!(
        out.contains("\n \n"),
        "Blank lines should be added to the file as context:\n {out}"
    );
    assert_eq!(reps[0].kind, "repaired", "{:?}", reps);
}

#[test]
fn blank_tolerant_skips_blank_line_deletion() {
    // `-` containing "delete a blank line" → blank-tolerant reconstruction cannot be expressed faithfully → leave unchanged (not silently converted to context).
    let (dir, name) = tmp_file("bd.txt", "x\ny\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Update File: {name}\n x\n-\n y\n+z\n");
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert_eq!(
        out, v4a,
        "Lines containing blank spaces should be deleted and left unchanged:\n {out}"
    );
    assert!(reps[0].kind.starts_with("skipped"), "{:?}", reps);
}

#[test]
fn blank_tolerant_ambiguous_passthrough() {
    // Exact mismatch (there is a blank line between file p/q, patch is not written) but **multiple** matches after ignoring the blank line → Ambiguity is allowed and no guessing is allowed.
    let (dir, name) = tmp_file("dup.txt", "p\n\nq\nX\np\n\nq\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!("*** Update File: {name}\n p\n q\n+r\n");
    let (out, reps) = preflight_repair(&v4a, Some(cwd));
    assert_eq!(
        out, v4a,
        "Ambiguities (ignore multiple places after blank lines) are not changed:\n {out}"
    );
    assert!(reps[0].kind.starts_with("skipped"), "{:?}", reps);
}

#[test]
fn optimize_pipeline_fixes_multiple_issues() {
    // One patch at the same time: leaky envelope + double-sided @@ + trailing whitespace context → full recovery.
    let (dir, name) = tmp_file("multi.txt", "fn main() {\n    let x = 1;\n}\n");
    let cwd = dir.path().to_str().unwrap();
    let v4a = format!(
        "*** Update File: {name}\n@@ fn main() {{ @@\n    let x = 1;   \n+    let y = 2;\n"
    );
    let (out, reps) = optimize_patch(&v4a, Some(cwd), true);
    assert!(
        out.starts_with("*** Begin Patch\n"),
        "Fill envelope:\n {out}"
    );
    assert!(
        out.trim_end().ends_with("*** End Patch"),
        "Supplement End:\n {out}"
    );
    assert!(
        out.contains("@@ fn main() {\n"),
        "Bilateral @@ to unilateral:\n {out}"
    );
    assert!(
        out.contains("    let x = 1;\n"),
        "Trailing space alignment:\n {out}"
    );
    assert!(out.contains("+    let y = 2;"), "Keep new rows");
    // At least 3 types of repairs are recorded
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
    // None Update File → short circuit returns unchanged (even to cwd).
    let (out, reps) = preflight_repair(v4a, Some("/tmp"));
    assert_eq!(out, v4a);
    assert!(reps.is_empty());
    // no cwd → as is
    let (out2, reps2) = preflight_repair(v4a, None);
    assert_eq!(out2, v4a);
    assert!(reps2.is_empty());
}
