//! One computation per question (d-6d90, ac-0dbb). A reading answer can take a
//! while on a large ledger, so the same question arriving again while one is
//! running waits for that one instead of starting another — a reloading screen
//! can no longer stack a computation per request. A computation that runs past
//! its limit is answered with 503 and the reason, and a failure says one line on
//! the server's log.
use crate::http::Response;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// The most a reading answer may take before it is refused.
pub const LIMIT: Duration = Duration::from_secs(5);

/// One question and the flight answering it.
type Entry = (String, Arc<Flight>);

/// The flights in hand; cloning shares them, so every connection sees one map.
#[derive(Clone, Default)]
pub struct Flights {
    entries: Arc<Mutex<Vec<Entry>>>,
}

/// One computation, and the answers of the requests waiting on it.
#[derive(Default)]
struct Flight {
    outcome: Mutex<Option<Outcome>>,
    ready: Condvar,
    /// Whether the failure of this computation has been written out once.
    logged: AtomicBool,
}

enum Outcome {
    Answered(Response),
    /// The computation panicked; the log carries the one line about it.
    Panicked,
}

impl Flights {
    /// Answer `key` once: the first caller computes it, the rest wait for its
    /// result, and a computation past `limit` is 503.
    pub fn answer(
        &self,
        key: &str,
        limit: Duration,
        compute: impl FnOnce() -> Response + Send + 'static,
    ) -> Response {
        let (leader, flight) = self.entry(key);
        if leader {
            self.spawn(key.to_string(), Arc::clone(&flight), compute);
        }
        flight.wait(key, limit)
    }

    /// The flight for a key, and whether this caller has to compute it.
    fn entry(&self, key: &str) -> (bool, Arc<Flight>) {
        let mut entries = self.entries.lock().unwrap();
        if let Some((_, flight)) = entries.iter().find(|(held, _)| held == key) {
            return (false, Arc::clone(flight));
        }
        let flight = Arc::new(Flight::default());
        entries.push((key.to_string(), Arc::clone(&flight)));
        (true, flight)
    }

    /// Compute on a thread of its own, so the waiting side can give up after
    /// its limit while the computation finishes and answers the next ask.
    fn spawn(
        &self,
        key: String,
        flight: Arc<Flight>,
        compute: impl FnOnce() -> Response + Send + 'static,
    ) {
        let flights = self.clone();
        std::thread::spawn(move || {
            let outcome = match catch_unwind(AssertUnwindSafe(compute)) {
                Ok(answer) => {
                    if answer.status != 200 {
                        line(&key, &format!("{} {}", answer.status, preview(&answer)));
                    }
                    Outcome::Answered(answer)
                }
                Err(_) => {
                    line(&key, "the answer panicked");
                    Outcome::Panicked
                }
            };
            *flight.outcome.lock().unwrap() = Some(outcome);
            flight.ready.notify_all();
            flights.forget(&key);
        });
    }

    /// Drop a finished flight, so the next ask computes afresh.
    fn forget(&self, key: &str) {
        let mut entries = self.entries.lock().unwrap();
        entries.retain(|(held, _)| held != key);
    }
}

impl Flight {
    /// The answer, or the reason it did not come in time.
    fn wait(&self, key: &str, limit: Duration) -> Response {
        let held = self.outcome.lock().unwrap();
        let (held, _) = self
            .ready
            .wait_timeout_while(held, limit, |outcome| outcome.is_none())
            .unwrap();
        match &*held {
            Some(Outcome::Answered(answer)) => answer.clone(),
            Some(Outcome::Panicked) => {
                Response::text(500, "the answer failed; the log has the line")
            }
            None => {
                drop(held);
                if !self.logged.swap(true, Ordering::Relaxed) {
                    line(key, &format!("no answer within {}s", limit.as_secs()));
                }
                Response::text(
                    503,
                    &format!(
                        "no answer within {}s; the same question is still being computed",
                        limit.as_secs()
                    ),
                )
            }
        }
    }
}

/// One line for the server's log: the question and why it failed.
fn line(key: &str, message: &str) {
    eprintln!("{key}: {message}");
}

/// The first line of an answer's body, short enough for one log line.
fn preview(answer: &Response) -> String {
    let body = String::from_utf8_lossy(&answer.body);
    let first = body.lines().next().unwrap_or_default();
    first.chars().take(160).collect()
}

#[cfg(test)]
mod tests {
    use super::Flights;
    use crate::http::Response;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    /// Two asks for one question compute it once, and both get the answer.
    #[test]
    fn one_computation_answers_every_waiter() {
        let flights = Flights::default();
        let computed = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&computed);
        let slow = move || {
            count.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(120));
            Response::json(&serde_json::json!({"ok": true}))
        };
        let (first, second) = (flights.clone(), flights.clone());
        let one = std::thread::spawn(move || first.answer("/api/x", Duration::from_secs(2), slow));
        std::thread::sleep(Duration::from_millis(20));
        let count = Arc::clone(&computed);
        let two = std::thread::spawn(move || {
            second.answer("/api/x", Duration::from_secs(2), move || {
                count.fetch_add(1, Ordering::SeqCst);
                Response::text(500, "a second computation")
            })
        });
        let (one, two) = (one.join().unwrap(), two.join().unwrap());
        assert_eq!((one.status, two.status), (200, 200));
        assert_eq!(computed.load(Ordering::SeqCst), 1);
    }

    /// A computation past the limit is 503 with the reason, and the next ask
    /// after it finished computes again.
    #[test]
    fn a_slow_answer_is_refused_with_the_reason() {
        let flights = Flights::default();
        let refused = flights.answer("/api/slow", Duration::from_millis(30), || {
            std::thread::sleep(Duration::from_millis(200));
            Response::text(200, "late")
        });
        assert_eq!(refused.status, 503);
        assert!(
            String::from_utf8_lossy(&refused.body).contains("no answer within"),
            "{}",
            String::from_utf8_lossy(&refused.body)
        );
        std::thread::sleep(Duration::from_millis(250));
        let again = flights.answer("/api/slow", Duration::from_secs(2), || {
            Response::text(200, "now")
        });
        assert_eq!(again.status, 200);
    }

    /// A failed answer keeps its own status; only the limit makes a 503.
    #[test]
    fn a_failure_keeps_its_status() {
        let flights = Flights::default();
        let answer = flights.answer("/api/bad", Duration::from_secs(2), || {
            Response::text(500, "no ledger")
        });
        assert_eq!(answer.status, 500);
    }
}
