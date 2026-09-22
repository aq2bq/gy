//! `gy remote join`: what a member needs, the copy, and who they write as (n-57c5,
//! ac-545c). The checks all run before anything is fetched, and a second run
//! is harmless.
use super::super::{Error, Result, log};
use super::git;
use super::prepare;
use serde::Serialize;
use std::fmt;
use std::path::Path;

/// What `gy remote join` prints, one line per prefix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Join {
    pub lines: Vec<String>,
}

impl fmt::Display for Join {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.lines {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

/// Check everything a join needs in one pass, before anything is fetched: git,
/// the remote (readable), and `git config user.name` and `user.email`. Every
/// missing item is listed with its fix; an empty list is Ok (ac-545c b).
pub fn check(root: &Path, url: &str) -> Result<()> {
    let mut missing = Vec::new();
    let git_ok = match git::remote_refs(root, url) {
        Ok(_) => true,
        Err(error) if error.message.contains("git is required") => {
            missing.push("git: install git and put it on your PATH".to_string());
            false
        }
        Err(error) => {
            missing.push(format!(
                "read {url}: {}\nget access to the repository (ask the owner) and check the URL",
                error.message
            ));
            true
        }
    };
    if git_ok {
        let (name, mail) = git::user(root)?;
        if name.is_none() {
            missing.push("user.name: git config --global user.name \"<your name>\"".to_string());
        }
        if mail.is_none() {
            missing.push(
                "user.email: git config --global user.email \"<you@example.com>\"".to_string(),
            );
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    Err(Error::invalid(format!(
        "joining needs these first:\n{}",
        missing.join("\n")
    )))
}

/// The copy and the words (ac-545c c, d). An existing copy reports itself and
/// fetches nothing; otherwise the remote's ledger is cloned and counted.
pub fn run(ledger: &Path, url: &str, actor: Option<&str>) -> Result<Join> {
    if ledger.join("remote").is_file() {
        let seq = last_seq(ledger);
        return Ok(Join {
            lines: vec![format!("already joined {url} (seq {seq})")],
        });
    }
    prepare::reconcile(ledger, Some(url))?;
    let (writes, writers) = stats(ledger)?;
    let name = user_name(ledger);
    Ok(Join {
        lines: vec![
            format!("checked: git, {url}, and git config user.name/user.email are ready"),
            format!(
                "fetched: {writes} writes by {} writers ({})",
                writers.len(),
                who(&writers)
            ),
            write_as(&name, actor),
            "joined. next: gy handover".to_string(),
        ],
    })
}

/// The one line the automatic clone prints on stderr (ac-545c, d-b1d4).
pub fn notice(ledger: &Path, url: &str) -> String {
    let (writes, writers) = stats(ledger).unwrap_or((0, Vec::new()));
    format!(
        "joined {url} as {} ({writes} writes by {} writers)",
        user_name(ledger),
        writers.len()
    )
}

/// How the member will be recorded: `user.name / GY_ACTOR`, with the fix when
/// the actor is not set yet.
fn write_as(name: &str, actor: Option<&str>) -> String {
    match actor {
        Some(actor) => format!("you write as \"{name} / {actor}\""),
        None => {
            format!("you write as \"{name} / <GY_ACTOR>\"; set GY_ACTOR before your first write")
        }
    }
}

/// The writes and their distinct writer names, in first-seen order. A writer is
/// the human before the agent, as the views show it (n-d36d).
fn stats(ledger: &Path) -> Result<(u64, Vec<String>)> {
    let events = log::read(ledger)?.0;
    let mut writers: Vec<String> = Vec::new();
    for event in &events {
        let name = match &event.by {
            Some(by) => format!("{by} / {}", event.actor),
            None => event.actor.clone(),
        };
        if !writers.contains(&name) {
            writers.push(name);
        }
    }
    Ok((events.len() as u64, writers))
}

/// The writer list, at most five and then an ellipsis.
fn who(writers: &[String]) -> String {
    let shown: Vec<&str> = writers.iter().take(5).map(String::as_str).collect();
    let mut text = shown.join(", ");
    if writers.len() > 5 {
        text.push_str(", …");
    }
    if text.is_empty() {
        text = "no writes yet".to_string();
    }
    text
}

/// The name the member writes as, from git, or a placeholder when there is
/// none.
fn user_name(ledger: &Path) -> String {
    git::user(ledger)
        .ok()
        .and_then(|(name, _)| name)
        .unwrap_or_else(|| "(no git user.name yet)".to_string())
}

/// The last write sequence in the copy.
fn last_seq(ledger: &Path) -> u64 {
    log::read(ledger)
        .ok()
        .and_then(|(events, _)| events.last().map(|event| event.seq))
        .unwrap_or(0)
}
