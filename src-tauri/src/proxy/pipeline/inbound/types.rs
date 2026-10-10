use super::*;

/// 客户端思考控制开关（三态枚举）
/// 遵循最高指令：思考开关（一票否决权） > 思考等级 > 思考预算
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientThinkingSwitch {
    /// 显式关闭（最高准则，一票否决：disabled / 0 / none / off）
    Disabled,
    /// 显式开启（enabled / on / 具体档位 / 具体预算）
    Enabled,
    /// 缺省（未传任何思考参数，或值为 default；业务铁律：缺省就是默认开）
    Default,
}

impl ClientThinkingSwitch {
    /// 是否允许开启思考（缺省即开，关则一票否决）
    pub fn is_allowed(self) -> bool {
        matches!(self, Self::Enabled | Self::Default)
    }

    /// 是否显式关闭
    pub fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled)
    }
}

/// 统一归一化提取客户端思考开关状态（适用于 OpenAI / Claude / Gemini / Codex 等所有协议）
pub fn extract_client_thinking_switch(
    thinking_type: Option<&str>,
    budget: Option<u64>,
    effort: Option<&str>,
) -> ClientThinkingSwitch {
    let t_type = thinking_type.map(|s| s.trim().to_lowercase());
    let eff = effort.map(|s| s.trim().to_lowercase());

    // 1. 显式关闭判定（最高优先级，一票否决）
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

    // 2. 显式开启判定（只要客户端带了有效开启标记、具体预算，或任何非空/非关闭的思考等级）
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

    // 3. 缺省（全未传，或仅为 "default"；铁律：缺省就是默认开）
    ClientThinkingSwitch::Default
}

/// 统一进站思考管线（InboundThinkingPipeline）
/// 接收任何协议转译成的 Google contents 统一报文，单向流转执行：
/// 1. 协议策略签名清洗 (Chat 协议丢弃客户端签名，其他协议验签)
/// 2. 思考块首位强制排序与占位符规范化
/// 3. 状态机历史思维链无损复活 (Hydration)
/// 4. 终审脱敏与前缀缓存格式规范化 (Finalize)
pub struct InboundThinkingPipeline;
