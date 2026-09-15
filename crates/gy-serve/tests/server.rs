use gy_serve::server::bind;
use std::net::TcpListener;

/// Hold the first free port of the range, then bind again: the server must take
/// a different port (7331 is not assumed free).
#[test]
fn bind_skips_a_bound_port() {
    let held = (7331..=7400u16).find_map(|port| TcpListener::bind(("127.0.0.1", port)).ok());
    let held = held.expect("a free port in the range");
    let (server, port) = bind().unwrap();
    assert_ne!(port, held.local_addr().unwrap().port());
    drop(server);
}
