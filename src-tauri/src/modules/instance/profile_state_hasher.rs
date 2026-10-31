// Instance Profile Configuration SHA-256 Hasher (Imp-531)
// Generates deterministic SHA-256 state fingerprints for instance configurations
// to detect out-of-band file modifications or profile drift.

use sha2::{Digest, Sha256};

pub struct ProfileStateHasher;

impl ProfileStateHasher {
    /// Generates a hex-encoded SHA-256 hash of instance core attributes.
    pub fn compute_profile_hash(
        instance_id: &str,
        data_dir: &str,
        home_dir: &str,
        custom_args: &[String],
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(instance_id.as_bytes());
        hasher.update(b"|");
        hasher.update(data_dir.as_bytes());
        hasher.update(b"|");
        hasher.update(home_dir.as_bytes());
        hasher.update(b"|");

        for arg in custom_args {
            hasher.update(arg.as_bytes());
            hasher.update(b",");
        }

        format!("{:x}", hasher.finalize())
    }
}
