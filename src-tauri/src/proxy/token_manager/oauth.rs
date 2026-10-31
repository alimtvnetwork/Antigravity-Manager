//! OAuth: authorization code exchange, user information, account addition.

use super::TokenManager;

impl TokenManager {
    /// Use Authorization Code to exchange Refresh Token (Web OAuth)
    pub async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<String, String> {
        crate::modules::oauth::exchange_code(code, redirect_uri)
            .await
            .and_then(|t| {
                t.refresh_token
                    .ok_or_else(|| "No refresh token returned by Google".to_string())
            })
    }

    /// Get OAuth URL (supports custom Redirect URI)
    pub fn get_oauth_url_with_redirect(&self, redirect_uri: &str, state: &str) -> String {
        crate::modules::oauth::get_auth_url(redirect_uri, state)
    }

    /// Get user information (Email, etc.)
    pub async fn get_user_info(
        &self,
        refresh_token: &str,
    ) -> Result<crate::modules::oauth::UserInfo, String> {
        // Get Access Token first
        let token = crate::modules::oauth::refresh_access_token(refresh_token, None)
            .await
            .map_err(|e| format!("Failed to refresh Access Token: {}", e))?;

        crate::modules::oauth::get_user_info(&token.access_token, None).await
    }

    /// Add new account (pure backend implementation, does not rely on Tauri AppHandle)
    pub async fn add_account(&self, email: &str, refresh_token: &str) -> Result<(), String> {
        // 1. Obtain Access Token (verify the validity of refresh_token)
        let token_info = crate::modules::oauth::refresh_access_token(refresh_token, None)
            .await
            .map_err(|e| format!("Invalid refresh token: {}", e))?;

        // 2. Get project ID
        let project_id = crate::proxy::project_resolver::fetch_project_id(&token_info.access_token)
            .await
            .unwrap_or_else(|_| "bamboo-precept-lgxtn".to_string()); // Fallback

        // 3. Delegate processing to modules::account::add_account (including file writing, index update, and locking)
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
                false, // Not enabled by default
                token_info.id_token,
            )
            .with_oauth_client_key(token_info.oauth_client_key.clone());

            crate::modules::account::upsert_account(email_clone, None, token_data)
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
        .map_err(|e| format!("Failed to save account: {}", e))?;

        // 4. Reload (update memory)
        self.reload_all_accounts().await.map(|_| ())
    }
}
