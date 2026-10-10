// Streaming shared types (split from streaming.rs)
use crate::proxy::mappers::claude::models::*;

pub fn remap_function_call_args(name: &str, args: &mut Value) {
    // 纯透传协议工具参数，不进行任何字段重命名与拦截改写
    if let Some(obj) = args.as_object() {
        tracing::debug!(
            "[Streaming] Tool Call (Passthrough): '{}' Args: {:?}",
            name,
            obj
        );
    }
}

/// 块类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    None,
    Text,
    Thinking,
    Function,
}

/// 签名管理器
pub struct SignatureManager {
    pending: Option<String>,
}

impl SignatureManager {
    pub fn new() -> Self {
        Self { pending: None }
    }

    pub fn store(&mut self, signature: Option<String>) {
        if signature.is_some() {
            self.pending = signature;
        }
    }

    pub fn consume(&mut self) -> Option<String> {
        self.pending.take()
    }

    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }
}
