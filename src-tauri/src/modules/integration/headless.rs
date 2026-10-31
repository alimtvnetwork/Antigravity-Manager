use crate::models::Account;

use super::*;

/// Headless/Docker implementation: only perform data layer operations, ignore UI and process control
pub struct HeadlessIntegration;

impl SystemIntegration for HeadlessIntegration {
    async fn on_account_switch(
        &self,
        account: &crate::models::Account,
        target_ide: Option<&str>,
    ) -> Result<(), String> {
        if target_ide == Some("agy") {
            return Err(
                "Switching to the agy CLI is not supported in headless mode (no host keyring access)."
                    .to_string(),
            );
        }

        crate::modules::logger::log_info(&format!(
            "[Headless] Delegating account switch for '{}' to DesktopIntegration without GUI handle",
            account.email
        ));
        let desktop = DesktopIntegration { app_handle: None };
        desktop.on_account_switch(account, target_ide).await
    }

    fn update_tray(&self) {
        // No-op
    }

    fn show_notification(&self, title: &str, body: &str) {
        crate::modules::logger::log_info(&format!("[Log Notification] {}: {}", title, body));
    }
}
