use gy_ledger::{CriterionAdd, FileStore, FormatVersion, Operation, Repository, Store, format};
use gy_serve::http::Request;
use gy_serve::watch::Watch;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// A throwaway directory under the system's temp root (no extra dependency).
fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gy-serve-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn ledger(dir: &Path) -> Repository<FileStore> {
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let store = FileStore::open_with(dir, |_| Some("piko".to_string())).unwrap();
    Repository::new(store).with_scopes(vec!["a".to_string()])
}

/// One write, answering the sequence it landed at.
fn add(repo: &mut Repository<FileStore>, title: &str) -> u64 {
    CriterionAdd {
        scope: "a".to_string(),
        title: title.to_string(),
        body: None,
    }
    .run(repo)
    .unwrap();
    repo.store().history().last().unwrap().seq
}

#[test]
fn the_watch_follows_the_log() {
    let dir = tempdir("watch");
    let mut repo = ledger(&dir);
    let first = add(&mut repo, "one");
    let watch = Watch::start(dir.clone());
    assert_eq!(watch.current(), first);

    // A waiter wakes when a write lands.
    let waiter = watch.clone();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(waiter.wait_past(first, Duration::from_secs(3)));
    });
    std::thread::sleep(Duration::from_millis(700));
    let second = add(&mut repo, "two");
    let seen = receiver
        .recv_timeout(Duration::from_millis(1500))
        .expect("the waiter did not wake");
    assert_eq!(seen, second);

    // Nothing written: the wait ends at its timeout with the value it has.
    let started = Instant::now();
    assert_eq!(watch.wait_past(second, Duration::from_millis(300)), second);
    assert!(started.elapsed() >= Duration::from_millis(250));

    // Already past: answered without waiting.
    let started = Instant::now();
    assert_eq!(watch.wait_past(0, Duration::from_secs(3)), second);
    assert!(
        started.elapsed() < Duration::from_millis(50),
        "the wait was not immediate"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_wait_handler_answers_the_sequence() {
    let dir = tempdir("wait");
    let mut repo = ledger(&dir);
    let seq = add(&mut repo, "one");
    let watch = Watch::start(dir.clone());

    let answered = |query: Option<&str>| {
        let res = gy_serve::api::wait::wait(&watch, &Request::new("GET", "/api/wait", query, &[]));
        (
            res.status,
            serde_json::from_slice::<serde_json::Value>(&res.body).ok(),
        )
    };

    let (status, body) = answered(Some("after=0"));
    assert_eq!(status, 200);
    assert_eq!(body.unwrap()["seq"], seq);
    assert_eq!(answered(None).0, 200);
    assert_eq!(answered(Some("after=abc")).0, 400);

    let _ = std::fs::remove_dir_all(&dir);
}
