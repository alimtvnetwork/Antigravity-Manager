use crate::proxy::config::{ProxyAuthMode, ProxyConfig, SecurityMonitorConfig};
use std::net::IpAddr;

/// Imp-53: Checks if an IP address belongs to a specified CIDR block (e.g. "192.168.1.0/24")
pub fn is_ip_in_cidr(ip: &IpAddr, cidr: &str) -> bool {
    let parts: Vec<&str> = cidr.trim().split('/').collect();
    if parts.is_empty() {
        return false;
    }

    let network_ip: IpAddr = match parts[0].parse() {
        Ok(ip) => ip,
        Err(_) => return false,
    };

    let prefix_len: u32 = if parts.len() > 1 {
        match parts[1].parse() {
            Ok(len) => len,
            Err(_) => return false,
        }
    } else {
        match network_ip {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        }
    };

    match (ip, network_ip) {
        (IpAddr::V4(ip4), IpAddr::V4(net4)) => {
            if prefix_len > 32 {
                return false;
            }
            if prefix_len == 0 {
                return true;
            }
            let mask = !((1u32 << (32 - prefix_len)) - 1);
            (u32::from(*ip4) & mask) == (u32::from(net4) & mask)
        }
        (IpAddr::V6(ip6), IpAddr::V6(net6)) => {
            if prefix_len > 128 {
                return false;
            }
            if prefix_len == 0 {
                return true;
            }
            let mask = !((1u128 << (128 - prefix_len)) - 1);
            (u128::from(*ip6) & mask) == (u128::from(net6) & mask)
        }
        _ => false,
    }
}

/// Imp-53: Checks if an IP matches any CIDR rule in a list
pub fn is_ip_allowed_by_cidr_list(ip: &IpAddr, cidr_list: &[String]) -> bool {
    if cidr_list.is_empty() {
        return true;
    }
    cidr_list.iter().any(|cidr| is_ip_in_cidr(ip, cidr))
}

#[derive(Debug, Clone)]
pub struct ProxySecurityConfig {
    pub auth_mode: ProxyAuthMode,
    pub api_key: String,
    pub admin_password: Option<String>,
    pub allow_lan_access: bool,
    pub port: u16,
    pub security_monitor: SecurityMonitorConfig,
}

impl ProxySecurityConfig {
    pub fn from_proxy_config(config: &ProxyConfig) -> Self {
        Self {
            auth_mode: config.auth_mode.clone(),
            api_key: config.api_key.clone(),
            admin_password: config.admin_password.clone(),
            allow_lan_access: config.allow_lan_access,
            port: config.port,
            security_monitor: config.security_monitor.clone(),
        }
    }

    pub fn effective_auth_mode(&self) -> ProxyAuthMode {
        match self.auth_mode {
            ProxyAuthMode::Auto => {
                if self.allow_lan_access {
                    ProxyAuthMode::AllExceptHealth
                } else {
                    ProxyAuthMode::Off
                }
            }
            ref other => other.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_mode_resolves_off_for_local_only() {
        let s = ProxySecurityConfig {
            auth_mode: ProxyAuthMode::Auto,
            api_key: "sk-test".to_string(),
            admin_password: None,
            allow_lan_access: false,
            port: 8080,
            security_monitor: crate::proxy::config::SecurityMonitorConfig::default(),
        };
        assert!(matches!(s.effective_auth_mode(), ProxyAuthMode::Off));
    }

    #[test]
    fn auto_mode_resolves_all_except_health_for_lan() {
        let s = ProxySecurityConfig {
            auth_mode: ProxyAuthMode::Auto,
            api_key: "sk-test".to_string(),
            admin_password: None,
            allow_lan_access: true,
            port: 8080,
            security_monitor: crate::proxy::config::SecurityMonitorConfig::default(),
        };
        assert!(matches!(
            s.effective_auth_mode(),
            ProxyAuthMode::AllExceptHealth
        ));
    }
}
