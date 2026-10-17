use std::fmt;

use super::*;

/// Canonical identifier for the default shared instance.
pub const DEFAULT_INSTANCE_ID: &str = "default";

/// Legacy internal identifier for the default instance preserved for backward database compatibility.
pub const LEGACY_DEFAULT_INSTANCE_ID: &str = "__default__";

/// Determine whether the given instance identifier designates the default instance.
/// Matches "default", "__default__", or an empty / whitespace-only string.
#[inline]
pub fn is_default_instance(id: &str) -> bool {
    let trimmed = id.trim();
    trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case(DEFAULT_INSTANCE_ID)
        || trimmed == LEGACY_DEFAULT_INSTANCE_ID
}

/// Normalize an arbitrary instance identifier: returns "default" if it represents
/// the default instance, or the trimmed identifier otherwise.
#[inline]
pub fn normalize_instance_id<'a>(id: &'a str) -> &'a str {
    if is_default_instance(id) {
        DEFAULT_INSTANCE_ID
    } else {
        id.trim()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceModeType {
    Default,
    Isolated,
}

impl InstanceModeType {
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Isolated => "isolated",
        }
    }

    #[inline]
    pub const fn from_bool(is_default: bool) -> Self {
        if is_default {
            Self::Default
        } else {
            Self::Isolated
        }
    }

    #[inline]
    pub const fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

impl fmt::Display for InstanceModeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

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
    let mut is_default = is_default_instance(instance_spec);

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
            is_default = inst.is_default || is_default_instance(&inst.id);

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

    let instance_mode = InstanceModeType::from_bool(is_default);

    (
        old_email,
        inst_id,
        inst_name,
        instance_mode.as_str().to_string(),
    )
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

/// Resolve the current node's physical and instance identity for telemetry broadcasting
pub fn resolve_current_telemetry_identity() -> (String, String, String, String) {
    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| crate::modules::email_watcher::detect_machine_name());
    let ip = crate::modules::email_watcher::detect_local_ip();

    let mut instance_id = DEFAULT_INSTANCE_ID.to_string();
    let mut bound_email = String::new();

    if let Ok(registry) = crate::modules::instance::load_registry() {
        let active = registry
            .instances
            .iter()
            .find(|i| i.id == registry.active_instance_id)
            .or_else(|| {
                registry
                    .instances
                    .iter()
                    .find(|i| i.is_default || is_default_instance(&i.id))
            })
            .or_else(|| registry.instances.first());

        if let Some(inst) = active {
            instance_id = inst.id.clone();
            if let Some(ref email) = inst.bound_email {
                if !email.is_empty() {
                    bound_email = email.clone();
                }
            }
        }
    }

    if bound_email.is_empty() {
        if let Ok(Some(acc)) = crate::modules::account::get_current_account() {
            bound_email = acc.email;
        } else if let Ok(idx) = crate::modules::account::load_account_index() {
            if let Some(cid) = idx.current_account_id {
                if let Some(acc) = idx.accounts.into_iter().find(|a| a.id == cid) {
                    bound_email = acc.email;
                }
            }
        }
    }

    if bound_email.is_empty() {
        bound_email = "none".to_string();
    }

    (hostname, ip, instance_id, bound_email)
}

/// Format standardized Telemetry Header in Telegram HTML format
pub fn format_telemetry_header_telegram_html(
    hostname: &str,
    ip: &str,
    instance_id: &str,
    bound_email: &str,
) -> String {
    format!(
        "🖥️ <b>Machine:</b> <code>{} ({})</code> | 🏷️ <b>Instance:</b> <code>{}</code> | 👤 <b>Bound:</b> <code>{}</code>",
        crate::modules::telegram_inbound::clean_for_telegram_html(hostname, 48),
        crate::modules::telegram_inbound::clean_for_telegram_html(ip, 48),
        crate::modules::telegram_inbound::clean_for_telegram_html(instance_id, 32),
        crate::modules::telegram_inbound::clean_for_telegram_html(bound_email, 64)
    )
}
