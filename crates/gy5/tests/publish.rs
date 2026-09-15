mod common;

use common::{criterion, fixture, stderr, stdout};

#[test]
fn publish_prints_the_reading() {
    let fx = fixture();
    fx.seed(&[criterion("0001", "a criterion")]);

    let out = fx.run(&["publish"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("# gy の公開物"), "{text}");
    assert!(text.contains("## ノード"), "{text}");
    assert!(text.contains("a criterion"), "{text}");
}

#[test]
fn publish_writes_to_the_out_path() {
    let fx = fixture();
    fx.seed(&[criterion("0001", "a criterion")]);
    let path = fx.root.join("out.md");

    let out = fx.run(&["publish", "--out", path.to_str().unwrap()]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(out.stdout.is_empty());
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("# gy の公開物"), "{text}");
}
