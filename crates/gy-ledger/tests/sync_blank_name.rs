//! `GIT_AUTHOR_NAME` set to nothing (d-a4f6). git does not fall back to the
//! config here — it stops with `fatal: empty ident name` — so gy does not
//! either: falling back would let gy accept a write whose commit cannot be
//! made. The variable is process-wide, which is why this case has a file of
//! its own.
use gy_ledger::{CriterionAdd, FileStore, FormatVersion, Operation, Repository, format, log};
use std::path::Path;
use std::process::Command;

fn blank() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        std::env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
        std::env::set_var("GIT_CONFIG_SYSTEM", "/dev/null");
        std::env::set_var("GIT_CONFIG_NOSYSTEM", "1");
        std::env::set_var("GIT_AUTHOR_NAME", "");
    });
}

fn git(cwd: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn a_blank_author_name_does_not_fall_back_to_the_config() {
    blank();
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("copy");
    std::fs::create_dir_all(&dir).unwrap();
    format::write(&dir, FormatVersion::CURRENT).unwrap();
    git(&dir, &["init", "-q"]);
    std::fs::write(dir.join("remote"), "file:///example/ledger.git").unwrap();
    git(&dir, &["config", "user.name", "alice"]);
    git(&dir, &["config", "user.email", "alice@example.com"]);

    let mut repo = Repository::new(FileStore::open_with(&dir, |_| Some("piko".into())).unwrap());
    let error = CriterionAdd {
        scope: "a".into(),
        title: "an ac".into(),
        body: None,
    }
    .run(&mut repo)
    .unwrap_err();

    assert!(
        error.message.contains("no name to write under"),
        "{}",
        error.message
    );
    assert!(log::read(&dir).unwrap().0.is_empty());
}
