//! Interactive Truncation Callout Parser & Omission Banner Guard.
//!
//! Enforces conversation rendering discipline per AGENTS.md:
//! Never render raw `<truncated N bytes>` or `<truncated N lines>` markers as plain text;
//! parse and render them as interactive dark-glass omission banners
//! (`⚡ [Omitted N bytes/lines of context - Click to inspect/expand]`).

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Regex detecting raw `<truncated N bytes>` or `<truncated N lines>` markers.
/// Handles singular/plural units, case-insensitivity, and optional range annotations
/// (e.g. `<truncated 923 of 10767 bytes from the middle of the output>`).
static TRUNCATION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)<truncated\s+(\d+)(?:\s+of\s+\d+)?\s+(bytes?|lines?|b)(?:[^>]*)?>")
        .expect("valid truncation callout regex")
});

/// Unit classification for omitted transcript or payload context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TruncationUnit {
    Bytes,
    Lines,
}

impl fmt::Display for TruncationUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bytes => write!(f, "bytes"),
            Self::Lines => write!(f, "lines"),
        }
    }
}

/// Represents a parsed truncation omission banner with interactive presentation representations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmissionBanner {
    pub raw_marker: String,
    pub quantity: u64,
    pub unit: TruncationUnit,
    pub banner_text: String,
    pub html_representation: String,
}

/// Structured parsing result across a text document or transcript turn.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TruncationParseResult {
    pub raw_text: String,
    pub formatted_text: String,
    pub html_text: String,
    pub callouts: Vec<OmissionBanner>,
    pub has_truncations: bool,
}

/// Formats the standard AGENTS.md omission banner plain-text string:
/// `⚡ [Omitted N bytes/lines of context - Click to inspect/expand]`.
pub fn format_omission_banner_text(quantity: u64, unit: TruncationUnit) -> String {
    let unit_label = match (unit, quantity) {
        (TruncationUnit::Bytes, 1) => "byte",
        (TruncationUnit::Bytes, _) => "bytes",
        (TruncationUnit::Lines, 1) => "line",
        (TruncationUnit::Lines, _) => "lines",
    };
    format!("⚡ [Omitted {quantity} {unit_label} of context - Click to inspect/expand]")
}

/// Formats the interactive dark-glass HTML omission banner capsule matching AGENTS.md specifications.
pub fn format_dark_glass_html(quantity: u64, unit: TruncationUnit) -> String {
    let banner_text = format_omission_banner_text(quantity, unit);
    let unit_str = match unit {
        TruncationUnit::Bytes => "bytes",
        TruncationUnit::Lines => "lines",
    };

    format!(
        "<div class=\"interactive-dark-glass-omission-banner my-3 flex items-center justify-between px-3.5 py-2 rounded-xl bg-amber-500/10 dark:bg-amber-950/40 border border-amber-500/30 dark:border-amber-600/40 text-amber-900 dark:text-amber-100 text-xs shadow-sm backdrop-blur-md cursor-pointer hover:bg-amber-500/20 dark:hover:bg-amber-900/40 focus:outline-none focus:ring-2 focus:ring-amber-500/50 transition-all select-none\" role=\"button\" tabindex=\"0\" data-omitted-quantity=\"{quantity}\" data-omitted-unit=\"{unit_str}\" title=\"Click to inspect/expand full un-truncated context\" aria-label=\"{banner_text}\"><div class=\"flex items-center gap-2 font-mono font-medium\"><span class=\"text-amber-500 text-sm\">⚡</span><span>[Omitted {quantity} {unit_str} of context - Click to inspect/expand]</span></div></div>"
    )
}

/// Instantiates an `OmissionBanner` from matched components.
pub fn create_omission_banner(
    raw_marker: &str,
    quantity: u64,
    unit: TruncationUnit,
) -> OmissionBanner {
    let banner_text = format_omission_banner_text(quantity, unit);
    let html_representation = format_dark_glass_html(quantity, unit);

    OmissionBanner {
        raw_marker: raw_marker.to_string(),
        quantity,
        unit,
        banner_text,
        html_representation,
    }
}

