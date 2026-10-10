use super::*;
use serde_json::Value;

#[derive(Debug, Default, Clone)]
pub struct TurnAccumulator {
    thought: String,
    signature: Option<String>,
    visible: String,
    tool_ids: Vec<String>,
    tool_names: Vec<String>,
    context_anchor: String,
    function_call_count: usize,
}

impl TurnAccumulator {
    pub fn new() -> Self {
        Self::with_anchor("root")
    }

    pub fn with_anchor(anchor: impl Into<String>) -> Self {
        Self {
            thought: String::new(),
            signature: None,
            visible: String::new(),
            tool_ids: Vec::new(),
            tool_names: Vec::new(),
            context_anchor: anchor.into(),
            function_call_count: 0,
        }
    }

    pub fn ingest_part(&mut self, part: &Value) {
        let is_thought = part
            .get("thought")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
            if is_thought {
                self.thought.push_str(text);
            } else {
                self.visible.push_str(text);
            }
        }
        if let Some(sig) = part
            .get("thoughtSignature")
            .or_else(|| part.get("thought_signature"))
            .and_then(|s| s.as_str())
        {
            if is_real_signature(sig)
                && self
                    .signature
                    .as_ref()
                    .map(|old| sig.len() > old.len())
                    .unwrap_or(true)
            {
                self.signature = Some(sig.to_string());
            }
        }
        if let Some(fc) = part.get("functionCall") {
            let name = fc
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let explicit_id = fc
                .get("id")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .map(|s| crate::proxy::common::utils::normalize_tool_id(s).into_owned());

            let synthetic = synthesize_tool_id(
                &name,
                fc.get("args"),
                &self.context_anchor,
                self.function_call_count,
            );
            self.function_call_count += 1;

            // 核心演进：全面转战因果伪哈希 ID！首位强制存入确定性 synthetic ID，客户端 real_id 紧随其后作为元数据兜底
            if !self.tool_ids.iter().any(|x| x == &synthetic) {
                self.tool_ids.push(synthetic.clone());
            }

            if let Some(ref real_id) = explicit_id {
                if !self.tool_ids.iter().any(|x| x == real_id) {
                    self.tool_ids.push(real_id.clone());
                }
            }

            if !self.tool_names.iter().any(|x| x == &name) {
                self.tool_names.push(name);
            }
        }
    }

    pub fn record_tool_id(&mut self, tool_name: &str, real_id: &str) {
        let norm_id = crate::proxy::common::utils::normalize_tool_id(real_id);
        let id_str = norm_id.to_string();
        if !self.tool_ids.iter().any(|x| x == &id_str) {
            self.tool_ids.push(id_str);
        }
        if !self.tool_names.iter().any(|x| x == tool_name) {
            self.tool_names.push(tool_name.to_string());
        }
    }

    pub fn is_empty(&self) -> bool {
        self.thought.trim().is_empty() && self.signature.is_none()
    }

    pub(crate) fn should_capture(&self) -> bool {
        !self.is_empty() && is_capturable_thought(&self.thought, self.signature.as_deref())
    }

    pub(crate) fn into_record(self) -> ThinkingRecord {
        let fp = fingerprint(&self.visible, &self.tool_ids, &self.tool_names);
        ThinkingRecord {
            fingerprint: fp,
            thought: self.thought,
            signature: self.signature,
            tool_ids: self.tool_ids,
            tool_names: self.tool_names,
            visible: self.visible,
        }
    }

    pub fn commit(self, store_key: &str) {
        if store_key.is_empty() || !self.should_capture() {
            return;
        }
        let rec = self.into_record();
        tracing::debug!(
            "[ThinkingStore] Capture thought len={} sig_len={} fp={} sid={}",
            rec.thought.len(),
            rec.signature.as_ref().map(|s| s.len()).unwrap_or(0),
            rec.fingerprint,
            store_key
        );
        ThinkingStore::global().record(store_key, rec);
    }
}
