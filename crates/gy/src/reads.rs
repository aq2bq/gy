//! The reading commands that need more than a single view call: list and the
//! publish reading.
use crate::output::emit;
use crate::repo;
use crate::{Cli, Command};
use gy_ledger::{
    Actor, Error, Filter, FormatVersion, Listing, MemoryStore, NodeKind, Publication, Repository,
    Result, Store, config, file, format, list, local_day_start, publish,
};
use gy_serve::server::Opened;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn read_list(cli: &Cli, ledger: &Path) -> Result<()> {
    let Command::List {
        kind,
        status,
        targets,
        grep,
        actor,
        since,
    } = &cli.command
    else {
        return Err(Error::invalid("not a list"));
    };
    let repository = repo::open(ledger)?;
    let filter = Filter {
        kind: kind.as_deref().map(parse_kind).transpose()?,
        status: status.clone(),
        targets: targets
            .as_deref()
            .map(|text| repository.resolve(text))
            .transpose()?,
        grep: grep.clone(),
        actor: actor.clone(),
        since: since
            .as_deref()
            .map(|text| parse_since(&repository, text))
            .transpose()?,
    };
    emit(cli.json, &list(&repository, &filter)?)
}

/// `gy remote sync`: the explicit step that prepares the copy and pulls (n-6f47,
/// d-39f6). It needs `remote` in gy.toml; a remote-less ledger is never
/// touched, and reading it back is an error instead of a change.
pub fn sync_command(cli: &Cli, root: &Path, ledger: &Path) -> Result<()> {
    watchdog(ledger);
    let Some(remote) = config::read(root)?.remote else {
        let error = Error::invalid(
            "gy.toml has no remote; gy remote sync needs one (add `remote = \"…\"`)",
        );
        // The sync body never ran, so its state write never happens: leave
        // the reason where serve's next tick reads it (n-9f9d).
        gy_ledger::record_error(ledger, &error.message);
        return Err(error);
    };
    emit(cli.json, &gy_ledger::sync(ledger, &remote)?)
}

/// A background sync gives up after 30 seconds and records why (n-ecbf). The
/// parent has already detached, so the child limits itself.
fn watchdog(ledger: &Path) {
    if std::env::var("GY_SYNC_BACKGROUND").ok().as_deref() != Some("1") {
        return;
    }
    let ledger = ledger.to_path_buf();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(30));
        gy_ledger::record_timeout(&ledger);
        std::process::exit(1);
    });
}

/// `handover`, after reaching the remote when the copy is shared: the fetch
/// gets five seconds, then the view is built from the copy as it stands
/// (n-ecbf 2B2).
pub fn handover(cli: &Cli, root: &Path, ledger: &Path) -> Result<()> {
    crate::release_check::poll();
    repo::refresh(root, ledger)?;
    let repository = repo::open(ledger)?;
    emit(
        cli.json,
        &gy_ledger::handover(&repository, cli.scope.as_deref())?,
    )
}

/// Serve the ledger over HTTP on localhost until stopped (n-a493). A shared
/// copy also syncs every ten seconds while it runs (n-94bb). A read may name a
/// sequence, which opens the ledger as it stood then (n-10e1).
pub fn serve(root: &Path, ledger: &Path, name: String) -> Result<()> {
    let remote = config::read(root)?.remote;
    // The marker may be gone while gy.toml's remote is back (n-9f9d): the
    // thread rebinds the copy on its first tick.
    if remote.is_some() {
        let (root, ledger) = (root.to_path_buf(), ledger.to_path_buf());
        std::thread::spawn(move || sync_log(&root, &ledger));
    }
    let watched = ledger.to_path_buf();
    let ledger = ledger.to_path_buf();
    gy_serve::server::serve(
        Box::new(move |at| {
            /* Before the first write there is no ledger: show a 0-write one
            instead of an error (n-3e6b). A read never creates it (N-63). */
            if !ledger.join(format::FILE).is_file() {
                return Ok(empty());
            }
            match at {
                None => repo::open(&ledger).map(Opened::Now),
                Some(seq) => {
                    // The same reader actor repo::open uses; open_at never writes.
                    let reader = |_: &str| Some("gy-read".to_string());
                    file::open_at(&ledger, seq, reader)
                        .map(|store| Opened::At(Repository::new(store)))
                }
            }
        }),
        watched,
        name,
        remote,
    )
}

