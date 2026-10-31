//! Cloned profile telemetry salt generator.
//! Generates distinct pseudo-random telemetry seed hashes for cloned instance profiles
//! to guarantee telemetry isolation across concurrent developer workspaces.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClonedTelemetryTokens {
    pub machine_id: String,
    pub mac_machine_id: String,
    pub dev_device_id: String,
    pub sqm_id: String,
}

pub struct ClonedTelemetrySalt;

impl ClonedTelemetrySalt {
    /// Generates isolated telemetry GUIDs salted by instance profile ID.
    pub fn generate_salted_tokens(instance_id: &str) -> ClonedTelemetryTokens {
        let salt = format!(
            "{}-{}",
            instance_id,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(42)
        );

        ClonedTelemetryTokens {
            machine_id: compute_hash_guid(&format!("{}-mach", salt)),
            mac_machine_id: compute_hash_guid(&format!("{}-mac", salt)),
            dev_device_id: compute_hash_guid(&format!("{}-dev", salt)),
            sqm_id: format!(
                "{{{}}}",
                compute_hash_guid(&format!("{}-sqm", salt)).to_uppercase()
            ),
        }
    }
}

fn compute_hash_guid(input: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in input.bytes() {
        h = (h ^ (b as u64)).wrapping_mul(0x100000001b3);
    }
    let h2 = h
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);

    format!(
        "{:08x}-{:04x}-4{:03x}-a{:03x}-{:012x}",
        (h >> 32) as u32,
        (h >> 16) as u16,
        h as u16 & 0x0fff,
        (h2 >> 48) as u16 & 0x0fff,
        h2 & 0xffffffffffff
    )
}
