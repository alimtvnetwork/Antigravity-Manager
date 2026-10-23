use super::*;

const SESSION_TTL_SECS: u64 = 3600; // Expires in 1 hour

#[derive(Debug, Clone)]
pub struct HttpSessionEntry {
    /// Conversation history: instructions + all input items (including historical response output)
    pub input_items: Vec<Value>,
    /// System commands
    pub instructions: String,
    /// Model name
    pub model: String,
    /// Last access time (for TTL elimination)
    pub last_accessed: Instant,
}

#[derive(Debug)]
pub(crate) struct SessionNode {
    pub(crate) parent: Option<Arc<SessionNode>>,
    pub(crate) input_delta: Vec<Value>,
    pub(crate) response_output: Vec<Value>,
    pub(crate) instructions: String,
    pub(crate) model: String,
    pub(crate) routing_session_id: String,
}

#[derive(Debug, Clone)]
pub struct SessionParent(pub(crate) Arc<SessionNode>);

impl SessionParent {
    pub fn routing_session_id(&self) -> &str {
        &self.0.routing_session_id
    }
}

pub(crate) struct StoredSession {
    pub(crate) node: Arc<SessionNode>,
    pub(crate) last_accessed: Instant,
}

pub(crate) struct HttpSessionStore {
    pub(crate) sessions: HashMap<String, StoredSession>,
}

impl HttpSessionStore {
    pub(crate) fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub(crate) fn get(&mut self, response_id: &str) -> Option<(HttpSessionEntry, SessionParent)> {
        let stored = self.sessions.get_mut(response_id)?;
        stored.last_accessed = Instant::now();
        let node = stored.node.clone();
        Some((
            HttpSessionEntry {
                input_items: materialize_history(&node),
                instructions: node.instructions.clone(),
                model: node.model.clone(),
                last_accessed: stored.last_accessed,
            },
            SessionParent(node),
        ))
    }

    pub(crate) fn insert(&mut self, response_id: String, entry: HttpSessionEntry) {
        self.insert_delta(
            response_id,
            None,
            entry.input_items,
            Vec::new(),
            entry.instructions,
            entry.model,
            None,
        );
    }

    pub(crate) fn insert_delta(
        &mut self,
        response_id: String,
        parent: Option<SessionParent>,
        input_delta: Vec<Value>,
        response_output: Vec<Value>,
        instructions: String,
        model: String,
        routing_session_id: Option<String>,
    ) {
        let routing_session_id = routing_session_id.unwrap_or_else(|| response_id.clone());
        self.sessions.insert(
            response_id,
            StoredSession {
                node: Arc::new(SessionNode {
                    parent: parent.map(|parent| parent.0),
                    input_delta,
                    response_output,
                    instructions,
                    model,
                    routing_session_id,
                }),
                last_accessed: Instant::now(),
            },
        );
        // By the way, eliminate expired sessions (lazy cleanup)
        self.evict_expired();
    }

    pub(crate) fn evict_expired(&mut self) {
        let ttl = Duration::from_secs(SESSION_TTL_SECS);
        self.sessions
            .retain(|_, stored| stored.last_accessed.elapsed() < ttl);
    }
}

pub(crate) fn materialize_history(node: &Arc<SessionNode>) -> Vec<Value> {
    let mut chain = Vec::new();
    let mut current = Some(node.clone());
    while let Some(node) = current {
        chain.push(node.clone());
        current = node.parent.clone();
    }

    let capacity = chain
        .iter()
        .map(|node| node.input_delta.len() + node.response_output.len())
        .sum();
    let mut history = Vec::with_capacity(capacity);
    for node in chain.into_iter().rev() {
        history.extend(node.input_delta.iter().cloned());
        history.extend(node.response_output.iter().cloned());
    }
    history
}

static STORE: OnceLock<Mutex<HttpSessionStore>> = OnceLock::new();

pub(crate) fn store() -> &'static Mutex<HttpSessionStore> {
    STORE.get_or_init(|| Mutex::new(HttpSessionStore::new()))
}

/// Find historical conversations based on previous_response_id
pub async fn get_session(previous_response_id: &str) -> Option<HttpSessionEntry> {
    store()
        .lock()
        .await
        .get(previous_response_id)
        .map(|(entry, _)| entry)
}

pub async fn get_session_with_parent(
    previous_response_id: &str,
) -> Option<(HttpSessionEntry, SessionParent)> {
    store().lock().await.get(previous_response_id)
}

/// Save the new session state (with response_id as key)
pub async fn save_session(response_id: String, entry: HttpSessionEntry) {
    store().lock().await.insert(response_id, entry);
}

