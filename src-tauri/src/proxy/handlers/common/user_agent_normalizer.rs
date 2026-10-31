//! User-Agent Header Normalizer
//! Normalizes client User-Agent strings to Antigravity IDE standard signatures,
//! ensuring consistent behavior across upstream Google Cloud Code and Gemini edge endpoints.

pub struct UserAgentNormalizer;

impl UserAgentNormalizer {
    pub const DEFAULT_UPSTREAM_USER_AGENT: &'static str =
        "Antigravity/4.250.0 (Windows NT 10.0; Win64; x64) GoogleCloudCode/1.24.0";

    /// Normalizes a client User-Agent string.
    /// If empty or missing, returns the default Antigravity signature.
    /// If present, preserves the client name while appending the upstream gateway tag.
    pub fn normalize(client_ua: Option<&str>) -> String {
        match client_ua {
            None => Self::DEFAULT_UPSTREAM_USER_AGENT.to_string(),
            Some(ua) if ua.trim().is_empty() => Self::DEFAULT_UPSTREAM_USER_AGENT.to_string(),
            Some(ua) => {
                let trimmed = ua.trim();
                if trimmed.contains("Antigravity") || trimmed.contains("GoogleCloudCode") {
                    trimmed.to_string()
                } else {
                    format!("{} Antigravity/4.250.0", trimmed)
                }
            }
        }
    }
}
