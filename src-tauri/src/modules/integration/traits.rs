use crate::models::Account;

use super::*;

#[allow(async_fn_in_trait)]
pub trait SystemIntegration: Send + Sync {
    /// 当切换账号时执行的系统层操作（如杀进程、写入文件、注入数据库）
    async fn on_account_switch(
        &self,
        account: &crate::models::Account,
        target_ide: Option<&str>,
    ) -> Result<(), String>;

    /// 更新系统托盘（如果适用）
    fn update_tray(&self);

    /// 发送系统通知
    fn show_notification(&self, title: &str, body: &str);
}
