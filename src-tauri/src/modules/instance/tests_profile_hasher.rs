// Unit Test Suite for Instance Profile Configuration SHA-256 Hasher (Imp-531)
// Validates deterministic SHA-256 state fingerprints, attribute order sensitivity,
// configuration drift detection, and boundary delimiter integrity.

#[cfg(test)]
mod tests {
    use crate::modules::instance::profile_state_hasher::ProfileStateHasher;
    use sha2::{Digest, Sha256};

    #[test]
    fn test_profile_state_hasher_deterministic_sha256_format_and_manual_vector() {
        let inst_id = "inst-primary-prod";
        let data_dir = "/data/antigravity/profiles/inst-primary";
        let home_dir = "/home/developer/.antigravity";
        let custom_args = vec![
            "--disable-gpu".to_string(),
            "--no-sandbox".to_string(),
            "--remote-debugging-port=9222".to_string(),
        ];

        let hash_1 =
            ProfileStateHasher::compute_profile_hash(inst_id, data_dir, home_dir, &custom_args);
        let hash_2 =
            ProfileStateHasher::compute_profile_hash(inst_id, data_dir, home_dir, &custom_args);

        // Assertions: Determinism & SHA-256 hex specification
        assert_eq!(
            hash_1, hash_2,
            "Repeated hash calculations must be strictly deterministic"
        );
        assert_eq!(
            hash_1.len(),
            64,
            "SHA-256 hex digest must be exactly 64 characters"
        );
        assert!(
            hash_1
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
            "Hash must consist solely of lowercase hex characters"
        );

        // Verification against manual baseline construction
        let mut manual_hasher = Sha256::new();
        manual_hasher.update(b"inst-primary-prod|");
        manual_hasher.update(b"/data/antigravity/profiles/inst-primary|");
        manual_hasher.update(b"/home/developer/.antigravity|");
        manual_hasher.update(b"--disable-gpu,");
        manual_hasher.update(b"--no-sandbox,");
        manual_hasher.update(b"--remote-debugging-port=9222,");
        let manual_hex = format!("{:x}", manual_hasher.finalize());

        assert_eq!(
            hash_1, manual_hex,
            "Hash must match canonical delimiter byte stream exactly"
        );
    }

    #[test]
    fn test_profile_state_hasher_detects_configuration_drift_across_all_fields() {
        let base_id = "inst-baseline";
        let base_data = "/var/lib/agm/data";
        let base_home = "/var/lib/agm/home";
        let base_args = vec!["--flag-a".to_string(), "--flag-b".to_string()];

        let baseline_hash =
            ProfileStateHasher::compute_profile_hash(base_id, base_data, base_home, &base_args);

        // Drift 1: instance_id mutation
        let drift_id = ProfileStateHasher::compute_profile_hash(
            "inst-mutated",
            base_data,
            base_home,
            &base_args,
        );
        assert_ne!(
            baseline_hash, drift_id,
            "Instance ID mutation must trigger drift"
        );

        // Drift 2: data_dir mutation
        let drift_data = ProfileStateHasher::compute_profile_hash(
            base_id,
            "/var/lib/agm/data-v2",
            base_home,
            &base_args,
        );
        assert_ne!(
            baseline_hash, drift_data,
            "Data directory mutation must trigger drift"
        );

        // Drift 3: home_dir mutation
        let drift_home = ProfileStateHasher::compute_profile_hash(
            base_id,
            base_data,
            "/var/lib/agm/home-alt",
            &base_args,
        );
        assert_ne!(
            baseline_hash, drift_home,
            "Home directory mutation must trigger drift"
        );

        // Drift 4: argument append
        let mut args_appended = base_args.clone();
        args_appended.push("--flag-c".to_string());
        let drift_arg_append =
            ProfileStateHasher::compute_profile_hash(base_id, base_data, base_home, &args_appended);
        assert_ne!(
            baseline_hash, drift_arg_append,
            "Adding an argument must trigger drift"
        );

        // Drift 5: argument re-ordering (order sensitivity)
        let mut args_reordered = base_args.clone();
        args_reordered.reverse();
        let drift_arg_order = ProfileStateHasher::compute_profile_hash(
            base_id,
            base_data,
            base_home,
            &args_reordered,
        );
        assert_ne!(
            baseline_hash, drift_arg_order,
            "Argument reordering must alter state hash"
        );

        // Drift 6: argument casing mutation
        let args_cased = vec!["--Flag-a".to_string(), "--flag-b".to_string()];
        let drift_arg_case =
            ProfileStateHasher::compute_profile_hash(base_id, base_data, base_home, &args_cased);
        assert_ne!(
            baseline_hash, drift_arg_case,
            "Single-byte argument mutation must trigger drift"
        );
    }

    #[test]
    fn test_profile_state_hasher_boundary_empty_inputs_and_delimiter_integrity() {
        let empty_hash = ProfileStateHasher::compute_profile_hash("", "", "", &[]);
        assert_eq!(empty_hash.len(), 64);

        let mut expected_empty = Sha256::new();
        expected_empty.update(b"|||");
        assert_eq!(empty_hash, format!("{:x}", expected_empty.finalize()));

        // Distinction between no args vs single empty string arg
        let single_empty_arg_hash =
            ProfileStateHasher::compute_profile_hash("", "", "", &[String::new()]);
        assert_ne!(
            empty_hash, single_empty_arg_hash,
            "Empty args slice (&[]) must not collide with slice containing empty arg (&[\"\"])"
        );

        let mut expected_single_empty = Sha256::new();
        expected_single_empty.update(b"|||,");
        assert_eq!(
            single_empty_arg_hash,
            format!("{:x}", expected_single_empty.finalize())
        );
    }
}
