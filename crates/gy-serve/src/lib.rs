//! gy serve: the local reading surface for the master (n-a493, d-25c4). The
//! HTTP layer is the standard library's, in `server` alone (d-e6c4); routes
//! see `http`.
pub mod api;
pub mod assets;
pub mod graph_cache;
pub mod http;
pub mod layout;
pub mod server;
pub mod watch;
