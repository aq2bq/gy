//! gy remote sync A1 (n-6f47, ac-fd1b): the first sync, clone, and fast-forward on
//! local `file://` remotes. No network.
use gy_ledger::{
    CriterionAdd, FileStore, FormatVersion, NeedAdd, Operation, Pulled, Range, Repository, format,
    log, sync,
};
use std::path::Path;
use std::process::Command;

mod common;
use common::ident;

fn git(cwd: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn url(dir: &Path) -> String {
    format!("file://{}", dir.display())
}

/// A bare remote with the default HEAD (`master`), which stays unborn.
fn bare(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "--bare"]);
}

/// A local ledger with two writes; returns its last sequence.
fn seed(dir: &Path) -> u64 {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let mut repo = Repository::new(FileStore::open_with(dir, |_| Some("piko".into())).unwrap());
    let ac = CriterionAdd {
        scope: "a".into(),
        title: "an ac".into(),
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    NeedAdd {
        scope: "a".into(),
        title: "a need".into(),
        targets: vec![ac.id.unwrap()],
        spawned_by: None,
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    last_seq(dir)
}

fn last_seq(dir: &Path) -> u64 {
    log::read(dir).unwrap().0.last().unwrap().seq
}

#[test]
fn first_sync_moves_the_ledger_to_an_empty_remote() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, remote) = (temp.path().join("ledger"), temp.path().join("remote.git"));
    let seq = seed(&ledger);
    bare(&remote);

    assert_eq!(
        sync(&ledger, &url(&remote)).unwrap().pushed,
        Some(Range { from: 1, to: seq })
    );
    let files = git(&remote, &["ls-tree", "-r", "--name-only", "main"]);
    let mut names: Vec<&str> = files.split_whitespace().collect();
    names.sort();
    assert_eq!(names, [".gitignore", "README.md", "events.jsonl", "format"]);
    assert!(git(&remote, &["log", "-1", "--format=%B", "main"]).contains("Gy-Seq: 1-"));
    assert!(git(&remote, &["show", "main:README.md"]).contains("written by gy"));
    assert_eq!(
        std::fs::read_to_string(ledger.join("remote")).unwrap(),
        url(&remote)
    );

    // Nothing left to move; a second copy clones the same sequence.
    let again = sync(&ledger, &url(&remote)).unwrap();
    assert_eq!((again.pushed, again.pulled), (None, None));
    let second = temp.path().join("second");
    let clone = sync(&second, &url(&remote)).unwrap();
    assert_eq!(clone.pushed, None);
    assert_eq!(
        clone.pulled,
        Some(Pulled {
            from: 0,
            to: seq,
            writers: vec!["piko".into()]
        })
    );
    assert_eq!(last_seq(&second), seq);
}

#[test]
fn sync_fast_forwards_a_copy_with_no_write_of_its_own() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, remote) = (temp.path().join("first"), temp.path().join("remote.git"));
    let seq = seed(&first);
    bare(&remote);
    sync(&first, &url(&remote)).unwrap();
    let second = temp.path().join("second");
    sync(&second, &url(&remote)).unwrap();

    // One more write reaches the remote through git (gy pushes it in n-8a52).
    let mut repo = Repository::new(FileStore::open_with(&second, |_| Some("piko".into())).unwrap());
    CriterionAdd {
        scope: "a".into(),
        title: "another".into(),
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    let next = last_seq(&second);
    git(&second, &["add", "-A"]);
    git(
        &second,
        &["commit", "-q", "-m", &format!("a write\n\nGy-Seq: {next}")],
    );
    git(&second, &["push", "-q", "origin", "HEAD:main"]);

    let report = sync(&first, &url(&remote)).unwrap();
    assert_eq!(
        report.pulled,
        Some(Pulled {
            from: seq,
            to: next,
            writers: vec!["piko".into()]
        })
    );
    assert_eq!(last_seq(&first), next);
    let snapshot = std::fs::read_to_string(first.join("snapshot.json")).unwrap();
    assert!(snapshot.contains(&format!("\"seq\":{next}")), "{snapshot}");
}

