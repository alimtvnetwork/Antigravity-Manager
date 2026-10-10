//! OAuth：授权码交换、用户信息、账号添加。

use super::TokenManager;

impl TokenManager {
    /// 使用 Authorization Code 交换 Refresh Token (Web OAuth)
    pub async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<String, String> {
        crate::modules::oauth::exchange_code(code, redirect_uri)
            .await
            .and_then(|t| {
                t.refresh_token
                    .ok_or_else(|| "No refresh token returned by Google".to_string())
            })
    }

    /// 获取 OAuth URL (支持自定义 Redirect URI)
    pub fn get_oauth_url_with_redirect(&self, redirect_uri: &str, state: &str) -> String {
        crate::modules::oauth::get_auth_url(redirect_uri, state)
    }

    /// 获取用户信息 (Email 等)
    pub async fn get_user_info(
        &self,
        refresh_token: &str,
    ) -> Result<crate::modules::oauth::UserInfo, String> {
        // 先获取 Access Token
        let token = crate::modules::oauth::refresh_access_token(refresh_token, None)
            .await
            .map_err(|e| format!("刷新 Access Token 失败: {}", e))?;

        crate::modules::oauth::get_user_info(&token.access_token, None).await
    }

    /// 添加新账号 (纯后端实现，不依赖 Tauri AppHandle)
    pub async fn add_account(&self, email: &str, refresh_token: &str) -> Result<(), String> {
        // 1. 获取 Access Token (验证 refresh_token 有效性)
        let token_info = crate::modules::oauth::refresh_access_token(refresh_token, None)
            .await
            .map_err(|e| format!("Invalid refresh token: {}", e))?;

        // 2. 获取项目 ID (Project ID)
        let project_id = crate::proxy::project_resolver::fetch_project_id(&token_info.access_token)
            .await
            .unwrap_or_else(|_| "bamboo-precept-lgxtn".to_string()); // Fallback

        // 3. 委托给 modules::account::add_account 处理 (包含文件写入、索引更新、锁)
        let email_clone = email.to_string();
        let refresh_token_clone = refresh_token.to_string();

        tokio::task::spawn_blocking(move || {
            let token_data = crate::models::TokenData::new(
                token_info.access_token,
                refresh_token_clone,
                token_info.expires_in,
                Some(email_clone.clone()),
                Some(project_id),
                None,  // session_id
                false, // 默认不开启
                token_info.id_token,
            )
            .with_oauth_client_key(token_info.oauth_client_key.clone());

            crate::modules::account::upsert_account(email_clone, None, token_data)
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
        .map_err(|e| format!("Failed to save account: {}", e))?;

        // 4. 重新加载 (更新内存)
        self.reload_all_accounts().await.map(|_| ())
    }
}
