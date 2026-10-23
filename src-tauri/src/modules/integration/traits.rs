use crate::models::Account;

use super::*;

#[allow(async_fn_in_trait)]
pub trait SystemIntegration: Send + Sync {
    /// System layer operations performed when switching accounts (such as killing processes, writing files, injecting into databases)
    async fn on_account_switch(
        &self,
        account: &crate::models::Account,
        target_ide: Option<&str>,
    ) -> Result<(), String>;

    /// Update system tray (if applicable)
    fn update_tray(&self);

    /// Send system notification
    fn show_notification(&self, title: &str, body: &str);
}
