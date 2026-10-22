//! Unit tests for Remote Debugging Port Allocator.

use super::remote_debugging_port_allocator::{
    RemoteDebuggingPortAllocator, DEFAULT_CDP_PORT_END, DEFAULT_CDP_PORT_START,
};
use std::net::TcpListener;

#[test]
fn test_default_port_range_constants() {
    assert_eq!(DEFAULT_CDP_PORT_START, 9222);
    assert_eq!(DEFAULT_CDP_PORT_END, 9322);
    assert!(DEFAULT_CDP_PORT_END > DEFAULT_CDP_PORT_START);
}

#[test]
fn test_is_port_available_detects_bound_port() {
    // Bind to a free ephemeral port
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind ephemeral port");
    let bound_port = listener.local_addr().expect("local addr").port();

    // Since listener is currently open, is_port_available should return false
    assert!(!RemoteDebuggingPortAllocator::is_port_available(bound_port));

    // Drop listener to release port
    drop(listener);

    // Port should now be available again
    assert!(RemoteDebuggingPortAllocator::is_port_available(bound_port));
}

#[test]
fn test_allocate_free_port_custom_range() {
    let port = RemoteDebuggingPortAllocator::allocate_free_port(Some(19222), Some(19230));
    assert!(port.is_some());
    let p = port.unwrap();
    assert!(p >= 19222 && p <= 19230);
}
