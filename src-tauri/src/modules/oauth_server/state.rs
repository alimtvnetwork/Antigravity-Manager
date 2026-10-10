use std::sync::{Mutex, OnceLock};
use tokio::sync::mpsc;
use tokio::sync::watch;

use super::*;

pub(crate) struct OAuthFlowState {
    pub(crate) auth_url: String,
    #[allow(dead_code)]
    pub(crate) redirect_uri: String,
    pub(crate) state: String,
    pub(crate) client_key: String,
    pub(crate) cancel_tx: watch::Sender<bool>,
    pub(crate) code_tx: mpsc::Sender<Result<String, String>>,
    pub(crate) code_rx: Option<mpsc::Receiver<Result<String, String>>>,
}

pub(crate) static OAUTH_FLOW_STATE: OnceLock<Mutex<Option<OAuthFlowState>>> = OnceLock::new();

pub(crate) fn get_oauth_flow_state() -> &'static Mutex<Option<OAuthFlowState>> {
    OAUTH_FLOW_STATE.get_or_init(|| Mutex::new(None))
}

pub(crate) fn oauth_success_html() -> &'static str {
    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
    <html>\
    <body style='font-family: sans-serif; text-align: center; padding: 50px;'>\
    <h1 style='color: green;'>✅ Authorization Successful!</h1>\
    <p>You can close this window and return to the application.</p>\
    <script>setTimeout(function() { window.close(); }, 2000);</script>\
    </body>\
    </html>"
}

pub(crate) fn oauth_fail_html() -> &'static str {
    "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
    <html>\
    <body style='font-family: sans-serif; text-align: center; padding: 50px;'>\
    <h1 style='color: red;'>❌ Authorization Failed</h1>\
    <p>Failed to obtain Authorization Code. Please return to the app and try again.</p>\
    </body>\
    </html>"
}
