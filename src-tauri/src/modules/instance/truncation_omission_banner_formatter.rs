//! Truncation Omission Banner Formatter (`Imp-683`).
//!
//! Enforces AGENTS.md Minimalist & Contextual UI invariant:
//! "Interactive Truncation Callouts: Never render raw `<truncated N bytes>` or `<truncated N lines>`
//! markers as plain text; parse and render them as interactive dark-glass omission banners
//! (`⚡ [Omitted N bytes/lines of context - Click to inspect/expand]`)."
//!
//! Formats raw transcript/prompt truncation markers into structured, interactive dark-glass UI
//! callout elements with human-readable size labels (B, KB, MB) and expansion hooks.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Categorization of truncated context markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TruncationKind {
    /// Omission specified in byte length.
    Bytes(usize),
    /// Omission specified in line count.
    Lines(usize),
}

impl fmt::Display for TruncationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bytes(b) => write!(f, "{} bytes", format_bytes_human(*b)),
            Self::Lines(l) => write!(f, "{l} lines"),
        }
    }
}

/// A structured record representing a parsed truncation marker and its interactive UI banner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmissionBannerRecord {
    /// Raw token text as found in source (e.g. `<truncated 4096 bytes>`).
    pub raw_token: String,
    /// Parsed kind and count.
    pub kind: TruncationKind,
    /// Formatted human-readable count string (e.g. `4.0 KB` or `25 lines`).
    pub formatted_label: String,
    /// Standard interactive text banner format.
    pub banner_text: String,
    /// Rich HTML/JSX markup with dark-glass styling tokens.
    pub banner_html: String,
}

/// Converts a raw byte count into a human-readable string (e.g. `512 B`, `2.4 KB`, `1.5 MB`).
pub fn format_bytes_human(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        let kb = bytes as f64 / 1024.0;
        format!("{kb:.1} KB")
    } else {
        let mb = bytes as f64 / (1024.0 * 1024.0);
        format!("{mb:.1} MB")
    }
}

/// Parses raw truncation tokens from content and generates structured omission records.
pub fn parse_truncation_markers(content: &str) -> Vec<OmissionBannerRecord> {
    let mut records = Vec::new();
    let mut search_from = 0;

    while let Some(start_idx) = content[search_from..].find("<truncated ") {
        let abs_start = search_from + start_idx;
        let rest = &content[abs_start..];

        if let Some(end_idx) = rest.find('>') {
            let full_token = &rest[..=end_idx];
            let inner = &full_token[11..full_token.len() - 1].trim();

            let parts: Vec<&str> = inner.split_whitespace().collect();
            if parts.len() == 2 {
                if let Ok(count) = parts[0].parse::<usize>() {
                    let unit = parts[1].to_ascii_lowercase();
                    let kind = if unit.starts_with("byte") {
                        Some(TruncationKind::Bytes(count))
                    } else if unit.starts_with("line") {
                        Some(TruncationKind::Lines(count))
                    } else {
                        None
                    };

                    if let Some(kind) = kind {
                        let formatted_label = match kind {
                            TruncationKind::Bytes(b) => format_bytes_human(b),
                            TruncationKind::Lines(l) => format!("{l} lines"),
                        };

                        let banner_text = format!(
                            "⚡ [Omitted {formatted_label} of context - Click to inspect/expand]"
                        );

                        let banner_html = format!(
                            "<div role=\"button\" tabIndex=\"0\" aria-label=\"Inspect omitted context\" \
                             className=\"my-2 flex items-center justify-center gap-2 rounded-lg border border-amber-500/30 \
                             bg-amber-950/20 px-3 py-1.5 text-xs font-medium text-amber-300 backdrop-blur-md \
                             hover:bg-amber-950/40 hover:border-amber-500/50 cursor-pointer transition-all\">\
                             <span>⚡</span>\
                             <span>[Omitted {formatted_label} of context - Click to inspect/expand]</span>\
                             </div>"
                        );

                        records.push(OmissionBannerRecord {
                            raw_token: full_token.to_string(),
                            kind,
                            formatted_label,
                            banner_text,
                            banner_html,
                        });
                    }
                }
            }

            search_from = abs_start + end_idx + 1;
        } else {
            break;
        }
    }

    records
}

/// Replaces all raw `<truncated N bytes/lines>` markers in text with interactive omission banner text.
pub fn replace_markers_with_banners(content: &str) -> (String, usize) {
    let records = parse_truncation_markers(content);
    if records.is_empty() {
        return (content.to_string(), 0);
    }

    let mut output = content.to_string();
    for rec in &records {
        output = output.replace(&rec.raw_token, &rec.banner_text);
    }

    (output, records.len())
}
