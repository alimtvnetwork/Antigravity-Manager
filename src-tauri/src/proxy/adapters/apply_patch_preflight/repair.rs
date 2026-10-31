// Hunk repair (split from apply_patch_preflight.rs).
use super::cwd::anchor_probe;
use super::read::read_patch_file;
// apply_patch **pre-flight automatic repair**: Before sending the V4A patch to Codex apply, read the target file for comparison.
// Automatically align **safe** context mismatch (trailing spaces / leading and trailing whitespace differences), eliminating the number one failure of V4A
// `apply_patch verification failed: Failed to find expected lines`。
//
// ## Why is it needed?
// Weaker chat models (non-OpenAI) are often unable to reproduce the context/delete lines of `Update File` byte-by-byte on large files.
// (Trailing spaces, indentation, memory deviation) → Codex cannot find the anchor point → apply fails → the entire model file is rewritten, wasting time and tokens.
// The error reported by the actual machine (rollout ground truth) is exactly this type.
//
// ## Safety boundary (never damage files - align users with the hard rule of "no destructive downgrades")
// - **Move anchor point only**: context(space prefix)/ delete (`-`) line in `Update File`. `+New` lines **never change**.
// - **Only fix when there is a unique match**: The anchor block is found in the file by "Ignore trailing spaces/leading and trailing blanks" to find candidates, and **exactly one** position will be aligned;
//   0 (the model is really wrong) or ≥2 (ambiguous) will be **released as is** and handed over to Codex parse_patch to expose the real bad, never rely on guessing.
// - **Add File / Delete File does not touch** (no anchor, no matching involved). Unable to read file / no cwd → release as is.
// - Each repair/release is recorded in the apply-patch diagnostic page and can be audited.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::proxy::adapters::apply_patch_preflight::Repair;
use serde_json::Value;

/// A record of pre-flight processing (for diagnostic pages/logs).

/// [MOC-263 P0] Find the longest consecutive block in `file[floor..]` that can be uniquely matched starting from `anchors[0]`.
/// Anchors are compared using "ignore trailing spaces" (intra-segment byte drift is left for subsequent repair_hunk alignment). Return `(block length = number of matching anchors,
/// File starting point)`. The longest and unique → Some; the longest non-empty match if >1 (ambiguous) → None (shorter will only be more ambiguous); all 0 → None.
fn longest_unique_block(anchors: &[&str], file: &[&str], floor: usize) -> Option<(usize, usize)> {
    if anchors.is_empty() || floor >= file.len() {
        return None;
    }
    // [MOC-263 P1] The first anchor point of the paragraph must be in `file[floor..]` **globally unique**, otherwise the starting point of the paragraph is ambiguous - when the same line also appears elsewhere,
    // Greedy longest block will rely on the "uniqueness" of the longer block to select **irrelevant earlier areas** (file has old `A/B/C` blocks + real `A/B…gap…C/D`
    // Area, body `A/-B/ C/-D` was cut into the old block hunk → delete B) from the wrong block, which would have failed safely without splitting. The starting point is not unique = segmentation
    // Non-unique determination → bail, self-corrected through cross-model as is (chatgpt-codex-connector review; don’t guess, don’t lose).
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
            0 => continue, // Too long (spanning a jump in the file) → shorten and try again
            _ => return None, // The longest non-empty match is ambiguity → give up (no guessing, no losing)
        }
    }
    None
}

