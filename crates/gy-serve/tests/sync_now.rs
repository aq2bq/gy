//! /api/now carries the shared copy's sync, and a local ledger has none
//! (n-94bb 3A2).
use gy_ledger::{FileStore, FormatVersion, Repository, format};
use gy_serve::api::route;
use gy_serve::http::Request;
use std::path::{Path, PathBuf};

/// A throwaway directory under the system's temp root (no extra dependency).
fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gy-serve-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn now(repo: &Repository<FileStore>) -> serde_json::Value {
    let res = route(repo, &Request::new("GET", "/api/now", None, &[]), "gy");
    serde_json::from_slice(&res.body).unwrap()
}

fn open(dir: &Path) -> Repository<FileStore> {
    Repository::new(FileStore::open_with(dir, |_| Some("piko".into())).unwrap())
}

#[test]
fn the_now_view_carries_sync_for_a_marked_copy() {
    let dir = tempdir("sync-now");
    format::write(&dir, FormatVersion::CURRENT).unwrap();
    std::fs::write(dir.join("remote"), "file:///example/ledger.git").unwrap();
    std::fs::write(
        dir.join("sync.state"),
        r#"{"last_ok_at":1,"last_ok_seq":3,"last_error":"peer moved ac-0001"}"#,
    )
    .unwrap();

    let value = now(&open(&dir));
    assert_eq!(value["resume"]["sync"]["last_ok_seq"], 3);
    assert!(
        value["resume"]["sync"]["last_error"]
            .as_str()
            .unwrap()
            .contains("peer moved")
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_now_view_has_no_sync_for_a_local_ledger() {
    let dir = tempdir("sync-none");
    format::write(&dir, FormatVersion::CURRENT).unwrap();

    let value = now(&open(&dir));
    assert!(value["resume"].get("sync").is_none());
    let _ = std::fs::remove_dir_all(&dir);
}
