use serde_json::Value;

// loadCodeAssist endpoint (fallback order: Daily → Sandbox → Prod)
//
// [FIX Issue #3525] Daily takes precedence, consistent with the outbound endpoint of the official Antigravity language_server.
// Sandbox will return termination 400 `User location is not supported for the API use.` to compliant accounts in some regions;
// The sandbox endpoint is never officially accessed, and is reserved only as a safety net for availability.
const LOAD_PROJECT_ENDPOINTS: [&str; 3] = [
    "https://daily-cloudcode-pa.googleapis.com/v1internal:loadCodeAssist",
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal:loadCodeAssist",
    "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist",
];

/// Get the project_id using Antigravity's loadCodeAssist API
/// This is the correct way to get cloudaicompanionProject
pub async fn fetch_project_id(access_token: &str) -> Result<String, String> {
    let request_body = serde_json::json!({
        "metadata": {
            "ideType": "ANTIGRAVITY"
        }
    });

    let client = crate::utils::http::get_client();
    let mut last_error = String::new();

    for url in LOAD_PROJECT_ENDPOINTS.iter() {
        let response = client
            .post(*url)
            .bearer_auth(access_token)
            .header("User-Agent", crate::constants::USER_AGENT.as_str())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let data: Value = resp
                        .json()
                        .await
                        .map_err(|e| format!("Failed to parse response: {}", e))?;

                    if let Some(project_id) =
                        data.get("cloudaicompanionProject").and_then(|v| v.as_str())
                    {
                        return Ok(project_id.to_string());
                    } else {
                        return Err("The account is not eligible to obtain the official cloudaicompanionProject".to_string());
                    }
                } else {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    last_error = format!(
                        "loadCodeAssist [ {} ] returns error {} : {}",
                        url, status, body
                    );
                    continue;
                }
            }
            Err(e) => {
                last_error = format!("loadCodeAssist [ {} ] request failed: {}", url, e);
                continue;
            }
        }
    }

    Err(if last_error.is_empty() {
        "loadCodeAssist All endpoints are unavailable".to_string()
    } else {
        last_error
    })
}
