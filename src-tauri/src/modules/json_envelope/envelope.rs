use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::*;

/// Strongly typed universal JSON envelope containing `attributes`, optional `variables`, and `data`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JsonEnvelope<T> {
    pub attributes: JsonAttributes,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, Value>>,
    pub data: T,
}

impl<T> JsonEnvelope<T> {
    pub fn new(data_type: impl Into<String>, data: T) -> Self {
        let mut vars = HashMap::new();
        vars.insert("workDir".to_string(), Value::String("D:\\work".to_string()));
        vars.insert(
            "repoDir".to_string(),
            Value::String("${workDir}\\antigravity-manager".to_string()),
        );
        vars.insert("secretsDir".to_string(), Value::String(".".to_string()));
        Self {
            attributes: JsonAttributes::new(data_type),
            variables: Some(vars),
            data,
        }
    }

    pub fn with_variables(mut self, vars: HashMap<String, Value>) -> Self {
        self.variables = Some(vars);
        self
    }

    pub fn with_work_directory(mut self, work_dir: WorkDirectoryConfig) -> Self {
        self.attributes.work_directory = Some(work_dir);
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.attributes.description = Some(desc.into());
        self
    }

    pub fn with_source(mut self, src: impl Into<String>) -> Self {
        let raw = src.into();
        let clean = std::path::Path::new(&raw)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&raw)
            .to_string();
        self.attributes.source = Some(clean);
        self
    }
}

/// Dynamically typed universal JSON envelope where `data` is arbitrary JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicJsonEnvelope {
    pub attributes: JsonAttributes,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, Value>>,
    pub data: Value,
}
