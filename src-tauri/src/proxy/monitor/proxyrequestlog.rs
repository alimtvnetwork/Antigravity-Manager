use super::*;

// Admission happens before spawn_blocking, so queued tasks cannot retain unlimited bodies.
static LOG_WRITERS: Semaphore = Semaphore::const_new(4);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRequestLog {
    pub id: String,
    pub timestamp: i64,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub duration: u64,                // ms
    pub model: Option<String>,        // 客户端请求的模型名
    pub mapped_model: Option<String>, // 实际路由后使用的模型名
    pub account_email: Option<String>,
    pub client_ip: Option<String>, // 客户端 IP 地址
    pub error: Option<String>,
    pub request_body: Option<String>,
    pub upstream_request_body: Option<String>, // Forwarded request body sent upstream to Antigravity
    pub response_body: Option<String>,
    #[serde(default)]
    pub request_headers: Option<String>,
    #[serde(default)]
    pub upstream_request_headers: Option<String>,
    #[serde(default)]
    pub response_headers: Option<String>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub cached_tokens: Option<u32>,
    pub protocol: Option<String>, // 协议类型: "openai", "anthropic", "gemini"
    pub username: Option<String>, // User token username
    #[serde(default)]
    pub session_id: Option<String>, // 会话标识 (如 sess-xxx)
}

#[cfg(test)]
pub(crate) mod prompt_log_tests {
    use super::*;

    pub(crate) fn sample_log(id: &str, bytes: usize) -> ProxyRequestLog {
        serde_json::from_value(serde_json::json!({
            "id": id, "timestamp": chrono::Utc::now().timestamp_millis(),
            "method": "POST", "url": "/v1/chat/completions", "status": 500, "duration": 10,
            "request_body": "q".repeat(bytes), "response_body": "错".repeat(bytes),
            "error": "错".repeat(2048)
        }))
        .unwrap()
    }

    // Tests using ABV_DATA_DIR run serially and restore the previous process setting.
    pub(crate) struct TestDataDir {
        _dir: tempfile::TempDir,
        previous: Option<std::ffi::OsString>,
        _guard: Option<std::sync::MutexGuard<'static, ()>>,
    }
    impl TestDataDir {
        pub(crate) fn new() -> Self {
            let guard = crate::modules::account::TEST_DATA_DIR_MUTEX
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            Self::new_with_guard(Some(guard))
        }
        pub(crate) fn new_nested() -> Self {
            Self::new_with_guard(None)
        }
        fn new_with_guard(guard: Option<std::sync::MutexGuard<'static, ()>>) -> Self {
            let dir = tempfile::tempdir().unwrap();
            let previous = std::env::var_os("ABV_DATA_DIR");
            std::env::set_var("ABV_DATA_DIR", dir.path());
            Self {
                _dir: dir,
                previous,
                _guard: guard,
            }
        }
        pub(crate) fn path(&self) -> &std::path::Path {
            self._dir.path()
        }
    }
    impl Drop for TestDataDir {
        fn drop(&mut self) {
            if let Some(previous) = &self.previous {
                std::env::set_var("ABV_DATA_DIR", previous);
            } else {
                std::env::remove_var("ABV_DATA_DIR");
            }
        }
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn prompt_log_memory_summary_and_database_detail() {
        let _dir = TestDataDir::new();
        crate::modules::proxy_db::init_db().unwrap();
        let monitor = ProxyMonitor {
            logs: RwLock::new(VecDeque::new()),
            stats: RwLock::new(ProxyStats::default()),
            max_logs: 2,
            enabled: Arc::new(AtomicBool::new(true)),
            capture_health_logs: Arc::new(AtomicBool::new(false)),
            app_handle: None,
        };
        let log = sample_log("detail", 2048);
        let response = log.response_body.clone();
        monitor.log_request(log).await;
        let _finished = LOG_WRITERS.acquire_many(4).await.unwrap();
        let logs = monitor.logs.read().await;
        assert!(logs[0].request_body.is_none() && logs[0].response_body.is_none());
        assert_eq!(logs[0].error.as_ref().unwrap().chars().count(), 1024);
        let detail = crate::modules::proxy_db::get_log_detail("detail").unwrap();
        assert_eq!(detail.response_body, response);
        assert_eq!(
            detail.request_body.as_deref(),
            Some("q".repeat(2048).as_str())
        );
        monitor.set_enabled(false);
        monitor.log_request(sample_log("disabled", 2048)).await;
        assert!(crate::modules::proxy_db::get_log_detail("disabled").is_err());
    }
}

impl ProxyRequestLog {
    pub(crate) fn summary(&self) -> Self {
        Self {
            id: self.id.clone(),
            timestamp: self.timestamp,
            method: self.method.clone(),
            url: self.url.clone(),
            status: self.status,
            duration: self.duration,
            model: self.model.clone(),
            mapped_model: self.mapped_model.clone(),
            account_email: self.account_email.clone(),
            client_ip: self.client_ip.clone(),
            error: self
                .error
                .as_ref()
                .map(|error| error.chars().take(1024).collect()),
            request_body: None,
            upstream_request_body: None,
            response_body: None,
            request_headers: None,
            upstream_request_headers: None,
            response_headers: None,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            cached_tokens: self.cached_tokens,
            protocol: self.protocol.clone(),
            username: self.username.clone(),
            session_id: self.session_id.clone(),
        }
    }
}

#[derive(Default)]
pub(crate) struct UpstreamCapture {
    body: Option<String>,
    headers: Option<String>,
}

#[derive(Clone, Default)]
pub struct UpstreamRequestBodyHolder(pub std::sync::Arc<std::sync::Mutex<UpstreamCapture>>);

pub static CURRENT_UPSTREAM_CAPTURE: UpstreamRequestBodyHolder;

impl UpstreamRequestBodyHolder {
    pub fn new() -> Self {
        Self(std::sync::Arc::new(std::sync::Mutex::new(
            UpstreamCapture::default(),
        )))
    }

