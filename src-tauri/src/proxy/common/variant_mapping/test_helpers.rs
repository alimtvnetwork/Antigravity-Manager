// Shared test helper (split from variant_mapping.rs)
use super::resolve::resolve;

/// Helper: call `resolve(canonical, budget)` and assert id / tb / mot.
pub(crate) fn check(canonical: &str, budget: Option<u32>, id: &str, tb: u32, mot: u32) {
    let s = resolve(canonical, budget)
        .unwrap_or_else(|| panic!("resolve({canonical:?}, {budget:?}) unexpectedly returned None"));
    assert_eq!(s.id, id, "resolve({canonical:?}, {budget:?}).id");
    assert_eq!(
        s.thinking_budget, tb,
        "resolve({canonical:?}, {budget:?}).thinking_budget"
    );
    assert_eq!(
        s.max_output_tokens, mot,
        "resolve({canonical:?}, {budget:?}).max_output_tokens"
    );
}
