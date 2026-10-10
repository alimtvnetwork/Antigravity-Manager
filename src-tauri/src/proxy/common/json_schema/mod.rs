//! JSON Schema cleaning/normalization (split from json_schema.rs).
//! Facade: implementation lives in submodules, each <= 500 lines.

pub mod args;
pub mod clean;
pub mod recursive;
pub mod refs;

#[cfg(test)]
mod tests_args;
#[cfg(test)]
mod tests_clean;

pub(crate) const MAX_RECURSION_DEPTH: usize = 10;
pub const MAX_DESCRIPTION_LENGTH: usize = 8192;

pub use args::{fix_single_arg_recursive, fix_tool_call_args};
pub use clean::{
    clean_json_schema, clean_json_schema_for_tool, clean_response_schema, sanitize_description,
};
pub use recursive::clean_json_schema_recursive;