    pub fn set(&self, body: String) {
        if let Ok(mut lock) = self.0.lock() {
            lock.body = Some(body);
        }
    }

    pub fn set_headers_json(&self, headers: String) {
        if let Ok(mut lock) = self.0.lock() {
            lock.headers = Some(headers);
        }
    }

    pub fn set_value(&self, val: &serde_json::Value) {
        let clean = sanitize_upstream_debug_value(val);
        if let Ok(s) = serde_json::to_string(&clean) {
            self.set(s);
        }
    }

    pub fn take(&self) -> Option<String> {
        self.0.lock().ok().and_then(|mut g| g.body.take())
    }

    pub fn take_headers(&self) -> Option<String> {
        self.0.lock().ok().and_then(|mut g| g.headers.take())
    }
}

pub(crate) fn sanitize_upstream_debug_value(val: &serde_json::Value) -> serde_json::Value {
    match val {
        serde_json::Value::String(s)
            if s.starts_with("data:image/") || s.starts_with("data:audio/") =>
        {
            serde_json::Value::String(format!("[inline data omitted: {} chars]", s.len()))
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(sanitize_upstream_debug_value).collect())
        }
        serde_json::Value::Object(map) => {
            let is_inline = map
                .get("mimeType")
                .and_then(serde_json::Value::as_str)
                .is_some()
                && map
                    .get("data")
                    .and_then(serde_json::Value::as_str)
                    .is_some();
            serde_json::Value::Object(
                map.iter()
                    .map(|(k, v)| {
                        let v = if is_inline && k == "data" {
                            serde_json::Value::String(format!(
                                "[inline data omitted: {} chars]",
                                v.as_str().map(str::len).unwrap_or_default()
                            ))
                        } else {
                            sanitize_upstream_debug_value(v)
                        };
                        (k.clone(), v)
                    })
                    .collect(),
            )
        }
        _ => val.clone(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyStats {
    pub total_requests: u64,
    pub success_count: u64,
    pub error_count: u64,
}

pub struct ProxyMonitor {
    pub logs: RwLock<VecDeque<ProxyRequestLog>>,
    pub stats: RwLock<ProxyStats>,
    pub max_logs: usize,
    pub enabled: Arc<AtomicBool>,
    pub capture_health_logs: Arc<AtomicBool>,
    app_handle: Option<tauri::AppHandle>,
}
