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

    #[test]
    pub(crate) fn test_ssh_connections_rsa_password_encryption() {
        let raw_record = SshConnectionRecord {
            alias: "vm-test".to_string(),
            ip_address: "10.0.0.42".to_string(),
            username: "Administrator".to_string(),
            port: 2222,
            encrypted_password: "MyPlaintextSshPassword!99".to_string(),
            encryption_algo: String::new(),
            key_path: String::new(),
            os: "windows".to_string(),
            os_group: None,
            os_version: None,
            build_version: None,
            first_run_at: None,
            created_at: None,
        };
        let encrypted_list = ensure_connections_rsa_encrypted(&[raw_record]);
        assert_eq!(encrypted_list.len(), 1);
        let rec = &encrypted_list[0];
        assert!(rec
            .encrypted_password
            .starts_with(crate::utils::crypto::ENCRYPTED_RSA_V1_PREFIX));
        assert!(!rec.encrypted_password.contains("MyPlaintextSshPassword!99"));
        assert_eq!(rec.encryption_algo, "RSA-2048");
        assert_eq!(rec.port, 2222);

        let decrypted =
            crate::utils::crypto::decrypt_password_rsa(&rec.encrypted_password).unwrap();
        assert_eq!(decrypted, "MyPlaintextSshPassword!99");
    }

    #[test]
    pub(crate) fn test_parse_openssh_config_hosts() {
        let sample_cfg = r#"
Host w1
    HostName 192.168.1.10
    User Administrator
    Port 2222

Host *
    ServerAliveInterval 60
"#;
        let hosts = parse_openssh_config_hosts(sample_cfg);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].alias, "w1");
        assert_eq!(hosts[0].ip_address, "192.168.1.10");
        assert_eq!(hosts[0].username, "Administrator");
        assert_eq!(hosts[0].port, 2222);
    }
}
