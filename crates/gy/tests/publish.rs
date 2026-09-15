mod common;

use common::{DATE, criterion, fixture, node_id, stderr};
use gy_ledger::{Node, NodeKind};

fn criterion_in(scope: &str, hash: &str, title: &str) -> Node {
    Node::criterion(node_id(NodeKind::Criterion, hash), scope, DATE, title).unwrap()
}

#[test]
fn publish_writes_a_scope_directory_tree() {
    let fx = fixture();
    fx.seed(&[criterion("0001", "a criterion")]);
    let out = fx.root.join("pub");

    let result = fx.run(&["publish", "--out", out.to_str().unwrap()]);
    assert!(result.status.success(), "{}", stderr(&result));
    let file = out.join("a/criteria/ac-0001-a-criterion.md");
    assert!(file.is_file(), "missing {}", file.display());
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("# ac-0001 a criterion"), "{text}");
}

#[test]
fn publish_replaces_only_the_target_scope() {
    let fx = fixture();
    fx.seed(&[
        criterion("0001", "a criterion"),
        criterion_in("b", "0002", "b criterion"),
    ]);
    let out = fx.root.join("pub");
    std::fs::create_dir_all(out.join("b")).unwrap();
    std::fs::write(out.join("keep.md"), "keep").unwrap();
    std::fs::write(out.join("b/keep.md"), "keep b").unwrap();

    let result = fx.run(&["publish", "--scope", "a", "--out", out.to_str().unwrap()]);
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(out.join("keep.md").is_file());
    assert!(out.join("b/keep.md").is_file());
    assert!(out.join("a/criteria/ac-0001-a-criterion.md").is_file());

    std::fs::write(out.join("a/stale.md"), "stale").unwrap();
    let result = fx.run(&["publish", "--scope", "a", "--out", out.to_str().unwrap()]);
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(!out.join("a/stale.md").exists());
}

#[test]
fn publish_needs_an_output_directory() {
    let fx = fixture();
    fx.seed(&[criterion("0001", "a criterion")]);

    let result = fx.run(&["publish"]);
    assert_eq!(result.status.code(), Some(2));
}
