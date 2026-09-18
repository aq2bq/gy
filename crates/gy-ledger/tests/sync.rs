//! gy sync A1 (n-6f47, ac-fd1b): the first sync, clone, and fast-forward on
//! local `file://` remotes. No network.
use gy_ledger::{
    CriterionAdd, FileStore, FormatVersion, NeedAdd, Operation, Repository, format, log, sync,
};
use std::path::Path;
use std::process::Command;

/// The commit identity the child git inherits.
fn ident() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        for (key, value) in [
            ("GIT_AUTHOR_NAME", "piko"),
            ("GIT_AUTHOR_EMAIL", "piko@example.com"),
            ("GIT_COMMITTER_NAME", "piko"),
            ("GIT_COMMITTER_EMAIL", "piko@example.com"),
        ] {
            std::env::set_var(key, value);
        }
    });
}

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

    assert_eq!(sync(&ledger, &url(&remote)).unwrap().pushed, Some((1, seq)));
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
    assert_eq!(sync(&second, &url(&remote)).unwrap().pushed, None);
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
    assert_eq!(report.pulled, Some((seq, next)));
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
