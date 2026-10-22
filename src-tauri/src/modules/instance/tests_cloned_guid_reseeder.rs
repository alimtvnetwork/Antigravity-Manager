#[cfg(test)]
mod tests {
    use super::cloned_guid_reseeder::{generate_synthetic_uuid, reseed_storage_json};
    use serde_json::json;

    #[test]
    fn test_generate_synthetic_uuid_format() {
        let uuid = generate_synthetic_uuid();
        assert_eq!(uuid.len(), 36);
        let parts: Vec<&str> = uuid.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
        // Version 4 UUID check
        assert!(parts[2].starts_with('4'));
    }

    #[test]
    fn test_generate_synthetic_uuid_distinctness() {
        let u1 = generate_synthetic_uuid();
        let u2 = generate_synthetic_uuid();
        assert_ne!(u1, u2);
    }

    #[test]
    fn test_reseed_storage_json_injects_all_target_keys() {
        let mut storage = json!({
            "existing_key": "some_value"
        });
        let modified = reseed_storage_json(&mut storage);
        assert_eq!(modified, 5);

        let obj = storage.as_object().expect("expected object");
        assert_eq!(obj.get("existing_key").unwrap(), "some_value");
        assert!(obj.contains_key("telemetry.machineId"));
        assert!(obj.contains_key("telemetry.devDeviceId"));
        assert!(obj.contains_key("telemetry.sqmId"));
        assert!(obj.contains_key("telemetry.macMachineId"));
        assert!(obj.contains_key("storage.machineGuid"));

        // Confirm injected values are valid strings
        let machine_id = obj.get("telemetry.machineId").unwrap().as_str().unwrap();
        assert_eq!(machine_id.len(), 36);
    }

    #[test]
    fn test_reseed_storage_json_overwrites_existing_telemetry() {
        let mut storage = json!({
            "telemetry.machineId": "old-machine-id",
            "telemetry.devDeviceId": "old-dev-id",
            "telemetry.sqmId": "old-sqm-id",
            "telemetry.macMachineId": "old-mac-id",
            "storage.machineGuid": "old-guid"
        });
        let modified = reseed_storage_json(&mut storage);
        assert_eq!(modified, 5);

        let obj = storage.as_object().unwrap();
        assert_ne!(obj.get("telemetry.machineId").unwrap(), "old-machine-id");
        assert_ne!(obj.get("storage.machineGuid").unwrap(), "old-guid");
    }

    #[test]
    fn test_reseed_storage_json_handles_non_object() {
        let mut storage = json!(["not", "an", "object"]);
        let modified = reseed_storage_json(&mut storage);
        assert_eq!(modified, 0);
    }

    #[test]
    fn test_reseed_storage_json_unique_across_runs() {
        let mut storage1 = json!({});
        let mut storage2 = json!({});
        reseed_storage_json(&mut storage1);
        reseed_storage_json(&mut storage2);

        let id1 = storage1["telemetry.machineId"].as_str().unwrap();
        let id2 = storage2["telemetry.machineId"].as_str().unwrap();
        assert_ne!(id1, id2);
    }
}
