// Model output Token quota management (DEPRECATED: logic has been migrated to crate::proxy::model_specs)
// For compatibility, this entry is retained and redirected to model_specs.

use crate::proxy::model_specs;

/// Get the output token limit of the model
///
/// # Parameters
/// - `model_name`: mapped model name
/// - `dynamic_limit`: passed in if the dynamic limit is known (deprecated, it is recommended to pass the ProxyToken directly to model_specs)
#[allow(dead_code)]
pub fn get_model_output_limit(model_name: &str, dynamic_limit: Option<u64>) -> u64 {
    // Compatible logic: If there is no dynamic_limit, call model_specs to obtain it (no token is currently passed in)
    if let Some(limit) = dynamic_limit {
        limit
    } else {
        model_specs::get_max_output_tokens(model_name, None)
    }
}
