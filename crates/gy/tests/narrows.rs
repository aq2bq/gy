mod common;
mod narrows_support;
use common::{stderr, stdout};
use narrows_support::*;

#[test]
fn three_passages_survive_every_new_reader_and_both_creation_routes() {
    for related in [false, true] {
        let (fx, x, y) = setup(related);
        three(&fx, &x, &y, usize::from(related));
        for args in [vec!["show", &y], vec!["show", &y, "--full"]] {
            text_marks(&stdout(&ok(fx.run(&args))), &x, false);
        }
        all_marks(&shown(&fx, &y), &x);
        let source = stdout(&ok(fx.run(&["show", &x, "--full"])));
        for mark in MARKS {
            assert!(
                source.contains(&format!("narrows {y} ({mark})")),
                "{source}"
            );
        }
        ok(fx.run(&["publish"]));
        published(&fx, &x, &y);
        let server = Server::start(&fx, false);
        all_marks(&server.get(&format!("/api/node/{y}")), &x);
        for (center, side, peer) in [(&x, "up", &y), (&y, "down", &x)] {
            let path = server.get(&format!("/api/node/{center}/path"));
            let matching: Vec<_> = path[side][0]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| n["id"] == *peer)
                .collect();
            assert_eq!(matching.len(), 1);
            assert_eq!(matching[0]["mark"], MARKS.join("\n"));
        }
    }
}

#[test]
fn refused_marks_do_not_append_and_supersedes_stays_pair_unique() {
    let (fx, x, y) = setup(false);
    three(&fx, &x, &y, 0);
    for mark in [MARKS[0], "absent passage"] {
        let before = std::fs::read(fx.ledger().join("events.jsonl")).unwrap();
        let out = fx.run(&["link", &x, "narrows", &y, "--mark", mark]);
        assert!(!out.status.success());
        assert_eq!(
            std::fs::read(fx.ledger().join("events.jsonl")).unwrap(),
            before
        );
    }
    ok(fx.run(&["link", &x, "supersedes", &y, "--mark", MARKS[0]]));
    let before = std::fs::read(fx.ledger().join("events.jsonl")).unwrap();
    let refused = fx.run(&["link", &x, "supersedes", &y, "--mark", MARKS[1]]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("already registered"));
    assert_eq!(
        std::fs::read(fx.ledger().join("events.jsonl")).unwrap(),
        before
    );
}

#[test]
fn selected_removal_undo_ambiguity_and_two_unresolved_passages() {
    let (fx, x, y) = setup(false);
    three(&fx, &x, &y, 0);
    for args in [
        vec!["link", &x, "narrows", &y, "--remove"],
        vec!["link", &x, "narrows", &y, "--remove", "--mark", "absent"],
    ] {
        let before = std::fs::read(fx.ledger().join("events.jsonl")).unwrap();
        let refused = fx.run(&args);
        assert!(!refused.status.success());
        if args.len() == 5 {
            for mark in MARKS {
                assert!(stderr(&refused).contains(mark));
            }
        }
        assert_eq!(
            std::fs::read(fx.ledger().join("events.jsonl")).unwrap(),
            before
        );
    }
    ok(fx.run(&["link", &x, "narrows", &y, "--remove", "--mark", MARKS[1]]));
    let node = shown(&fx, &y);
    assert_eq!(
        node["cancellation"]["narrowed"].as_array().unwrap().len(),
        2
    );
    assert!(
        !node["body_marked"]
            .as_str()
            .unwrap()
            .contains(&format!("[[retracted by {x}: {}]]", MARKS[1]))
    );
    ok(fx.run(&["undo", "--reason", "restore one passage"]));
    all_marks(&shown(&fx, &y), &x);
    let body = fx.root.join("edited.md");
    std::fs::write(&body, "## Decision\nReplacement.\n").unwrap();
    let edit = stdout(&ok(fx.run(&[
        "edit",
        &y,
        "--reason",
        "replace body",
        "--body-file",
        body.to_str().unwrap(),
    ])));
    for mark in &MARKS[..2] {
        assert_eq!(
            edit.lines().filter(|line| line.contains(mark)).count(),
            1,
            "{edit}"
        );
    }
    let unresolved_line = edit
        .lines()
        .find(|line| line.starts_with("unresolved: "))
        .unwrap();
    assert_eq!(unresolved_line.matches("narrows:").count(), 2, "{edit}");
    ok(fx.run(&["undo", "--reason", "measure JSON too"]));
    let edit_json: serde_json::Value = serde_json::from_slice(
        &ok(fx.run(&[
            "edit",
            &y,
            "--reason",
            "replace body",
            "--body-file",
            body.to_str().unwrap(),
            "--json",
        ]))
        .stdout,
    )
    .unwrap();
    let unresolved = edit_json["unresolved"].as_array().unwrap();
    assert_eq!(unresolved.len(), 2);
    for mark in &MARKS[..2] {
        assert_eq!(
            unresolved
                .iter()
                .filter(|v| v.as_str().unwrap().contains(mark))
                .count(),
            1
        );
    }
    ok(fx.run(&["link", &x, "narrows", &y, "--remove", "--mark", MARKS[0]]));
    ok(fx.run(&["link", &x, "narrows", &y, "--remove", "--mark", MARKS[1]]));
    ok(fx.run(&["link", &x, "narrows", &y, "--remove"]));
    assert!(shown(&fx, &y)["cancellation"].is_null());
}

#[test]
fn retirement_counts_all_three_narrows_boundaries() {
    let (fx, x, y) = setup(false);
    three(&fx, &x, &y, 0);
    let before = std::fs::read(fx.ledger().join("events.jsonl")).unwrap();
    let out = fx.run(&["retire", &x, "--reason", "retire one"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out).matches(&format!("{x} narrows {y}")).count(),
        3,
        "{}",
        stderr(&out)
    );
    assert_eq!(
        std::fs::read(fx.ledger().join("events.jsonl")).unwrap(),
        before
    );
}
