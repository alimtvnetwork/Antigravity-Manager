//! Canonical model + variant -> real model ID + real params.
//! Facade: implementation lives in submodules, each <= 500 lines.

pub mod resolve;
pub mod specs;
pub mod types;

#[cfg(test)]
mod test_helpers;
#[cfg(test)]
mod tests_baseline;
#[cfg(test)]
mod tests_unit;

pub use resolve::{
    infer_tier, resolve, resolve_non_variant_model, resolve_real_model, resolve_with_tier,
    tier_from_effort,
};
pub use specs::GEMINI_FAMILIES;
pub use types::{AliasPolicy, CanonicalFamily, RealModelSpec, VariantTier};
