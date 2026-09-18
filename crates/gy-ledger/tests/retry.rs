//! n-fe59: a lost race is retried on the ledger as it then stands. The retry
//! count rides on the log line; an invariant failure returns at once.
use gy_ledger::{
    Actor, Closure, CriterionAdd, FileStore, FormatVersion, MemoryStore, Operation, QuestionAdd,
    QuestionClose, Repository, Result, format, log, retry,
};
use std::path::Path;

const SCOPE: &str = "a";

fn actor(_: &str) -> Option<String> {
    Some("piko".to_string())
}

fn opened(dir: &Path) -> Repository<FileStore> {
    Repository::new(FileStore::open_with(dir, actor).unwrap())
}

fn add(title: &str) -> CriterionAdd {
    CriterionAdd {
        scope: SCOPE.into(),
        title: title.into(),
        body: None,
    }
}

#[test]
fn a_conflicted_write_retries_and_records_the_count() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    format::write(dir, FormatVersion::CURRENT).unwrap();

    // `stale` is opened before the first write, so it still points at seq 0.
    let mut first = opened(dir);
    let stale = opened(dir);
    add("the first").run(&mut first).unwrap();
    let fresh = opened(dir);

    // The first try uses the stale repository, the second the reopened one.
    let mut queue = vec![fresh, stale];
    let open = move || -> Result<Repository<FileStore>> { Ok(queue.pop().expect("an opener")) };
    let (_, outcome) = retry(open, add("the second")).unwrap();
    assert!(outcome.id.is_some());

    let events = log::read(dir).unwrap().0;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].retries, 0);
    assert_eq!(events[1].retries, 1);
}

#[test]
fn a_write_that_an_earlier_write_invalidated_fails_once() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    format::write(dir, FormatVersion::CURRENT).unwrap();

    let mut repository = opened(dir);
    let question = QuestionAdd {
        scope: SCOPE.into(),
        title: "a question".into(),
        decider: "master".into(),
        options: vec!["one".into(), "two".into()],
        body: None,
    }
    .run(&mut repository)
    .unwrap()
    .id
    .unwrap();
    QuestionClose {
        id: question.clone(),
        by: Closure::Fact,
        evidence: "x".into(),
        decision: None,
    }
    .run(&mut repository)
    .unwrap();
    let lines = log::read(dir).unwrap().0.len();

    let error = retry(
        || Ok(opened(dir)),
        QuestionClose {
            id: question,
            by: Closure::Fact,
            evidence: "x".into(),
            decision: None,
        },
    )
    .err()
    .unwrap();
    assert!(!error.is_conflict());
    assert!(
        error.message.contains("already closed"),
        "{}",
        error.message
    );
    assert_eq!(log::read(dir).unwrap().0.len(), lines);
}

#[test]
fn a_writer_that_never_reopens_gives_up_after_five_retries() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    format::write(dir, FormatVersion::CURRENT).unwrap();

    // Six stale repositories, opened before the one write that advances the log:
    // the first try and the five retries each get one.
    let mut first = opened(dir);
    let stale: Vec<Repository<FileStore>> = (0..6).map(|_| opened(dir)).collect();
    add("the first").run(&mut first).unwrap();

    let mut queue = stale;
    let open = move || -> Result<Repository<FileStore>> { Ok(queue.pop().expect("an opener")) };
    let error = retry(open, add("never")).err().unwrap();
    assert!(error.is_conflict());
    assert!(
        error.message.contains("gave up after 5 retries"),
        "{}",
        error.message
    );
}

#[test]
fn a_write_that_is_not_a_conflict_returns_at_once() {
    let calls = std::cell::Cell::new(0);
    let open = || -> Result<Repository<MemoryStore>> {
        calls.set(calls.get() + 1);
        Ok(Repository::new(MemoryStore::with_actor(
            FormatVersion::CURRENT,
            Actor::new("piko").unwrap(),
        )))
    };
    let error = retry(
        open,
        QuestionAdd {
            scope: SCOPE.into(),
            title: "a question".into(),
            decider: "master".into(),
            options: vec!["only".into()],
            body: None,
        },
    )
    .err()
    .unwrap();
    assert!(!error.is_conflict());
    assert_eq!(calls.get(), 1);
}
