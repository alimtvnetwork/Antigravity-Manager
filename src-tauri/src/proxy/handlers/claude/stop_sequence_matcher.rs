//! Claude Stream Custom Stop Sequence Detector
//!
//! Evaluates streaming response text deltas against client-requested `stop_sequences`
//! in Claude API requests. Safely intercepts and truncates the stream when a matched
//! stop token appears, setting `stop_reason: "stop_sequence"`.

use tracing::debug;

/// Result of evaluating text against stop sequences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopSequenceMatch {
    /// No stop sequence encountered.
    NoMatch,
    /// Exact match found; provides truncated text up to the match and matched sequence.
    Matched {
        matched_sequence: String,
        truncated_text: String,
    },
    /// Text ends with a potential prefix of a stop sequence (needs buffering).
    PartialPrefixMatch { candidate: String },
}

/// Matches streaming text accumulator against a list of custom stop sequences.
pub fn check_stop_sequences(
    accumulated_text: &str,
    stop_sequences: &[String],
) -> StopSequenceMatch {
    if stop_sequences.is_empty() || accumulated_text.is_empty() {
        return StopSequenceMatch::NoMatch;
    }

    // Step 1: Check for exact full match
    for stop_seq in stop_sequences {
        if stop_seq.is_empty() {
            continue;
        }

        if let Some(idx) = accumulated_text.find(stop_seq) {
            debug!(
                stop_sequence = %stop_seq,
                index = idx,
                "Encountered exact custom stop sequence in stream"
            );
            return StopSequenceMatch::Matched {
                matched_sequence: stop_seq.clone(),
                truncated_text: accumulated_text[..idx].to_string(),
            };
        }
    }

    // Step 2: Check for trailing partial prefix match
    for stop_seq in stop_sequences {
        for len in (1..stop_seq.len()).rev() {
            let prefix = &stop_seq[..len];
            if accumulated_text.ends_with(prefix) {
                return StopSequenceMatch::PartialPrefixMatch {
                    candidate: stop_seq.clone(),
                };
            }
        }
    }

    StopSequenceMatch::NoMatch
}
