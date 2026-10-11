// Facade for the OpenAI handlers. Code lives in the submodules below;
// this module only declares them and re-exports the public API so that
// `crate::proxy::handlers::openai::X` paths keep resolving.
mod chat_completions;
mod chat_completions_error;
mod chat_completions_normalize;
mod chat_completions_send;
mod chat_completions_success;
mod chat_conversion;
mod codex_convert;
mod completions;
mod completions_error;
mod completions_prep_codex;
mod completions_prep_convert;
mod completions_prep_session;
mod completions_prep_setup;
mod completions_send;
mod completions_success;
mod completions_success_nonstream;
mod completions_success_stream;
mod image_input;
mod images_edits;
mod images_edits_dispatch;
mod images_generations;
mod images_intercept;
mod models;
mod responses_history;
mod responses_media;
mod tool_cache;
#[cfg(test)]
mod variant_tests;
mod websocket;
mod websocket_codex;
mod websocket_compress;
mod websocket_finalize;
mod websocket_normalize;
mod websocket_translate;

pub use chat_completions::handle_chat_completions;
pub use completions::handle_completions;
pub use images_edits::handle_images_edits;
pub use images_generations::{handle_images_generations, handle_images_generations_internal};
pub use images_intercept::handle_chat_redirection;
pub use models::handle_list_models;
pub use tool_cache::{get_cached_tool_call, insert_cached_tool_call};
pub use websocket::handle_responses_websocket;
