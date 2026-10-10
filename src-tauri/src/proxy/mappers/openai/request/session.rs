// Session transform orchestrator (split from request.rs).
// NOTE: transform_openai_request_with_session underwent behavior-preserving
// phase extraction (setup/system/contents/body/tools/finalize) to bring files under 500 lines.
use super::session_body::phase_body;
use super::session_contents::phase_contents;
use super::session_finalize::phase_finalize;
use super::session_setup::{phase_setup, SetupState};
use super::session_system::{phase_system, SystemState};
use super::session_tools::phase_tools;
use serde_json::Value;
use super::super::models::OpenAIRequest;
use crate::proxy::token_manager::ProxyToken;

pub fn transform_openai_request_with_session(
    request: &OpenAIRequest,
    project_id: &str,
    mapped_model: &str,
    token: Option<&ProxyToken>,
    routing_session_id: &str,
    _signature_read_key: Option<&str>,
    is_responses_api: bool,
) -> (Value, String, usize, String) {
    // Phase 1: Setup
    let setup = phase_setup(
        request,
        project_id,
        mapped_model,
        token,
        routing_session_id,
        is_responses_api,
    );

    // Phase 2: System instructions
    let system = phase_system(request);

    // Phase 3: Contents
    let contents = phase_contents(
        request,
        mapped_model,
        is_responses_api,
        setup.actual_include_thinking,
        &system.tool_id_to_name,
        &system.tool_name_to_schema,
        &setup.session_id,
    );

    // Phase 4: Body
    let (mut inner_request, _gen_config) =
        phase_body(request, mapped_model, token, &setup, &contents);

    // Phase 5: Tools
    phase_tools(
        request,
        mapped_model,
        token,
        &setup,
        &system.system_instructions,
        &mut inner_request,
    );

    // Phase 6: Finalize
    phase_finalize(request, project_id, token, &setup, &contents, inner_request)
}