/// Save Responses for this round of increments; the Arc strong reference of the parent node ensures that the branches share ancestors.
pub async fn save_session_delta(
    response_id: String,
    parent: Option<SessionParent>,
    input_delta: Vec<Value>,
    response_output: Vec<Value>,
    instructions: String,
    model: String,
    routing_session_id: String,
) {
    store().lock().await.insert_delta(
        response_id,
        parent,
        input_delta,
        response_output,
        instructions,
        model,
        Some(routing_session_id),
    );
}

pub struct PreparedSessionInput {
    pub merged: Vec<Value>,
    pub delta: Vec<Value>,
    pub reset_parent: bool,
}

/// Merge request history and only extract new items when the client replays the full history.
pub fn prepare_session_input(
    history: Vec<Value>,
    new_input: Vec<Value>,
    tool_call_cache: &HashMap<String, Value>,
) -> PreparedSessionInput {
    prepare_session_input_with_storage(history, new_input, tool_call_cache, true)
}

/// Restore the complete model input while optionally retaining a delta for storage.
pub fn prepare_session_input_with_storage(
    history: Vec<Value>,
    new_input: Vec<Value>,
    tool_call_cache: &HashMap<String, Value>,
    retain_delta: bool,
) -> PreparedSessionInput {
    let reset_parent = new_input.iter().any(|item| {
        matches!(
            item.get("type").and_then(Value::as_str),
            Some("compaction") | Some("compaction_summary")
        )
    });
    let exact_replay = !history.is_empty() && new_input.starts_with(&history);
    let replayed_through = if reset_parent || exact_replay {
        None
    } else {
        let history_ids: std::collections::HashSet<&str> = history
            .iter()
            .filter_map(|item| item.get("id").and_then(Value::as_str))
            .filter(|id| !id.is_empty())
            .collect();
        new_input.iter().rposition(|item| {
            item.get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| history_ids.contains(id))
        })
    };

    // Helper: check if two input items are semantically equivalent (ignoring volatile fields like id)
    let items_semantically_equal = |a: &Value, b: &Value| -> bool {
        let role_a = a.get("role").and_then(Value::as_str);
        let role_b = b.get("role").and_then(Value::as_str);
        let type_a = a.get("type").and_then(Value::as_str);
        let type_b = b.get("type").and_then(Value::as_str);
        let content_a = a.get("content").or_else(|| a.get("text"));
        let content_b = b.get("content").or_else(|| b.get("text"));
        role_a == role_b && type_a == type_b && content_a == content_b
    };

    // Semantic prefix match: check if new_input starts with history semantically
    let semantic_prefix_match = !history.is_empty()
        && new_input.len() >= history.len()
        && history
            .iter()
            .zip(new_input.iter())
            .all(|(h, n)| items_semantically_equal(h, n));

    // Semantic suffix find: find last item of history in new_input
    let semantic_suffix_idx = if !history.is_empty()
        && !reset_parent
        && !exact_replay
        && replayed_through.is_none()
        && !semantic_prefix_match
    {
        let last_h = &history[history.len() - 1];
        new_input
            .iter()
            .rposition(|n| items_semantically_equal(last_h, n))
    } else {
        None
    };

    let (delta_source, use_new_input_as_merged) = if reset_parent || history.is_empty() {
        (new_input.clone(), false)
    } else if exact_replay {
        (new_input[history.len()..].to_vec(), false)
    } else if semantic_prefix_match {
        (new_input[history.len()..].to_vec(), false)
    } else if let Some(index) = replayed_through {
        (new_input[index + 1..].to_vec(), false)
    } else if let Some(index) = semantic_suffix_idx {
        (new_input[index + 1..].to_vec(), false)
    } else if new_input.len() >= history.len() {
        // [FIX #3382] Fallback protection:
        // When the client sends full conversation history but formatting/IDs differed
        // such that no boundary was identified, appending all of new_input to history
        // would double the history (2x, 4x, ...). Instead, treat new_input as the authoritative
        // current history, extracting the last element as delta.
        tracing::warn!(
            "[Session] Match failed but new_input (len: {}) >= history (len: {}). Preventing history duplication.",
            new_input.len(),
            history.len()
        );
        let delta_slice = if new_input.is_empty() {
            Vec::new()
        } else {
            vec![new_input.last().unwrap().clone()]
        };
        (delta_slice, true)
    } else {
        (new_input.clone(), false)
    };

    let delta = merge_history_with_new_input(Vec::new(), &[], delta_source, tool_call_cache);
    let stored_delta = if retain_delta {
        delta.clone()
    } else {
        Vec::new()
    };
    let merged = if reset_parent || history.is_empty() {
        delta
    } else if use_new_input_as_merged {
        merge_history_with_new_input(Vec::new(), &[], new_input, tool_call_cache)
    } else {
        merge_history_with_new_input(history, &[], delta, tool_call_cache)
    };

    PreparedSessionInput {
        merged,
        delta: stored_delta,
        reset_parent,
    }
}

