// Dynamic model forwarding rules (split from model_mapping.rs)
use dashmap::DashMap;
use once_cell::sync::Lazy;

pub static DYNAMIC_MODEL_FORWARDING_RULES: Lazy<DashMap<String, String>> =
    Lazy::new(|| DashMap::new());

pub fn update_dynamic_forwarding_rules(old_model: String, new_model: String) {
    if !DYNAMIC_MODEL_FORWARDING_RULES.contains_key(&old_model) {
        crate::modules::logger::log_info(&format!(
            "[Mapping] Registered automatic forwarding rule: {} -> {}",
            old_model, new_model
        ));
    }
    DYNAMIC_MODEL_FORWARDING_RULES.insert(old_model, new_model);
}
