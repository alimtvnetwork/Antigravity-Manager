use super::*;

/// 端点降级尝试的记录信息
#[derive(Debug, Clone)]
pub struct FallbackAttemptLog {
    /// 尝试的端点 URL
    pub endpoint_url: String,
    /// HTTP 状态码 (网络错误时为 None)
    pub status: Option<u16>,
    /// 错误描述
    pub error: String,
}

/// 上游调用结果，包含响应和降级尝试记录
pub struct UpstreamCallResult {
    /// 最终的 HTTP 响应
    pub response: Response,
    /// 降级过程中失败的端点尝试记录 (成功时为空)
    pub fallback_attempts: Vec<FallbackAttemptLog>,
}

/// 邮箱脱敏：只显示前3位 + *** + @域名前2位 + ***
/// 例: "userexample@gmail.com" → "use***@gm***"
pub fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let local = &email[..at_pos];
        let domain = &email[at_pos + 1..];
        let local_prefix: String = local.chars().take(3).collect();
        let domain_prefix: String = domain.chars().take(2).collect();
        format!("{}***@{}***", local_prefix, domain_prefix)
    } else {
        // 不是合法邮箱格式，直接截取前5位
        let prefix: String = email.chars().take(5).collect();
        format!("{}***", prefix)
    }
}

/// [NEW] 错误日志脱敏：抹除报错信息中的 access_token, proxy_url 等敏感凭证
pub fn sanitize_error_for_log(error_text: &str) -> String {
    // 抹除常见敏感 key 的值
    let re = regex::Regex::new(r#"(?i)(access_token|refresh_token|id_token|authorization|api_key|secret|password|proxy_url|http_proxy|https_proxy)\s*[:=]\s*[^"'\\\s,}\]]+"#).unwrap();
    let redacted = re.replace_all(error_text, "$1=<redacted>");

    // 抹除 Bearer token
    let re_bearer = regex::Regex::new(r#"(?i)(bearer\s+)[^"'\\\s,}\]]+"#).unwrap();
    let redacted = re_bearer.replace_all(&redacted, "$1<redacted>");

    // 限制长度防止日志炸弹 (UTF-8 字符边界安全保护)
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
// Daily 优先 —— 它是官方 IDE 原生唯一主力端点（`language_server` 的启动参数即指向它），
// 稳定支持思维链与工具调用；Sandbox 为沙箱备用、Prod 为生产兜底，
// 后两者均易触发 Prod 环境的 429（Ref: Issue #1176, Issue #3523）。
//
// [FIX Issue #3525 / PR #3526] 顺序同时是**正确性**要求，不只是可用性偏好：
// 同一账号 / 模型 / 代理下，Sandbox 对部分地区返回**终止性 400**
// `User location is not supported for the API use.`，而 `should_try_next_endpoint`
// 只对 408 / 404 / 5xx 回退 → 400 不触发回退，故 Sandbox 排首位会让已验证可用的
// Daily 永远不被尝试。Sandbox 保留为回退项、回退判定规则不变 ——
// **不对所有 400 无条件重试**。

const V1_INTERNAL_BASE_URL_PROD: &str = "https://cloudcode-pa.googleapis.com/v1internal";
const V1_INTERNAL_BASE_URL_DAILY: &str = "https://daily-cloudcode-pa.googleapis.com/v1internal";
const V1_INTERNAL_BASE_URL_SANDBOX: &str =
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal";

pub(crate) const V1_INTERNAL_BASE_URL_FALLBACKS: [&str; 3] = [
    V1_INTERNAL_BASE_URL_DAILY, // 优先级 1: Daily (官方 IDE 原生唯一主力端点，稳定支持思维链与工具调用)
    V1_INTERNAL_BASE_URL_SANDBOX, // 优先级 2: Sandbox (沙箱备用；部分地区对合规账号返回终止性 400)
    V1_INTERNAL_BASE_URL_PROD,  // 优先级 3: Prod (生产兜底，易触发 429)
];

pub struct UpstreamClient {
    pub(crate) default_client: RwLock<Client>,
    pub(crate) proxy_pool: Option<Arc<crate::proxy::proxy_pool::ProxyPoolManager>>,
    pub(crate) client_cache: DashMap<String, Client>, // proxy_id -> Client
    pub(crate) user_agent_override: RwLock<Option<String>>,
}
