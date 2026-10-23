// Unit Test Suite for Port Collision Evictor (Imp-535)
// Validates TCP port availability checks, port range bounds [9222..9399],
// and dynamic alternative port resolution upon collision.

#[cfg(test)]
mod tests {
    use crate::modules::instance::port_collision_evictor::{
        is_port_available, resolve_collision_free_port, CDP_PORT_MAX, CDP_PORT_MIN,
    };
    use std::net::{SocketAddr, TcpListener};

    #[test]
    fn test_port_collision_constants() {
        assert_eq!(CDP_PORT_MIN, 9222);
        assert_eq!(CDP_PORT_MAX, 9399);
        assert!(CDP_PORT_MIN < CDP_PORT_MAX);
    }

    #[test]
    fn test_port_availability_detection() {
        // Find a free port in a high ephemeral range by binding port 0
        let listener = TcpListener::bind("127.0.0.1:0").expect("Ephemeral port bind must succeed");
        let bound_port = listener.local_addr().unwrap().port();

        // While listener is held, port is not available
        assert!(
            !is_port_available(bound_port),
            "Bound port must report unavailable"
        );

        // Once dropped, the port becomes available again
        drop(listener);
        assert!(
            is_port_available(bound_port),
            "Released port must report available"
        );
    }

    #[test]
    fn test_resolve_collision_free_port_when_requested_is_free() {
        // If a high safe port is free, resolve_collision_free_port returns it directly
        let test_port = 9388;
        if is_port_available(test_port) {
            let resolved = resolve_collision_free_port(test_port);
            assert_eq!(resolved, Some(test_port));
        }
    }

    #[test]
    fn test_resolve_collision_free_port_evicts_collision() {
        let test_port = 9370;
        // Occupy test_port intentionally
        if let Ok(occupier) = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], test_port))) {
            let resolved = resolve_collision_free_port(test_port);
            assert!(resolved.is_some());
            let alt_port = resolved.unwrap();
            assert_ne!(
                alt_port, test_port,
                "Must evict collision and select alternative port"
            );
            assert!(alt_port >= CDP_PORT_MIN && alt_port <= CDP_PORT_MAX);
            drop(occupier);
        }
    }
}