/// Convert the previous round of response output items into input items and append them to the history
/// At the same time, add new user input items
/// Return the merged input items
pub fn merge_history_with_new_input(
    mut history: Vec<Value>,
    response_output: &[Value],
    new_input: Vec<Value>,
    tool_call_cache: &HashMap<String, Value>,
) -> Vec<Value> {
    // Check whether the new input contains compaction / compaction_summary. If it does, the client is sending a new and complete history after compression.
    let has_compaction = new_input.iter().any(|item| {
        let t = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
        t == "compaction" || t == "compaction_summary"
    });

    if has_compaction {
        tracing::info!(
            "[Session] Compaction detected in new input. Overwriting stale history (new items: {})",
            new_input.len()
        );
        // filter out compaction itself
        let mut filtered = Vec::new();
        for item in new_input {
            let t = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if t == "compaction" || t == "compaction_summary" {
                continue;
            }
            filtered.push(item);
        }
        repair_tool_calls(&mut filtered, tool_call_cache);
        return dedupe_input_items(filtered);
    }

    // Append the previous round of response output (assistant messages, tool calls, etc.)
    for item in response_output {
        history.push(item.clone());
    }

    // Append new input items
    for item in new_input {
        let t = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if t == "compaction" || t == "compaction_summary" {
            continue;
        }
        history.push(item);
    }

    // Fix tool calls (make sure there is a corresponding function_call before function_call_output)
    repair_tool_calls(&mut history, tool_call_cache);

    // Remove duplicates
    dedupe_input_items(history)
}

pub(crate) fn repair_tool_calls(items: &mut Vec<Value>, tool_call_cache: &HashMap<String, Value>) {
    let mut call_present = std::collections::HashSet::new();
    for item in items.iter() {
        let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if item_type == "function_call" || item_type == "custom_tool_call" {
            if let Some(call_id) = item.get("call_id").and_then(|v| v.as_str()) {
                call_present.insert(call_id.to_string());
            }
        }
    }

    let mut new_items = Vec::new();
    let mut inserted = std::collections::HashSet::new();
    for item in items.drain(..) {
        let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if item_type == "function_call_output" || item_type == "custom_tool_call_output" {
            if let Some(call_id) = item.get("call_id").and_then(|v| v.as_str()) {
                if !call_id.is_empty()
                    && !call_present.contains(call_id)
                    && !inserted.contains(call_id)
                {
                    if let Some(cached_call) = tool_call_cache
                        .get(call_id)
                        .cloned()
                        .or_else(|| get_cached_tool_call(call_id))
                    {
                        new_items.push(cached_call.clone());
                        inserted.insert(call_id.to_string());
                    }
                }
            }
        }
        new_items.push(item);
    }
    *items = new_items;
}

pub(crate) fn dedupe_input_items(items: Vec<Value>) -> Vec<Value> {
    use std::collections::{HashMap, HashSet};
    let mut referenced_call_ids = HashSet::new();
    for item in &items {
        let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if item_type == "function_call_output" || item_type == "custom_tool_call_output" {
            if let Some(call_id) = item.get("call_id").and_then(|v| v.as_str()) {
                if !call_id.is_empty() {
                    referenced_call_ids.insert(call_id.to_string());
                }
            }
        }
    }

    let mut keep_map: HashMap<String, usize> = HashMap::new();
    for (idx, item) in items.iter().enumerate() {
        let item_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if item_id.is_empty() {
            continue;
        }
        let call_id = item.get("call_id").and_then(|v| v.as_str()).unwrap_or("");
        let is_referenced = !call_id.is_empty() && referenced_call_ids.contains(call_id);
        if let Some(&existing_idx) = keep_map.get(item_id) {
            let existing_call_id = items[existing_idx]
                .get("call_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let existing_referenced =
                !existing_call_id.is_empty() && referenced_call_ids.contains(existing_call_id);
            if is_referenced || !existing_referenced {
                keep_map.insert(item_id.to_string(), idx);
            }
        } else {
            keep_map.insert(item_id.to_string(), idx);
        }
    }

    let mut keep_indices = std::collections::HashSet::new();
    for (_, idx) in keep_map {
        keep_indices.insert(idx);
    }

    let mut filtered = Vec::new();
    for (idx, item) in items.into_iter().enumerate() {
        let item_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if !item_id.is_empty() && !keep_indices.contains(&idx) {
            continue;
        }
        filtered.push(item);
    }
    filtered
}
