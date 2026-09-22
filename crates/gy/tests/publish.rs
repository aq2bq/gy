//! `gy publish` at the command line (n-b6c9, n-a391, ac-e58f): the wiki's
//! entry README, a page per vertex, `loose.md`, and only the target scope
//! rewritten. `--since` is accepted and deprecated (d-3c54).
mod common;

use common::{DATE, criterion, fixture, node_id, stderr, stdout};
use gy_ledger::{Link, Node, NodeKind, Relation};
use std::process::Command;

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
    assert!(text.contains("[← All of it](README.md)"), "{text}");
    assert!(text.contains("Criterion `ac-0001`"), "{text}");
    let loose = out.join("a/loose.md");
    assert!(loose.is_file(), "missing {}", loose.display());
    assert!(
        std::fs::read_to_string(&loose)
            .unwrap()
            .contains("an unbuilt criterion")
    );
    let readme = out.join("a/README.md");
    assert!(readme.is_file(), "missing {}", readme.display());
    let text = std::fs::read_to_string(&readme).unwrap();
    assert!(text.contains("## Needs (1)"), "{text}");
    assert!(text.contains("seq: 1"), "{text}");
    assert!(!out.join("a/criteria").exists(), "no per-kind directory");
}

#[test]
fn publish_since_is_ignored_and_says_so() {
    let fx = fixture();
    let ac = criterion("0001", "a criterion");
    let need = need_targeting("0002", "a need", &ac);
    fx.seed(&[ac, need]);
    let with = fx.root.join("with");
    let without = fx.root.join("without");

    let result = fx.run(&["publish", "--since", "3", "--out", with.to_str().unwrap()]);
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(stdout(&result).is_empty(), "{}", stdout(&result));
    assert_eq!(
        stderr(&result).lines().next(),
        Some("--since has no effect on the wiki and will be removed in a later version")
    );
    let plain = fx.run(&["publish", "--out", without.to_str().unwrap()]);
    assert!(plain.status.success(), "{}", stderr(&plain));
    assert!(stderr(&plain).is_empty(), "{}", stderr(&plain));
    for path in ["a/README.md", "a/n-0002.md"] {
        assert_eq!(
            std::fs::read_to_string(with.join(path)).unwrap(),
            std::fs::read_to_string(without.join(path)).unwrap(),
            "{path} changed with --since"
        );
    }
}

#[test]
fn publish_help_hides_since() {
    let out = Command::new(env!("CARGO_BIN_EXE_gy"))
        .args(["publish", "--help"])
        .output()
        .unwrap();
    let text = stdout(&out);
    assert!(text.contains("--out"), "{text}");
    assert!(!text.contains("--since"), "{text}");
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
