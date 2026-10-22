#[cfg(test)]
mod tests {
    use super::cloned_telemetry_salt::ClonedTelemetrySalt;

    #[test]
    fn test_generate_salted_tokens_fields_populated() {
        let tokens = ClonedTelemetrySalt::generate_salted_tokens("inst-alpha");
        assert_eq!(tokens.machine_id.len(), 36);
        assert_eq!(tokens.mac_machine_id.len(), 36);
        assert_eq!(tokens.dev_device_id.len(), 36);
        // sqm_id has format {UUID} (38 chars)
        assert_eq!(tokens.sqm_id.len(), 38);
        assert!(tokens.sqm_id.starts_with('{'));
        assert!(tokens.sqm_id.ends_with('}'));
    }

    #[test]
    fn test_generate_salted_tokens_sqm_uppercase() {
        let tokens = ClonedTelemetrySalt::generate_salted_tokens("inst-beta");
        let inner = &tokens.sqm_id[1..tokens.sqm_id.len() - 1];
        for c in inner.chars() {
            if c.is_alphabetic() {
                assert!(c.is_uppercase(), "character {} should be uppercase in sqmId", c);
            }
        }
    }

    #[test]
    fn test_generate_salted_tokens_unique_tokens_within_instance() {
        let tokens = ClonedTelemetrySalt::generate_salted_tokens("inst-gamma");
        assert_ne!(tokens.machine_id, tokens.mac_machine_id);
        assert_ne!(tokens.machine_id, tokens.dev_device_id);
        assert_ne!(tokens.mac_machine_id, tokens.dev_device_id);
    }

    #[test]
    fn test_generate_salted_tokens_distinct_instances() {
        let t1 = ClonedTelemetrySalt::generate_salted_tokens("inst-1");
        let t2 = ClonedTelemetrySalt::generate_salted_tokens("inst-2");
        assert_ne!(t1.machine_id, t2.machine_id);
        assert_ne!(t1.sqm_id, t2.sqm_id);
    }

    #[test]
    fn test_generate_salted_tokens_handles_empty_instance_id() {
        let tokens = ClonedTelemetrySalt::generate_salted_tokens("");
        assert_eq!(tokens.machine_id.len(), 36);
        assert!(tokens.sqm_id.starts_with('{'));
    }
}
