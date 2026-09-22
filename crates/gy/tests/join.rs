//! `gy remote join` at the command line (n-57c5, ac-545c): the checks, the prefixes,
//! the exit codes, and the automatic clone's one line. Local `file://`.
mod common;
use common::{fixture, stderr, stdout};
use std::path::Path;
use std::process::Command;

/// A bare remote holding a gy ledger, made by one source's sync.
fn populated_remote(temp: &Path) -> String {
    let remote = temp.join("remote.git");
    std::fs::create_dir_all(&remote).unwrap();
    let bare = Command::new("git")
        .args(["init", "-q", "--bare"])
        .arg(&remote)
        .output()
        .unwrap();
    assert!(bare.status.success());
    let source = fixture();
    std::fs::write(
        source.root.join("gy.toml"),
        format!("remote = \"file://{}\"\n\n[scopes.a]\n", remote.display()),
    )
    .unwrap();
    source.seed(&[common::criterion("0001", "an ac")]);
    let out = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(&source.root)
        .env("XDG_DATA_HOME", &source.data)
        .env("GY_ACTOR", "piko")
        .env("GIT_AUTHOR_NAME", "piko")
        .env("GIT_AUTHOR_EMAIL", "piko@example.com")
        .env("GIT_COMMITTER_NAME", "piko")
        .env("GIT_COMMITTER_EMAIL", "piko@example.com")
        .arg("remote")
        .arg("sync")
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    format!("file://{}", remote.display())
}

/// An empty git config (no user.name) unless `user` says otherwise. HOME is
/// the empty config's directory, so no per-user file is read.
fn config_file(dir: &Path, user: bool) -> std::path::PathBuf {
    let path = dir.join("gitconfig");
    let text = if user {
        "[user]\n\tname = gitpiko\n\temail = gitpiko@example.com\n"
    } else {
        ""
    };
    std::fs::write(&path, text).unwrap();
    path
}

fn join(
    fx: &common::Fixture,
    args: &[&str],
    config: &Path,
    actor: Option<&str>,
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_gy"));
    command
        .current_dir(&fx.root)
        .env("XDG_DATA_HOME", &fx.data)
        .env("HOME", config.parent().unwrap())
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        // The author environment names the writer too (d-a4f6), so the config
        // decides only when nothing here does.
        .env_remove("GIT_AUTHOR_NAME")
        .env_remove("GIT_AUTHOR_EMAIL")
        .args(args);
    match actor {
        Some(actor) => command.env("GY_ACTOR", actor),
        None => command.env_remove("GY_ACTOR"),
    };
    command.output().unwrap()
}

#[test]
fn join_without_a_remote_says_not_shared() {
    let fx = fixture();
    let config = config_file(&fx.root, true);
    let out = join(&fx, &["remote", "join"], &config, Some("piko"));
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("not shared"), "{}", stderr(&out));
}

#[test]
fn join_lists_every_missing_item_and_fetches_nothing() {
    let temp = tempfile::tempdir().unwrap();
    let url = populated_remote(temp.path());
    let fx = fixture();
    std::fs::write(
        fx.root.join("gy.toml"),
        format!("remote = \"{url}\"\n\n[scopes.a]\n"),
    )
    .unwrap();
    let config = config_file(temp.path(), false);

    let out = join(&fx, &["remote", "join"], &config, None);
    assert_eq!(out.status.code(), Some(2));
    let err = stderr(&out);
    assert!(
        err.contains("user.name: git config --global user.name"),
        "{err}"
    );
    assert!(
        err.contains("user.email: git config --global user.email"),
        "{err}"
    );
    assert!(!fx.ledger().join("remote").exists(), "nothing is fetched");
}

#[test]
fn join_fetches_the_copy_and_says_who_writes() {
    let temp = tempfile::tempdir().unwrap();
    let url = populated_remote(temp.path());
    let fx = fixture();
    std::fs::write(
        fx.root.join("gy.toml"),
        format!("remote = \"{url}\"\n\n[scopes.a]\n"),
    )
    .unwrap();
    let config = config_file(temp.path(), true);

    let out = join(&fx, &["remote", "join"], &config, Some("myagent"));
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.lines().next().unwrap().starts_with("checked: "),
        "{text}"
    );
    assert!(
        text.contains("fetched: 1 writes by 1 writers (piko)"),
        "{text}"
    );
    assert!(
        text.contains("you write as \"gitpiko / myagent\""),
        "{text}"
    );
    assert!(text.contains("joined. next: gy handover"), "{text}");
    assert!(fx.ledger().join("remote").is_file());

    let again = join(&fx, &["remote", "join"], &config, Some("myagent"));
    assert!(again.status.success(), "{}", stderr(&again));
    assert!(
        stdout(&again).starts_with(&format!("already joined {url} (seq 1)")),
        "{}",
        stdout(&again)
    );
}

#[test]
fn the_first_other_command_prints_the_joined_line() {
    let temp = tempfile::tempdir().unwrap();
    let url = populated_remote(temp.path());
    let fx = fixture();
    std::fs::write(
        fx.root.join("gy.toml"),
        format!("remote = \"{url}\"\n\n[scopes.a]\n"),
    )
    .unwrap();
    let config = config_file(temp.path(), true);

    let out = join(
        &fx,
        &["list", "--type", "criterion"],
        &config,
        Some("myagent"),
    );
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stderr(&out).contains(&format!("joined {url} as gitpiko (1 writes by 1 writers)")),
        "{}",
        stderr(&out)
    );
    assert!(stdout(&out).contains("an ac"), "{}", stdout(&out));
}
