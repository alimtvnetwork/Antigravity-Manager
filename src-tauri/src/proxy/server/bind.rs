//! TCP listener binding with SO_REUSEADDR and IPv4/IPv6 dual-stack support.

/// Binds a single socket address (IPv4 or specific IPv6)
fn bind_single_socket(
    socket_addr: std::net::SocketAddr,
) -> Result<tokio::net::TcpListener, String> {
    let domain = if socket_addr.is_ipv6() {
        socket2::Domain::IPV6
    } else {
        socket2::Domain::IPV4
    };

    let socket = socket2::Socket::new(domain, socket2::Type::STREAM, Some(socket2::Protocol::TCP))
        .map_err(|e| format!("Failed to create socket ({}): {}", socket_addr, e))?;

    // Enable SO_REUSEADDR on Windows/Unix to prevent WSAEADDRINUSE (10048) on restarts
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(socket.set_reuse_address(true), "set_reuse_address");

    #[cfg(unix)]
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(socket.set_reuse_port(true), "set_reuse_port");

    socket
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking mode ({}): {}", socket_addr, e))?;

    socket
        .bind(&socket_addr.into())
        .map_err(|e| format!("Failed to bind address {}: {}", socket_addr, e))?;

    socket
        .listen(1024)
        .map_err(|e| format!("Failed to listen on address {}: {}", socket_addr, e))?;

    let std_listener: std::net::TcpListener = socket.into();
    tokio::net::TcpListener::from_std(std_listener).map_err(|e| {
        format!(
            "Failed to convert to Tokio TcpListener ({}): {}",
            socket_addr, e
        )
    })
}

/// Binds IPv6/IPv4 dual-stack wildcard listener ([::]:port), accepting both IPv6 and IPv4 on a single socket
fn bind_dual_stack_socket(port: u16) -> Result<tokio::net::TcpListener, String> {
    use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};

    let socket = socket2::Socket::new(
        socket2::Domain::IPV6,
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP),
    )
    .map_err(|e| format!("Failed to create IPv6 dual-stack socket: {}", e))?;

    // Critical: Windows defaults only_v6 to true, must explicitly set false for dual-stack IPv4 acceptance
    if let Err(e) = socket.set_only_v6(false) {
        return Err(format!(
            "Failed to enable dual-stack support (set_only_v6(false)): {}",
            e
        ));
    }

    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(socket.set_reuse_address(true), "set_reuse_address");

    #[cfg(unix)]
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(socket.set_reuse_port(true), "set_reuse_port");

    socket
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking mode: {}", e))?;

    let addr = SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0));
    socket
        .bind(&addr.into())
        .map_err(|e| format!("Failed to bind dual-stack address [::]:{}: {}", port, e))?;

    socket.listen(1024).map_err(|e| {
        format!(
            "Failed to listen on dual-stack address [::]:{}: {}",
            port, e
        )
    })?;

    let std_listener: std::net::TcpListener = socket.into();
    tokio::net::TcpListener::from_std(std_listener).map_err(|e| {
        format!(
            "Failed to convert to Tokio TcpListener ([::]:{}): {}",
            port, e
        )
    })
}

/// Binds TCP listener with SO_REUSEADDR and dual-stack IPv6/IPv4 wildcard support
pub(crate) fn bind_tcp_listener(host: &str, port: u16) -> Result<tokio::net::TcpListener, String> {
    let clean_host = host.trim_matches('[').trim_matches(']');
    let is_wildcard = clean_host == "0.0.0.0" || clean_host == "::";

    if is_wildcard {
        // Try binding in IPv6 / IPv4 dual-stack mode on [::]:port
        match bind_dual_stack_socket(port) {
            Ok(listener) => {
                tracing::info!(
                    "TCP listener ready on [::]:{} (IPv6/IPv4 dual-stack mode)",
                    port
                );
                return Ok(listener);
            }
            Err(e) => {
                tracing::warn!(
                    "IPv6 dual-stack bind failed ({}), gracefully falling back to IPv4 listener (0.0.0.0:{})",
                    e,
                    port
                );
            }
        }
        // Graceful fallback to IPv4 0.0.0.0
        let v4_addr =
            std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), port);
        return bind_single_socket(v4_addr);
    }

    // Exact address binding (e.g. 127.0.0.1, ::1, or specific interface IP)
    use std::net::ToSocketAddrs;
    let addr_str = if clean_host.contains(':') {
        format!("[{}]:{}", clean_host, port)
    } else {
        format!("{}:{}", clean_host, port)
    };
    let socket_addr = addr_str
        .to_socket_addrs()
        .map_err(|e| format!("Failed to resolve address {}: {}", addr_str, e))?
        .next()
        .ok_or_else(|| format!("Failed to resolve address: {}", addr_str))?;

    bind_single_socket(socket_addr)
}
