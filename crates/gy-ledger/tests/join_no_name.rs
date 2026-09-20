//! The join notice when the machine has no name to write under yet (n-57c5,
//! ac-545c, d-a4f6). Neither the author environment nor the config holds one,
//! and the name is process-wide, so this case has a file of its own.
//!
//! The copy is made by hand rather than cloned: a write lands while the copy
//! is still local, where no name is asked for, and the marker follows. That
//! keeps git out of it entirely, which is the state being described.
use gy_ledger::{
    CriterionAdd, FileStore, FormatVersion, Operation, Repository, format, join_notice,
};

fn nameless() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        std::env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
        std::env::set_var("GIT_CONFIG_SYSTEM", "/dev/null");
        std::env::set_var("GIT_CONFIG_NOSYSTEM", "1");
        for key in ["GIT_AUTHOR_NAME", "GIT_AUTHOR_EMAIL"] {
            std::env::remove_var(key);
        }
    });
}

#[test]
fn the_notice_says_a_name_is_still_missing() {
    nameless();
    let temp = tempfile::tempdir().unwrap();
    let ledger = temp.path().join("copy");
    std::fs::create_dir_all(&ledger).unwrap();
    format::write(&ledger, FormatVersion::CURRENT).unwrap();

    let mut repo = Repository::new(FileStore::open_with(&ledger, |_| Some("piko".into())).unwrap());
    CriterionAdd {
        scope: "a".into(),
        title: "an ac".into(),
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    std::fs::write(ledger.join("remote"), "file:///example/ledger.git").unwrap();

    assert_eq!(
        join_notice(&ledger, "file:///example/ledger.git"),
        "joined file:///example/ledger.git as (no git user.name yet) (1 writes by 1 writers)"
    );
}
