mod common;

use common::{criterion, fixture, stderr, stdout};

#[test]
fn publish_prints_the_record() {
    let fx = fixture();
    fx.seed(&[criterion("0001", "a criterion")]);

    let out = fx.run(&["publish"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    for marker in [
        "# gy の公開物",
        "## 読み方",
        "## 記録",
        "## 履歴",
        "## 診断",
        "a criterion",
    ] {
        assert!(text.contains(marker), "missing {marker}:\n{text}");
    }
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

#[test]
fn publish_replaces_seq_in_the_out_path() {
    let fx = fixture();
    fx.seed(&[criterion("0001", "a criterion")]);
    let path = fx.root.join("pub-{seq}.md");

    let out = fx.run(&["publish", "--out", path.to_str().unwrap()]);
    assert!(out.status.success(), "{}", stderr(&out));
    let written = fx.root.join("pub-1.md");
    assert!(written.is_file(), "{}", stdout(&out));
}