/// [MOC-263 P0] Split the Update body without `@@`** into multiple hunks according to the real location of the file.
///
/// The main reason for apply failure on real machines (phase-1 seg1/seg3): the model combines multiple **discontinuous** editing groups into one Update File block,
/// Missing `@@` delimitation → applier Matches the whole block as a continuous context → `Failed to find expected lines`.
/// Here, according to the anchor point (context/delete) sequence, it is greedily cut into N segments that are ordered, non-overlapping, and each uniquely matching** (each segment = from the previous segment
/// Starting from the longest consecutive anchor point block that only matches), `+` new lines are retained along with adjacent segments. Only returned if **N≥2 and each segment can be uniquely positioned**
/// `Some`; single paragraph / any paragraph is ambiguous or cannot be located → `None` (the caller passes it through as it is, no guessing, no loss). The caller uses bare `@@` to concatenate the segments.
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

    // Greedy segmentation: each segment = uniquely matched longest continuous anchor block starting from floor.
    // Note (anchor_start, anchor_end_excl, file_start, file_end_excl); floor monotonically advances to ensure ordering and non-overlapping.
    let mut raw: Vec<(usize, usize, usize, usize)> = Vec::new();
    let mut ai = 0usize;
    let mut floor = 0usize;
    while ai < anchors.len() {
        let (len, pos) = longest_unique_block(&anchor_contents[ai..], file, floor)?;
        raw.push((ai, ai + len, pos, pos + len));
        ai += len;
        floor = pos + len;
    }

    // Merge adjacent segments: If the file gap between segments is **all blank lines** (the model misses the blank lines in the file) → the same hunk, it is not cut here.
    // Handle repair_hunk's EP-1 blank-tolerant processing (otherwise, the blank line drift will be mistakenly cut into two segments, destroying the existing behavior).
    // Only remain as independent segments if the gap contains **non-empty lines** (true · non-continuous editing area).
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
        return None; // Single segment (or merged into a single segment due to blank line gaps) → No need to cut, return to the regular path
    }

    // [MOC-263 P0 Security Protection] Inter-segment "floating `+` insert line" placement ambiguity → always bail.
    // If there is **any** `+` new line between the last anchor point of the previous paragraph and the first anchor point of the following paragraph, the placement point of the `+` cannot be uniquely determined from V4A——
    // It may be an insertion at the end of the previous paragraph, or it may be an "introduction line" written by the model to the subsequent paragraph; when the paragraphs are separated by non-empty content, the two placement points are in the file.
    // Different positions, guessing wrong is a **silent error apply** (violation, no guessing, no loss). **Key**: Even if the anchor point at the end of the previous paragraph is `-`, it is not safe to delete it
    // (chatgpt-codex-connector review points out a mix of replace+insert:`-return 1`/`+return 42`/`+@memoize`/
    // ` def beta():` —— `+return 42` is a replacement, `+@memoize` is an introduction line for the later paragraph, the two are indistinguishable)→ Do not do it
    // The exemption of "let it pass if the previous segment is deleted", as long as there is `+` in the gap, the entire segmentation will be abandoned and passed through as it is (subject to model self-correction).
    // Pure deletion / pure context multi-region (gap without `+`) is still safe.
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

    // The body line interval of segment g: the first paragraph includes the leading line (body[0..first anchor point]); the remaining paragraphs start from their first anchor point,
    // Ending at the first anchor point of the next paragraph → `+` lines within/after the paragraph are retained with the **previous** paragraph.
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

/// Fix the body of an `Update File` section. `path` is the (relative) path in patch.
/// `cwd` is the current request cwd (primary hint), read the disk through [`read_patch_file`] and then analyze the candidate history (MOC-263).
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
                detail: format!(
                    "Unable to read file {path} (no candidate cwd exists) → release as is"
                ),
            },
        );
    };
    let file_lines: Vec<&str> = content.lines().collect();

    // [MOC-263 P0] body has no `@@` but contains multiple discontinuous editing groups (the model omits `@@` separation) → automatically cuts into segments according to the file position,
    // Use naked `@@` to concatenate, so that the applier will position each segment as an independent hunk (otherwise the whole block will be mismatched as a continuous context).
    // Only move if the only one is segmentable; single segment / ambiguity → keep the original body intersection regular path.
    let mut split_owned: Vec<&str> = Vec::new();
    // Use **column 0** `@@` (without trim_start) to determine whether there is hunk separation, and compare it with the actual splitter below (`l.starts_with("@@")`)
    // Consistent - otherwise context lines `@@ ...` (leading spaces, content starting with @@, such as markdown/diff text) will be mistaken for delimiters,
    // Error disabling automatic splitting and the splitter doesn't cut it → still fails (chatgpt-codex-connector review).
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

    // Cut the body into hunks (segmented by `@@` lines; `@@` lines themselves are retained and do not participate in anchor point matching).
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
        "{} hunk: Repair {repaired_hunks} / Already match {clean_hunks} / Release {} {}",
        if did_split {
            "Multiple hunk without @@ separation → automatically cut into segments according to file location @@;"
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
    /// The anchor point matches the file exactly and does not need to be modified.
    Clean,
    /// The anchor is aligned to the entire hunk (including the original `+` line) after the actual bytes of the file.
    Repaired(Vec<String>),
    /// Not fixed (0 or more matches), with reason.
    Skipped(String),
}

