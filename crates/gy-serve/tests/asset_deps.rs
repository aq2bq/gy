//! The shape of the assets (d-81a4): the role of every file, the direction of
//! the calls, who holds the document, who opens a socket, and who writes the
//! state. The dictionary's guard (assets.rs) reads the words; this one reads
//! the structure. A new file must name its role here, so its place is asked
//! before it is written.
use gy_serve::assets;

/// The role of every asset (d-81a4): a file must say what it is, so a new file
/// cannot arrive without someone naming its role. The checks below read this
/// table, never a file name. `network` is the one role that may open a socket;
/// `asset` is for the files that are not code (html, css, words).
const ROLES: [(&str, &str); 34] = [
    ("index.html", "asset"),
    ("app.css", "asset"),
    ("state.js", "core"),
    ("reads.js", "network"),
    ("watch.js", "network"),
    ("ease.js", "core"),
    ("events.js", "core"),
    ("root.js", "core"),
    ("sidebar.js", "region"),
    ("topbar.js", "region"),
    ("band.js", "region"),
    ("i18n.json", "asset"),
    ("now.css", "asset"),
    ("sky.js", "part"),
    ("now.js", "region"),
    ("palette.css", "asset"),
    ("palette.js", "region"),
    ("list.css", "asset"),
    ("list.js", "region"),
    ("eye.css", "asset"),
    ("eye.js", "region"),
    ("history.css", "asset"),
    ("history.js", "region"),
    ("node.css", "asset"),
    ("node.js", "region"),
    ("time.css", "asset"),
    ("time.js", "part"),
    ("graph.css", "asset"),
    ("graph-draw.js", "part"),
    ("graph.js", "region"),
    ("writes.js", "part"),
    ("copy.js", "part"),
    ("rail.css", "asset"),
    ("rail.js", "region"),
];

const ROLE_NAMES: [&str; 5] = ["core", "region", "part", "asset", "network"];

/// The file's code, with its block comments taken out: the rules are about what
/// the code does, not what a comment says (the assets use no line comments).
fn code(body: &str) -> String {
    let mut out = String::new();
    let mut rest = body;
    loop {
        match rest.find("/*") {
            None => {
                out.push_str(rest);
                break;
            }
            Some(start) => {
                out.push_str(&rest[..start]);
                match rest[start + 2..].find("*/") {
                    None => break,
                    Some(end) => rest = &rest[start + 2 + end + 2..],
                }
            }
        }
    }
    out
}

fn body(name: &str) -> String {
    code(assets::get(name).expect(name).0)
}

