//! Prompt sanitizer pipeline.
//! Facade: implementation lives in submodules, each <= 500 lines.

mod parts;
pub(crate) mod patterns;
mod text;

#[cfg(test)]
mod tests_parts;
#[cfg(test)]
mod tests_text;

pub struct PromptSanitizer;
