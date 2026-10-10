use super::*;

pub(crate) fn resolve_switch_context(
    new_email: &str,
    instance_spec: &str,
) -> (String, String, String, String) {
    let mut old_email = String::new();
    if let Ok(mut guard) = PREVIOUS_EMAIL_STATE.lock() {
        if let Some(prev) = guard.take() {
            if !prev.is_empty() {
                old_email = prev;
            }
        }
    }

    let mut inst_id = instance_spec.to_string();
    let mut inst_name = instance_spec.to_string();
    let mut is_default = instance_spec.eq_ignore_ascii_case("default");

    if let Ok(registry) = crate::modules::instance::load_registry() {
        let found = registry
            .instances
            .iter()
            .find(|i| {
                i.id.eq_ignore_ascii_case(instance_spec)
                    || i.name.eq_ignore_ascii_case(instance_spec)
            })
            .or_else(|| {
                registry
                    .instances
                    .iter()
                    .find(|i| i.id == registry.active_instance_id)
            });

        if let Some(inst) = found {
            inst_id = inst.id.clone();
            inst_name = inst.name.clone();
            is_default = inst.is_default || inst.id == "default";

            if old_email.is_empty() || old_email.eq_ignore_ascii_case(new_email) {
                if let Some(ref b_email) = inst.bound_email {
                    if !b_email.is_empty() && !b_email.eq_ignore_ascii_case(new_email) {
                        old_email = b_email.clone();
                    }
                }
                if old_email.is_empty() || old_email.eq_ignore_ascii_case(new_email) {
                    if let Some(ref b_id) = inst.bound_account_id {
                        if let Ok(acc) = crate::modules::account::load_account(b_id) {
                            if !acc.email.is_empty() && !acc.email.eq_ignore_ascii_case(new_email) {
                                old_email = acc.email;
                            }
                        }
                    }
                }
            }
        }
    }

    if old_email.is_empty() || old_email.eq_ignore_ascii_case(new_email) {
        if let Ok(Some(cur_acc)) = crate::modules::account::get_current_account() {
            if !cur_acc.email.is_empty() && !cur_acc.email.eq_ignore_ascii_case(new_email) {
                old_email = cur_acc.email;
            }
        }
    }

    if old_email.trim().eq_ignore_ascii_case(new_email.trim()) {
        old_email.clear();
    }

    if !new_email.trim().is_empty() {
        if let Ok(mut guard) = PREVIOUS_EMAIL_STATE.lock() {
            *guard = Some(new_email.trim().to_string());
        }
    }

    let instance_mode = if is_default {
        "default".to_string()
    } else {
        "isolated".to_string()
    };

    (old_email, inst_id, inst_name, instance_mode)
}

#[derive(Debug, Clone, Default)]
pub struct SwitchNotificationDetails {
    pub previous_email: Option<String>,
    pub previous_quota_4h: Option<f64>,
    pub previous_quota_weekly: Option<f64>,
    pub predicted_next_email: Option<String>,
    pub selected_email: String,
    pub target_quota_4h: Option<f64>,
    pub target_quota_weekly: Option<f64>,
    pub credit_before_switch: Option<f64>,
    pub threshold_activated: Option<f64>,
    pub instance_id: String,
    pub instance_name: String,
    pub instance_mode: String,
    pub reason: String,
    pub is_auto: bool,
    pub backed_up_projects: Vec<String>,
    pub backed_up_prompts_count: Option<usize>,
    pub restored_prompts_count: Option<usize>,
}
