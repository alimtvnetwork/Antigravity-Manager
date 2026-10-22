// Cloned Profile Telemetry Isolation Verifier (Imp-501)
// Verifies that a newly created or cloned instance profile does not share
// machine telemetry identifiers or machine GUIDs with the source profile.

use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TelemetryIsolationReport {
    pub is_isolated: bool,
    pub machine_id_differs: bool,
    pub dev_device_id_differs: bool,
    pub error: Option<String>,
}

pub struct ProfileTelemetryIsolation;

impl ProfileTelemetryIsolation {
    /// Compares storage.json between source and target instance directories.
    pub fn verify_isolation<P: AsRef<Path>>(source_dir: P, target_dir: P) -> TelemetryIsolationReport {
        let source_storage = source_dir.as_ref().join("storage.json");
        let target_storage = target_dir.as_ref().join("storage.json");

        if !source_storage.exists() || !target_storage.exists() {
            return TelemetryIsolationReport {
                is_isolated: true,
                machine_id_differs: true,
                dev_device_id_differs: true,
                error: None,
            };
        }

        let source_content = match fs::read_to_string(&source_storage) {
            Ok(c) => c,
            Err(e) => return Self::err_report(e.to_string()),
        };
        let target_content = match fs::read_to_string(&target_storage) {
            Ok(c) => c,
            Err(e) => return Self::err_report(e.to_string()),
        };

        let source_json: Value = serde_json::from_str(&source_content).unwrap_or(Value::Null);
        let target_json: Value = serde_json::from_str(&target_content).unwrap_or(Value::Null);

        let s_machine = source_json.get("telemetry.machineId").and_then(|v| v.as_str());
        let t_machine = target_json.get("telemetry.machineId").and_then(|v| v.as_str());
        let machine_differs = match (s_machine, t_machine) {
            (Some(s), Some(t)) => s != t,
            _ => true,
        };

        let s_device = source_json.get("telemetry.devDeviceId").and_then(|v| v.as_str());
        let t_device = target_json.get("telemetry.devDeviceId").and_then(|v| v.as_str());
        let device_differs = match (s_device, t_device) {
            (Some(s), Some(t)) => s != t,
            _ => true,
        };

        TelemetryIsolationReport {
            is_isolated: machine_differs && device_differs,
            machine_id_differs: machine_differs,
            dev_device_id_differs: device_differs,
            error: None,
        }
    }

    fn err_report(err: String) -> TelemetryIsolationReport {
        TelemetryIsolationReport {
            is_isolated: false,
            machine_id_differs: false,
            dev_device_id_differs: false,
            error: Some(err),
        }
    }
}
