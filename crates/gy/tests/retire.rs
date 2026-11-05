mod common;
use common::{fixture, stderr, stdout};
use gy_ledger::{
    Alias, Approval, Closed, ClosedBy, Closure, Completion, FileStore, FormatVersion, Node,
    NodeData, NodeId, NodeKind, Open, Repository, RequirementState, format,
};
use std::sync::Arc;
const MARK: &str = "UNIQUE_RETIRED_TEXT";
fn seed() -> (common::Fixture, Vec<String>) {
    let f = fixture();
    let mut need = common::need("0001", MARK);
    if let NodeData::Need(d) = need.data_mut() {
        d.closed = Some(Closed {
            by: ClosedBy::Fact,
            evidence: MARK.into(),
        });
    }
    let mut q = common::question("0001", MARK);
    if let NodeData::Question(d) = q.data_mut() {
        d.closure = Some(Closure::Fact);
        d.evidence = Some(MARK.into());
    }
    let mut ac = common::criterion("0001", MARK);
    if let NodeData::Criterion(d) = ac.data_mut() {
        d.satisfied = true;
        d.evidence = Some(MARK.into());
    }
    let mut req = Node::requirement(
        NodeId::from_hash(NodeKind::Requirement, "0001").unwrap(),
        "a",
        common::DATE,
        MARK,
        RequirementState::Done,
    )
    .unwrap();
    if let NodeData::Requirement(d) = req.data_mut() {
        d.approval = Some(
            Approval::new(
                MARK.into(),
                "master".into(),
                MARK.into(),
                common::DATE.into(),
            )
            .unwrap(),
        );
        d.completion = Some(Completion::new(MARK.into(), common::DATE.into()).unwrap());
    }
    let mut nodes = vec![need, q, ac, req, common::decision("0001", MARK)];
    for n in &mut nodes {
        n.set_body(MARK);
    }
    nodes[4].add_alias(Alias("D-06".into()));
    let ids = nodes.iter().map(|n| n.id().to_string()).collect();
    format::write(&f.ledger(), FormatVersion::CURRENT).unwrap();
    let mut r = Repository::with_gate(
        FileStore::open_with(&f.ledger(), |_| Some("piko".into())).unwrap(),
        Arc::new(Open),
    );
    r.transaction(MARK, MARK, |r| {
        for n in &nodes {
            r.put(n)?;
        }
        Ok(())
    })
    .unwrap();
    r.transaction("remaining", "remaining", |r| {
        r.put(&common::decision("ffff", "still visible"))
    })
    .unwrap();
    (f, ids)
}
fn output(f: &common::Fixture, args: &[&str]) -> String {
    let out = f.run(args);
    assert!(out.status.success(), "{args:?}: {}", stderr(&out));
    stdout(&out)
}
#[test]
fn every_cli_read_excludes_retired_payloads_and_publication_is_rewritten() {
    let (f, ids) = seed();
    let mut reads = vec![
        vec!["list"],
        vec!["list", "--since", "0"],
        vec!["list", "--actor", "piko"],
        vec!["list", "--grep", MARK],
        vec!["next"],
        vec!["handover"],
        vec!["show", "d-ffff", "--full"],
    ];
    for k in ["need", "question", "decision", "criterion", "requirement"] {
        reads.push(vec!["list", "--type", k]);
    }
    for s in [
        "open",
        "closed",
        "done",
        "satisfied",
        "unsatisfied",
        "filed",
        "approved",
        "cancelled",
    ] {
        reads.push(vec!["list", "--status", s]);
    }
    reads.push(vec!["list", "--targets", "d-ffff"]);
    let before_live = output(&f, &["show", "d-ffff", "--full", "--json"]);
    for args in [
        &["show", "D-6", "--full"][..],
        &["list"][..],
        &["list", "--since", "0"][..],
        &["list", "--actor", "piko"][..],
        &["list", "--grep", MARK][..],
    ] {
        assert!(output(&f, args).contains(MARK), "precondition: {args:?}");
    }
    let outdir = f.root.join("pub");
    let out = outdir.to_str().unwrap();
    output(&f, &["publish", "--out", out]);
    assert!(
        std::fs::read_to_string(outdir.join("a/decisions/d-0001.md"))
            .unwrap_or_default()
            .contains(MARK)
            || publication_text(&outdir).contains(MARK)
    );
    let logpath = f.ledger().join("events.jsonl");
    let before = std::fs::read(&logpath).unwrap();
    let mut args = vec!["retire"];
    args.extend(ids.iter().map(String::as_str));
    args.extend(["--reason", "earlier process records"]);
    assert!(output(&f, &args).contains("元の文はログと remote の git に残る"));
    let after = std::fs::read(&logpath).unwrap();
    assert!(after.starts_with(&before));
    assert_eq!(
        after[before.len()..]
            .iter()
            .filter(|b| **b == b'\n')
            .count(),
        1
    );
    for args in reads {
        for json in [false, true] {
            let mut args = args.clone();
            if json {
                args.push("--json");
            }
            let text = output(&f, &args);
            assert!(!text.contains(MARK), "{args:?}: {text}");
            if !args.contains(&"--since") && !args.contains(&"--actor") {
                for id in &ids {
                    assert!(!text.contains(id), "{args:?}: {id}");
                }
            }
        }
    }
    for scope in [None, Some("a")] {
        for command in ["next", "handover"] {
            for json in [false, true] {
                let mut a = vec![command];
                if let Some(scope) = scope {
                    a.extend(["--scope", scope]);
                }
                if json {
                    a.push("--json");
                }
                let t = output(&f, &a);
                assert!(!t.contains(MARK));
            }
        }
    }
    assert_eq!(
        output(&f, &["show", "d-ffff", "--full", "--json"]),
        before_live
    );
    for id in ids.iter().map(String::as_str).chain(["D-6"]) {
        let o = f.run(&["show", id, "--full", "--json"]);
        let text = stdout(&o);
        assert!(
            text.contains("seq 3") && text.contains("--revoke"),
            "{text}"
        );
        assert!(!text.contains(MARK));
        for a in [
            vec!["edit", id, "--reason", "test", "--title", "changed"],
            vec!["req", "revise", id, "--reason", "test", "--source", "test"],
            vec!["link", "d-ffff", "completes", id],
        ] {
            assert!(!f.run(&a).status.success());
        }
    }
    output(&f, &["publish", "--out", out]);
    assert!(!publication_text(&outdir).contains(MARK));
    for mode in [
        vec!["list", "--since", "0", "--json"],
        vec!["list", "--actor", "piko", "--json"],
    ] {
        let text = output(&f, &mode);
        for id in &ids {
            assert!(text.contains(id));
        }
        assert!(text.contains("earlier process records"));
    }
    // Restore the full set; payload and ordinary historical rows return.
    args.push("--revoke");
    output(&f, &args);
    assert!(output(&f, &["list", "--since", "0"]).contains(MARK));
}
fn publication_text(dir: &std::path::Path) -> String {
    let mut out = String::new();
    for p in std::fs::read_dir(dir).unwrap() {
        let p = p.unwrap().path();
        if p.is_dir() {
            out.push_str(&publication_text(&p));
        } else {
            out.push_str(&std::fs::read_to_string(p).unwrap());
        }
    }
    out
}
#[test]
fn publish_clears_the_pages_when_the_last_node_of_a_scope_retires() {
    let f = fixture();
    f.seed(&[common::decision("0001", MARK)]);
    let dir = f.root.join("pub");
    let out = dir.to_str().unwrap();
    output(&f, &["publish", "--out", out]);
    assert!(publication_text(&dir).contains(MARK));
    output(&f, &["retire", "d-0001", "--reason", "process record"]);
    output(&f, &["publish", "--out", out]);
    assert!(!publication_text(&dir).contains(MARK));
}
