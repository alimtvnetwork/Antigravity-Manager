use super::*;
use crate::proxy::mappers::claude::models::ClaudeRequest;
use crate::proxy::handlers::claude::attempt::AttemptState;

/// Apply the 3-layer progressive compression to the request when the
/// compression level is "high" (and thinking was not stripped for retry).
/// Extracted from `handle_messages`. Pure code move: no logic changes.
///
/// Returns `Err(Response)` when Layer-3 fork+summary fails, mirroring the
/// early `return` previously done inline.
pub(crate) async fn apply_compression(
    request_for_body: &ClaudeRequest,
    mapped_model: &str,
    st: &AttemptState,
) -> Result<ClaudeRequest, Response> {
    let mut request_with_mapped = request_for_body.clone();
    // ===== [3-Layer Progressive Compression + Calibrated Estimation] Context Management =====
    // [ENHANCED] 整合 3.3.47 的三层压缩框架 + PR #925 的动态校准机制
    // [NEW] 只有当 st.scaling_enabled 为 true 时才执行压缩逻辑 (联动机制)
    // Layer 1 (60%): Tool message trimming - Does NOT break cache
    // Layer 2 (75%): Thinking purification - Breaks cache but preserves signatures
    // Layer 3 (90%): Fork conversation + XML summary - Ultimate optimization
    let mut compression_applied = false;

    if !st.retried_without_thinking && st.compression_level == "high" {
        // 新增 st.scaling_enabled 联动判断
        // 1. Determine context limit (Flash: ~1M, Pro: ~2M)
        let context_limit = if mapped_model.contains("flash") {
            1_000_000
        } else {
            2_000_000
        };

        // 2. [ENHANCED] 使用校准器提高估算准确度 (PR #925)
        let raw_estimated = ContextManager::estimate_token_usage(&request_with_mapped);
        let calibrator = get_calibrator();
        let mut estimated_usage = calibrator.calibrate(raw_estimated);
        let mut usage_ratio = estimated_usage as f32 / context_limit as f32;

        info!(
        "[{}] [ContextManager] Context pressure: {:.1}% (raw: {}, calibrated: {} / {}), Calibration factor: {:.2}",
        st.trace_id, usage_ratio * 100.0, raw_estimated, estimated_usage, context_limit, calibrator.get_factor()
    );

        // ===== Layer 1: Tool Message Trimming (L1 threshold) =====
        // Borrowed from Practical-Guide-to-Context-Engineering
        // Advantage: Completely cache-friendly (only removes messages, doesn't modify content)
        if usage_ratio > st.threshold_l1 && !compression_applied {
            if ContextManager::trim_tool_messages(&mut request_with_mapped.messages, 5) {
                info!(
                    "[{}] [Layer-1] Tool trimming triggered (usage: {:.1}%, threshold: {:.1}%)",
                    st.trace_id,
                    usage_ratio * 100.0,
                    st.threshold_l1 * 100.0
                );
                compression_applied = true;

                // Re-estimate after trimming (with calibration)
                let new_raw = ContextManager::estimate_token_usage(&request_with_mapped);
                let new_usage = calibrator.calibrate(new_raw);
                let new_ratio = new_usage as f32 / context_limit as f32;

                info!(
                    "[{}] [Layer-1] Compression result: {:.1}% → {:.1}% (saved {} tokens)",
                    st.trace_id,
                    usage_ratio * 100.0,
                    new_ratio * 100.0,
                    estimated_usage - new_usage
                );

                // If compression is sufficient, skip further layers
                if new_ratio < 0.7 {
                    estimated_usage = new_usage;
                    usage_ratio = new_ratio;
                    // Success, no need for Layer 2
                } else {
                    // Still high pressure, update for Layer 2
                    usage_ratio = new_ratio;
                    compression_applied = false; // Allow Layer 2 to run
                }
            }
        }

        // ===== Layer 3: Fork Conversation + XML Summary (L3 threshold) =====
        // Ultimate optimization: Generate structured summary and start fresh conversation
        // Advantage: Completely cache-friendly (append-only), extreme compression ratio
        if usage_ratio > st.threshold_l3 && !compression_applied {
            info!(
            "[{}] [Layer-3] Context pressure ({:.1}%) exceeded threshold ({:.1}%), attempting Fork+Summary",
            st.trace_id, usage_ratio * 100.0, st.threshold_l3 * 100.0
        );

            // Clone st.token_manager Arc to avoid borrow issues
            let token_manager_clone = st.token_manager.clone();

            match try_compress_with_summary(
                &request_with_mapped,
                &st.trace_id,
                &token_manager_clone,
                &state.st.upstream,
            )
            .await
            {
                Ok(forked_request) => {
                    info!(
                        "[{}] [Layer-3] Fork successful: {} → {} messages",
                        st.trace_id,
                        request_with_mapped.messages.len(),
                        forked_request.messages.len()
                    );

                    request_with_mapped = forked_request;
                    // Re-estimate after fork (with calibration)
                    let new_raw = ContextManager::estimate_token_usage(&request_with_mapped);
                    let new_usage = calibrator.calibrate(new_raw);
                    let new_ratio = new_usage as f32 / context_limit as f32;

                    info!(
                        "[{}] [Layer-3] Compression result: {:.1}% → {:.1}% (saved {} tokens)",
                        st.trace_id,
                        usage_ratio * 100.0,
                        new_ratio * 100.0,
                        estimated_usage - new_usage
                    );
                }
                Err(e) => {
                    error!(
                        "[{}] [Layer-3] Fork+Summary failed: {}, falling back to error response",
                        st.trace_id, e
                    );

                    // Return friendly error to user
                    return Err(
                    (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "type": "error",
                        "error": {
                            "type": "invalid_request_error",
                            "message": format!("Context too long and automatic compression failed: {}", e),
                            "suggestion": "Please use /compact or /clear command in Claude Code, or switch to a model with larger context window."
                        }
                    }))
                ).into_response()
                );
                }
            }
        }
    }
    Ok(request_with_mapped)
}
