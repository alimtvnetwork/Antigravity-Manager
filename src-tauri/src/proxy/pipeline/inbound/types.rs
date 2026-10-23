use super::*;

/// Client thinking control switch (three-state enumeration)
/// Follow the Prime Directive: Think Switch (veto power) > Think Level > Think Budget
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientThinkingSwitch {
    /// Explicit shutdown (highest guideline, one vote veto: disabled / 0 / none / off)
    Disabled,
    /// Explicitly turned on (enabled / on / specific gear / specific budget)
    Enabled,
    /// Default (no thinking parameter is passed, or the value is default; iron rule of business: the default is to enable it by default)
    Default,
}

impl ClientThinkingSwitch {
    /// Whether to allow thinking (default is on, off means veto)
    pub fn is_allowed(self) -> bool {
        matches!(self, Self::Enabled | Self::Default)
    }

    /// Whether to explicitly close
    pub fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled)
    }
}

/// Unified and normalized extraction of client thinking switch status (applicable to all protocols such as OpenAI / Claude / Gemini / Codex)
pub fn extract_client_thinking_switch(
    thinking_type: Option<&str>,
    budget: Option<u64>,
    effort: Option<&str>,
) -> ClientThinkingSwitch {
    let t_type = thinking_type.map(|s| s.trim().to_lowercase());
    let eff = effort.map(|s| s.trim().to_lowercase());

    // 1. Explicitly close the judgment (highest priority, one vote veto)
    if matches!(
        t_type.as_deref(),
        Some("disabled") | Some("off") | Some("false") | Some("0")
    ) || budget == Some(0)
        || matches!(
            eff.as_deref(),
            Some("none") | Some("off") | Some("false") | Some("0") | Some("disabled")
        )
    {
        return ClientThinkingSwitch::Disabled;
    }

    // 2. Explicit opening determination (as long as the client has a valid opening tag, a specific budget, or any non-empty/non-closed thinking level)
    if matches!(
        t_type.as_deref(),
        Some("enabled") | Some("on") | Some("true")
    ) || budget.map_or(false, |b| b > 0)
        || eff.as_deref().map_or(false, |e| {
            !e.is_empty()
                && e != "none"
                && e != "off"
                && e != "false"
                && e != "0"
                && e != "disabled"
                && e != "default"
        })
    {
        return ClientThinkingSwitch::Enabled;
    }

    // 3. Default (not transmitted at all, or only "default"; iron rule: the default is to enable it by default)
    ClientThinkingSwitch::Default
}

/// Unified InboundThinkingPipeline
/// Receive unified messages from Google contents translated by any protocol, and perform one-way transfer:
/// 1. Protocol policy signature cleaning (Chat protocol discards client signatures, other protocols verify signatures)
/// 2. Think about forced sorting at the top of blocks and placeholder normalization
/// 3. Lossless resurrection of the state machine’s historical thinking chain (Hydration)
/// 4. Final desensitization and prefix cache format standardization (Finalize)
pub struct InboundThinkingPipeline;