#[test]
fn sync_refuses_a_remote_that_is_not_a_ledger() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    bare(&remote);
    let work = temp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();
    git(&work, &["init", "-q"]);
    std::fs::write(work.join("notes.txt"), "hello").unwrap();
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "notes"]);
    git(&work, &["push", "-q", &url(&remote), "HEAD:master"]);

    // No local ledger: the clone path rejects it and leaves no half-clone.
    let copy = temp.path().join("copy");
    let error = sync(&copy, &url(&remote)).unwrap_err();
    assert!(
        error.message.contains("not a gy ledger"),
        "{}",
        error.message
    );
    assert!(std::fs::read_dir(&copy).unwrap().next().is_none());

    // A local ledger: the dedicated check still comes before any divergence.
    let own = temp.path().join("own");
    seed(&own);
    let error = sync(&own, &url(&remote)).unwrap_err();
    assert!(
        error.message.contains("not a gy ledger"),
        "{}",
        error.message
    );
}

#[test]
fn sync_pushes_each_write_as_its_own_commit() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, remote) = (temp.path().join("ledger"), temp.path().join("remote.git"));
    seed(&ledger);
    bare(&remote);
    sync(&ledger, &url(&remote)).unwrap();
    let count = |remote: &Path| {
        git(remote, &["rev-list", "--count", "main"])
            .trim()
            .to_string()
    };

    // A write stays local until gy remote sync.
    let mut repo = Repository::new(FileStore::open_with(&ledger, |_| Some("piko".into())).unwrap());
    CriterionAdd {
        scope: "a".into(),
        title: "another".into(),
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    let next = last_seq(&ledger);
    assert_eq!(count(&remote), "1");

    let report = sync(&ledger, &url(&remote)).unwrap();
    assert_eq!(
        report.pushed,
        Some(Range {
            from: next,
            to: next
        })
    );
    assert_eq!(count(&remote), "2");
    let message = git(&remote, &["log", "-1", "--format=%B", "main"]);
    assert!(message.contains("criterion add"), "{message}");
    assert!(message.contains("actor: piko"), "{message}");
    assert!(message.contains(&format!("Gy-Seq: {next}")), "{message}");
}

#[test]
fn sync_reports_pulled_pushed_and_up_to_date() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, remote) = (temp.path().join("ledger"), temp.path().join("remote.git"));
    let seq = seed(&ledger);
    bare(&remote);

    let first = sync(&ledger, &url(&remote)).unwrap();
    let first_text = format!("{first}");
    assert!(first_text.starts_with(&format!("pushed: {seq} writes (seq 1 → {seq})\n")));
    assert!(first_text.contains("block force pushes"), "{first_text}");
    assert!(first.guard.is_some());
    // The guidance is only on the first push.
    let up = sync(&ledger, &url(&remote)).unwrap();
    assert_eq!(format!("{up}"), format!("up to date: seq {seq}\n"));

    let second = temp.path().join("second");
    let clone = sync(&second, &url(&remote)).unwrap();
    assert_eq!(
        format!("{clone}"),
        format!("pulled: seq 0 → {seq} ({seq} writes by piko)\n")
    );
    let json = serde_json::to_value(&clone).unwrap();
    assert_eq!(json["pulled"]["from"], 0);
    assert_eq!(json["pulled"]["to"], seq);
    assert_eq!(json["pushed"], serde_json::Value::Null);
}

#[test]
fn sync_says_a_refused_push_is_a_permission_problem() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, remote) = (temp.path().join("ledger"), temp.path().join("remote.git"));
    seed(&ledger);
    bare(&remote);
    sync(&ledger, &url(&remote)).unwrap();

    let objects = remote.join("objects");
    let original = std::fs::metadata(&objects).unwrap().permissions();
    let mut readonly = original.clone();
    readonly.set_readonly(true);
    std::fs::set_permissions(&objects, readonly).unwrap();

    let mut repo = Repository::new(FileStore::open_with(&ledger, |_| Some("piko".into())).unwrap());
    CriterionAdd {
        scope: "a".into(),
        title: "another".into(),
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    let error = sync(&ledger, &url(&remote)).unwrap_err();

    std::fs::set_permissions(&objects, original).unwrap();
    assert!(
        error.message.contains("refused the push"),
        "{}",
        error.message
    );
}
