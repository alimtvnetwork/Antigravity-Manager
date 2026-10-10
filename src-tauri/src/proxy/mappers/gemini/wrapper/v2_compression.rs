// V2 compression phase (split from wrapper.rs).
// NOTE: wrap_request_v2 underwent behavior-preserving phase extraction
// to bring files under 500 lines.

pub(crate) fn phase_compression(
    inner_request: &mut serde_json::Value,
    compression_level: &str,
    final_model_name: &str,
    compression_applied: &mut bool,
    token_manager: Option<&std::sync::Arc<crate::proxy::TokenManager>>,
    upstream: Option<&std::sync::Arc<crate::proxy::upstream::client::UpstreamClient>>,
    session_id: Option<&str>,
    project_id: &str,
) {
    if compression_level == "high" {
        let tm = token_manager;
        let context_limit = if final_model_name.contains("flash") {
            1_000_000
        } else {
            2_000_000
        };

        let raw_estimated =
            crate::proxy::mappers::context_manager::ContextManager::estimate_gemini_token_usage(
                &inner_request,
            );
        let calibrator = crate::proxy::mappers::estimation_calibrator::get_calibrator();
        let mut estimated_usage = calibrator.calibrate(raw_estimated);
        let mut usage_ratio = estimated_usage as f32 / context_limit as f32;

        let threshold_l1 = crate::proxy::config::get_global_threshold_l1();
        let threshold_l2 = crate::proxy::config::get_global_threshold_l2();
        let threshold_l3 = crate::proxy::config::get_global_threshold_l3();

        let trace_id = format!(
            "gemini_req_{}",
            chrono::Utc::now().timestamp_subsec_millis()
        );

        tracing::info!(
            "[{}] [ContextManager] [Gemini] Context pressure: {:.1}% (raw: {}, calibrated: {} / {}), Calibration factor: {:.2}",
            trace_id, usage_ratio * 100.0, raw_estimated, estimated_usage, context_limit, calibrator.get_factor()
        );

        // ===== Layer 1: Tool Message Trimming =====
        if usage_ratio > threshold_l1 && !compression_applied {
            if crate::proxy::mappers::context_manager::ContextManager::trim_gemini_tool_messages(
                &mut inner_request,
                5,
            ) {
                tracing::info!(
                    "[{}] [Layer-1] [Gemini] Tool trimming triggered (usage: {:.1}%, threshold: {:.1}%)",
                    trace_id, usage_ratio * 100.0, threshold_l1 * 100.0
                );
                compression_applied = true;

                let new_raw = crate::proxy::mappers::context_manager::ContextManager::estimate_gemini_token_usage(&inner_request);
                let new_usage = calibrator.calibrate(new_raw);
                let new_ratio = new_usage as f32 / context_limit as f32;

                tracing::info!(
                    "[{}] [Layer-1] [Gemini] Compression result: {:.1}% → {:.1}% (saved {} tokens)",
                    trace_id,
                    usage_ratio * 100.0,
                    new_ratio * 100.0,
                    estimated_usage - new_usage
                );

                if new_ratio < 0.7 {
                    estimated_usage = new_usage;
                    usage_ratio = new_ratio;
                } else {
                    usage_ratio = new_ratio;
                    compression_applied = false;
                }
            }
        }

        // ===== Layer 2: Thinking Content Compression =====
        if usage_ratio > threshold_l2 && !compression_applied {
            tracing::info!(
                "[{}] [Layer-2] [Gemini] Thinking compression triggered (usage: {:.1}%, threshold: {:.1}%)",
                trace_id, usage_ratio * 100.0, threshold_l2 * 100.0
            );

            if crate::proxy::mappers::context_manager::ContextManager::compress_gemini_thinking_preserve_signature(
                &mut inner_request,
                4,
            ) {
                compression_applied = true;

                let new_raw = crate::proxy::mappers::context_manager::ContextManager::estimate_gemini_token_usage(&inner_request);
                let new_usage = calibrator.calibrate(new_raw);
                let new_ratio = new_usage as f32 / context_limit as f32;

                tracing::info!(
                    "[{}] [Layer-2] [Gemini] Compression result: {:.1}% → {:.1}% (saved {} tokens)",
                    trace_id, usage_ratio * 100.0, new_ratio * 100.0, estimated_usage - new_usage
                );

                usage_ratio = new_ratio;
            }
        }

        // ===== Layer 3: Fork Conversation + XML Summary =====
        if usage_ratio > threshold_l3 && !compression_applied {
            tracing::info!(
                "[{}] [Layer-3] [Gemini] Context pressure ({:.1}%) exceeded threshold ({:.1}%), spawning Fork+Summary in background",
                trace_id, usage_ratio * 100.0, threshold_l3 * 100.0
            );

            let tm_opt = tm.cloned();
            let upstream_opt = upstream.cloned();
            let sid_str = session_id.unwrap_or_default().to_string();
            let body_clone = inner_request.clone();
            let trace_id_clone = trace_id.clone();
            let proj_clone = project_id.to_string();
            let acc_clone = account_id.unwrap_or_default().to_string();

            // 两个依赖缺一不可：token_manager 取凭据，upstream 选对代理
            // （否则账号绑定专属代理时摘要会从真实 IP 出去）。
            if let (Some(tm_arc), Some(upstream_arc)) = (tm_opt, upstream_opt) {
                tokio::spawn(async move {
                    match try_compress_gemini_with_summary(
                        &body_clone,
                        &trace_id_clone,
                        &tm_arc,
                        &upstream_arc,
                        &sid_str,
                        &proj_clone,
                        &acc_clone,
                    )
                    .await
                    {
                        Ok(_) => {
                            tracing::info!(
                                "[{}] [Layer-3] [Gemini] Background Fork+Summary completed successfully",
                                trace_id_clone
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                "[{}] [Layer-3] [Gemini] Background Fork+Summary failed: {}",
                                trace_id_clone,
                                e
                            );
                        }
                    }
                });
            }
        }
    }

    if compression_level != "disabled" {
        if let Some(contents) = inner_request
            .get_mut("contents")
            .and_then(|c| c.as_array_mut())
        {
            let total_turns = contents.len();
            let protected_last_n = 4;
            let start_protection_idx = total_turns.saturating_sub(protected_last_n);

            for (i, content) in contents.iter_mut().enumerate() {
                if let Some(parts) = content.get_mut("parts").and_then(|p| p.as_array_mut()) {
                    for part in parts {
                        if let Some(obj) = part.as_object_mut() {
                            if compression_level == "medium" || compression_level == "high" {
                                if i < start_protection_idx {
                                    if let Some(text_val) =
                                        obj.get_mut("text").and_then(|t| t.as_str())
                                    {
                                        let cleaned = crate::proxy::mappers::caveman_cleaner::CavemanCleaner::clean(text_val);
                                        if cleaned != text_val {
                                            obj.insert("text".to_string(), json!(cleaned));
                                        }
                                    }
                                }
                            }
                            if let Some(fr) = obj.get_mut("functionResponse") {
                                if let Some(resp_obj) =
                                    fr.get_mut("response").and_then(|r| r.as_object_mut())
                                {
                                    for (_key, val) in resp_obj.iter_mut() {
                                        if let Some(s) = val.as_str() {
                                            let cleaned = crate::proxy::mappers::rtk_cleaner::RtkCleaner::clean(s, 48);
                                            if cleaned != s {
                                                *val = json!(cleaned);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
