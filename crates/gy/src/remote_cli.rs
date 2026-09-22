//! The `remote` group's leaves (n-8d0e): the three operations that decide where
//! the record lives, under one name. `set` / `join` / `sync` do what the old
//! top-level `share` / `join` / `sync` did; main.rs wires them.
use clap::Subcommand;

#[derive(Subcommand)]
pub enum RemoteAction {
    /// Start sharing: check the remote, write gy.toml, upload the ledger.
    Set {
        #[arg(value_name = "URL")]
        url: String,
    },
    /// Join a shared project: check what you need, take the copy, say who you write as.
    Join,
    /// Sync this copy with the remote named in gy.toml.
    Sync,
}
