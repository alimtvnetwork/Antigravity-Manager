use crate::modules::oauth;
use tauri::Url;
use tokio::sync::mpsc;
use tokio::sync::watch;

use super::*;

/// Start OAuth flow and wait for callback, then exchange token
pub async fn start_oauth_flow(
    app_handle: Option<tauri::AppHandle>,
    oauth_client_key: Option<String>,
) -> Result<oauth::TokenResponse, String> {
    // Ensure URL + listener are ready (this way if the user authorizes first, it won't get stuck)
    let auth_url = ensure_oauth_flow_prepared(app_handle.clone(), oauth_client_key).await?;

    if let Some(h) = app_handle {
        // Open default browser
        use tauri_plugin_opener::OpenerExt;
        h.opener()
            .open_url(&auth_url, None::<String>)
            .map_err(|e| format!("failed_to_open_browser: {}", e))?;
    }

    // Take code_rx to wait for it
    let (mut code_rx, redirect_uri, client_key) = {
        let mut lock = get_oauth_flow_state()
            .lock()
            .map_err(|_| "OAuth state lock corrupted".to_string())?;
        let Some(state) = lock.as_mut() else {
            return Err("OAuth state does not exist".to_string());
        };
        let rx = state
            .code_rx
            .take()
            .ok_or_else(|| "OAuth authorization already in progress".to_string())?;
        (rx, state.redirect_uri.clone(), state.client_key.clone())
    };

    // Wait for code (if user has already authorized, this returns immediately)
    // For mpsc, we use recv()
    let code = match code_rx.recv().await {
        Some(Ok(code)) => code,
        Some(Err(e)) => return Err(e),
        None => return Err("OAuth flow channel closed unexpectedly".to_string()),
    };

    // Clean up flow state (release cancel_tx, etc.)
    if let Ok(mut lock) = get_oauth_flow_state().lock() {
        *lock = None;
    }

    oauth::exchange_code_with_client(&code, &redirect_uri, Some(&client_key)).await
}

/// Завершить OAuth flow без открытия браузера.
/// Предполагается, что пользователь открыл ссылку вручную (или ранее была открыта),
/// а мы только ждём callback и обмениваем code на token.
pub async fn complete_oauth_flow(
    app_handle: Option<tauri::AppHandle>,
) -> Result<oauth::TokenResponse, String> {
    // Ensure URL + listeners exist
    let _ = ensure_oauth_flow_prepared(app_handle, None).await?;

    // Take receiver to wait for code
    let (mut code_rx, redirect_uri, client_key) = {
        let mut lock = get_oauth_flow_state()
            .lock()
            .map_err(|_| "OAuth state lock corrupted".to_string())?;
        let Some(state) = lock.as_mut() else {
            return Err("OAuth state does not exist".to_string());
        };
        let rx = state
            .code_rx
            .take()
            .ok_or_else(|| "OAuth authorization already in progress".to_string())?;
        (rx, state.redirect_uri.clone(), state.client_key.clone())
    };

    let code = match code_rx.recv().await {
        Some(Ok(code)) => code,
        Some(Err(e)) => return Err(e),
        None => return Err("OAuth flow channel closed unexpectedly".to_string()),
    };

    if let Ok(mut lock) = get_oauth_flow_state().lock() {
        *lock = None;
    }

    oauth::exchange_code_with_client(&code, &redirect_uri, Some(&client_key)).await
}

/// Manually submit an OAuth code to complete the flow.
/// This is used when the user manually copies the code/URL from the browser
/// because the localhost callback couldn't be reached (e.g. in Docker/remote).
pub async fn submit_oauth_code(
    code_input: String,
    state_input: Option<String>,
) -> Result<(), String> {
    let tx = {
        let lock = get_oauth_flow_state().lock().map_err(|e| e.to_string())?;
        if let Some(state) = lock.as_ref() {
            // Verify state if provided
            if let Some(provided_state) = state_input {
                if provided_state != state.state {
                    return Err("OAuth state mismatch (CSRF protection)".to_string());
                }
            }
            state.code_tx.clone()
        } else {
            return Err("No active OAuth flow found".to_string());
        }
    };

    // Extract code if it's a URL
    let code = if code_input.starts_with("http") {
        if let Ok(url) = Url::parse(&code_input) {
            url.query_pairs()
                .find(|(k, _)| k == "code")
                .map(|(_, v)| v.to_string())
                .unwrap_or(code_input)
        } else {
            code_input
        }
    } else {
        code_input
    };

    crate::modules::logger::log_info("Received manual OAuth code submission");

    // Send to the channel
    tx.send(Ok(code))
        .await
        .map_err(|_| "Failed to send code to OAuth flow (receiver dropped)".to_string())?;

    Ok(())
}

/// Manually prepare an OAuth flow without starting listeners.
/// Useful for Web/Docker environments where we only need manual code submission.
pub fn prepare_oauth_flow_manually(
    redirect_uri: String,
    state_str: String,
    oauth_client_key: Option<String>,
) -> Result<(String, mpsc::Receiver<Result<String, String>>), String> {
    let (auth_url, resolved_client_key) =
        oauth::get_auth_url_with_client(&redirect_uri, &state_str, oauth_client_key.as_deref())?;

    // Check if we can reuse existing state
    if let Ok(mut lock) = get_oauth_flow_state().lock() {
        if let Some(s) = lock.as_mut() {
            // If we already have a code_rx, we can't easily "steal" it again because it's already returned.
            // But if this is a NEW request (different state), we should overwrite.
            // For now, let's just clear and restart to be safe.
            // Justification: best-effort channel notification; the receiver may already be gone during shutdown
            crate::error::record_ignored(s.cancel_tx.send(true), "channel send");
            *lock = None;
        }
    }

    let (cancel_tx, _cancel_rx) = watch::channel(false);
    let (code_tx, code_rx) = mpsc::channel(1);

    if let Ok(mut state) = get_oauth_flow_state().lock() {
        *state = Some(OAuthFlowState {
            auth_url: auth_url.clone(),
            redirect_uri: redirect_uri.clone(),
            state: state_str,
            client_key: resolved_client_key,
            cancel_tx,
            code_tx,
            code_rx: None, // We return it directly
        });
    }

    Ok((auth_url, code_rx))
}
