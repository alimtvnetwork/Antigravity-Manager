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

/// Affirmative predicate checking if match result represents a terminal stop.
pub fn is_stop_sequence_matched(match_result: &StopSequenceMatch) -> bool {
    matches!(match_result, StopSequenceMatch::Matched { .. })
}

/// Affirmative predicate checking if match result requires buffering for potential stop sequence.
pub fn is_partial_match(match_result: &StopSequenceMatch) -> bool {
    matches!(match_result, StopSequenceMatch::PartialPrefixMatch { .. })
}

/// Checks if text contains an exact stop sequence.
pub fn contains_exact_stop(text: &str, stop_seq: &str) -> bool {
    !stop_seq.is_empty() && text.contains(stop_seq)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_stop_sequence_matched_predicates() {
        let no_match = StopSequenceMatch::NoMatch;
        let matched = StopSequenceMatch::Matched {
            matched_sequence: "</stop>".to_string(),
            truncated_text: "output".to_string(),
        };
        let partial = StopSequenceMatch::PartialPrefixMatch {
            candidate: "</stop>".to_string(),
        };

        assert!(!is_stop_sequence_matched(&no_match));
        assert!(is_stop_sequence_matched(&matched));
        assert!(!is_stop_sequence_matched(&partial));

        assert!(!is_partial_match(&no_match));
        assert!(!is_partial_match(&matched));
        assert!(is_partial_match(&partial));
    }

    #[test]
    fn test_check_stop_sequences_exact_match() {
        let text = "Hello world! <stop> More tokens";
        let stops = vec!["<stop>".to_string()];

        let res = check_stop_sequences(text, &stops);
        assert_eq!(
            res,
            StopSequenceMatch::Matched {
                matched_sequence: "<stop>".to_string(),
                truncated_text: "Hello world! ".to_string(),
            }
        );
    }

    #[test]
    fn test_check_stop_sequences_partial_prefix() {
        let text = "Hello world! <st";
        let stops = vec!["<stop>".to_string()];

        let res = check_stop_sequences(text, &stops);
        assert_eq!(
            res,
            StopSequenceMatch::PartialPrefixMatch {
                candidate: "<stop>".to_string(),
            }
        );
    }

    #[test]
    fn test_check_stop_sequences_no_match() {
        let text = "Hello world!";
        let stops = vec!["<stop>".to_string()];

        let res = check_stop_sequences(text, &stops);
        assert_eq!(res, StopSequenceMatch::NoMatch);
    }
}