/// Scans text and extracts all raw truncation markers into structured `OmissionBanner` instances.
pub fn extract_truncation_callouts(text: &str) -> Vec<OmissionBanner> {
    let mut callouts = Vec::new();

    for caps in TRUNCATION_REGEX.captures_iter(text) {
        let raw_marker = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
        let quantity_str = caps.get(1).map(|m| m.as_str()).unwrap_or("0");
        let unit_str = caps
            .get(2)
            .map(|m| m.as_str())
            .unwrap_or("bytes")
            .to_ascii_lowercase();

        let quantity: u64 = quantity_str.parse().unwrap_or(0);
        let unit = if unit_str.starts_with('l') {
            TruncationUnit::Lines
        } else {
            TruncationUnit::Bytes
        };

        callouts.push(create_omission_banner(raw_marker, quantity, unit));
    }

    callouts
}

/// Replaces all raw `<truncated ...>` markers with standard plain-text omission banners.
pub fn replace_raw_truncation_markers_with_banners(text: &str) -> String {
    TRUNCATION_REGEX
        .replace_all(text, |caps: &regex::Captures| {
            let quantity_str = caps.get(1).map(|m| m.as_str()).unwrap_or("0");
            let unit_str = caps
                .get(2)
                .map(|m| m.as_str())
                .unwrap_or("bytes")
                .to_ascii_lowercase();
            let quantity: u64 = quantity_str.parse().unwrap_or(0);
            let unit = if unit_str.starts_with('l') {
                TruncationUnit::Lines
            } else {
                TruncationUnit::Bytes
            };
            format_omission_banner_text(quantity, unit)
        })
        .to_string()
}

/// Replaces all raw `<truncated ...>` markers with interactive dark-glass HTML omission elements.
pub fn replace_raw_truncation_markers_with_html(text: &str) -> String {
    TRUNCATION_REGEX
        .replace_all(text, |caps: &regex::Captures| {
            let quantity_str = caps.get(1).map(|m| m.as_str()).unwrap_or("0");
            let unit_str = caps
                .get(2)
                .map(|m| m.as_str())
                .unwrap_or("bytes")
                .to_ascii_lowercase();
            let quantity: u64 = quantity_str.parse().unwrap_or(0);
            let unit = if unit_str.starts_with('l') {
                TruncationUnit::Lines
            } else {
                TruncationUnit::Bytes
            };
            format_dark_glass_html(quantity, unit)
        })
        .to_string()
}

/// Returns true if the string contains any unparsed raw truncation markers.
pub fn has_raw_truncation_markers(text: &str) -> bool {
    TRUNCATION_REGEX.is_match(text)
}

/// Validates that content contains zero unparsed raw truncation markers.
pub fn validate_no_raw_truncation_markers(text: &str) -> Result<(), Vec<String>> {
    let callouts = extract_truncation_callouts(text);
    if callouts.is_empty() {
        Ok(())
    } else {
        let violations = callouts
            .into_iter()
            .map(|c| {
                format!(
                    "Unparsed raw truncation marker detected: '{}' (must be parsed and rendered as interactive dark-glass omission banner: '{}')",
                    c.raw_marker, c.banner_text
                )
            })
            .collect();
        Err(violations)
    }
}

/// Processes text, detecting truncation markers and producing both banner and HTML representations.
pub fn parse_and_transform_truncations(text: &str) -> TruncationParseResult {
    let callouts = extract_truncation_callouts(text);
    let has_truncations = !callouts.is_empty();
    let formatted_text = replace_raw_truncation_markers_with_banners(text);
    let html_text = replace_raw_truncation_markers_with_html(text);

    TruncationParseResult {
        raw_text: text.to_string(),
        formatted_text,
        html_text,
        callouts,
        has_truncations,
    }
}