/// The state field the code assigns to, if it does. A read (`state.x`), and a
/// comparison (`state.x === y`), are not writes; `state.a.b = c` is one.
fn writes_the_state(text: &str) -> Option<String> {
    let mut rest = text;
    while let Some(at) = rest.find("state.") {
        let after = &rest[at + 6..];
        let mut path = String::from("state.");
        let mut len = 0;
        loop {
            let name_len: usize = after[len..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .map(char::len_utf8)
                .sum();
            if name_len == 0 {
                break;
            }
            path.push_str(&after[len..len + name_len]);
            len += name_len;
            if after[len..].starts_with('.') {
                path.push('.');
                len += 1;
            } else {
                break;
            }
        }
        let tail = after[len..].trim_start();
        if len > 0 && tail.starts_with('=') && !tail.starts_with("==") && !tail.starts_with("=>") {
            return Some(path);
        }
        rest = after;
    }
    None
}

/// Every embedded file names its role, and every named role is a real one.
#[test]
fn every_asset_names_its_role() {
    let mut table: Vec<&str> = ROLES.iter().map(|(name, _)| *name).collect();
    let mut files: Vec<&str> = assets::FILES.iter().map(|(name, _)| *name).collect();
    table.sort_unstable();
    files.sort_unstable();
    assert_eq!(
        table, files,
        "the role table and the embedded files must name the same set"
    );
    for (name, role) in ROLES {
        assert!(
            ROLE_NAMES.contains(&role),
            "{name} has the unknown role {role}"
        );
    }
}

/// A region draws what it is handed: it reaches for a part (a pure helper) and
/// its own name, never for another region, the core, or the network.
#[test]
fn a_region_does_not_reach_for_another_region_or_the_core() {
    let globals = [
        ("state.js", "GyState"),
        ("reads.js", "GyReads"),
        ("watch.js", "GyWatch"),
        ("ease.js", "GyEase"),
        ("events.js", "GyEvents"),
        ("sidebar.js", "GySidebar"),
        ("topbar.js", "GyTopbar"),
        ("band.js", "GyBand"),
        ("rail.js", "GyRail"),
        ("palette.js", "GyPalette"),
        ("now.js", "GyNow"),
        ("list.js", "GyList"),
        ("eye.js", "GyEye"),
        ("node.js", "GyNode"),
        ("history.js", "GyHistory"),
        ("graph.js", "GyGraph"),
    ];
    for (name, kind) in ROLES {
        if kind != "region" {
            continue;
        }
        let text = body(name);
        for (other, global) in globals {
            if other == name {
                continue;
            }
            assert!(
                !text.contains(&format!("window.{global}")),
                "{name} reaches for {other} ({global}); the root hands it what it draws"
            );
        }
    }
}

/// The element comes from the root: a region or a part reaches for neither the
/// document nor an event listener.
#[test]
fn only_the_core_holds_the_document() {
    for (name, kind) in ROLES {
        if kind != "region" && kind != "part" {
            continue;
        }
        let text = body(name);
        for needle in ["document.getElementById", "addEventListener"] {
            assert!(
                !text.contains(needle),
                "{name} ({kind}) contains {needle}; the element comes from the root"
            );
        }
    }
}

/// A read asks and answers, a wait holds a line: both live in the network, and
/// nowhere else does the app open a socket.
#[test]
fn only_the_network_opens_a_socket() {
    for (name, kind) in ROLES {
        if !name.ends_with(".js") || kind == "network" {
            continue;
        }
        let text = body(name);
        assert!(
            !text.contains("fetch("),
            "{name} ({kind}) fetches; the reads live in reads.js and the wait in watch.js"
        );
    }
}

/// A region draws the state it is handed and the parts help it; only the root
/// turns an intent into a new state.
#[test]
fn a_region_or_part_does_not_write_the_state() {
    for (name, kind) in ROLES {
        if kind != "region" && kind != "part" {
            continue;
        }
        let text = body(name);
        assert!(
            !text.contains("GyState.apply"),
            "{name} ({kind}) applies an intent; only the root moves the state"
        );
        assert!(
            !text.contains("state = "),
            "{name} ({kind}) assigns the state; only the root moves the state"
        );
        if let Some(field) = writes_the_state(&text) {
            panic!("{name} ({kind}) writes {field}; the root hands the state in");
        }
        assert!(
            !text.contains("state["),
            "{name} ({kind}) indexes the state; a region reads the fields it is handed"
        );
    }
}

/// The state is the one pure core: it knows neither the document nor the net.
#[test]
fn the_state_is_apart_from_the_document_and_the_network() {
    let text = body("state.js");
    for needle in ["document", "fetch"] {
        assert!(
            !text.contains(needle),
            "state.js mentions {needle}; the state is pure"
        );
    }
}

/// The state is built in one place: the root asks `state.js` for every new
/// state, so the reason a field is cleared lives in the intent that names it
/// (d-03ca, n-ca6d). A hand-built `{ ...state }` in the root is the drift this
/// guards against.
#[test]
fn only_the_state_builds_the_state() {
    let text = body("root.js");
    let mut rest = text.as_str();
    while let Some(at) = rest.find("state = ") {
        let after = &rest[at + "state = ".len()..];
        let named = after.starts_with("window.GyState.initial(")
            || after.starts_with("window.GyState.apply(");
        assert!(
            named,
            "root.js builds the state by hand: {}",
            rest[at..].lines().next().unwrap_or("")
        );
        rest = after;
    }
}
