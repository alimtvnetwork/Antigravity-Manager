//! Telemetry Session UUID Rotator.
//! Generates fresh random UUIDv4 identifiers for `telemetry.sessionId` and `telemetry.sqmId` during profile initialization.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Fresh telemetry identifiers generated for an instance session or clone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySessionIds {
    pub session_id: String,
    pub sqm_id: String,
    pub dev_device_id: String,
}

/// Generates a set of unique, conforming synthetic telemetry identifiers.
pub fn generate_fresh_telemetry_ids() -> TelemetrySessionIds {
    let session_uuid = Uuid::new_v4();
    let sqm_uuid = Uuid::new_v4();
    let dev_device_uuid = Uuid::new_v4();

    TelemetrySessionIds {
        session_id: session_uuid.to_string(),
        sqm_id: format!("{{{}}}", sqm_uuid.to_string().to_uppercase()),
        dev_device_id: dev_device_uuid.to_string(),
    }
}
