//! gy serve: the local reading surface for the master (n-a493, d-25c4). The
//! HTTP framework appears in `server` alone (ac-a49a); routes see `http`.
pub mod api;
pub mod assets;
pub mod http;
pub mod server;
pub mod watch;
