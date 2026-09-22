//! `gy publish` at the command line (n-b6c9, ac-9bc1): the wiki's pages and
//! `loose.md` under `<out>/<scope>/`, and only the target scope rewritten.
mod common;

use common::{DATE, criterion, fixture, node_id, stderr};
use gy_ledger::{Link, Node, NodeKind, Relation};

fn need_targeting(hash: &str, title: &str, ac: &Node) -> Node {
    let mut node = Node::need(node_id(NodeKind::Need, hash), "a", DATE, title).unwrap();
    node.link(Link::new(node.id().clone(), Relation::Targets, ac.id().clone()).unwrap());
    node
}

#[test]
fn publish_writes_a_page_per_vertex_and_a_loose_file() {
    let fx = fixture();
    let ac = criterion("0001", "a criterion");
    let loose = criterion("0002", "an unbuilt criterion");
    let need = need_targeting("0003", "a need", &ac);
    fx.seed(&[ac, loose, need]);
    let out = fx.root.join("pub");

    let result = fx.run(&["publish", "--out", out.to_str().unwrap()]);
    assert!(result.status.success(), "{}", stderr(&result));
    let page = out.join("a/n-0003.md");
    assert!(page.is_file(), "missing {}", page.display());
    let text = std::fs::read_to_string(&page).unwrap();
    assert!(text.contains("# a need"), "{text}");
    assert!(text.contains("Criterion `ac-0001`"), "{text}");
    let loose = out.join("a/loose.md");
    assert!(loose.is_file(), "missing {}", loose.display());
    assert!(
        std::fs::read_to_string(&loose)
            .unwrap()
            .contains("an unbuilt criterion")
    );
    assert!(!out.join("a/README.md").exists(), "no index in this need");
    assert!(!out.join("a/criteria").exists(), "no per-kind directory");
}

#[test]
fn publish_replaces_only_the_target_scope() {
    let fx = fixture();
    let ac = criterion("0001", "a criterion");
    let need = need_targeting("0002", "a need", &ac);
    let mut other = Node::need(node_id(NodeKind::Need, "0003"), "b", DATE, "b need").unwrap();
    other.link(
        Link::new(
            other.id().clone(),
            Relation::Targets,
            criterion("0004", "b criterion").id().clone(),
        )
        .unwrap(),
    );
    let b_ac = criterion("0004", "b criterion");
    fx.seed(&[ac, need, b_ac, other]);
    let out = fx.root.join("pub");
    std::fs::create_dir_all(out.join("b")).unwrap();
    std::fs::write(out.join("keep.md"), "keep").unwrap();
    std::fs::write(out.join("b/keep.md"), "keep b").unwrap();

    let result = fx.run(&["publish", "--scope", "a", "--out", out.to_str().unwrap()]);
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(out.join("keep.md").is_file());
    assert!(out.join("b/keep.md").is_file());
    assert!(out.join("a/n-0002.md").is_file());

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
