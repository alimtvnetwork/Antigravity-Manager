use chrono::Utc;

use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_is_account_or_email_leased_by_other() {
        let local_node = supabase_sync::get_local_node_id();
        let now = Utc::now().timestamp();

        let mut cache = ACTIVE_REMOTE_LEASES.write().unwrap();
        cache.clear();

        // 1. Lease owned by local node (should not be considered "leased by other")
        cache.insert(
            "acc_local".to_string(),
            WorkspaceLease {
                account_id: "acc_local".to_string(),
                account_email: "local@example.com".to_string(),
                node_id: local_node.clone(),
                node_alias: "Node-Local".to_string(),
                ip_address: "127.0.0.1".to_string(),
                profile_name: "Profile-Local".to_string(),
                leased_at: now - 60,
                expires_at: now + 3600,
            },
        );

        // 2. Active lease owned by a remote node
        cache.insert(
            "acc_remote_active".to_string(),
            WorkspaceLease {
                account_id: "acc_remote_active".to_string(),
                account_email: "remote@example.com".to_string(),
                node_id: "node-remote-alpha".to_string(),
                node_alias: "Node-Alpha".to_string(),
                ip_address: "192.168.1.100".to_string(),
                profile_name: "Profile-Alpha".to_string(),
                leased_at: now - 60,
                expires_at: now + 3600,
            },
        );

        // 3. Expired lease owned by a remote node
        cache.insert(
            "acc_remote_expired".to_string(),
            WorkspaceLease {
                account_id: "acc_remote_expired".to_string(),
                account_email: "expired@example.com".to_string(),
                node_id: "node-remote-beta".to_string(),
                node_alias: "Node-Beta".to_string(),
                ip_address: "192.168.1.101".to_string(),
                profile_name: "Profile-Beta".to_string(),
                leased_at: now - 7200,
                expires_at: now - 60,
            },
        );

        // 4. Stale lease (> 6 hours since leased_at)
        cache.insert(
            "acc_remote_stale".to_string(),
            WorkspaceLease {
                account_id: "acc_remote_stale".to_string(),
                account_email: "stale@example.com".to_string(),
                node_id: "node-remote-gamma".to_string(),
                node_alias: "Node-Gamma".to_string(),
                ip_address: "192.168.1.102".to_string(),
                profile_name: "Profile-Gamma".to_string(),
                leased_at: now - 25000, // ~7 hours ago
                expires_at: now + 3600,
            },
        );

        // 5. Active lease matched by account_email (even when profile_name differs)
        cache.insert(
            "acc_by_email".to_string(),
            WorkspaceLease {
                account_id: "acc_by_email".to_string(),
                account_email: "team-shared@example.com".to_string(),
                node_id: "node-remote-delta".to_string(),
                node_alias: "Node-Delta".to_string(),
                ip_address: "192.168.1.103".to_string(),
                profile_name: "Profile-Different-Name".to_string(),
                leased_at: now - 100,
                expires_at: now + 3600,
            },
        );

        // 6. Recently leased by remote node within lockout window (even if expires_at has passed)
        cache.insert(
            "acc_remote_locked".to_string(),
            WorkspaceLease {
                account_id: "acc_remote_locked".to_string(),
                account_email: "locked@example.com".to_string(),
                node_id: "node-remote-epsilon".to_string(),
                node_alias: "Node-Epsilon".to_string(),
                ip_address: "192.168.1.104".to_string(),
                profile_name: "Profile-Epsilon".to_string(),
                leased_at: now - 300, // 5 min ago (< 60m lockout window)
                expires_at: now - 10,
            },
        );

        drop(cache);

        // Assertions:
        assert!(!is_account_or_email_leased_by_other("acc_local", ""));
        assert!(is_account_or_email_leased_by_other("acc_remote_active", ""));
        assert!(is_account_or_email_leased_by_other("ACC_REMOTE_ACTIVE", "")); // Case-insensitive ID
        assert!(!is_account_or_email_leased_by_other(
            "acc_remote_expired",
            ""
        ));
        assert!(!is_account_or_email_leased_by_other("acc_remote_stale", ""));
        assert!(is_account_or_email_leased_by_other(
            "some_random_id",
            "team-shared@example.com"
        ));
        assert!(is_account_or_email_leased_by_other(
            "some_random_id",
            "TEAM-SHARED@EXAMPLE.COM" // Case-insensitive email
        ));
        assert!(is_account_or_email_leased_by_other("acc_remote_locked", ""));
        assert!(!is_account_or_email_leased_by_other(
            "acc_non_existent",
            "other@example.com"
        ));
        assert!(has_active_lease_in_cooldown("acc_remote_active", "", 3600));
        assert!(has_active_lease_in_cooldown(
            "",
            "TEAM-SHARED@example.com",
            3600
        ));
        assert!(!has_active_lease_in_cooldown("acc_non_existent", "", 3600));

        // Cleanup
        let mut cache = ACTIVE_REMOTE_LEASES.write().unwrap();
        cache.clear();
    }
}
