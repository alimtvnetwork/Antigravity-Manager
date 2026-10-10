use base64::prelude::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboundAction {
    PromptInjection {
        project_name: String,
        prompt_name: String,
        prompt: String,
        instance_id: Option<String>,
    },
    NamedPromptExecution {
        prompt_query: String,
    },
    CliExecution {
        target_ip: String,
        command: String,
    },
    GitMapExecution {
        target: String,
        command: String,
    },
    UpdateExecution {
        target: String,
        is_gitmap: bool,
    },
    ListInstances {
        target: String,
    },
    ListPrompts {
        target: String,
        is_gitmap: bool,
    },
    InstanceCreate {
        profile_name: String,
    },
    AccountRotate {
        target: String,
        instance_id: Option<String>,
    },
    FastForward {
        target_node: String,
        instance_id: Option<String>,
    },
    MultiNodeSnapshotQuery,
    StatusQuery {
        target: String,
    },
    DoctorDiagnostic {
        target: String,
    },
    ListAccounts {
        target: String,
    },
    AccountSwitch {
        target: String,
        email_query: String,
        instance_id: Option<String>,
    },
    ProxyStatus {
        target: String,
        is_test: bool,
    },
    SystemClean {
        target: String,
    },
    SyncState {
        target: String,
    },
    HelpRequest {
        target: String,
        instance_id: Option<String>,
    },
    Ignored {
        reason: String,
    },
}

/// Raw message header and body
#[derive(Debug, Clone)]
pub struct RawEmailMessage {
    pub message_id: String,
    pub from: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct DebounceRecord {
    pub first_seen: i64,
    pub count: usize,
    pub has_acked: bool,
    pub has_completed: bool,
}

pub(crate) static DEBOUNCE_STACK: Lazy<Mutex<HashMap<String, DebounceRecord>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
