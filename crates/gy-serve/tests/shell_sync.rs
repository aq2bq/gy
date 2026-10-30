//! `/api/shell` carries the synced copy's state for the band (d-6d90): the same
//! object `/api/now` answered under `resume.sync` before that route was removed,
//! and nothing at all on a local ledger (n-ecbf, ac-7ae3).
use gy_ledger::{CriterionAdd, FileStore, FormatVersion, Operation, Repository, format};
use gy_serve::api::route;
use gy_serve::http::Request;
use std::path::{Path, PathBuf};

/// A throwaway directory under the system's temp root.
fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gy-serve-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// One written ledger, a remote marker, and the sync state a push would leave.
fn ledger(dir: &Path, sync: &str) {
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let store = FileStore::open_with(dir, |_| Some("piko".to_string())).unwrap();
    let mut repo = Repository::new(store).with_scopes(vec!["a".to_string()]);
    CriterionAdd {
        scope: "a".to_string(),
        title: "measurable".to_string(),
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    std::fs::write(dir.join("remote"), "https://example.com/ledger.git\n").unwrap();
    std::fs::write(dir.join("sync.state"), sync).unwrap();
}

fn shell(dir: &Path) -> serde_json::Value {
    let store = FileStore::open_with(dir, |_| Some("piko".to_string())).unwrap();
    let repo: Repository<FileStore> = Repository::new(store);
    let response = route(&repo, &Request::new("GET", "/api/shell", None, &[]), "gy");
    assert_eq!(response.status, 200);
    serde_json::from_slice(&response.body).unwrap()
}

#[test]
fn the_shell_carries_the_sync_state() {
    let dir = tempdir("shell-sync");
    ledger(
        &dir,
        r#"{"last_ok_at": 1793000000, "last_ok_seq": 1, "last_error": "unreachable\nsecond line"}"#,
    );
    let json = shell(&dir);
    let sync = &json["sync"];
    assert_eq!(sync["pending"], 1, "{json}");
    assert_eq!(sync["last_ok_at"], 1793000000);
    assert_eq!(sync["last_ok_seq"], 1);
    assert_eq!(sync["last_error"], "unreachable\nsecond line");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A local ledger keeps nothing: the band has no mark to draw.
#[test]
fn a_local_ledger_has_no_sync() {
    let dir = tempdir("shell-no-sync");
    format::write(&dir, FormatVersion::CURRENT).unwrap();
    let store = FileStore::open_with(&dir, |_| Some("piko".to_string())).unwrap();
    let mut repo = Repository::new(store).with_scopes(vec!["a".to_string()]);
    CriterionAdd {
        scope: "a".to_string(),
        title: "measurable".to_string(),
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    let json = shell(&dir);
    assert!(json.get("sync").is_none(), "{json}");
    let _ = std::fs::remove_dir_all(&dir);
}