/// The serve sync thread: one round every ten seconds, with the failure
/// throttle (n-08ae). A failure shows its first line and the way out on the
/// next; a recovery shows once.
fn sync_log(root: &Path, ledger: &Path) {
    let mut rounds = gy_ledger::Rounds::new();
    loop {
        let at = clock();
        match repo::sync_tick(root, ledger) {
            repo::Tick::Out(line) => {
                for note in rounds.recovered(&at) {
                    eprintln!("{note}");
                }
                eprintln!("{at} sync: {line}");
            }
            repo::Tick::Failed(fallback) => {
                let (first, advice) = gy_ledger::sync_error(ledger).unwrap_or((fallback, None));
                for note in rounds.failed(&first, advice.as_deref(), &at) {
                    eprintln!("{note}");
                }
            }
            repo::Tick::Silent => {
                for note in rounds.recovered(&at) {
                    eprintln!("{note}");
                }
            }
            repo::Tick::Skipped => {}
        }
        std::thread::sleep(Duration::from_secs(10));
    }
}

/// The wall clock for serve's log lines, `HH:MM:SS` in the reader's own place
/// (d-b1f8, n-94bb).
fn clock() -> String {
    gy_ledger::local_clock()
}

/// A ledger with nothing in it, for a directory whose first write has not
/// happened yet. Reads only; it is never saved.
fn empty() -> Opened {
    let actor = Actor::new("gy-read").expect("gy-read is a valid actor");
    Opened::At(Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        actor,
    )))
}

/// The publication goes under `--out`, else gy.toml's `output` (an error when
/// neither is given). Each target scope's directory is removed and rewritten.
/// The wiki always shows the record as it is now, so `--since` is accepted for
/// compatibility and named once on stderr (n-a391, d-3c54).
pub fn write_publish(
    cli: &Cli,
    root: &Path,
    ledger: &Path,
    since: Option<u64>,
    out: Option<&Path>,
) -> Result<()> {
    if since.is_some() {
        eprintln!("--since has no effect on the wiki and will be removed in a later version");
    }
    let repository = repo::open(ledger)?;
    let writer = Actor::from_env()
        .map(|actor| actor.name().to_string())
        .unwrap_or_default();
    let publication = publish(
        &repository,
        cli.scope.as_deref(),
        since,
        &writer,
        &ledger.display().to_string(),
    )?;
    let out = match out {
        Some(path) => path.to_path_buf(),
        None => output_path(root)?
            .ok_or_else(|| Error::invalid("publish needs --out or gy.toml output"))?,
    };
    write_publication(&out, &publication)
}

fn write_publication(out: &Path, publication: &Publication) -> Result<()> {
    for scope in &publication.scopes {
        let dir = out.join(&scope.name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        std::fs::create_dir_all(&dir)?;
        for file in &scope.files {
            let path = dir.join(&file.path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, &file.text)?;
        }
    }
    Ok(())
}

fn output_path(root: &Path) -> Result<Option<PathBuf>> {
    Ok(config::read(root)?.output.map(|out| root.join(out)))
}

fn parse_kind(text: &str) -> Result<NodeKind> {
    let wanted = text.to_lowercase();
    NodeKind::ALL
        .into_iter()
        .find(|kind| kind.name() == wanted || kind.prefix() == wanted)
        .ok_or_else(|| {
            Error::invalid(format!(
                "unknown type {text}; expected one of Need, Question, Decision, Requirement, Criterion (or n, q, d, r, ac)"
            ))
        })
}

/// The `--since` value: a write sequence, or a date (`YYYY-MM-DD`) whose local
/// day onwards is meant. A date becomes the sequence just before that day, so
/// every write from the day's start is kept (n-a56f, n-b6b6).
fn parse_since<S: Store>(repository: &Repository<S>, text: &str) -> Result<u64> {
    if let Ok(seq) = text.parse::<u64>() {
        return Ok(seq);
    }
    let start = local_day_start(text)?;
    let rows = match list(
        repository,
        &Filter {
            since: Some(0),
            ..Default::default()
        },
    )? {
        Listing::History(rows) => rows,
        Listing::Nodes(_) => Vec::new(),
    };
    Ok(rows
        .iter()
        .filter(|row| row.at < start)
        .map(|row| row.seq)
        .max()
        .unwrap_or(0))
}
