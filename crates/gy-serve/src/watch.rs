//! Watching the canonical log so a browser can wait for the next write
//! (n-ff71, d-8a24). One thread looks at the log file; waiting requests park on
//! a condition variable. No extra dependency.
use gy_ledger::log;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant, SystemTime};

/// How often the log file is looked at.
const TICK: Duration = Duration::from_millis(500);

/// The last write the watcher has seen, and the requests waiting for a later
/// one. Cloning shares the same state.
#[derive(Clone)]
pub struct Watch {
    seen: Arc<(Mutex<u64>, Condvar)>,
}

impl Watch {
    /// Watch `ledger`'s log from now on, reading its last write once.
    pub fn start(ledger: PathBuf) -> Self {
        let watch = Watch {
            seen: Arc::new((Mutex::new(0), Condvar::new())),
        };
        let file = ledger.join(log::FILE);
        watch.update(last_seq(&file));
        let poller = watch.clone();
        std::thread::spawn(move || poller.poll(file));
        watch
    }

    /// The sequence of the last write seen.
    pub fn current(&self) -> u64 {
        *self.seen.0.lock().unwrap()
    }

    /// Wait until the last write passes `after`, or `timeout` runs out, and
    /// answer the sequence seen then.
    pub fn wait_past(&self, after: u64, timeout: Duration) -> u64 {
        let (lock, condvar) = &*self.seen;
        let mut seq = lock.lock().unwrap();
        let deadline = Instant::now() + timeout;
        while *seq <= after {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                break;
            }
            seq = condvar.wait_timeout(seq, left).unwrap().0;
        }
        *seq
    }

    /// Look at the file every tick; a changed length or time means a write.
    fn poll(&self, file: PathBuf) {
        let mut stamp = fingerprint(&file);
        loop {
            std::thread::sleep(TICK);
            let now = fingerprint(&file);
            if now == stamp {
                continue;
            }
            stamp = now;
            self.update(last_seq(&file));
        }
    }

    /// Raise the seen sequence and wake every waiter; it never goes down.
    fn update(&self, seq: u64) {
        let (lock, condvar) = &*self.seen;
        let mut seen = lock.lock().unwrap();
        if seq > *seen {
            *seen = seq;
            condvar.notify_all();
        }
    }
}

/// The log's length and time, to notice a write without reading it.
fn fingerprint(file: &Path) -> Option<(u64, SystemTime)> {
    std::fs::metadata(file).ok().map(|meta| {
        (
            meta.len(),
            meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        )
    })
}

/// The last sequence in the log, or 0 when it is empty or not there yet.
fn last_seq(file: &Path) -> u64 {
    let dir = file.parent().unwrap_or(Path::new("."));
    log::read(dir).map_or(0, |(events, _)| events.last().map_or(0, |event| event.seq))
}
