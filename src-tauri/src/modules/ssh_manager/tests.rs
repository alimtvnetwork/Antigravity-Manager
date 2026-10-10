use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_validate_ssh_public_key() {
        let sample = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl user@host";
        let (line, blob) = validate_ssh_public_key(sample).unwrap();
        assert_eq!(line, sample);
        assert_eq!(
            blob,
            "AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl"
        );
    }

    #[test]
    pub(crate) fn test_decode_connections_from_envelope() {
        let json = r#"{
            "schema_version": "1.0",
            "exported_at": "2026-09-27T10:10:07Z",
            "total_nodes": 1,
            "nodes": [
                {
                    "worker_id": "worker-1",
                    "id": 1,
                    "alias": "w1",
                    "ip_address": "192.168.1.3",
                    "username": "Administrator",
                    "port": 22,
                    "os": "windows",
                    "auth_method": "key",
                    "key_path": ""
                }
            ],
            "connections": []
        }"#;
        let conns = decode_connections_from_json(json).unwrap();
        assert_eq!(conns.len(), 1);
        assert_eq!(conns[0].alias, "w1");
        assert_eq!(conns[0].ip_address, "192.168.1.3");
    }
}
