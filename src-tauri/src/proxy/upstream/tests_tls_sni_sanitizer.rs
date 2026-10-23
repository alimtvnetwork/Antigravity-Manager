use super::*;

#[cfg(test)]
mod tests {
    use super::tls_sni_sanitizer::{sanitize_tls_sni, SanitizedSni};

    #[test]
    fn test_standard_fqdn() {
        let res = sanitize_tls_sni("api.openai.com").unwrap();
        assert_eq!(
            res,
            SanitizedSni {
                hostname: "api.openai.com".to_string(),
                port: None,
                is_idn: false,
                is_ip_literal: false,
            }
        );
    }

    #[test]
    fn test_strips_https_scheme_and_path() {
        let res = sanitize_tls_sni("https://api.anthropic.com/v1/messages?query=1#frag").unwrap();
        assert_eq!(
            res,
            SanitizedSni {
                hostname: "api.anthropic.com".to_string(),
                port: None,
                is_idn: false,
                is_ip_literal: false,
            }
        );
    }

    #[test]
    fn test_extracts_explicit_port() {
        let res = sanitize_tls_sni("http://localhost:8080/path").unwrap();
        assert_eq!(
            res,
            SanitizedSni {
                hostname: "localhost".to_string(),
                port: Some(8080),
                is_idn: false,
                is_ip_literal: false,
            }
        );
    }

    #[test]
    fn test_ipv4_literal() {
        let res = sanitize_tls_sni("127.0.0.1:443").unwrap();
        assert_eq!(
            res,
            SanitizedSni {
                hostname: "127.0.0.1".to_string(),
                port: Some(443),
                is_idn: false,
                is_ip_literal: true,
            }
        );
    }

    #[test]
    fn test_ipv6_literal() {
        let res = sanitize_tls_sni("[::1]:8443").unwrap();
        assert_eq!(
            res,
            SanitizedSni {
                hostname: "::1".to_string(),
                port: Some(8443),
                is_idn: false,
                is_ip_literal: true,
            }
        );
    }

    #[test]
    fn test_detects_idn_punycode() {
        let res = sanitize_tls_sni("xn--fiqs8s.cn").unwrap();
        assert_eq!(res.hostname, "xn--fiqs8s.cn");
        assert!(res.is_idn);
        assert!(!res.is_ip_literal);
    }

    #[test]
    fn test_empty_host_rejected() {
        assert!(sanitize_tls_sni("   ").is_err());
    }

    #[test]
    fn test_invalid_label_hyphens_rejected() {
        assert!(sanitize_tls_sni("-invalid.com").is_err());
        assert!(sanitize_tls_sni("invalid-.com").is_err());
    }
}
