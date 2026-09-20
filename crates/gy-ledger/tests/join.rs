//! `gy join`'s copy and words (n-57c5, ac-545c). Local `file://` remotes.
use gy_ledger::{
    CriterionAdd, FileStore, FormatVersion, Operation, Repository, format, join_notice, join_run,
    sync,
};
use std::path::Path;
use std::process::Command;

fn url(dir: &Path) -> String {
    format!("file://{}", dir.display())
}

/// One name, given the way git gives it, and no per-user config to argue with
/// it: the notice names the same writer the commits carry (d-a4f6). The copy
/// with no name at all is in `join_no_name.rs`. Process-wide, once.
fn isolate() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        let path = std::env::temp_dir().join(format!("gy-join-empty-{}", std::process::id()));
        std::fs::write(&path, "").unwrap();
        for (key, value) in [
            ("GIT_CONFIG_GLOBAL", path.display().to_string()),
            ("GIT_CONFIG_SYSTEM", "/dev/null".to_string()),
            ("GIT_CONFIG_NOSYSTEM", "1".to_string()),
            ("GIT_AUTHOR_NAME", "piko".to_string()),
            ("GIT_AUTHOR_EMAIL", "piko@example.com".to_string()),
            ("GIT_COMMITTER_NAME", "piko".to_string()),
            ("GIT_COMMITTER_EMAIL", "piko@example.com".to_string()),
        ] {
            std::env::set_var(key, value);
        }
    });
}

fn bare(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    let out = Command::new("git")
        .args(["init", "-q", "--bare"])
        .arg(dir)
        .output()
        .unwrap();
    assert!(out.status.success());
}

/// A source ledger with one write, pushed to `remote`.
fn source(remote: &Path) {
    let dir = remote.with_extension("source");
    std::fs::create_dir_all(&dir).unwrap();
    format::write(&dir, FormatVersion::CURRENT).unwrap();
    let mut repo = Repository::new(FileStore::open_with(&dir, |_| Some("piko".into())).unwrap());
    CriterionAdd {
        scope: "a".into(),
        title: "an ac".into(),
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    sync(&dir, &url(remote)).unwrap();
}

#[test]
fn run_clones_the_copy_and_says_who_writes() {
    isolate();
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    bare(&remote);
    source(&remote);
    let target = temp.path().join("target");

    let joined = join_run(&target, &url(&remote), Some("myagent")).unwrap();
    assert!(
        joined.lines[0].starts_with("checked: "),
        "{:?}",
        joined.lines
    );
    assert!(joined.lines[1].starts_with("fetched: 1 writes by 1 writers (piko)"));
    assert!(joined.lines[2].contains("myagent"), "{:?}", joined.lines);
    assert_eq!(joined.lines[3], "joined. next: gy handover");
    assert!(target.join("remote").is_file());
    assert!(target.join("events.jsonl").is_file());
}

#[test]
fn run_without_an_actor_tells_how_to_set_it() {
    isolate();
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    bare(&remote);
    source(&remote);
    let target = temp.path().join("target");

    let joined = join_run(&target, &url(&remote), None).unwrap();
    assert!(joined.lines[2].contains("<GY_ACTOR>"), "{:?}", joined.lines);
    assert!(joined.lines[2].contains("set GY_ACTOR before your first write"));
}

#[test]
fn run_is_idempotent() {
    isolate();
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    bare(&remote);
    source(&remote);
    let target = temp.path().join("target");
    join_run(&target, &url(&remote), Some("myagent")).unwrap();

    let again = join_run(&target, &url(&remote), Some("myagent")).unwrap();
    assert_eq!(
        again.lines,
        [format!("already joined {} (seq 1)", url(&remote))]
    );
}

#[test]
fn notice_names_the_writer_count() {
    isolate();
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    bare(&remote);
    source(&remote);
    let target = temp.path().join("target");
    join_run(&target, &url(&remote), Some("myagent")).unwrap();

    // The name git signs with, which here is the author environment's.
    assert_eq!(
        join_notice(&target, &url(&remote)),
        format!("joined {} as piko (1 writes by 1 writers)", url(&remote))
    );
}
