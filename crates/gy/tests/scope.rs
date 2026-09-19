mod common;

use common::{fixture, need, stderr, stdout};

#[test]
fn scope_rename_rewrites_gy_toml_and_moves_nodes() {
    let fx = fixture();
    std::fs::write(
        fx.root.join("gy.toml"),
        "# kept\noutput = \"docs/publication\"\n\n[scopes.a]\n[scopes.keep]\n",
    )
    .unwrap();
    fx.seed(&[need("0001", "a need")]);

    let out = fx.run(&["scope", "rename", "a", "b"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("changed: scope: a → b (1 nodes)"),
        "{}",
        stdout(&out)
    );

    let toml = std::fs::read_to_string(fx.root.join("gy.toml")).unwrap();
    assert_eq!(
        toml,
        "# kept\noutput = \"docs/publication\"\n\n[scopes.b]\n[scopes.keep]\n"
    );

    let text = stdout(&fx.run(&["show", "--full", "n-0001"]));
    assert!(text.contains("scope: b"), "{text}");

    let log = stdout(&fx.run(&["list", "--since", "0"]));
    assert!(log.contains("scope renamed a → b (1 nodes)"), "{log}");
}

#[test]
fn scope_rename_errors_leave_gy_toml_alone() {
    let fx = fixture();
    fx.seed(&[need("0001", "a need")]);

    let before = std::fs::read_to_string(fx.root.join("gy.toml")).unwrap();
    let out = fx.run(&["scope", "rename", "missing", "b"]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        std::fs::read_to_string(fx.root.join("gy.toml")).unwrap(),
        before
    );

    std::fs::write(fx.root.join("gy.toml"), "[scopes.a]\n[scopes.b]\n").unwrap();
    let out = fx.run(&["scope", "rename", "a", "b"]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        std::fs::read_to_string(fx.root.join("gy.toml")).unwrap(),
        "[scopes.a]\n[scopes.b]\n"
    );

    let out = fx.run(&["scope", "rename", "a", "x/y"]);
    assert_eq!(out.status.code(), Some(2));

    let text = stdout(&fx.run(&["show", "--full", "n-0001"]));
    assert!(text.contains("scope: a"), "{text}");
}

#[test]
fn scope_rename_names_the_declared_scopes() {
    let fx = fixture();
    std::fs::write(fx.root.join("gy.toml"), "[scopes.a]\n[scopes.b]\n").unwrap();
    fx.seed(&[need("0001", "a need")]);

    let out = fx.run(&["scope", "rename", "missing", "c"]);
    assert_eq!(out.status.code(), Some(2));
    let message = stderr(&out);
    assert!(message.contains("expected one of a, b"), "{message}");
}

#[test]
fn a_write_names_the_declared_scopes() {
    let fx = fixture();
    let out = fx.run(&["--scope", "nope", "criterion", "add", "an ac"]);
    assert_eq!(out.status.code(), Some(2));
    let message = stderr(&out);
    assert!(message.contains("expected one of a"), "{message}");
}

#[test]
fn scope_rename_rolls_the_log_back_when_gy_toml_cannot_follow() {
    use std::os::unix::fs::PermissionsExt;
    let fx = fixture();
    fx.seed(&[need("0001", "a need")]);

    let path = fx.root.join("gy.toml");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();
    let out = fx.run(&["scope", "rename", "a", "b"]);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    assert_eq!(out.status.code(), Some(2), "{}", stdout(&out));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "[scopes.a]\n");
    let text = stdout(&fx.run(&["show", "--full", "n-0001"]));
    assert!(text.contains("scope: a"), "{text}");
}
