use crate::modules::supabase_client::{normalize_supabase_url, SupabaseClient, SupabaseEndpoint};
use serde_json::json;
use std::path::PathBuf;

use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_default_supabase_config() {
        let def = SupabaseConfig::default();
        assert!(!def.is_sync_enabled);
        assert_eq!(def.heartbeat_interval_secs, 30);
        assert_eq!(def.auto_prune_root_mb, 400);
        assert_eq!(def.auto_prune_secondary_mb, 200);
        assert!(def.node_alias.starts_with("Node-"));
        assert!(def.endpoints.is_empty());
    }

    #[test]
    pub(crate) fn test_supabase_config_serde_and_url_normalization() {
        let json_data = r#"{
            "endpoints": [
                {
                    "id": "ep1",
                    "name": "Endpoint 1",
                    "url": "https://sample.supabase.co/rest/v1/",
                    "api_key": "sample-key",
                    "role": "root",
                    "is_enabled": true,
                    "prune_threshold_mb": 400,
                    "priority": 1,
                    "notes": null,
                    "tags": []
                }
            ],
            "node_alias": "Node-Test",
            "is_sync_enabled": true,
            "auto_prune_root_mb": 400,
            "auto_prune_secondary_mb": 200,
            "heartbeat_interval_secs": 30
        }"#;

        let mut config: SupabaseConfig = serde_json::from_str(json_data).expect("valid json");
        assert_eq!(
            config.endpoints[0].url,
            "https://sample.supabase.co/rest/v1/"
        );

        // Normalize endpoints as done in load_config
        for ep in &mut config.endpoints {
            ep.url = normalize_supabase_url(&ep.url);
        }
        assert_eq!(config.endpoints[0].url, "https://sample.supabase.co");

        // Verify re-serialization maintains normalized URL
        let serialized = serde_json::to_string(&config).expect("serialization works");
        assert!(serialized.contains("\"https://sample.supabase.co\""));
        assert!(!serialized.contains("/rest/v1/\""));
    }

    #[test]
    pub(crate) fn test_postgrest_error_message_pgrst205() {
        let err_json = json!({
            "code": "PGRST205",
            "details": null,
            "hint": null,
            "message": "relation \"nodes\" does not exist"
        });
        let msg = postgrest_error_message(&err_json).expect("should extract error");
        assert_eq!(msg, "relation \"nodes\" does not exist (PGRST205)");

        let err_only_code = json!({ "code": "PGRST205" });
        let msg_code = postgrest_error_message(&err_only_code).expect("should extract error");
        assert_eq!(msg_code, "PostgREST error (PGRST205)");

        let ok_payload = json!({ "id": "node1", "status": "online" });
        assert!(postgrest_error_message(&ok_payload).is_none());
    }

    #[test]
    pub(crate) fn test_email_from_postgrest_readback() {
        let row_array = json!([{ "active_account_email": "test@example.com" }]);
        assert_eq!(
            email_from_postgrest(&row_array),
            Some("test@example.com".to_string())
        );

        let single_row = json!({ "active_account_email": "hello@example.com" });
        assert_eq!(
            email_from_postgrest(&single_row),
            Some("hello@example.com".to_string())
        );

        let empty_array = json!([]);
        assert_eq!(email_from_postgrest(&empty_array), None);

        let empty_email = json!([{ "active_account_email": "   " }]);
        assert_eq!(email_from_postgrest(&empty_email), None);
    }

    #[test]
    pub(crate) fn test_candidate_repo_secrets_paths() {
        let paths = candidate_repo_secrets_paths();
        assert!(!paths.is_empty());
        let path_strs: Vec<String> = paths
            .iter()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .collect();
        assert!(path_strs
            .iter()
            .any(|p| p.contains("03-supabase/01-own/supabase-credentials.json")));
        assert!(path_strs
            .iter()
            .any(|p| p.contains("03-supabase/02-lovable/supabase-credentials.json")));
    }

    #[test]
    pub(crate) fn test_auto_seed_from_repo_secrets_execution() {
        let mut cfg = SupabaseConfig::default();
        assert!(cfg.endpoints.is_empty());
        assert!(!cfg.is_sync_enabled);

        let seeded = auto_seed_from_repo_secrets(&mut cfg);
        let d_own =
            PathBuf::from("D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json");
        if d_own.exists() {
            assert!(seeded);
            assert!(!cfg.endpoints.is_empty());
            assert!(cfg.is_sync_enabled);
            if let Some(first) = cfg.endpoints.first() {
                assert_eq!(first.priority, 1);
                assert_eq!(first.role, "root");
            }
        }
    }

    #[test]
    pub(crate) fn test_build_local_fallback_machine() {
        let machines = build_local_fallback_machine();
        assert_eq!(machines.len(), 1);
        let m = &machines[0];
        assert!(m.is_local);
        assert!(m.is_online);
        assert!(!m.node_id.is_empty());
    }

    #[test]
    pub(crate) fn test_fleet_machine_info_sorting() {
        let mut machines = vec![
            FleetMachineInfo {
                node_id: "node-offline".to_string(),
                node_alias: "Bravo Offline".to_string(),
                os_info: None,
                ip_address: "10.0.0.2".to_string(),
                is_online: false,
                last_heartbeat_timestamp: 0,
                uptime_seconds: 10,
                in_flight_prompts_count: 0,
                active_instances: vec![],
                bound_emails: vec![],
                leases: vec![],
                is_local: false,
                ..Default::default()
            },
            FleetMachineInfo {
                node_id: "node-online".to_string(),
                node_alias: "Charlie Online".to_string(),
                os_info: None,
                ip_address: "10.0.0.3".to_string(),
                is_online: true,
                last_heartbeat_timestamp: 100,
                uptime_seconds: 50,
                in_flight_prompts_count: 0,
                active_instances: vec![],
                bound_emails: vec![],
                leases: vec![],
                is_local: false,
                ..Default::default()
            },
            FleetMachineInfo {
                node_id: "node-local".to_string(),
                node_alias: "Alpha Local".to_string(),
                os_info: None,
                ip_address: "127.0.0.1".to_string(),
                is_online: true,
                last_heartbeat_timestamp: 100,
                uptime_seconds: 100,
                in_flight_prompts_count: 1,
                active_instances: vec![],
                bound_emails: vec![],
                leases: vec![],
                is_local: true,
                ..Default::default()
            },
        ];

        machines.sort_by(|a, b| {
            if a.is_local != b.is_local {
                return b.is_local.cmp(&a.is_local);
            }
            if a.is_online != b.is_online {
                return b.is_online.cmp(&a.is_online);
            }
            a.node_alias
                .to_lowercase()
                .cmp(&b.node_alias.to_lowercase())
        });

        assert_eq!(machines[0].node_id, "node-local");
        assert!(machines[0].is_local);
        assert_eq!(machines[1].node_id, "node-online");
        assert_eq!(machines[2].node_id, "node-offline");
    }

    #[test]
    pub(crate) fn test_fleet_machine_info_and_local_construction() {
        let cfg = SupabaseConfig::default();
        let local_info = construct_local_machine_info(&cfg, &[]);
        assert!(local_info.is_local);
        assert_eq!(local_info.status, "online");
        assert_eq!(local_info.source, "local");
        assert!(!local_info.node_id.is_empty());

        let serialized = serde_json::to_string(&local_info).expect("serialize works");
        assert!(serialized.contains("\"is_local\":true"));
    }
}
