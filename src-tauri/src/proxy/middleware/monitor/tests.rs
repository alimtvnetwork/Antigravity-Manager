use super::*;

use super::next_chunk_while_receiver_open;
use futures::stream;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::task::Poll;

struct DropFlag(Arc<AtomicBool>);

impl Drop for DropFlag {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn receiver_close_stops_waiting_and_drops_source_stream() {
    let dropped = Arc::new(AtomicBool::new(false));
    let drop_flag = DropFlag(dropped.clone());
    let (polled_tx, polled_rx) = tokio::sync::oneshot::channel();
    let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);

    let task = tokio::spawn(async move {
        let mut polled_tx = Some(polled_tx);
        let mut source = stream::poll_fn(move |_| {
            // Justification: forces the closure to capture drop_flag, keeping the DropFlag alive for the stream's lifetime; not a fallible operation.
            let _ = &drop_flag;
            if let Some(polled_tx) = polled_tx.take() {
                // Justification: oneshot::Sender::send returns Result<(), ()> — the unit error carries no information to log; a dropped receiver is benign here.
                let _ = polled_tx.send(());
            }
            Poll::<Option<()>>::Pending
        });

        assert!(next_chunk_while_receiver_open(&mut source, &tx)
            .await
            .is_none());
    });

    polled_rx.await.expect("source stream was not polled");
    drop(rx);
    tokio::time::timeout(std::time::Duration::from_secs(1), task)
        .await
        .expect("forwarder did not stop after receiver closed")
        .expect("forwarder task panicked");
    assert!(dropped.load(Ordering::SeqCst));
}

#[test]
fn test_extract_tokens_anthropic() {
    use serde_json::json;
    let usage = json!({
        "input_tokens": 1200,
        "cache_read_input_tokens": 8800,
        "cache_creation_input_tokens": 0,
        "output_tokens": 150
    });
    assert_eq!(super::extract_input_tokens(&usage), Some(10000));
    assert_eq!(super::extract_cached_tokens(&usage), Some(8800));
    assert_eq!(super::extract_output_tokens(&usage), Some(150));
}

#[test]
fn test_extract_tokens_openai_chat() {
    use serde_json::json;
    let usage = json!({
        "prompt_tokens": 10000,
        "completion_tokens": 150,
        "total_tokens": 10150,
        "prompt_tokens_details": {
            "cached_tokens": 8800
        }
    });
    assert_eq!(super::extract_input_tokens(&usage), Some(10000));
    assert_eq!(super::extract_cached_tokens(&usage), Some(8800));
    assert_eq!(super::extract_output_tokens(&usage), Some(150));
}

#[test]
fn test_extract_tokens_openai_responses() {
    use serde_json::json;
    let usage = json!({
        "input_tokens": 10000,
        "output_tokens": 150,
        "total_tokens": 10150,
        "input_tokens_details": {
            "cached_tokens": 8800
        }
    });
    assert_eq!(super::extract_input_tokens(&usage), Some(10000));
    assert_eq!(super::extract_cached_tokens(&usage), Some(8800));
    assert_eq!(super::extract_output_tokens(&usage), Some(150));
}

#[test]
fn test_extract_tokens_gemini_raw() {
    use serde_json::json;
    let usage = json!({
        "promptTokenCount": 10000,
        "candidatesTokenCount": 150,
        "totalTokenCount": 10150,
        "cachedContentTokenCount": 8800
    });
    assert_eq!(super::extract_input_tokens(&usage), Some(10000));
    assert_eq!(super::extract_cached_tokens(&usage), Some(8800));
    assert_eq!(super::extract_output_tokens(&usage), Some(150));
}

#[test]
fn test_should_skip_request_log_only_suppresses_successful_get() {
    use axum::http::StatusCode;

    // 胶囊开关关闭（默认）：GET 成功请求一律不记录 —— 含 /v1/models 轮询（本次修复的核心场景）
    // 与 /health 探针，且与路径无关（GET 成功即噪音）
    for path in [
        "/v1/models",
        "/v1/models?limit=100",
        "/v1beta/models",
        "/v1/models/claude",
        "/health",
        "/healthz?t=123",
        "/api/health/",
        "/stats/overview",
    ] {
        for status in [200, 204, 201] {
            assert!(
                super::should_skip_request_log("GET", StatusCode::from_u16(status).unwrap(), false),
                "GET {status} {path} 应被抑制"
            );
        }
    }

    // 大写的 GET 同样识别
    assert!(super::should_skip_request_log("get", StatusCode::OK, false));

    // 失败请求始终记录（供排障）—— 无论路径与方法
    for status in [400, 401, 403, 404, 429, 500, 502, 503, 504] {
        assert!(
            !super::should_skip_request_log("GET", StatusCode::from_u16(status).unwrap(), false),
            "GET {status} 必须记录"
        );
    }

    // 非 GET 业务请求始终记录
    for method in ["POST", "PUT", "PATCH", "DELETE", "post"] {
        assert!(
            !super::should_skip_request_log(method, StatusCode::OK, false),
            "{method} 200 必须记录"
        );
    }

    // 开关开启（或 ABV_LOG_HEALTH_CHECKS=true）→ 全部记录并落库
    for (method, status) in [("GET", 200), ("GET", 503), ("POST", 200)] {
        assert!(
            !super::should_skip_request_log(method, StatusCode::from_u16(status).unwrap(), true),
            "开关开启时 {method} {status} 必须记录"
        );
    }
}
