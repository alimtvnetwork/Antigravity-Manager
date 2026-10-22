//! Subagent Hierarchy Nesting Depth Guard.
//! Validates conversation tree recursion depths and caps runaway subagent chains.

use thiserror::Error;

pub const DEFAULT_MAX_SUBAGENT_DEPTH: usize = 10;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DepthGuardError {
    #[error("Subagent hierarchy depth {current_depth} exceeds maximum allowable depth {max_allowed_depth}")]
    DepthExceeded {
        current_depth: usize,
        max_allowed_depth: usize,
    },
}

/// Validates whether the current recursion level is within safe operational limits.
pub fn validate_tree_depth(
    current_depth: usize,
    max_depth: Option<usize>,
) -> Result<(), DepthGuardError> {
    let limit = max_depth.unwrap_or(DEFAULT_MAX_SUBAGENT_DEPTH);
    if current_depth > limit {
        Err(DepthGuardError::DepthExceeded {
            current_depth,
            max_allowed_depth: limit,
        })
    } else {
        Ok(())
    }
}

/// Generic depth calculator for hierarchical items that have child elements.
pub fn calculate_hierarchy_depth<T, F>(node: &T, children_fn: F) -> usize
where
    F: Fn(&T) -> &[T] + Copy,
{
    let children = children_fn(node);
    if children.is_empty() {
        1
    } else {
        1 + children
            .iter()
            .map(|c| calculate_hierarchy_depth(c, children_fn))
            .max()
            .unwrap_or(0)
    }
}
