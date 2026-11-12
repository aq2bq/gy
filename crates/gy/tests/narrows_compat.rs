//! ac-a204: real new CLI writes, read and extended by the frozen 3.0.0.
mod common;
mod narrows_support;
use common::{fixture, ident, stdout};
use narrows_support::*;
use std::process::Command;

fn sync(fx: &common::Fixture, old: bool) {
    let out = if old {
        fx.compat(&["remote", "sync", "--json"])
    } else {
        fx.run(&["remote", "sync", "--json"])
    };
    let text = stdout(&ok(out));
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(value["rejected"].is_null(), "{value}");
}

fn reads(fx: &common::Fixture, x: &str, y: &str) {
    let default = stdout(&ok(fx.compat(&["show", y])));
    let full_text = stdout(&ok(fx.compat(&["show", y, "--full"])));
    text_marks(&default, x, true);
    text_marks(&full_text, x, false);
    let json: serde_json::Value =
        serde_json::from_slice(&ok(fx.compat(&["show", y, "--json"])).stdout).unwrap();
    // The default JSON carries every passage in cancellation/edges, with the
    // same projected body as default text. Also verify its full JSON body.
    assert_eq!(
        json[0]["cancellation"]["narrowed"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    for mark in MARKS {
        assert!(
            json[0]["cancellation"]["narrowed"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["by"] == x && n["mark"] == mark)
        );
        assert!(
            json[0]["edges"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["name"] == "narrowed-by" && e["to"] == x && e["mark"] == mark)
        );
        for (raw, marked) in [
            (&json[0]["body"], &json[0]["body_marked"]),
            (
                &json[0]["data"]["Decision"]["scope"]["text"],
                &json[0]["decision_scope_marked"],
            ),
        ] {
            if raw.as_str().is_some_and(|text| text.contains(mark)) {
                assert!(
                    marked
                        .as_str()
                        .unwrap()
                        .contains(&format!("[[retracted by {x}: {mark}]]"))
                );
            }
        }
    }
    let full: serde_json::Value =
        serde_json::from_slice(&ok(fx.compat(&["show", y, "--full", "--json"])).stdout).unwrap();
    all_marks(&full[0], x);
    ok(fx.compat(&["publish"]));
    published(fx, x, y);
    let server = Server::start(fx, true);
    all_marks(&server.get(&format!("/api/node/{y}")), x);
    println!(
        "ac-a204 3.0.0: default text summary={}, displayed inline={}; default JSON narrowed={}; full text inline={}; full JSON/publish/node API=3 passages",
        default.matches("narrowed-by ").count(),
        default.matches("[[retracted by ").count(),
        json[0]["cancellation"]["narrowed"]
            .as_array()
            .unwrap()
            .len(),
        full_text.matches("[[retracted by ").count(),
    );
}

#[test]
#[ignore = "requires the frozen gy 3.0.0 executable in GY_COMPAT_BIN (CI runs it explicitly)"]
fn release_3_0_0_reads_syncs_and_writes_above_three_new_cli_narrows() {
    ident();
    let (new, x, y) = setup(false);
    let remote = new.root.parent().unwrap().join("remote.git");
    assert!(
        Command::new("git")
            .args(["init", "--bare", "-q"])
            .arg(&remote)
            .status()
            .unwrap()
            .success()
    );
    let url = format!("file://{}", remote.display());
    ok(new.run(&["remote", "set", &url]));
    let old = fixture();
    std::fs::write(
        old.root.join("gy.toml"),
        format!("remote = \"{url}\"\noutput = \"pub\"\n[scopes.a]\n"),
    )
    .unwrap();
    ok(old.compat(&["remote", "join"]));
    // This is deliberately the actual new link path, never seeded JSON.
    three(&new, &x, &y, 0);
    sync(&new, false);
    sync(&old, true);
    assert_eq!(
        std::fs::read_to_string(new.ledger().join("format"))
            .unwrap()
            .trim(),
        "4"
    );
    reads(&old, &x, &y);
    ok(old.compat(&[
        "edit",
        &x,
        "--title",
        "written by release 3.0.0",
        "--reason",
        "another write",
    ]));
    sync(&old, true);
    sync(&new, false);
    all_marks(&shown(&new, &y), &x);
    assert_eq!(shown(&new, &x)["title"], "written by release 3.0.0");
    assert_eq!(
        std::fs::read(new.ledger().join("events.jsonl")).unwrap(),
        std::fs::read(old.ledger().join("events.jsonl")).unwrap()
    );
    reads(&old, &x, &y);
}
