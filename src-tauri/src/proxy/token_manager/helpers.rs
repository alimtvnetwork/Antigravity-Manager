//! 内部辅助函数（账号 JSON 更新、图片调度等待、限流原因分类等）。

pub(crate) fn classify_rate_limit_reason(
    error_body: &str,
) -> crate::proxy::rate_limit::RateLimitReason {
    use crate::proxy::rate_limit::RateLimitReason;

    let body = error_body.to_lowercase();
    let generic_resource_exhausted =
        body.contains("resource has been exhausted") || body.contains("resource_exhausted");
    let explicit_quota_exhausted = body.contains("quota_exhausted")
        || body.contains("quotaresetdelay")
        || body.contains("quota reset")
        || body.contains("quota limit")
        || body.contains("per day")
        || body.contains("daily quota")
        || body.contains("credits");

    if body.contains("model_capacity") {
        RateLimitReason::ModelCapacityExhausted
    } else if body.contains("per minute")
        || body.contains("rate limit")
        || body.contains("too many requests")
        || (generic_resource_exhausted && !explicit_quota_exhausted)
    {
        RateLimitReason::RateLimitExceeded
    } else if explicit_quota_exhausted || body.contains("exhausted") || body.contains("quota") {
        RateLimitReason::QuotaExhausted
    } else {
        RateLimitReason::Unknown
    }
}

const IMAGE_ACCOUNT_RESELECT_INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);

pub(crate) async fn wait_for_image_account_change(
    changes: &mut tokio::sync::watch::Receiver<u64>,
    remaining: std::time::Duration,
) -> bool {
    if remaining.is_zero() {
        return false;
    }
    tokio::select! {
        result = changes.changed() => result.is_ok(),
        _ = tokio::time::sleep(remaining.min(IMAGE_ACCOUNT_RESELECT_INTERVAL)) => true,
    }
}

pub(crate) async fn wait_for_image_token_selection<T>(
    deadline: tokio::time::Instant,
    selection: impl std::future::Future<Output = T>,
) -> Option<T> {
    let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
    if remaining.is_zero() {
        return None;
    }
    tokio::time::timeout(remaining, selection).await.ok()
}

/// 异步安全的账号 JSON 更新函数
///
/// 使用 `tokio::task::spawn_blocking` 将阻塞的文件 I/O 与 `std::sync::Mutex`
/// 的获取操作转移到 Tokio 的阻塞线程池中，避免占用 Tokio Worker Thread，
/// 防止高并发场景下因同步锁争抢导致 Tokio 运行时饥饿（runtime starvation）。
pub(crate) async fn update_account_json(
    path: &std::path::Path,
    update: impl FnOnce(&mut serde_json::Value) + Send + 'static,
) -> Result<(), String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let _account_write = crate::modules::account::lock_account_file_updates()?;
        let raw = std::fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {}", e))?;
        let mut content: serde_json::Value =
            serde_json::from_str(&raw).map_err(|e| format!("解析 JSON 失败: {}", e))?;
        update(&mut content);
        let serialized = serde_json::to_string_pretty(&content)
            .map_err(|e| format!("序列化 JSON 失败: {}", e))?;
        std::fs::write(&path, serialized).map_err(|e| format!("写入文件失败: {}", e))
    })
    .await
    .map_err(|e| format!("spawn_blocking panicked: {}", e))?
}

pub(crate) fn unix_timestamp_ceil(time: std::time::SystemTime) -> Option<i64> {
    let since_epoch = time
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .ok()?;
    let seconds = since_epoch
        .as_secs()
        .saturating_add(u64::from(since_epoch.subsec_nanos() > 0));
    i64::try_from(seconds).ok()
}

/// 截断过长的原因字符串
pub(crate) fn truncate_reason(reason: &str, max_len: usize) -> String {
    if reason.len() <= max_len {
        reason.to_string()
    } else {
        let budget = max_len.saturating_sub(3);
        let end = crate::proxy::mappers::common_utils::safe_truncate_str(reason, budget);
        format!("{}...", end)
    }
}
