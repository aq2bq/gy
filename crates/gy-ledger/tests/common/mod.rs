//! What every sync test needs from git before it runs (n-8f60).

/// The identity the child git works under, and the only git config it can see.
/// A write to a shared copy reads `git config user.name`, which the author
/// environment variables do not set, so borrowing the machine's global config
/// made the result depend on the machine: a developer has a name and a CI
/// runner has none, and every local run passed while the CI stayed red.
/// `join.rs` isolates the same way, for the opposite value (no name at all).
/// Process-wide, once.
pub fn ident() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        let path = std::env::temp_dir().join(format!("gy-ident-{}", std::process::id()));
        std::fs::write(&path, "[user]\n\tname = piko\n\temail = piko@example.com\n").unwrap();
        for (key, value) in [
            ("GIT_CONFIG_GLOBAL", path.display().to_string()),
            ("GIT_CONFIG_SYSTEM", "/dev/null".to_string()),
            ("GIT_CONFIG_NOSYSTEM", "1".to_string()),
            ("GIT_AUTHOR_NAME", "piko".to_string()),
            ("GIT_AUTHOR_EMAIL", "piko@example.com".to_string()),
            ("GIT_COMMITTER_NAME", "piko".to_string()),
            ("GIT_COMMITTER_EMAIL", "piko@example.com".to_string()),
        ] {
            std::env::set_var(key, value);
        }
    });
}