/// Fix a hunk: anchor point = context (space prefix) + delete the **content** (remove the prefix) of the (`-`) line, which should be in the file in order
/// Continuous blocks. Exact match→Clean; otherwise press "Ignore trailing spaces/leading and trailing blanks" to find candidates, unique→Align, otherwise let go.
fn repair_hunk(hunk: &[&str], file_lines: &[&str]) -> HunkOutcome {
    // The index + content of the anchor line in hunk (removing the single-character prefix).
    let anchors: Vec<(usize, &str)> = hunk
        .iter()
        .enumerate()
        .filter_map(|(idx, l)| match l.chars().next() {
            Some(' ') => Some((idx, &l[1..])),
            Some('-') => Some((idx, &l[1..])),
            _ => None, // '+' new line/blank line/others not used as anchor points
        })
        .collect();
    if anchors.is_empty() {
        return HunkOutcome::Clean; // Pure new addition, no anchor point
    }
    let anchor_contents: Vec<&str> = anchors.iter().map(|(_, c)| *c).collect();

    // Exact match: There are consecutive blocks in the file that are exactly equal to the anchor content → no need to modify (Codex can find it by itself).
    if !find_block(file_lines, &anchor_contents, |a, b| a == b).is_empty() {
        return HunkOutcome::Clean;
    }

    // Fuzzy matching: line by line "ignore trailing spaces" are equal; still 0 and then return "ignore both leading and trailing spaces".
    let mut matches = find_block(file_lines, &anchor_contents, |a, b| {
        a.trim_end() == b.trim_end()
    });
    let mut mode = "Trailing spaces";
    if matches.is_empty() {
        matches = find_block(file_lines, &anchor_contents, |a, b| a.trim() == b.trim());
        mode = "Blank space at first and last";
    }
    match matches.len() {
        1 => {
            let pos = matches[0];
            // Align anchor lines to actual file bytes (preserving +/- /space interleaving and `+` lines in hunks).
            let mut fixed: Vec<String> = hunk.iter().map(|l| (*l).to_owned()).collect();
            for (k, (idx, _)) in anchors.iter().enumerate() {
                let prefix = hunk[*idx].chars().next().unwrap(); // ' ' or '-'
                let file_line = file_lines[pos + k];
                fixed[*idx] = format!("{prefix}{file_line}");
            }
            HunkOutcome::Repaired(fixed)
        }
        n if n > 1 => HunkOutcome::Skipped(format!("Matches {n} under {mode} (ambiguity)")),
        // 0 Continuous matching → Try "Ignore blank line differences" (EP-1: The model misses/writes more blank lines, causing the entire block to mismatch). Anchor ** non-blank line ** sequence
        // Unique positioning in the file (allowing this section of the file to contain blank lines missed by the model), if hit, use the real section of the file (containing blank lines + bytes)
        // Rebuild the anchor point, `+` and the inserted row remains in place. 0/Many places are still allowed (no guessing).
        _ => {
            // blank-tolerant rebuild will discard blank anchor lines and use file blank lines instead → `-` cannot faithfully express "remove a blank line"
            // (It will be silently converted into context = should be deleted or not). If hunk contains blank lines, delete them, discard blank-tolerant, and pass (no guessing).
            let has_blank_deletion = hunk
                .iter()
                .any(|l| l.starts_with('-') && l[1..].trim().is_empty());
            if has_blank_deletion {
                return HunkOutcome::Skipped(
                    "Delete lines containing blanks, blank-tolerant is unsafe → release".to_owned(),
                );
            }
            let regions = find_regions_blank_tolerant(file_lines, &anchor_contents);
            match regions.len() {
                1 => {
                    let (s, e) = regions[0];
                    HunkOutcome::Repaired(rebuild_hunk_with_region(hunk, &file_lines[s..e]))
                }
                0 => HunkOutcome::Skipped("The anchor point has 0 matches in the file (suspected model correction content)".to_owned()),
                n => HunkOutcome::Skipped(format!("Ignore matches at {n} under empty lines (ambiguity)")),
            }
        }
    }
}

/// EP-1 Auxiliary: Find the interval in `file_lines` that can be uniquely positioned by the anchor **non-empty line** sequence (allowing the file interval to contain missing data from the model)
/// Blank lines, but no additional non-blank lines allowed). Returns all matching intervals `[start, end)` (end is the next bit of the last matching non-empty line).
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
                fi += 1; // Skip empty lines in the file (the model may miss writing)
                continue;
            }
            if fl.trim_end() == nb[ai] {
                ai += 1;
                fi += 1;
            } else {
                ok = false; // Extra non-empty lines appear → this start does not match
                break;
            }
        }
        if ok && ai == nb.len() {
            regions.push((start, fi));
        }
    }
    regions
}

/// EP-1 Auxiliary: Rebuild hunk using the file's real range `region` (including blank lines) - anchor point (context/`-`) aligned into file bytes,
/// Fill in the blank lines in the file that the model misses (used as context), `+` and insert lines in the original order of hunk. The blank anchor lines that come with the model are discarded
/// (Use blank lines of the file instead to avoid duplication).
fn rebuild_hunk_with_region(hunk: &[&str], region: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut fi = 0usize; // region cursor
    for &hl in hunk {
        match hl.chars().next() {
            Some('+') => out.push(hl.to_owned()), // Keep inserted rows in place
            Some(' ') | Some('-') => {
                let prefix = hl.chars().next().unwrap();
                let content = &hl[1..];
                if content.trim().is_empty() {
                    continue; // Empty anchor lines of the model are discarded and empty lines of the file are used.
                }
                // First fill in the blank lines that the model missed in the file (called context)
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
            _ => {} // Empty lines without prefix are discarded, and empty lines in the file are used.
        }
    }
    out
}

/// Find all starting points `i` in `file_lines`, make `file_lines[i..i+anchor.len()]` and `anchor` line by line `eq` true.
/// Returns all matching starting points.
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

/// Resolve the patch path to an absolute path. Absolute paths as-is; relative paths concatenated with `cwd`.
pub(crate) fn resolve_path(path: &str, cwd: &str) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        Path::new(cwd).join(p)
    }
}
