// Upstream TLS ALPN Protocol Negotiator (Imp-514)
// Configures Application-Layer Protocol Negotiation (ALPN) suites for TLS connections,
// enforcing prioritize-HTTP/2 with automatic graceful fallback to HTTP/1.1.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlpnProtocol {
    Http2,
    Http11,
}

pub struct TlsAlpnNegotiator;

impl TlsAlpnNegotiator {
    /// Returns default wire-format ALPN protocols prioritizing h2.
    pub fn default_alpn_protocols() -> Vec<Vec<u8>> {
        vec![b"h2".to_vec(), b"http/1.1".to_vec()]
    }

    /// Converts negotiated wire bytes into a strongly typed AlpnProtocol enum.
    pub fn parse_negotiated_protocol(wire_bytes: Option<&[u8]>) -> AlpnProtocol {
        match wire_bytes {
            Some(b"h2") => AlpnProtocol::Http2,
            Some(b"http/1.1") => AlpnProtocol::Http11,
            _ => AlpnProtocol::Http11, // Safe fallback
        }
    }

    /// Checks if HTTP/2 multiplexing is supported by the negotiated protocol.
    pub fn supports_http2(protocol: AlpnProtocol) -> bool {
        protocol == AlpnProtocol::Http2
    }
}
