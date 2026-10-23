use super::*;

/// Logging information for endpoint downgrade attempts
#[derive(Debug, Clone)]
pub struct FallbackAttemptLog {
    /// Endpoint URL to try
    pub endpoint_url: String,
    /// HTTP status code (None for network error)
    pub status: Option<u16>,
    /// Error description
    pub error: String,
}

/// Upstream call results, including response and downgrade attempt records
pub struct UpstreamCallResult {
    /// final HTTP response
    pub response: Response,
    /// Log of failed endpoint attempts during downgrade (empty on success)
    pub fallback_attempts: Vec<FallbackAttemptLog>,
}

/// Email desensitization: display the first 4 digits + ***, completely hide the @gmail / @googlemail domain name
/// Example: "userexample@gmail.com" → "user***"
pub fn mask_email(email: &str) -> String {
    if let Some((local, domain)) = email.split_once('@') {
        let prefix: String = local.chars().take(4).collect();
        let lower_domain = domain.to_ascii_lowercase();
        if lower_domain.contains("gmail") || lower_domain.contains("googlemail") {
            format!("{prefix}***")
        } else {
            format!("{prefix}***@{domain}")
        }
    } else {
        // It is not a legal email format, the first 4 digits are intercepted.
        let prefix: String = email.chars().take(4).collect();
        format!("{prefix}***")
    }
}

/// [NEW] Error log desensitization: Erase sensitive credentials such as access_token and proxy_url in the error message
pub fn sanitize_error_for_log(error_text: &str) -> String {
    // Erase the value of common sensitive keys
    let re = regex::Regex::new(r#"(?i)(access_token|refresh_token|id_token|authorization|api_key|secret|password|proxy_url|http_proxy|https_proxy)\s*[:=]\s*[^"'\\\s,}\]]+"#).unwrap();
    let redacted = re.replace_all(error_text, "$1=<redacted>");

    // Erase Bearer token
    let re_bearer = regex::Regex::new(r#"(?i)(bearer\s+)[^"'\\\s,}\]]+"#).unwrap();
    let redacted = re_bearer.replace_all(&redacted, "$1<redacted>");

    // Limit length to prevent log bombs (UTF-8 character boundary security protection)
    if redacted.len() > 1000 {
        format!(
            "{}... (truncated)",
            crate::proxy::mappers::common_utils::safe_truncate_str(&redacted, 1000)
        )
    } else {
        redacted.into_owned()
    }
}

// Cloud Code v1internal endpoints (fallback order: Daily → Sandbox → Prod)
//
// Daily priority - it is the only main endpoint native to the official IDE (the startup parameter of `language_server` points to it),
// Stable support for thought chains and tool calls; Sandbox is a backup sandbox, and Prod is a backup for production.
// The latter two are prone to triggering 429 of the Prod environment (Ref: Issue #1176, Issue #3523).
//
// [FIX Issue #3525 / PR #3526] Order is both a correctness requirement, not just a usability preference:
// Under the same account/model/agent, Sandbox returns **terminability 400** in some regions.
// `User location is not supported for the API use.` and `should_try_next_endpoint`
// Only rollback for 408 / 404 / 5xx → 400 does not trigger rollback, so the Sandbox will be ranked first and has been verified to be available.
// Daily is never attempted. Sandbox remains as a rollback item, and the rollback determination rules remain unchanged——
// **Do not retry unconditionally on all 400**.

const V1_INTERNAL_BASE_URL_PROD: &str = "https://cloudcode-pa.googleapis.com/v1internal";
const V1_INTERNAL_BASE_URL_DAILY: &str = "https://daily-cloudcode-pa.googleapis.com/v1internal";
const V1_INTERNAL_BASE_URL_SANDBOX: &str =
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal";

pub(crate) const V1_INTERNAL_BASE_URL_FALLBACKS: [&str; 3] = [
    V1_INTERNAL_BASE_URL_DAILY, // Priority 1: Daily (the only main endpoint native to the official IDE, stably supports thought chains and tool calls)
    V1_INTERNAL_BASE_URL_SANDBOX, // Priority 2: Sandbox (sandbox backup; some regions return termination 400 to compliant accounts)
    V1_INTERNAL_BASE_URL_PROD,    // Priority 3: Prod (Production is low, easy to trigger 429)
];

pub struct UpstreamClient {
    pub(crate) default_client: RwLock<Client>,
    pub(crate) proxy_pool: Option<Arc<crate::proxy::proxy_pool::ProxyPoolManager>>,
    pub(crate) client_cache: DashMap<String, Client>, // proxy_id -> Client
    pub(crate) user_agent_override: RwLock<Option<String>>,
}
